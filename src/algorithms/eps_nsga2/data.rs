use crate::algorithms::NumThreads;
use crate::core::OError;
use crate::operators::{PolynomialMutationArgs, SimulatedBinaryCrossoverArgs};

use std::path::PathBuf;

#[cfg(feature = "python")]
use crate::algorithms::PyStoppingConditionValue;
use nsga_rs_macros::algorithm_args;
#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const IDENTIFICATION_ARRAY_KEY: &str = "identification_array";
pub const SCALED_OBJECTIVE_KEY: &str = "scaled_objectives_to_hyperbox_coordinate_system";
pub const DISTANCE_FROM_BOX_CORNER: &str = "distance_from_box_corner";

/// The stopping condition configuration for NSGA2 intra-runs. A intra-run stops when the
/// archive is not improving above `threshold` after a lag window which depends on the
/// generation number. When the following condition is met:
///
/// >`ABS( K - A ) / A <= threshold `
///
/// the intra-run is stopped the population size is adjusted, reseeded from the Eps
/// archive, and a new NSGA2 run is started.
///
/// `A` is the number of ε-non-dominated individuals in the archive derived when the lag
/// window is reached and `K` the archive size at the current generation minus the lag
/// window.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "python", pyclass(get_all, from_py_object))]
pub struct IntraRunStoppingCondition {
    /// The lag window to use for the first intra-run generation.
    pub(crate) lag_window_first_generation: u32,
    /// The lag size to use after the first intra-run generation. After the first generation
    /// the archive is expected to grow faster
    pub(crate) lag_window_after: u32,
    /// The threshold below which the intra-run stops.
    pub(crate) threshold: f64,
    /// The maximum intra-run generations when to start a new NSGA2 run regardless of the
    /// archive gains.
    pub(crate) max_intra_run_generations: u32,
}

impl IntraRunStoppingCondition {
    /// Initialise the intra-run stopping conditions.
    ///
    /// # Arguments
    /// * `lag_window_first_generation`: The lag window to use for the first intra-run generation.
    /// * `lag_window_after`: The lag size to use after the first intra-run generation. After the
    /// first generation the archive is expected to grow faster.
    /// * `threshold`: The threshold below which the intra-run stops.
    /// * `max_intra_run_generations`: The maximum intra-run generations when to start
    /// a new NSGA2 run regardless of the archive gains.
    ///
    /// returns: `Result<MinLagWindow, OError>`
    pub fn new(
        lag_window_first_generation: u32,
        lag_window_after: u32,
        threshold: f64,
        max_intra_run_generations: u32,
    ) -> Result<Self, OError> {
        if lag_window_first_generation == 0 || lag_window_after == 0 {
            return Err(OError::Generic(
                "first_generation and after must be larger than 0".to_string(),
            ));
        }
        Ok(IntraRunStoppingCondition {
            lag_window_first_generation,
            lag_window_after,
            threshold,
            max_intra_run_generations,
        })
    }

    /// Get the lag window. Kollat & Reed (2005) use a longer window for the first run
    /// as the archive grows more slowly from the small initial population.
    ///
    /// # Arguments
    /// * `is_first_run`: Whether this is the first NSGA2 run.
    ///
    /// returns `u32`
    pub fn get_lag_window(&self, is_first_run: bool) -> u32 {
        if is_first_run {
            self.lag_window_first_generation
        } else {
            self.lag_window_after
        }
    }
}

impl Default for IntraRunStoppingCondition {
    fn default() -> Self {
        Self {
            lag_window_first_generation: 50,
            lag_window_after: 10,
            threshold: 0.1,
            max_intra_run_generations: 250,
        }
    }
}

/// The optional stopping condition for the inter-runs. This checks whether the archive
/// stopped improving after the NSGA2 intra-runs and is off by default. When a threshold
/// is given, the algorithm stops when the archive size changes between two intra-runs by
/// no more than the given threshold (Eq. 1 in Kollat & Reed (2005), which uses 0.1 for 10%). When on this may lead to an early termination with
/// a less-than desirable number of solutions.
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "python", pyclass(get_all, from_py_object))]
pub enum InterRunStoppingCondition {
    Threshold(f64),
    Off(),
}

impl Default for InterRunStoppingCondition {
    fn default() -> Self {
        Self::Off()
    }
}

