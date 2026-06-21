use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use optirustic_macros::as_algorithm_args;

use crate::algorithms::nsga2::crowding::CrowdingDistanceOperator;
use crate::algorithms::nsga2::survival::NSGA2Survival;
use crate::algorithms::nsga2::{CoreNSGA2, CoreNSGA2Arg};
use crate::algorithms::Algorithm;
use crate::core::{OError, Population, Problem};
use crate::operators::{PolynomialMutationArgs, SimulatedBinaryCrossoverArgs};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Input arguments for the NSGA2 algorithm.
#[as_algorithm_args]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct NSGA2Arg {
    /// The number of individuals to use in the population. This must be a multiple of `2`.
    pub number_of_individuals: usize,
    /// The options of the Simulated Binary Crossover (SBX) operator. This operator is used to
    /// generate new children by recombining the variables of parent solutions. This defaults to
    /// [`SimulatedBinaryCrossoverArgs::default()`].
    pub crossover_operator_options: Option<SimulatedBinaryCrossoverArgs>,
    /// The options to Polynomial Mutation (PM) operator used to mutate the variables of an
    /// individual. This defaults to [`PolynomialMutationArgs::default()`],
    /// with a distribution index or index parameter of `20` and variable probability equal `1`
    /// divided by the number of real variables in the problem (i.e., each variable will have the
    /// same probability of being mutated).
    pub mutation_operator_options: Option<PolynomialMutationArgs>,
    /// Instead of initialising the population with random variables, see the initial population
    /// with  the variable values from a JSON files exported with this tool. This option lets you
    /// restart the evolution from a previous generation; you can use any history file (exported
    /// when the field `export_history`) or the file exported when the stopping condition was reached.
    pub resume_from_file: Option<PathBuf>,
    /// The seed used in the random number generator (RNG). You can specify a seed in case you want
    /// to try to reproduce results. NSGA2 is a stochastic algorithm that relies on a RNG at
    /// different steps (when population is initially generated, during selection, crossover and
    /// mutation) and, as such, may lead to slightly different solutions. The seed is randomly
    /// picked if this is `None`.
    pub seed: Option<u64>,
}

