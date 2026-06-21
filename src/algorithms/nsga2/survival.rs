use core::f64;

use serde::{Deserialize, Serialize};

use crate::algorithms::nsga2::CROWDING_DIST_KEY;
use crate::core::{Individual, OError};

/// The data for [`NSGA2Survival::Epsilon`].
#[derive(Serialize, Deserialize, Clone)]
pub struct EpsilonData {
    /// The epsilon parameter.
    pub epsilon: u64,
    /// The reference point coordinates.
    pub reference_points: Vec<Vec<f64>>,
    /// The objective weights.
    pub weights: Vec<f64>,
}

/// Enum to select the survival operator in [`CoreNSGA2`].
///
/// Each type selects the survivors from the last front. The last given front is obtained
/// after applying the fast non-dominated sorting. This front is usually the one that
/// cannot be entirely chosen to maintain the population size of the next generation
/// (i.e. `last_front.len() + population.len() > number_of_individuals`).
#[derive(Serialize, Deserialize, Clone)]
pub enum NSGA2Survival {
    /// Default survival operator implemented in the NSGA2 paper. This selects individuals
    /// with the largest crowding distance first to prevent crowding.
    LargestCrowdingDistanceFirst,
    /// This is the approach used in R-NSGA2 which relies on reference points and simply
    /// alters the survival approach with respect to NSGA2. This field needs the epsilon
    /// parameter, the reference point coordinates and the max and min. See paragraph
    /// IV of Deb et al. (2005).
    Epsilon(EpsilonData),
}

impl NSGA2Survival {
    /// This function receives `last_front` and select the individuals to keep to
    /// complete the population.
    ///
    /// # Arguments
    ///
    /// * `last_front`: The last front obtained after applying the fast non-dominated
    /// sorting.
    /// `new_population_length`. The current number of individuals in the new
    /// population.
    /// * `number_of_individuals`: The number of individuals to reach at the next
    /// generation.
    ///
    /// returns: `Vec<Individual>`. The survivors.
    pub fn select_survivors(
        &self,
        mut last_front: Vec<Individual>,
        new_population_length: usize,
        number_of_individuals: usize,
    ) -> Result<Vec<Individual>, OError> {
        match self {
            Self::LargestCrowdingDistanceFirst => {
                // Sort in descending order. Prioritise individuals with the largest
                // distance to prevent crowding
                last_front.sort_by(|i, o| {
                    i.get_data(CROWDING_DIST_KEY)
                        .unwrap()
                        .as_real()
                        .unwrap()
                        .total_cmp(&o.get_data(CROWDING_DIST_KEY).unwrap().as_real().unwrap())
                });
                last_front.reverse();

                // add the items to complete the population
                last_front.truncate(number_of_individuals - new_population_length);
            }
            Self::Epsilon(data) => {
                todo!()
            }
        }

        // check the number of survivors
        let expected_number = number_of_individuals - new_population_length;
        if last_front.len() != expected_number {
            return Err(OError::AlgorithmRun(
                    "NSGA2Survival".to_string(),
                    format!("The survival algorithm returned a too little or many individuals ({}). Expected {}.", last_front.len(), expected_number),
                ));
        }

        Ok(last_front)
    }
}
