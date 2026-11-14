use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use log::LevelFilter;

use nsga_rs::algorithms::{Algorithm, EpsNSGA2, EpsNSGA2Arg, NumThreads, StoppingCondition};
use nsga_rs::core::builtin_problems::DTLZ1Problem;

/// Solve the DTLZ1 problem from Deb et al. (2013) with 3 objectives. This is a problem where the
/// optimal solutions or objectives lie on the hyper-plane passing through the intercept point
/// at 0.5 on each objective axis. This code replicates the first testing problem in Deb et al.
/// (2013).
///
/// Make sure to compile this in release mode to speed up the calculation:
///
/// `cargo run --example eps_nsga2_dtlz1 -p nsga_rs --release`
///
/// The example solves the problem twice: first with a hyper-box resolution of 0.1, then
/// with 0.3 to showcase how the number affects the resolution on the Pareto front and
/// in the solution space charts.
fn main() -> Result<(), Box<dyn Error>> {
    // Add log
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Load the built-in problem.

    let number_objectives: usize = 3;
    // Set the number of variables to use in the DTLZ1 problem
    let k: usize = 5;
    let number_variables: usize = number_objectives + k - 1;
    // Get the built-in problem
    let problem = DTLZ1Problem::create(number_variables, number_objectives, false)?;

    // Setup and run the Eps-NSGA2 algorithm
    let args = EpsNSGA2Arg {
        // set the hyper-box size for the two objectives
        eps: [0.05; 3].to_vec(),
        // stop the algorithm at 250 generations
        stopping_condition: StoppingCondition::MaxGeneration(1000),
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

    // Export serialised results at last generation
    let out_path = PathBuf::from(&env::current_dir().unwrap())
        .join("examples")
        .join("results");
    let data_file = algo.save_to_json(&out_path, Some("DTLZ1_3obj"))?;

    // Plot the Pareto front using the exported data at the last generation
    let serialised_data = EpsNSGA2::read_json_file(&data_file)?;
    serialised_data.plot_front(&out_path.join("DTLZ1_3obj_EpsNSGA2_Pareto_front.png"), None)?;

    Ok(())
}