#[cfg(feature = "python")]
#[pymethods]
impl NSGA2Arg {
    #[new]
    #[pyo3(signature = (number_of_individuals, stopping_condition, crossover_operator_options=None, mutation_operator_options=None, resume_from_file=None, parallel=None, export_history=None, seed=None))]
    fn py_new(
        number_of_individuals: usize,
        stopping_condition: StoppingCondition,
        crossover_operator_options: Option<SimulatedBinaryCrossoverArgs>,
        mutation_operator_options: Option<PolynomialMutationArgs>,
        resume_from_file: Option<PathBuf>,
        parallel: Option<bool>,
        export_history: Option<ExportHistory>,
        seed: Option<u64>,
    ) -> PyResult<Self> {
        Ok(NSGA2Arg {
            number_of_individuals,
            crossover_operator_options,
            mutation_operator_options,
            resume_from_file,
            seed,
            stopping_condition,
            parallel,
            export_history,
        })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "NSGA2Arg(number_of_individuals={}, stopping_condition={})",
            self.number_of_individuals, self.stopping_condition
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

/// The Non-dominated Sorting Genetic Algorithm (NSGA2).
///
/// Implemented based on:
/// > K. Deb, A. Pratap, S. Agarwal and T. Meyarivan, "A fast and elitist multi-objective genetic
/// > algorithm: NSGA-II," in IEEE Transactions on Evolutionary Computation, vol. 6, no. 2, pp.
/// > 182-197, April 2002, doi: 10.1109/4235.996017.
///
/// See: <https://doi.org/10.1109/4235.996017>.
///
/// # Examples
/// ## Solve the Schaffer’s problem
/// ```rust
#[doc = include_str!("../../../examples/nsga2_sch.rs")]
/// ```
/// ## Solve the ZDT1 problem
/// ```rust
#[doc = include_str!("../../../examples/nsga2_zdt1.rs")]
/// ```
pub struct NSGA2 {
    /// The core NSGA2 algorithm.
    core: CoreNSGA2,
    /// The options sued to initialise the algorithm
    args: NSGA2Arg,
}

impl NSGA2 {
    /// Initialise the NSGA2 algorithm.
    ///
    /// # Arguments
    ///
    /// * `problem`: The problem being solved.
    /// * `args`: The [`NSGA2Arg`] arguments to customise the algorithm behaviour.
    ///
    /// returns: `NSGA2`.
    pub fn new(problem: Problem, options: NSGA2Arg) -> Result<Self, OError> {
        let nsga2_options = options.clone();
        let core_options = CoreNSGA2Arg {
            stopping_condition: options.stopping_condition,
            number_of_individuals: options.number_of_individuals,
            crossover_operator_options: options.crossover_operator_options,
            mutation_operator_options: options.mutation_operator_options,
            resume_from_file: options.resume_from_file,
            seed: options.seed,
            survival_operator: NSGA2Survival::LargestCrowdingDistanceFirst,
            crowding_operator: CrowdingDistanceOperator::Default,
            parallel: options.parallel,
            export_history: options.export_history,
        };

        Ok(Self {
            args: nsga2_options,
            core: CoreNSGA2::new(problem, core_options)?,
        })
    }
}

/// Implementation of Section IIIC of the paper.
impl Algorithm<NSGA2Arg> for NSGA2 {
    /// This assesses the initial random population and sets the individual's ranks and crowding
    /// distance needed in [`self.evolve`].
    ///
    /// return: `Result<(), OError>`
    fn initialise(&mut self) -> Result<(), OError> {
        self.core.initialise()
    }

    fn evolve(&mut self) -> Result<(), OError> {
        self.core.evolve()
    }

    fn stopping_condition(&self) -> &StoppingCondition {
        &self.core.stopping_condition
    }

    fn name(&self) -> String {
        "NSGA2".to_string()
    }

    fn problem(&self) -> Arc<Problem> {
        self.core.problem.clone()
    }

    fn population(&self) -> &Population {
        &self.core.population
    }

    fn export_history(&self) -> Option<&ExportHistory> {
        self.core.export_history.as_ref()
    }

    fn generation(&self) -> u32 {
        self.core.generation
    }

    fn number_of_function_evaluations(&self) -> u32 {
        self.core.nfe
    }

    fn algorithm_options(&self) -> NSGA2Arg {
        self.args.clone()
    }

    fn start_time(&self) -> &Instant {
        &self.core.start_time
    }
}

impl Display for NSGA2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name().as_str())
    }
}

#[cfg(test)]
mod test_problems {
    use optirustic_macros::test_with_retries;

    use crate::algorithms::{Algorithm, NSGA2Arg, StoppingCondition, NSGA2};
    use crate::core::builtin_problems::{
        SCHProblem, ZTD1Problem, ZTD2Problem, ZTD3Problem, ZTD4Problem,
    };
    use crate::core::test_utils::{check_exact_value, check_value_in_range};

    const BOUND_TOL: f64 = 1.0 / 1000.0;
    const LOOSE_BOUND_TOL: f64 = 0.1;

    #[test_with_retries(10)]
    /// Test problem 1 from Deb et al. (2002). Optional solution x in [0; 2]
    fn test_sch_problem() {
        let problem = SCHProblem::create().unwrap();
        let args = NSGA2Arg {
            number_of_individuals: 10,
            stopping_condition: StoppingCondition::MaxGeneration(1000),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(10),
        };
        let mut algo = NSGA2::new(problem, args).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        // increase tolerance
        let bounds = -0.1..2.1;
        let invalid_x = check_value_in_range(&results.get_real_variables("x").unwrap(), &bounds);
        if !invalid_x.is_empty() {
            panic!("Some variables are outside the bounds: {:?}", invalid_x);
        }
    }

