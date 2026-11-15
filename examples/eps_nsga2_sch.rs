use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use gnuplot::PlotOption::{Caption, Color, LineWidth, PointSymbol};
use gnuplot::{AutoOption, AxesCommon, Figure};
use log::{info, LevelFilter};

use nsga_rs::algorithms::{
    Algorithm, AlgorithmExport, EpsNSGA2, EpsNSGA2Arg, NumThreads, StoppingCondition,
};
use nsga_rs::core::builtin_problems::SCHProblem;

/// Solve the Schaffer’s problem (SCH) where the following 2 objectives are minimised:
/// - `f_1(x) = x^2`
/// - `f_2(x) = (x - 2)^2`
///
/// The problem has 1 variable (`x`) bounded to -1000 and 1000. The optional solution is expected
/// to lie in the [0; 2] range. The algorithm converges in about 1 second.
///
/// Make sure to compile this in release mode to speed up the calculation:
///
/// `cargo run --example eps_nsga2_sch -p nsga_rs --release`
///
/// The example solves the problem twice: first with a hyper-box resolution of 0.1, then
/// with 0.3 to showcase how the number affects the resolution on the Pareto front and
/// in the solution space charts.
fn main() -> Result<(), Box<dyn Error>> {
    // Add log
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Load the built-in problem.

    for hyper_box_resolution in [0.1, 0.3] {
        info!("Starting example with hyper-box resolution {hyper_box_resolution}");

        let problem = SCHProblem::create()?;
        // ANCHOR: args
        // Setup and run the Eps-NSGA2 algorithm
        let args = EpsNSGA2Arg {
            // set the hyper-box size for the two objectives
            eps: vec![hyper_box_resolution, hyper_box_resolution],
            // stop the algorithm at 250 generations
            stopping_condition: StoppingCondition::MaxGeneration(2500),
            // use default values for the initial individuals (`10`)
            number_of_initial_individuals: None,
            // use default stopping conditions for the intra NSGA2 runs
            intra_run_stop_cond: None,
            // use the default stopping conditions with no threshold for the inter runs
            inter_run_stopping_condition: None,
            // use default options for the SBX and PM operators
            crossover_operator_options: None,
            mutation_operator_options: None,
            // no need to evaluate the objective in parallel
            threads: NumThreads::Off,
            // do not export intermediate solutions
            export_history: None,
            // to reproduce results
            resume_from_file: None,
            seed: Some(10),
        };
        let mut algo = EpsNSGA2::new(Arc::new(problem), args)?;
        algo.run()?;
        // ANCHOR_END: args

        // Export serialised results at last generation
        let out_path = PathBuf::from(&env::current_dir().unwrap())
            .join("examples")
            .join("results");
        let data_file =
            algo.save_to_json(&out_path, Some(&format!("SCH_2obj_{hyper_box_resolution}")))?;

        // Plot the Pareto front using the exported data at the last generation
        let serialised_data = EpsNSGA2::read_json_file(&data_file)?;
        serialised_data.plot_front(
            &out_path.join(format!(
                "SCH_2obj_{hyper_box_resolution}_EpsNSGA2_Pareto_front.png"
            )),
            None,
        )?;

        // Plot the solution chart SCH_2obj_EpsNSGA2_solutions.png
        let export_data: AlgorithmExport = serialised_data.try_into()?;

        let x = (-60..=60).map(|v| v as f64 / 10.0).collect::<Vec<_>>();
        let f1: Vec<f64> = x.iter().map(|v| v.powi(2)).collect();
        let f2: Vec<f64> = x.iter().map(|v| (v - 2.0).powi(2)).collect();

        let mut fg = Figure::new();
        let ax = fg.axes2d();

        // theoretical curves
        ax.lines(
            &x,
            &f1,
            &[
                Color("blue".into()),
                Caption("Objective f_1"),
                LineWidth(2.0),
            ],
        );
        ax.lines(
            &x,
            f2,
            &[
                Color("green".into()),
                Caption("Objective f_2"),
                LineWidth(2.0),
            ],
        );

        // data from algorithm
        let var = export_data.get_real_variables("x")?;
        let calc_f1 = export_data.get_objective("x^2")?;
        let calc_f2 = export_data.get_objective("(x-2)^2")?;
        ax.points(
            &var,
            calc_f1,
            &[Color("red".into()), PointSymbol('O'), Caption("Solution")],
        );
        ax.points(&var, calc_f2, &[Color("red".into()), PointSymbol('O')]);

        ax.set_x_label("x", &[])
            .set_y_label("Objective", &[])
            .set_x_grid(true)
            .set_x_range(AutoOption::Fix(-3.0), AutoOption::Fix(5.0))
            .set_y_grid(true)
            .set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(20.0))
            .set_title("SCH problem solved with Eps-NSGA2", &[]);
        fg.save_to_png(
            &data_file.parent().unwrap().join(format!(
                "SCH_2obj_{hyper_box_resolution}_EpsNSGA2_solutions.png"
            )),
            600,
            500,
        )?;
    }
    Ok(())
}
