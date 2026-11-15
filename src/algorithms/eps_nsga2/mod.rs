use crate::algorithms::{
    Algorithm, EvolveStatus, ExportHistory, NSGA2Arg, StoppingCondition, NSGA2,
};
use crate::core::{DataValue, Individual, OError};
use crate::utils::{
    euclidean_distance, get_pareto_constrained_dominance,
    PreferredSolution,
};
use data::has_archive_stalled;
pub use data::{
    EpsNSGA2Arg, InterRunStoppingCondition, IntraRunData, IntraRunStoppingCondition,
    DISTANCE_FROM_BOX_CORNER, IDENTIFICATION_ARRAY_KEY, SCALED_OBJECTIVE_KEY,
};
use crate::core::utils::get_rng;
use log::{debug, info, LevelFilter};
use rand::RngCore;
use nsga_rs_macros::{algorithm, algorithm_trait_items};
use rayon::ThreadPool;
use std::fmt::{Display, Formatter};
use std::mem;

mod data;

/// The ϵ-Domination NSGA2 (Eps-NSGA2) algorithm using population doubling with 25%
/// injection rate and adaptive population sizing scheme. [`Self::population`] contains
/// the archived individuals selected with the ϵ-dominance approach.
///
/// # References
/// The algorithm was implemented from:
///
/// > Joshua B. Kollat and Patrick M. Reed. 2005. The value of online adaptive search:
/// > a performance comparison of NSGAII, ε-NSGAII and εMOEA. In Proceedings of the Third
/// > international conference on Evolutionary Multi-Criterion Optimization (EMO'05).
/// > Springer-Verlag, Berlin, Heidelberg, 386–398. <https://doi.org/10.1007/978-3-540-31880-4_27?
///
/// and:
///
/// > J.B. Kollat, P.M. Reed, Comparing state-of-the-art evolutionary multi-objective
/// > algorithms for long-term groundwater monitoring design, Advances in Water Resources,
/// > Volume 29, Issue 6, 2006, Pages 792-807, <https://doi.org/10.1016/j.advwatres.2005.07.010>
///
/// Dynamic population sizing was first presented for `NSGA2` in:
///
/// > Reed, P., B. S. Minsker, and D. E. Goldberg (2003), Simplifying multiobjective
/// > optimization: An automated design methodology for the nondominated sorted genetic
/// > algorithm-II, Water Resour. Res., 39, 1196, doi:10.1029/2002WR001483, 7.
#[algorithm(EpsNSGA2Arg)]
pub struct EpsNSGA2 {
    /// The stopping condition configuration for NSGA2 intra-runs.
    intra_run_stop_cond: IntraRunStoppingCondition,
    /// The hyper-box size.
    eps: Vec<f64>,
    /// The intra-run data used to check the stopping condition in NSGA2 runs.
    intra_run_data: IntraRunData,
    /// The last generation when the history data was exported.
    history_gen_step: usize,
    /// Copy of `history_export` to prevent `Algorithm` to export data as this is done
    /// by the `after_evolve` closure.
    _export_history: Option<ExportHistory>,
    /// The random number generator used to create new individuals.
    rng: Box<dyn RngCore>,
    /// Whether to enable the inter-run stopping condition.
    inter_run_stopping_condition: InterRunStoppingCondition,
}