    #[test_with_retries(10)]
    /// Test the ZTD1 problem from Deb et al. (2002) with 30 variables. Solution x1 in [0; 1] and
    /// x2 to x30 = 0. The exact solutions are tested using a strict and loose bounds.
    fn test_ztd1_problem() {
        let number_of_individuals: usize = 30;
        let problem = ZTD1Problem::create(number_of_individuals).unwrap();
        let args = NSGA2Arg {
            number_of_individuals,
            stopping_condition: StoppingCondition::MaxGeneration(2500),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        };
        let mut algo = NSGA2::new(problem, args).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        let x_bounds = 0.0 - BOUND_TOL..1.0 + BOUND_TOL;
        let invalid_x1 =
            check_value_in_range(&results.get_real_variables("x1").unwrap(), &x_bounds);
        if !invalid_x1.is_empty() {
            panic!("Some X1 variables are outside the bounds: {:?}", invalid_x1);
        }

        let x_bounds = -BOUND_TOL..BOUND_TOL;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_individuals {
            let var_values = results
                .get_real_variables(format!("x{xi}").as_str())
                .unwrap();
            let (x_other_outside_bounds, breached_range, b_type) =
                check_exact_value(&var_values, &x_bounds, &x_other_bounds, 5);
            if !x_other_outside_bounds.is_empty() {
                panic!(
                    "Found {} X2 to X30 solutions ({:?}) outside the {} bounds {:?}",
                    x_other_outside_bounds.len(),
                    x_other_outside_bounds,
                    b_type,
                    breached_range
                );
            }
        }
    }

    #[test_with_retries(10)]
    /// Test the ZTD2 problem from Deb et al. (2002) with 30 variables. Solution x1 in [0; 1] and
    /// x2 to x30 = 0. The exact solutions are tested using a strict and loose bounds.
    fn test_ztd2_problem() {
        let number_of_individuals: usize = 30;
        let problem = ZTD2Problem::create(number_of_individuals).unwrap();
        let args = NSGA2Arg {
            number_of_individuals,
            stopping_condition: StoppingCondition::MaxGeneration(2500),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        };
        let mut algo = NSGA2::new(problem, args).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        let x_bounds = 0.0 - BOUND_TOL..1.0 + BOUND_TOL;
        let invalid_x1 =
            check_value_in_range(&results.get_real_variables("x1").unwrap(), &x_bounds);
        if !invalid_x1.is_empty() {
            panic!(
                "Found {} X1 variables outside the bounds {:?}",
                invalid_x1.len(),
                invalid_x1
            );
        }

        let x_bounds = -BOUND_TOL..BOUND_TOL;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_individuals {
            let var_name = format!("x{xi}");
            let var_values = results.get_real_variables(&var_name).unwrap();

            let (x_other_outside_bounds, breached_range, b_type) =
                check_exact_value(&var_values, &x_bounds, &x_other_bounds, 3);
            if !x_other_outside_bounds.is_empty() {
                panic!(
                    "Found {} {} solutions ({:?}) outside the {} bounds {:?}",
                    x_other_outside_bounds.len(),
                    var_name,
                    x_other_outside_bounds,
                    b_type,
                    breached_range
                );
            }
        }
    }

    #[test_with_retries(10)]
    /// Test the ZTD3 problem from Deb et al. (2002) with 30 variables. Solution x1 in [0; 1] and
    /// x2 to x30 = 0. The exact solutions are tested using a strict and loose bounds.
    fn test_ztd3_problem() {
        let number_of_individuals: usize = 30;
        let problem = ZTD3Problem::create(number_of_individuals).unwrap();
        let args = NSGA2Arg {
            number_of_individuals,
            stopping_condition: StoppingCondition::MaxGeneration(2500),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        };
        let mut algo = NSGA2::new(problem, args).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        let x_bounds = 0.0 - BOUND_TOL..1.0 + BOUND_TOL;
        let invalid_x1 =
            check_value_in_range(&results.get_real_variables("x1").unwrap(), &x_bounds);
        if !invalid_x1.is_empty() {
            panic!(
                "Found {} X1 variables outside the bounds {:?}",
                invalid_x1.len(),
                invalid_x1
            );
        }

        let x_bounds = -BOUND_TOL..BOUND_TOL;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_individuals {
            let var_name = format!("x{xi}");
            let var_values = results.get_real_variables(&var_name).unwrap();

            let (x_other_outside_bounds, breached_range, b_type) =
                check_exact_value(&var_values, &x_bounds, &x_other_bounds, 3);
            if !x_other_outside_bounds.is_empty() {
                panic!(
                    "Found {} {} solutions ({:?}) outside the {} bounds {:?}",
                    x_other_outside_bounds.len(),
                    var_name,
                    x_other_outside_bounds,
                    b_type,
                    breached_range
                );
            }
        }
    }

