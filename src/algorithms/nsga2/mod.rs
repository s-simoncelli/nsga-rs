use std::fmt::{Display, Formatter};
use std::ops::Rem;
use std::path::PathBuf;

use log::{debug, info};
use rand::RngCore;

use optirustic_macros::{as_algorithm, as_algorithm_args, impl_algorithm_trait_items};

use crate::algorithms::nsga2::crowding::CrowdingDistanceOperator;
use crate::algorithms::Algorithm;
use crate::core::utils::get_rng;
use crate::core::{Individual, OError};
use crate::operators::{
    Crossover, CrowdedComparison, Mutation, PolynomialMutation, PolynomialMutationArgs, Selector,
    SimulatedBinaryCrossover, SimulatedBinaryCrossoverArgs, TournamentSelector,
};
use crate::utils::fast_non_dominated_sort;
use survival::NSGA2Survival;

// Declare nested modules
mod crowding;
mod nsga2;
mod survival;

// Re-export algorithms
pub use nsga2::{NSGA2Arg, NSGA2};

/// The data key where the crowding distance is stored for each [`Individual`].
pub const CROWDING_DIST_KEY: &str = "crowding_distance";

/// Input arguments for the NSGA2 algorithm.
#[as_algorithm_args]
pub struct CoreNSGA2Arg {
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
    /// The survival operator used to select the individuals when the last front does
    /// not match the number to fill the new population.
    pub survival_operator: NSGA2Survival,
    /// The operator to use to calculate the crowding distance.
    pub crowding_operator: CrowdingDistanceOperator,
}

/// The Non-dominated Sorting Genetic Algorithm (NSGA2).
///
/// Implemented based on:
/// > K. Deb, A. Pratap, S. Agarwal and T. Meyarivan, "A fast and elitist multi-objective genetic
/// > algorithm: NSGA-II," in IEEE Transactions on Evolutionary Computation, vol. 6, no. 2, pp.
/// > 182-197, April 2002, doi: 10.1109/4235.996017.
///ƒ
/// See: <https://doi.org/10.1109/4235.996017>.
///
/// This struct allows implementing a custom survival operator to include individuals
/// from the last front.
#[as_algorithm(CoreNSGA2Arg)]
pub struct CoreNSGA2 {
    /// The operator to use to select the individuals for reproduction. This is a binary tournament
    /// selector ([`TournamentSelector`]) with the [`CrowdedComparison`] comparison operator.
    selector_operator: TournamentSelector<CrowdedComparison>,
    /// The SBX operator to use to generate a new children by recombining the variables of parent
    /// solutions.
    crossover_operator: SimulatedBinaryCrossover,
    /// The PM operator to use to mutate the variables of an individual.
    mutation_operator: PolynomialMutation,
    /// The operator to use to calculate the crowding distance.
    crowding_operator: CrowdingDistanceOperator,
    /// The survival operator.
    survival_operator: NSGA2Survival,
    /// The seed to use.
    rng: Box<dyn RngCore>,
}

impl CoreNSGA2 {
    /// Initialise the NSGA2 algorithm.
    ///
    /// # Arguments
    ///
    /// * `problem`: The problem being solved.
    /// * `args`: The [`CoreNSGA2Arg`] arguments to customise the algorithm behaviour.
    ///
    /// returns: `CoreNSGA2`.
    pub fn new(problem: Problem, options: CoreNSGA2Arg) -> Result<Self, OError> {
        let name = "NSGA2".to_string();
        if options.number_of_individuals < 3 {
            return Err(OError::AlgorithmInit(
                name,
                "The population size must have at least 3 individuals".to_string(),
            ));
        }
        // force the population size as multiple of 2 so that the new number of generated offsprings
        // matches `number_of_individuals`
        if options.number_of_individuals.rem(2) != 0 {
            return Err(OError::AlgorithmInit(
                name,
                "The population size must be a multiple of 2".to_string(),
            ));
        }

        let nsga2_args = options.clone();
        let problem = Arc::new(problem);
        let population = if let Some(init_file) = options.resume_from_file {
            info!("Loading initial population from {:?}", init_file);
            CoreNSGA2::seed_population_from_file(
                problem.clone(),
                &name,
                options.number_of_individuals,
                &init_file,
            )?
        } else {
            info!("Created initial random population");
            Population::init(problem.clone(), options.number_of_individuals)?
        };

        let mutation_options = match options.mutation_operator_options {
            Some(o) => o,
            None => PolynomialMutationArgs::default(problem.clone().as_ref()),
        };
        let mutation_operator = PolynomialMutation::new(mutation_options.clone())?;

        let crossover_options = options.crossover_operator_options.unwrap_or_default();
        let crossover_operator = SimulatedBinaryCrossover::new(crossover_options.clone())?;

        info!(
            "{}",
            Self::algorithm_option_str(&problem, &crossover_options, &mutation_options)
        );

        Ok(Self {
            number_of_individuals: options.number_of_individuals,
            problem,
            population,
            selector_operator: TournamentSelector::<CrowdedComparison>::new(2),
            crossover_operator,
            mutation_operator,
            generation: 0,
            nfe: 0,
            stopping_condition: options.stopping_condition,
            start_time: Instant::now(),
            parallel: options.parallel.unwrap_or(true),
            export_history: options.export_history,
            rng: get_rng(options.seed),
            args: nsga2_args,
            survival_operator: options.survival_operator,
            crowding_operator: options.crowding_operator,
        })
    }