impl EpsNSGA2 {
    /// Initialise the `EpsNSGA2` algorithm.
    ///
    /// # Arguments
    ///
    /// * `problem`: The problem being solved.
    /// * `args`: The [`EpsNSGA2Arg`] arguments to customise the algorithm behaviour.
    ///
    /// returns: `NSGA2`.
    pub fn new(problem: Arc<Problem>, options: EpsNSGA2Arg) -> Result<Self, OError> {
        let name = "EpsNSGA2".to_string();
        crate::algorithms::ensure_numeric_variables(&problem, &name)?;
        let number_of_initial_individuals = options.number_of_initial_individuals.unwrap_or(10);
        if number_of_initial_individuals < 5 {
            return Err(OError::AlgorithmInit(
                name,
                "The initial population size must have at least 5 individuals".to_string(),
            ));
        }
        if options.eps.len() != problem.number_of_objectives() {
            return Err(OError::Generic(format!(
                "The size of eps ({}) must equal the number of problem objectives ({})",
                options.eps.len(),
                problem.number_of_objectives()
            )));
        }
        let eps_nsga2_args = options.clone();

        // init population and individual's box data
        let mut rng = get_rng(options.seed);
        let population = if let Some(init_file) = options.resume_from_file {
            info!("Loading initial population from {:?}", init_file);
            let data = Self::read_json_file(&init_file)?;
            EpsNSGA2::seed_population_from_file(
                problem.clone(),
                &name,
                data.individuals.len(),
                &init_file,
            )?
        } else {
            info!("Created initial random population");
            Population::init(problem.clone(), number_of_initial_individuals, &mut rng)?
        };

        // Init other options
        let intra_run_stop_cond = options.intra_run_stop_cond.unwrap_or_default();
        let inter_run_stopping_condition = options.inter_run_stopping_condition.unwrap_or_default();

        // Info message
        let mut log_opts: String = "Algorithm options are:\n".to_owned();
        log_opts.push_str(
        format!("\t* Number of variables {:>13}\n\t* Number of objectives {:>12}\n\t* Number of constraints {:>11}\n", problem.number_of_variables(),
                    problem.number_of_objectives(),
                    problem.number_of_constraints()
            ).as_str());
        log_opts.push_str(
            format!(
                "\t* Initial population size {:>9}\n\t* Epsilon [{:>9}]\n",
                number_of_initial_individuals,
                options
                    .eps
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
        log_opts.push_str(
            format!(
                "\t* Initial lag window size: {:>9}\n\t* Next lag window size {:>13}\n\t* Archive size threshold {:>12}\n",
                intra_run_stop_cond.lag_window_first_generation,
                intra_run_stop_cond.lag_window_after,
                intra_run_stop_cond.threshold,
            )
            .as_str(),
        );
        log_opts.push_str(
        format!(
            "\t* Maximum generations of intra-NSGA2: {:>6}\n\t* Enable inter-run stop criterion: {:>9?}\n",
            intra_run_stop_cond.max_intra_run_generations, matches!(inter_run_stopping_condition, InterRunStoppingCondition::Threshold(_))
        ).as_str());
        info!("{}", log_opts);

        Ok(EpsNSGA2 {
            problem,
            eps: options.eps,
            number_of_individuals: number_of_initial_individuals,
            intra_run_stop_cond,
            // this is the eps archive which is initialised with `first_nsga2_population`
            // and updated in evolve()
            population,
            intra_run_data: IntraRunData::default(),
            generation: 0,
            history_gen_step: 0,
            nfe: 0,
            inter_run_stopping_condition,
            stopping_condition: options.stopping_condition,
            start_time: Instant::now(),
            thread_pool: Self::build_thread_pool(options.threads)?,
            // Prevent `Algorithm` to export data as this is done by the `after_evolve`
            // closure via `_export_history`.
            export_history: None,
            _export_history: options.export_history,
            rng,
            args: eps_nsga2_args,
        })
    }

    /// Set the NSGA2 to an even number as this is a NSGA2 requirement. This adds a new
    /// individual.
    fn adjust_nsga2_population(&mut self, population: &mut Population) {
        if population.len() % 2 != 0 {
            population.add_individual(Individual::new_random(self.problem.clone(), &mut self.rng));
        }
    }

    /// Calculate the identification array `B` as `floor [ f_j / eps_j ]` where `f_j`
    /// is then j-th objective and `e_j` is the objective tolerance below which two
    /// difference between the values are not important to the user.
    /// This function stores the array in the [`Individual`]' data field ([`IDENTIFICATION_ARRAY_KEY`]).
    /// The vector provides the coordinates of the hyper-box.
    ///
    /// This function also stores the objective in the hyper-box coordinate system
    /// ([`SCALED_OBJECTIVE_KEY`] and its distance from the box left-bottom corner
    /// ([`DISTANCE_FROM_BOX_CORNER`]).
    ///
    /// # Notes
    /// The paper uses `ceiling` when `f_j` is maximised, but [`Individual`] stores a
    /// maximised objective with the opposite sign, therefore all the objectives are
    /// minimised and `floor` is used.
    ///
    /// # Arguments
    /// * `individual`: The individual to update.
    /// * `eps`: The hyper-box size.
    ///
    /// returns: `Result<(), OError>`
    fn set_box_data(individual: &mut Individual, eps: &[f64]) -> Result<(), OError> {
        // skip if individuals has the data already
        if individual.get_data(IDENTIFICATION_ARRAY_KEY).is_ok() {
            return Ok(());
        }

        let mut b_vector = vec![];
        let mut obj_box_coords = vec![];
        for (j, f_j) in individual.get_objective_values()?.iter().enumerate() {
            if f_j.is_nan() {
                return Err(OError::Generic(format!(
                    "The individual has no objective value set for objective #{j}"
                )));
            }
            // Scale the objectives using the grid size to calculate the objective in box
            // coordinates.
            let obj_box_coord = f_j / eps[j];
            obj_box_coords.push(obj_box_coord);

            b_vector.push(f64::floor(obj_box_coord));
        }
        let distance_from_corner = euclidean_distance(&b_vector, &obj_box_coords);

        individual.set_data(IDENTIFICATION_ARRAY_KEY, DataValue::Vector(b_vector));
        individual.set_data(SCALED_OBJECTIVE_KEY, DataValue::Vector(obj_box_coords));
        individual.set_data(
            DISTANCE_FROM_BOX_CORNER,
            DataValue::Real(distance_from_corner),
        );

        Ok(())
    }

    /// Update [`self.population`] (i.e. the archive) with new offsprings. Each offspring
    /// is compared with all the archived solutions (see Fig. 6 in Deb et al., 2005):
    /// - if it is ε-dominated by an archived solution, it is rejected.
    /// - if it ε-dominates one or more archived solutions, these are removed and the
    ///   offspring is accepted.
    /// - if it lies in the same hyper-box as an archived solution, the closest solution
    ///   to the box corner is kept.
    /// - otherwise the offspring lies in a new ε-non-dominated box and is accepted.
    ///
    /// # Arguments
    /// * `offsprings`: The vector of [`Individual`]s to add.
    ///
    /// # Notes
    /// This assumes that the hyper-box data is already available in `offsprings`.
    ///
    /// returns: `Result<(), OError>`
    pub fn update_archive(&mut self, offsprings: Vec<Individual>) -> Result<(), OError> {
        let key = Some(IDENTIFICATION_ARRAY_KEY.to_string());
        let mut archive = mem::take(&mut self.population).individuals().to_vec();
        'offspring: for offspring in offsprings {
            let mut keep = vec![true; archive.len()];
            for (i, member) in archive.iter().enumerate() {
                match get_pareto_constrained_dominance(&offspring, member, key.clone())? {
                    PreferredSolution::First => keep[i] = false,
                    PreferredSolution::Second => continue 'offspring,
                    PreferredSolution::MutuallyPreferred => {
                        if offspring.get_data(IDENTIFICATION_ARRAY_KEY)?
                            != member.get_data(IDENTIFICATION_ARRAY_KEY)?
                        {
                            continue;
                        }
                        // same hyper-box
                        if offspring.get_data(DISTANCE_FROM_BOX_CORNER)?.as_real()?
                            < member.get_data(DISTANCE_FROM_BOX_CORNER)?.as_real()?
                        {
                            keep[i] = false;
                        } else {
                            continue 'offspring;
                        }
                    }
                }
            }
            let mut keep = keep.into_iter();
            archive.retain(|_| keep.next().unwrap());
            archive.push(offspring);
        }

        self.population = Population::new_with(archive);
        Ok(())
    }

    /// Get the archive.
    fn archive(&self) -> &Population {
        &self.population
    }
}

#[algorithm_trait_items(EpsNSGA2Arg)]
impl Algorithm<EpsNSGA2Arg> for EpsNSGA2 {
    fn initialise(&mut self) -> Result<(), OError> {
        // the first population is already initialised, but lacks the objective and
        // hyper-box values
        info!("Evaluating initial population");
        NSGA2::do_evaluation(
            self.population.individuals_as_mut(),
            &mut self.nfe,
            &self.thread_pool,
        )?;
        debug!("Calculating rank and crowding distance");
        self.population = NSGA2::rank_and_crowd(&mut self.population)?;

        // assign the hyper-box data after objective evaluation
        if self.generation() == 0 {
            for individual in self.population.individuals_as_mut() {
                Self::set_box_data(individual, &self.eps)?;
            }
        }

        info!("Initial evaluation completed");

        Ok(())
    }