    #[test_with_retries(10)]
    /// Test the ZTD4 problem from Deb et al. (2002) with 30 variables. Solution x1 in [0; 1] and
    /// x2 to x10 = 0. The exact solutions are tested using a strict and loose bounds.
    fn test_ztd4_problem() {
        let number_of_individuals: usize = 10;
        let args = NSGA2Arg {
            number_of_individuals,
            stopping_condition: StoppingCondition::MaxGeneration(3000),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        };
        let problem = ZTD4Problem::create(number_of_individuals).unwrap();
        let mut algo = NSGA2::new(problem, args.clone()).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        let x_bounds = 0.0 - BOUND_TOL..1.0 + BOUND_TOL;
        let invalid_x1 =
            check_value_in_range(&results.get_real_variables("x1").unwrap(), &x_bounds);
        if !invalid_x1.is_empty() {
            panic!(
                "Found {} X1 variables outside the bounds {:?}",
                invalid_x1.len(),
                invalid_x1
            );
        }

        // relax strict bounds O(2). The final solution is still acceptable.
        let x_bounds = -BOUND_TOL * 10.0..BOUND_TOL * 10.0;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_individuals {
            let var_name = format!("x{xi}");
            let var_values = results.get_real_variables(&var_name).unwrap();

            let (x_other_outside_bounds, breached_range, b_type) =
                check_exact_value(&var_values, &x_bounds, &x_other_bounds, 3);
            if !x_other_outside_bounds.is_empty() {
                panic!(
                    "Found {} {} solutions ({:?}) outside the {} bounds {:?}",
                    x_other_outside_bounds.len(),
                    var_name,
                    x_other_outside_bounds,
                    b_type,
                    breached_range
                );
            }
        }
    }

    #[test_with_retries(10)]
    /// Test the ZTD6 problem from Deb et al. (2002) with 30 variables. Solution x1 in [0; 1] and
    /// x2 to x10 = 0. The exact solutions are tested using a strict and loose bounds.
    fn test_ztd6_problem() {
        let number_of_individuals: usize = 10;
        let problem = ZTD4Problem::create(number_of_individuals).unwrap();
        let args = NSGA2Arg {
            number_of_individuals,
            stopping_condition: StoppingCondition::MaxGeneration(1000),
            crossover_operator_options: None,
            mutation_operator_options: None,
            parallel: Some(false),
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        };
        let mut algo = NSGA2::new(problem, args).unwrap();
        algo.run().unwrap();
        let results = algo.get_results();

        let x_bounds = 0.0 - BOUND_TOL..1.0 + BOUND_TOL;
        let invalid_x1 =
            check_value_in_range(&results.get_real_variables("x1").unwrap(), &x_bounds);
        if !invalid_x1.is_empty() {
            panic!(
                "Found {} X1 variables outside the bounds {:?}",
                invalid_x1.len(),
                invalid_x1
            );
        }

        // relax strict bounds O(2). The final solution is still acceptable.
        let x_bounds = -BOUND_TOL * 10.0..BOUND_TOL * 10.0;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_individuals {
            let var_name = format!("x{xi}");
            let var_values = results.get_real_variables(&var_name).unwrap();

            let (x_other_outside_bounds, breached_range, b_type) =
                check_exact_value(&var_values, &x_bounds, &x_other_bounds, 3);
            if !x_other_outside_bounds.is_empty() {
                panic!(
                    "Found {} {} solutions ({:?}) outside the {} bounds {:?}",
                    x_other_outside_bounds.len(),
                    var_name,
                    x_other_outside_bounds,
                    b_type,
                    breached_range
                );
            }
        }
    }
}