/// Input arguments for the Eps-NSGA2 algorithm.
#[algorithm_args]
pub struct EpsNSGA2Arg {
    /// The resolution to use for the Pareto front for all objectives. This corresponds to
    /// the size of the boxes on the objective space in which the algorithm will find one
    /// solution. Smaller ε values will result in a smaller grid size with more solutions
    /// on the front.
    pub eps: Vec<f64>,
    /// The number of individuals to use in the first population when the algorithm starts.
    /// The individual size will dynamically changes as the algorithm runs depending on the
    /// size of the Eps archive. Default to `10` as suggested in the paper.
    pub number_of_initial_individuals: Option<usize>,
    /// The stopping condition configuration for NSGA2 intra-runs. A intra-run stops when the
    /// archive is not improving above `threshold` after a lag window which depends on the
    /// generation number. When the following condition is met:
    /// >`ABS( K - A ) / A <= threshold `
    /// the intra-run is stopped the population size is adjusted, reseeded from the Eps
    /// archive, and a new NSGA2 run is started.
    /// `A` is the number of ε-non-dominated individuals in the archive derived when the lag
    /// window is reached and `K` the archive size at the current generation minus the lag
    /// window. Based on the paper suggestion, the lag window is set to `50` for the first
    /// generation and `10` to the following intra-runs. The threshold defaults to `0.1`.
    /// When the generation reaches `[IntraRunStoppingCondition::max_intra_run_generations`]
    /// the intra-run stops regardless on the archive gains. Default to `250`.
    pub intra_run_stop_cond: Option<IntraRunStoppingCondition>,
    /// Enable the inter-run stop criterion. When `ABS(K-A)/A <= threshold`, where `A` is the
    /// number of ε-non-dominated individuals in the archive derived at the last NSGA2 run,
    /// and `K` a the previous intra-run. When `stopping_condition` is reached OR the
    /// archive does not gain any more individuals, the algorithm stops. This is enabled
    /// with a threshold of `0.1` as suggested by the paper.
    pub inter_run_stopping_condition: Option<InterRunStoppingCondition>,
}