    fn evolve(&mut self) -> Result<EvolveStatus, OError> {
        let is_first_run = self.generation() == 0;
        let mut next_nsga2_population = if is_first_run {
            // the first NSGA2 run uses the initial population, whose ε-non-dominated
            // solutions form the first archive
            let population = mem::take(&mut self.population);
            self.update_archive(population.individuals().to_vec())?;
            population
        } else {
            // set the new population using the 25%-injection scheme - this also ensures that
            // the population is made of at least 10 individuals, if the archive size is <= 2.
            let next_population_size = usize::max(10, 4 * self.archive().len());
            let mut population = Population::new_with(self.archive().individuals().to_vec());
            while population.len() < next_population_size {
                population.add_individual(Individual::new_random(
                    self.problem.clone(),
                    &mut self.rng,
                ));
            }
            population
        };
        self.adjust_nsga2_population(&mut next_nsga2_population);

        info!(
            "Starting NSGA intra-runs with {} individuals",
            next_nsga2_population.len()
        );

        // data to check the stopping conditions - A in Eq. 1 of the paper is the archive
        // size at the end of the previous run
        let prev_run_archive_size = self.archive().len();
        self.intra_run_data = IntraRunData::new(prev_run_archive_size);
        let nfe_at_start = self.nfe;
        let mut archive_stalled_size = None;

        // the log level is global, so logs from other threads are muted during the intra-run too
        let prev_log_level = log::max_level();
        log::set_max_level(LevelFilter::Off);
        // use closure to drop borrowed `self` by `NSGA2` and restore the log level on error
        let run_result = (|| -> Result<(), OError> {
            let args = NSGA2Arg {
                number_of_individuals: next_nsga2_population.len(),
                crossover_operator_options: self.algorithm_options().crossover_operator_options,
                mutation_operator_options: self.algorithm_options().mutation_operator_options,
                stopping_condition: StoppingCondition::MaxGeneration(
                    self.intra_run_stop_cond.max_intra_run_generations,
                ),
                resume_from_file: None,
                // do not replay the same random numbers at each run
                seed: self
                    .algorithm_options()
                    .seed
                    .map(|s| s + self.generation as u64),
                threads: self.algorithm_options().threads,
                export_history: None,
            };
            let mut intra_run_algo = NSGA2::new(self.problem.clone(), args)?;
            intra_run_algo.set_population(next_nsga2_population);

            // add the intra-run stopping condition based on the number of new dominated
            // solutions in the archive
            intra_run_algo.set_after_evolve(Box::new(|nsga2: &mut NSGA2| {
                // update overall progress
                self.generation += 1;
                self.nfe = nfe_at_start + nsga2.number_of_function_evaluations();
                debug!("NSGA2 generation #{} completed", self.generation());

                // set box data on new individuals created by NSGA2 (this is the whole
                // population at the first generation). Box data is carried through NSGA2
                // runs and archive
                debug!("Setting hyper-box data for new individuals");
                for individual in nsga2.population.individuals_as_mut() {
                    Self::set_box_data(individual, &self.eps)?;
                }

                // update the archive/population with the new solutions
                debug!("Updating archive");
                let nsga2_last_population = (*nsga2.population().individuals()).to_vec();
                self.update_archive(nsga2_last_population)?;
                self.number_of_individuals = self.archive().len();

                // export data
                if let Some(export) = &self._export_history {
                    if self.history_gen_step + 1 >= export.generation_step {
                        self.save_to_json(&export.destination, None)?;
                        self.history_gen_step = 0;
                    } else {
                        self.history_gen_step += 1;
                    }
                }

                // the algorithm stopping condition also applies within a run
                if self.is_stopping_condition_met(&self.stopping_condition)? {
                    return Ok(EvolveStatus::Stop);
                }

                let archive_size = self.archive().len();
                if self.intra_run_data.should_stop(
                    &self.intra_run_stop_cond,
                    is_first_run,
                    nsga2.generation(),
                    archive_size,
                ) {
                    archive_stalled_size = Some(archive_size);
                    return Ok(EvolveStatus::Stop);
                }
                Ok(EvolveStatus::Continue)
            }));

            debug!("Running NSGA2");
            intra_run_algo.run()
        })();
        log::set_max_level(prev_log_level);
        run_result?;

        info!("NSGA2 intra-run ended at generation #{}", self.generation());
        if let Some(archive_size) = archive_stalled_size {
            info!("Archive has stopped improving above threshold (size is {archive_size})");
        }

        if self.is_stopping_condition_met(&self.stopping_condition)? {
            return Ok(EvolveStatus::Stop);
        }

        // check inter-run stopping condition - Eq. 1 from the paper
        if let InterRunStoppingCondition::Threshold(threshold) = self.inter_run_stopping_condition {
            if has_archive_stalled(prev_run_archive_size, self.archive().len(), threshold) {
                info!(
                    "Stopping algorithm as the archive has not improved above the threshold. \
                    Previous inter-run archive size was {}. Last archive size is {}",
                    prev_run_archive_size,
                    self.archive().len(),
                );
                return Ok(EvolveStatus::Stop);
            }
        }

        Ok(EvolveStatus::Continue)
    }
}

/// Test the function handling the eps archive update.
#[cfg(test)]
mod test_eps_archive {