    /// Get a string listing the algorithm options.
    ///
    /// # Arguments
    ///
    /// * `problem`: The problem.
    /// * `crossover_options`: The crossover operator options.
    /// * `mutation_options`: The mutation operator options.
    ///
    /// returns: `String`
    pub fn algorithm_option_str(
        problem: &Arc<Problem>,
        crossover_options: &SimulatedBinaryCrossoverArgs,
        mutation_options: &PolynomialMutationArgs,
    ) -> String {
        let mut log_opts: String = "Algorithm options are:\n".to_owned();
        log_opts.push_str(
            format!("\t* Number of variables {:>13}\n\t* Number of objectives {:>12}\n\t* Number of constraints {:>11}\n",
                    problem.number_of_variables(),
                    problem.number_of_objectives(),
                    problem.number_of_constraints()
            ).as_str()
        );
        log_opts.push_str(
            format!(
                "\t* Crossover distribution index {:>5}\n\t* Crossover probability {:>11}\n\t* Crossover var probability {:>9}\n",
                crossover_options.distribution_index, crossover_options.crossover_probability, crossover_options.variable_probability,
            )
                .as_str(),
        );
        log_opts.push_str(
            format!(
                "\t* Mutation index parameter {:>9}\n\t* Mutation var probability {:>10}",
                mutation_options.index_parameter, crossover_options.variable_probability,
            )
            .as_str(),
        );
        log_opts
    }
}

/// Implementation of Section IIIC of the paper.
#[impl_algorithm_trait_items(CoreNSGA2Arg)]
impl Algorithm<CoreNSGA2Arg> for CoreNSGA2 {
    /// This assesses the initial random population and sets the individual's ranks and crowding
    /// distance needed in [`self.evolve`].
    ///
    /// return: `Result<(), OError>`
    fn initialise(&mut self) -> Result<(), OError> {
        info!("Evaluating initial population");
        if self.parallel {
            CoreNSGA2::do_parallel_evaluation(self.population.individuals_as_mut(), &mut self.nfe)?;
        } else {
            CoreNSGA2::do_evaluation(self.population.individuals_as_mut(), &mut self.nfe)?;
        }

        debug!("Calculating rank");
        fast_non_dominated_sort(self.population.individuals_as_mut(), false)?;

        debug!("Calculating crowding distance");
        self.crowding_operator
            .set_crowding_distance(self.population.individuals_as_mut())?;

        info!("Initial evaluation completed");
        self.generation += 1;

        Ok(())
    }

    fn evolve(&mut self) -> Result<(), OError> {
        // Create the new population, based on the population at the previous time-step, of size
        // self.number_of_individuals. The loop adds two individuals at the time.
        debug!("Generating new population (selection + crossover + mutation)");
        let mut offsprings: Vec<Individual> = Vec::new();
        for _ in 0..self.number_of_individuals / 2 {
            let parents =
                self.selector_operator
                    .select(self.population.individuals(), 2, &mut self.rng)?;

            // generate the 2 children with crossover
            let children = self.crossover_operator.generate_offsprings(
                &parents[0],
                &parents[1],
                &mut self.rng,
            )?;

            // mutate them
            offsprings.push(
                self.mutation_operator
                    .mutate_offspring(&children.child1, &mut self.rng)?,
            );
            offsprings.push(
                self.mutation_operator
                    .mutate_offspring(&children.child2, &mut self.rng)?,
            );
        }
        debug!("Combining parents and offsprings in new population");
        self.population.add_new_individuals(offsprings);
        debug!("New population size is {}", self.population.len());

        debug!("Evaluating population");
        if self.parallel {
            CoreNSGA2::do_parallel_evaluation(self.population.individuals_as_mut(), &mut self.nfe)?;
        } else {
            CoreNSGA2::do_evaluation(self.population.individuals_as_mut(), &mut self.nfe)?;
        }
        debug!("Evaluation done");

        debug!("Calculating fronts and ranks for new population");
        let sorting_results = fast_non_dominated_sort(self.population.individuals_as_mut(), false)?;
        debug!("Collected {} fronts", sorting_results.fronts.len());

        debug!("Selecting best individuals");
        let mut new_population = Population::new();

        // This selects the best individuals that will form the new population which contains the
        // population at the previous generation and the new offsprings. The new population is created
        // by keeping/adding ranked non-dominated fronts until the population size almost reaches
        // `self.number_of_individuals`. When the last front does not fit, the individuals are then
        // added based on their crowding distance.
        //
        // This implements the algorithm at the bottom of page 186 in Deb et al. (2002).
        let mut last_front: Option<Vec<Individual>> = None;
        for (fi, front) in sorting_results.fronts.into_iter().enumerate() {
            if new_population.len() + front.len() <= self.number_of_individuals {
                debug!("Adding front #{} (size: {})", fi + 1, front.len());
                new_population.add_new_individuals(front);
            } else if new_population.len() == self.number_of_individuals {
                debug!("Population reached target size");
                break;
            } else {
                debug!(
                    "Population almost full ({} individuals)",
                    new_population.len()
                );
                last_front = Some(front.clone());
                break;
            }
        }

        // Complete the population with the last front
        if let Some(mut last_front) = last_front {
            self.crowding_operator
                .set_crowding_distance(&mut last_front)?;
            let survivors = self.survival_operator.select_survivors(
                last_front,
                new_population.len(),
                self.number_of_individuals,
            )?;
            new_population.add_new_individuals(survivors);
        }

        // update the population and the distance for the CrowdedComparison operator at the next
        // loop
        self.population = new_population;
        self.crowding_operator
            .set_crowding_distance(self.population.individuals_as_mut())?;

        self.generation += 1;
        Ok(())
    }
}