#[cfg(feature = "python")]
#[pymethods]
impl EpsNSGA2Arg {
    #[new]
    #[pyo3(signature = ( eps, stopping_condition, number_of_initial_individuals=None, inter_run_stopping_condition=None, intra_run_stop_cond=None, crossover_operator_options=None, mutation_operator_options=None, resume_from_file=None, threads=None, export_history=None, seed=None))]
    fn py_new(
        eps: Vec<f64>,
        stopping_condition: PyStoppingConditionValue,
        number_of_initial_individuals: Option<usize>,
        inter_run_stopping_condition: Option<InterRunStoppingCondition>,
        intra_run_stop_cond: Option<IntraRunStoppingCondition>,
        crossover_operator_options: Option<SimulatedBinaryCrossoverArgs>,
        mutation_operator_options: Option<PolynomialMutationArgs>,
        resume_from_file: Option<PathBuf>,
        threads: Option<NumThreads>,
        export_history: Option<ExportHistory>,
        seed: Option<u64>,
    ) -> PyResult<Self> {
        let threads = threads.unwrap_or_default();
        Ok(EpsNSGA2Arg {
            number_of_initial_individuals,
            eps,
            intra_run_stop_cond,
            inter_run_stopping_condition,
            crossover_operator_options,
            mutation_operator_options,
            resume_from_file,
            seed,
            stopping_condition: stopping_condition.into(),
            threads,
            export_history,
        })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "EspNSGA2Arg(stopping_condition={}",
            self.stopping_condition,
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

/// Check whether the archive stopped improving. This implements Eq. 1 of Kollat & Reed
/// (2005): the search has stalled when the change in the archive size (`ABS(K - A) / A`)
/// is within `threshold`.
///
/// # Arguments
/// * `prev_size`: The archive size `A` at the previous check.
/// * `size`: The current archive size `K`.
/// * `threshold`: The threshold as a fraction (`0.1` is 10%).
///
/// returns: `bool`
pub(crate) fn has_archive_stalled(prev_size: usize, size: usize, threshold: f64) -> bool {
    // an empty archive cannot be used as reference; any solution is progress
    if prev_size == 0 {
        return size == 0;
    }
    let a = prev_size as f64;
    size.abs_diff(prev_size) as f64 / a <= threshold
}

/// The struct containing the data to handle the NSGA2 intra-run stopping condition.
/// Based on Kollat & Reed (2005), Section 2.1, Paragraph 4, the NSGA2 run with population
/// `N` is stopped when:
///  1) The search within `w` generations or lag windows does not lead to an increase
///  in the number of archived solutions.
///  2) A maximum duration is reached.
///
/// This struct only implements the first condition. The second condition is expressed
/// as number of maximum generations.
/// After this the population is re-seeded form the archive and the algorithm restarts.
#[derive(Clone, Default)]
pub struct IntraRunData {
    /// The last NSGA2 generation when the intra-run stopping condition was checked
    /// against the lag window.
    pub(crate) last_checked_generation: u32,
    /// The size of the archive when the stopping condition was last checked.
    pub(crate) prev_gen_archive_size: usize,
}

impl IntraRunData {
    /// Start tracking a new NSGA2 run.
    ///
    /// # Arguments
    /// * `archive_size`: The archive size when the run starts.
    pub(crate) fn new(archive_size: usize) -> Self {
        Self {
            last_checked_generation: 0,
            prev_gen_archive_size: archive_size,
        }
    }

    /// Check whether the NSGA2 run must stop. Call this once per NSGA2 generation.
    ///
    /// # Arguments
    /// * `cond`: The intra-run stopping condition.
    /// * `is_first_run`: Whether this is the first NSGA2 run.
    /// * `nsga2_generation`: The generation of the NSGA2 run.
    /// * `archive_size`: The current archive size.
    ///
    /// returns: `bool`
    pub(crate) fn should_stop(
        &mut self,
        cond: &IntraRunStoppingCondition,
        is_first_run: bool,
        nsga2_generation: u32,
        archive_size: usize,
    ) -> bool {
        if nsga2_generation - self.last_checked_generation < cond.get_lag_window(is_first_run) {
            return false;
        }
        let stalled = has_archive_stalled(self.prev_gen_archive_size, archive_size, cond.threshold);
        *self = Self {
            last_checked_generation: nsga2_generation,
            prev_gen_archive_size: archive_size,
        };
        stalled
    }
}

#[cfg(test)]
mod test_stopping_conditions {
    use super::{has_archive_stalled, IntraRunData, IntraRunStoppingCondition};

    #[test]
    fn test_condition_new() {
        // the paper defaults are valid
        assert!(IntraRunStoppingCondition::new(50, 10, 0.1, 250).is_ok());
        assert!(IntraRunStoppingCondition::new(0, 10, 0.1, 250).is_err());
        assert!(IntraRunStoppingCondition::new(50, 0, 0.1, 250).is_err());
    }

    #[test]
    fn test_has_archive_stalled() {
        // archive grew by 50%
        assert!(!has_archive_stalled(20, 30, 0.1));
        // archive grew by 5%
        assert!(has_archive_stalled(20, 21, 0.1));
        // 10% is not above the threshold (Eq. 1)
        assert!(has_archive_stalled(20, 22, 0.1));
        assert!(has_archive_stalled(20, 20, 0.1));
        // archive shrank
        assert!(!has_archive_stalled(20, 10, 0.1));
        assert!(has_archive_stalled(20, 19, 0.1));
        // empty reference
        assert!(!has_archive_stalled(0, 3, 0.1));
    }

    /// Feed `should_stop` with one archive size per generation, starting
    /// at `first_gen`, and return the generation when the run stops.
    fn stop_generation(
        data: &mut IntraRunData,
        is_first_run: bool,
        first_gen: u32,
        generations: &[usize],
    ) -> Option<u32> {
        let cond = IntraRunStoppingCondition::default();
        (first_gen..)
            .zip(generations)
            .find(|(gen, size)| data.should_stop(&cond, is_first_run, *gen, **size))
            .map(|(gen, _)| gen)
    }

    #[test]
    /// The first run is checked after 50 generations.
    fn test_lag_window_first_run() {
        let mut data = IntraRunData::new(10);
        assert_eq!(stop_generation(&mut data, true, 1, &[10; 49]), None);
        assert_eq!(stop_generation(&mut data, true, 50, &[10]), Some(50));
    }

    #[test]
    /// The following runs are checked every 10 generations.
    fn test_lag_window_next_runs() {
        let mut data = IntraRunData::new(10);
        assert_eq!(stop_generation(&mut data, false, 1, &[10; 9]), None);
        assert_eq!(stop_generation(&mut data, false, 10, &[10]), Some(10));
    }

    #[test]
    /// The run continues while the archive grows and the reference size is the size
    /// at the previous check.
    fn test_growing_archive() {
        let mut data = IntraRunData::new(10);
        // +50% at generation 10, +33% at generation 20 and +5% at generation 30
        let mut sizes = vec![15; 10];
        sizes.extend(vec![20; 10]);
        sizes.extend(vec![21; 10]);
        assert_eq!(stop_generation(&mut data, false, 1, &sizes), Some(30));
        assert_eq!(data.prev_gen_archive_size, 21);
        assert_eq!(data.last_checked_generation, 30);
    }
}