    use std::sync::Arc;

    use crate::{
        algorithms::{
            EpsNSGA2, EpsNSGA2Arg, NumThreads, StoppingCondition, DISTANCE_FROM_BOX_CORNER,
            IDENTIFICATION_ARRAY_KEY, SCALED_OBJECTIVE_KEY,
        },
        core::{
            utils::dummy_evaluator, BoundedNumber, Individual, OError, Objective,
            ObjectiveDirection, Population, Problem, VariableType,
        },
        utils::euclidean_distance,
    };

    fn pareto_points() -> Vec<(f64, f64)> {
        vec![
            (0.83, 9.74),
            (1.41, 8.95),
            (2.17, 8.38),
            (2.96, 7.61),
            (3.84, 6.92),
            (4.71, 6.4),
            (5.63, 5.41),
            (6.89, 4.27),
            (8.02, 3.46),
            (9.58, 2.11),
        ]
    }

    fn test_data() -> Result<EpsNSGA2, OError> {
        let objectives = vec![
            Objective::new("obj1", ObjectiveDirection::Minimise),
            Objective::new("obj2", ObjectiveDirection::Minimise),
        ];
        let var_types = vec![VariableType::Real(
            BoundedNumber::new("X1", 0.0, 2.0).unwrap(),
        )];
        let problem = Arc::new(Problem::new(
            objectives,
            var_types,
            None,
            dummy_evaluator(),
        )?);
        let eps = vec![0.5, 0.5];

        // initialise custom population
        let mut individuals = vec![];
        for point in pareto_points() {
            let mut solution = Individual::new(problem.clone());
            solution.update_objective("obj1", point.0)?;
            solution.update_objective("obj2", point.1)?;
            EpsNSGA2::set_box_data(&mut solution, &eps)?;
            individuals.push(solution);
        }

        let options = EpsNSGA2Arg {
            eps: vec![0.5, 0.5],
            number_of_initial_individuals: None,
            crossover_operator_options: None,
            mutation_operator_options: None,
            stopping_condition: StoppingCondition::MaxGeneration(1000),
            resume_from_file: None,
            seed: None,
            threads: NumThreads::Max,
            export_history: None,
            intra_run_stop_cond: None,
            inter_run_stopping_condition: None,
        };
        let mut algo = EpsNSGA2::new(problem, options)?;
        algo.population = Population::new_with(individuals);

        Ok(algo)
    }

