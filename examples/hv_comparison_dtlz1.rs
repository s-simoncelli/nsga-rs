use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use gnuplot::{AxesCommon, Caption, Color, Figure, PointSymbol};

use nsga_rs::algorithms::{
    Algorithm, EpsNSGA2, EpsNSGA2Arg, ExportHistory, NSGA2Arg, NumThreads, StoppingCondition, NSGA2,
};
use nsga_rs::core::builtin_problems::DTLZ1Problem;
use nsga_rs::core::{Individual, Individuals, OError};
use nsga_rs::metrics::HyperVolume;

/// Compare the convergence and Pareto fronts of NSGA2 and ε-NSGA2 on the DTLZ1 problem
/// with 3 objectives by calculating the hyper-volume at different generations.
///
/// Both algorithms export their population history every 50 generations. The reference point
/// is the worst value of each objective across both histories and is used for both, so that the
/// hyper-volume values are comparable. The history JSON files are deleted at the end.
///
/// Make sure to compile this in release mode to speed up the calculation:
///
/// `cargo run --example hv_comparison_dtlz1 -p nsga_rs --release`
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let number_objectives: usize = 3;
    let k: usize = 5;
    let problem = Arc::new(DTLZ1Problem::create(
        number_objectives + k - 1,
        number_objectives,
        false,
    )?);

    let out_path = PathBuf::from(&env::current_dir()?)
        .join("examples")
        .join("results");
    let history_path = out_path.join("hv_comparison_dtlz1");
    let nsga2_path = history_path.join("NSGA2");
    let eps_nsga2_path = history_path.join("EpsNSGA2");
    fs::create_dir_all(&nsga2_path)?;
    fs::create_dir_all(&eps_nsga2_path)?;

    // Run NSGA2
    let args = NSGA2Arg {
        number_of_individuals: 92,
        stopping_condition: StoppingCondition::MaxGeneration(1000),
        crossover_operator_options: None,
        mutation_operator_options: None,
        threads: NumThreads::Off,
        export_history: Some(ExportHistory::new(50, &nsga2_path)?),
        resume_from_file: None,
        seed: Some(10),
    };
    let mut algo = NSGA2::new(problem.clone(), args)?;
    algo.run()?;
    let nsga2_front = algo.get_results().individuals;

    // Run ε-NSGA2
    let args = EpsNSGA2Arg {
        eps: [0.05; 3].to_vec(),
        stopping_condition: StoppingCondition::MaxGeneration(1000),
        number_of_initial_individuals: None,
        intra_run_stop_cond: None,
        inter_run_stopping_condition: None,
        crossover_operator_options: None,
        mutation_operator_options: None,
        threads: NumThreads::Off,
        export_history: Some(ExportHistory::new(50, &eps_nsga2_path)?),
        resume_from_file: None,
        seed: Some(10),
    };
    let mut algo = EpsNSGA2::new(problem, args)?;
    algo.run()?;
    let eps_nsga2_front = algo.get_results().individuals;

    // ANCHOR: ref_point
    // Use the same reference point for both algorithms: the worst value of each objective
    // across both histories, plus an offset so that the point is strictly dominated
    let nsga2_data = NSGA2::read_json_files(&nsga2_path)?;
    let eps_nsga2_data = EpsNSGA2::read_json_files(&eps_nsga2_path)?;
    let nsga2_ref = HyperVolume::estimate_reference_point_from_files(&nsga2_data, None)?;
    let eps_nsga2_ref = HyperVolume::estimate_reference_point_from_files(&eps_nsga2_data, None)?;
    let ref_point: Vec<f64> = nsga2_ref
        .iter()
        .zip(&eps_nsga2_ref)
        .map(|(n, e)| n.max(*e) + 1.0)
        .collect();
    println!("Reference point: {:?}", ref_point);

    let nsga2_hv = HyperVolume::from_files(&nsga2_data, &ref_point)?;
    let eps_nsga2_hv = HyperVolume::from_files(&eps_nsga2_data, &ref_point)?;
    // ANCHOR_END: ref_point

    println!("{:>10} {:>15} {:>15}", "Generation", "NSGA2", "ε-NSGA2");
    let eps_values = eps_nsga2_hv.values();
    for ((generation, n), e) in nsga2_hv.nfe().iter().zip(nsga2_hv.values()).zip(eps_values) {
        println!("{:>10} {:>15.4e} {:>15.4e}", generation, n, e);
    }

    // Plot both hyper-volume lines on the same chart
    let mut fg = Figure::new();
    fg.axes2d()
        .lines_points(
            nsga2_hv.duration_as_secs(),
            nsga2_hv.values(),
            &[Caption("NSGA2"), Color("black".into())],
        )
        .lines_points(
            eps_nsga2_hv.duration_as_secs(),
            eps_nsga2_hv.values(),
            &[Caption("EpsNSGA2"), Color("red".into())],
        )
        .set_x_grid(true)
        .set_x_label("Time", &[])
        .set_x_time(true)
        .set_y_grid(true)
        .set_y_label("Hyper volume", &[])
        .set_title("DTLZ1 (3 objectives) - Hyper-volume comparison", &[]);
    fg.save_to_png(
        out_path.join("DTLZ1_3obj_NSGA2_vs_EpsNSGA2_convergence_chart.png"),
        800,
        600,
    )?;

    // Plot both Pareto fronts at the last generation on the same chart
    let objectives = |front: &[Individual]| -> Result<[Vec<f64>; 3], OError> {
        Ok([
            front.objective_values("f1")?,
            front.objective_values("f2")?,
            front.objective_values("f3")?,
        ])
    };
    let [n1, n2, n3] = objectives(&nsga2_front)?;
    let [e1, e2, e3] = objectives(&eps_nsga2_front)?;
    let mut fg = Figure::new();
    fg.axes3d()
        .points(
            n1,
            n2,
            n3,
            &[Caption("NSGA2"), PointSymbol('O'), Color("black".into())],
        )
        .points(
            e1,
            e2,
            e3,
            &[Caption("EpsNSGA2"), PointSymbol('x'), Color("red".into())],
        )
        .set_x_label("f1", &[])
        .set_y_label("f2", &[])
        .set_z_label("f3", &[])
        .set_x_grid(true)
        .set_y_grid(true)
        .set_z_grid(true)
        .set_view(40.0, 110.0)
        .set_title(
            "DTLZ1 (3 objectives) - Pareto fronts at last generation",
            &[],
        );
    fg.save_to_png(
        out_path.join("DTLZ1_3obj_NSGA2_vs_EpsNSGA2_Pareto_front.png"),
        800,
        600,
    )?;

    // Remove the history JSON files
    fs::remove_dir_all(&history_path)?;

    Ok(())
}