    #[test]
    /// Test that the hyper-box coordinates are properly calculated.
    fn test_box_coordinates() -> Result<(), OError> {
        // this map each point to the manually-calculated box coordinates (assuming
        // miniminsation).
        let coords: Vec<(f64, f64)> = vec![
            (1.0, 19.0),
            (2.0, 17.0),
            (4.0, 16.0),
            (5.0, 15.0),
            (7.0, 13.0),
            (9.0, 12.0),
            (11.0, 10.0),
            (13.0, 8.0),
            (16.0, 6.0),
            (19.0, 4.0),
        ];
        let algo = test_data()?;
        for (individual, expected_point) in algo.population.individuals().iter().zip(coords) {
            assert_eq!(
                vec![expected_point.0, expected_point.1],
                *individual
                    .get_data(IDENTIFICATION_ARRAY_KEY)?
                    .as_f64_vec()?
            );
        }
        Ok(())
    }

    #[test]
    /// Test when an offspring is in the same box with another solution, and it is rejected
    /// because it is further from the box corner.
    fn test_offspring_same_box_but_further() -> Result<(), OError> {
        let mut algo = test_data()?;

        // solution is in the same box as point #9 (19, 4) - distance is 0.91, but archived
        // individual is 0.27
        let mut offspring = Individual::new(algo.problem.clone());
        let new_objectives = (9.885, 2.24);
        offspring.update_objective("obj1", new_objectives.0)?;
        offspring.update_objective("obj2", new_objectives.1)?;
        EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;

        assert_eq!(
            *offspring.get_data(IDENTIFICATION_ARRAY_KEY)?.as_f64_vec()?,
            vec![19.0, 4.0]
        );
        // offspring is rejected
        algo.update_archive(vec![offspring])?;

        assert!(algo
            .population
            .individuals()
            .iter()
            .find(|i| i.get_objective_values().unwrap() == vec![new_objectives.0, new_objectives.1])
            .is_none());
        Ok(())
    }

    #[test]
    /// Test when an offspring is in a new box and it is accepted because it is not
    /// dominated. Also test a new non-dominated solution in the same box of existing
    /// archived individual.
    fn test_offspring_new_box() -> Result<(), OError> {
        let mut algo = test_data()?;

        // new solution in box (14, 7), which is non dominated
        let mut offspring = Individual::new(algo.problem.clone());
        let new_objectives = (7.335, 3.795);
        offspring.update_objective("obj1", new_objectives.0)?;
        offspring.update_objective("obj2", new_objectives.1)?;
        EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;

        // offspring is accepted
        algo.update_archive(vec![offspring])?;
        assert_eq!(algo.population.len(), 11);

        // new solution in box (11, 8), dominates (11, 10) / (5.63, 5.41)
        let mut offspring = Individual::new(algo.problem.clone());
        let new_objectives = (5.785, 4.6);
        offspring.update_objective("obj1", new_objectives.0)?;
        offspring.update_objective("obj2", new_objectives.1)?;
        EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;
        // offspring is accepted
        algo.update_archive(vec![offspring])?;
        assert_eq!(algo.population.len(), 11);
        assert!(algo
            .population
            .individuals()
            .iter()
            .find(|i| i.get_objective_values().unwrap() == vec![new_objectives.0, new_objectives.1])
            .is_some());
        assert!(algo
            .population
            .individuals()
            .iter()
            .find(|i| i.get_objective_values().unwrap() == vec![5.63, 5.41])
            .is_none());
        Ok(())
    }

    #[test]
    /// Test when an offspring is in the same box with another solution, and it is accepted
    /// because it is closer to the box corner.
    fn test_offspring_same_box_but_closer() -> Result<(), OError> {
        let mut algo = test_data()?;
        let mut offspring = Individual::new(algo.problem.clone());
        let to_compare = algo.population.individuals().last().unwrap();

        // solution is in the same box as point #9 (19, 4). It is worst in the first objective,
        // but better in the second, therefore solutions are mutually preferred
        let new_objectives = (9.6, 2.0);
        offspring.update_objective("obj1", new_objectives.0)?;
        offspring.update_objective("obj2", new_objectives.1)?;
        EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;

        assert_eq!(
            *offspring.get_data(IDENTIFICATION_ARRAY_KEY)?.as_f64_vec()?,
            vec![19.0, 4.0]
        );

        // check distances
        let archived_distance = euclidean_distance(
            &offspring
                .get_data(IDENTIFICATION_ARRAY_KEY)?
                .as_f64_vec()?
                .iter()
                .map(|v| *v as f64)
                .collect::<Vec<f64>>(),
            to_compare.get_data(SCALED_OBJECTIVE_KEY)?.as_f64_vec()?,
        );
        let offspring_distance = euclidean_distance(
            &offspring
                .get_data(IDENTIFICATION_ARRAY_KEY)?
                .as_f64_vec()?
                .iter()
                .map(|v| *v as f64)
                .collect::<Vec<f64>>(),
            offspring.get_data(SCALED_OBJECTIVE_KEY)?.as_f64_vec()?,
        );
        assert!(archived_distance > offspring_distance);

        // offspring is accepted
        algo.update_archive(vec![offspring])?;
        assert!(algo
            .population
            .individuals()
            .iter()
            .find(|i| i.get_objective_values().unwrap() == vec![new_objectives.0, new_objectives.1])
            .is_some());
        assert!(algo
            .population
            .individuals()
            .iter()
            .find(|i| i.get_objective_values().unwrap() == vec![9.58, 2.11])
            .is_none());
        Ok(())
    }

    #[test]
    /// Test that only the offspring closest to the corner is kept, when two offsprings
    /// are in the same new box.
    fn test_offsprings_same_new_box() -> Result<(), OError> {
        let mut algo = test_data()?;

        // both solutions are in the new non-dominated box (14, 7)
        let mut offsprings = vec![];
        for objectives in [(7.4, 3.9), (7.335, 3.795)] {
            let mut offspring = Individual::new(algo.problem.clone());
            offspring.update_objective("obj1", objectives.0)?;
            offspring.update_objective("obj2", objectives.1)?;
            EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;
            assert_eq!(
                *offspring.get_data(IDENTIFICATION_ARRAY_KEY)?.as_f64_vec()?,
                vec![14.0, 7.0]
            );
            offsprings.push(offspring);
        }

        // the second offspring replaces the first
        algo.update_archive(offsprings)?;
        assert_eq!(algo.population.len(), 11);
        let objectives: Vec<Vec<f64>> = algo
            .population
            .individuals()
            .iter()
            .map(|i| i.get_objective_values().unwrap())
            .collect();
        assert!(objectives.contains(&vec![7.335, 3.795]));
        assert!(!objectives.contains(&vec![7.4, 3.9]));
        Ok(())
    }

    #[test]
    /// Test that the archive can hold one solution.
    fn test_one_individual() -> Result<(), OError> {
        let mut algo = test_data()?;
        let archived = algo.population.individuals()[0].clone();
        algo.population = Population::new_with(vec![archived]);

        // solution is in the same box (1, 19), but it is further from the corner
        let mut offspring = Individual::new(algo.problem.clone());
        offspring.update_objective("obj1", 0.9)?;
        offspring.update_objective("obj2", 9.8)?;
        EpsNSGA2::set_box_data(&mut offspring, &algo.eps)?;

        algo.update_archive(vec![offspring])?;
        assert_eq!(algo.population.len(), 1);
        assert_eq!(
            algo.population.individuals()[0].get_objective_values()?,
            vec![0.83, 9.74]
        );
        Ok(())
    }

    #[test]
    /// Test the box of a maximised objective, whose value is stored with the opposite
    /// sign. The box corner must be the best corner.
    fn test_maximised_objective_box() -> Result<(), OError> {
        let objectives = vec![
            Objective::new("obj1", ObjectiveDirection::Maximise),
            Objective::new("obj2", ObjectiveDirection::Minimise),
        ];
        let var_types = vec![VariableType::Real(
            BoundedNumber::new("X1", 0.0, 2.0).unwrap(),
        )];
        let problem = Arc::new(Problem::new(
            objectives,
            var_types,
            None,
            dummy_evaluator(),
        )?);

        let mut individual = Individual::new(problem.clone());
        individual.update_objective("obj1", 1.3)?;
        individual.update_objective("obj2", 1.3)?;
        EpsNSGA2::set_box_data(&mut individual, &[0.5, 0.5])?;
        // obj1 is in [1; 1.5] and 1.5 is the best value
        assert_eq!(
            *individual
                .get_data(IDENTIFICATION_ARRAY_KEY)?
                .as_f64_vec()?,
            vec![-3.0, 2.0]
        );

        // a larger obj1 in the same box is closer to the corner
        let mut better = Individual::new(problem);
        better.update_objective("obj1", 1.4)?;
        better.update_objective("obj2", 1.3)?;
        EpsNSGA2::set_box_data(&mut better, &[0.5, 0.5])?;
        assert!(
            better.get_data(DISTANCE_FROM_BOX_CORNER)?.as_real()?
                < individual.get_data(DISTANCE_FROM_BOX_CORNER)?.as_real()?
        );
        Ok(())
    }
}

/// Test the algorithm on the problems from Deb et al. (2002).
#[cfg(test)]
mod test_problems {
    use nsga_rs_macros::test_with_retries;
    use std::sync::Arc;

    use crate::algorithms::{
        Algorithm, EpsNSGA2, EpsNSGA2Arg, InterRunStoppingCondition, NumThreads, StoppingCondition,
    };
    use crate::core::builtin_problems::{
        SCHProblem, ZTD1Problem, ZTD2Problem, ZTD3Problem, ZTD4Problem,
    };
    use crate::core::test_utils::{check_exact_value, check_value_in_range};
    use crate::core::Problem;

    const BOUND_TOL: f64 = 1.0 / 1000.0;
    const LOOSE_BOUND_TOL: f64 = 0.1;

    /// Get the algorithm options. The inter-run stopping condition is disabled to
    /// stop at the maximum generation.
    fn args(eps: f64, stopping_condition: StoppingCondition) -> EpsNSGA2Arg {
        EpsNSGA2Arg {
            eps: vec![eps, eps],
            number_of_initial_individuals: None,
            intra_run_stop_cond: None,
            inter_run_stopping_condition: Some(InterRunStoppingCondition::Off()),
            stopping_condition,
            crossover_operator_options: None,
            mutation_operator_options: None,
            threads: NumThreads::Off,
            export_history: None,
            resume_from_file: None,
            seed: Some(1),
        }
    }

    /// Solve a ZTD problem and check the solutions: x1 in [0; 1] and the other variables
    /// equal to 0. The exact solutions are tested using a strict and loose bounds.
    fn check_ztd_problem(
        problem: Problem,
        number_of_vars: usize,
        strict_tol: f64,
        max_outside_strict_range: usize,
    ) {
        let args = args(0.05, StoppingCondition::MaxGeneration(2500));
        let mut algo = EpsNSGA2::new(Arc::new(problem), args).unwrap();
        algo.run().unwrap();
        assert_eq!(algo.generation(), 2500);
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

        let x_bounds = -strict_tol..strict_tol;
        let x_other_bounds = -LOOSE_BOUND_TOL..LOOSE_BOUND_TOL;
        for xi in 2..=number_of_vars {
            let var_name = format!("x{xi}");
            let var_values = results.get_real_variables(&var_name).unwrap();

            let (x_other_outside_bounds, breached_range, b_type) = check_exact_value(
                &var_values,
                &x_bounds,
                &x_other_bounds,
                max_outside_strict_range,
            );
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
    /// Test problem 1 from Deb et al. (2002). Optional solution x in [0; 2]
    fn test_sch_problem() {
        let problem = SCHProblem::create().unwrap();
        let args = args(0.1, StoppingCondition::MaxGeneration(1000));
        let mut algo = EpsNSGA2::new(Arc::new(problem), args).unwrap();
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
    /// Test that the algorithm stops when the archive does not improve between two
    /// NSGA2 runs.
    fn test_sch_problem_inter_run_stop() {
        let problem = SCHProblem::create().unwrap();
        let mut args = args(0.1, StoppingCondition::MaxGeneration(100000));
        args.inter_run_stopping_condition = Some(InterRunStoppingCondition::Threshold(0.1));
        let mut algo = EpsNSGA2::new(Arc::new(problem), args).unwrap();
        algo.run().unwrap();
        assert!(algo.generation() < 100000);
        let results = algo.get_results();

        let bounds = -0.1..2.1;
        let invalid_x = check_value_in_range(&results.get_real_variables("x").unwrap(), &bounds);
        if !invalid_x.is_empty() {
            panic!("Some variables are outside the bounds: {:?}", invalid_x);
        }
    }

    #[test]
    /// Test that the algorithm stopping condition is checked within a NSGA2 run and
    /// that the function evaluations are not counted more than once.
    fn test_max_nfe() {
        let problem = SCHProblem::create().unwrap();
        let args = args(0.1, StoppingCondition::MaxFunctionEvaluations(500));
        let mut algo = EpsNSGA2::new(Arc::new(problem), args).unwrap();
        algo.run().unwrap();

        // the first run has 10 individuals: 10 evaluations for the initial population
        // and 10 for each generation
        assert_eq!(algo.number_of_function_evaluations(), 500);
        assert_eq!(algo.generation(), 49);
    }

    #[test]
    /// Test that the maximum generation is checked within a NSGA2 run.
    fn test_max_generation() {
        let problem = SCHProblem::create().unwrap();
        let args = args(0.1, StoppingCondition::MaxGeneration(20));
        let mut algo = EpsNSGA2::new(Arc::new(problem), args).unwrap();
        algo.run().unwrap();
        assert_eq!(algo.generation(), 20);
    }

    #[test_with_retries(10)]
    /// Test the ZTD1 problem from Deb et al. (2002) with 30 variables.
    fn test_ztd1_problem() {
        check_ztd_problem(ZTD1Problem::create(30).unwrap(), 30, BOUND_TOL, 5);
    }

    #[test_with_retries(10)]
    /// Test the ZTD2 problem from Deb et al. (2002) with 30 variables.
    fn test_ztd2_problem() {
        check_ztd_problem(ZTD2Problem::create(30).unwrap(), 30, BOUND_TOL, 3);
    }

    #[test_with_retries(10)]
    /// Test the ZTD3 problem from Deb et al. (2002) with 30 variables.
    fn test_ztd3_problem() {
        check_ztd_problem(ZTD3Problem::create(30).unwrap(), 30, BOUND_TOL, 3);
    }

    #[test_with_retries(10)]
    /// Test the ZTD4 problem from Deb et al. (2002) with 10 variables. The strict
    /// bounds are relaxed as in the NSGA2 test.
    fn test_ztd4_problem() {
        check_ztd_problem(ZTD4Problem::create(10).unwrap(), 10, BOUND_TOL * 10.0, 3);
    }
}
