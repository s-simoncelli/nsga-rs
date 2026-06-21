use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    algorithms::nsga2::CROWDING_DIST_KEY,
    core::{DataValue, Individual, Individuals, IndividualsMut, OError},
    utils::{argsort, vector_max, vector_min, Sort},
};

/// The data for [`CrowdingDistanceOperator::Rank`].
#[derive(Serialize, Deserialize, Clone)]
pub struct DistanceData {
    /// The reference point coordinates.
    pub reference_points: Vec<Vec<f64>>,
    /// The objective weights.
    pub weights: Vec<f64>,
}

/// Enum to select the crowding distance operator to calculate crowding in
/// `NSGA2` and `R-NSGA2`.
#[derive(Serialize, Deserialize, Clone)]
pub enum CrowdingDistanceOperator {
    /// This is the default approach used in the NSGA2 paper (paragraph 3B, doi: 10.1109/4235.996017)
    Default,
    /// Approach used in R-NSGA2 (doi.org/10.1145/1143997.1144112), where the distance
    /// is the min rank from the reference points
    Rank(DistanceData),
}

impl CrowdingDistanceOperator {
    /// Calculate the crowding distance. This sets the distance on the individual's data,
    /// to retrieve it, use `Individual::get_data("crowding_distance").unwrap()`.
    ///
    /// > NOTE: the individuals must be a non-dominated front.
    ///
    /// With:
    ///
    /// - `Default`: this is calculated with complexity $O(M * log(N))$, where `M` is the
    ///  number of objectives and `N` the number of individuals. This is implemented
    ///  based on paragraph 3B in:
    ///   > K. Deb, A. Pratap, S. Agarwal and T. Meyarivan, "A fast and elitist multi-objective genetic
    ///   > algorithm: NSGA-II," in IEEE Transactions on Evolutionary Computation, vol. 6, no. 2, pp.
    ///   > 182-197, April 2002, doi: 10.1109/4235.996017.
    /// - `Rank`: this is calculated using the minimum rank from all reference points. This
    ///  is implemented based on:
    ///   > Kalyanmoy Deb and J. Sundar. 2006. Reference point based multi-objective
    ///   > optimization using evolutionary algorithms. In Proceedings of the 8th annual
    ///   > conference on Genetic and evolutionary computation (GECCO '06). Association
    ///   > for Computing Machinery, New York, NY, USA, 635–642.
    ///
    /// # Arguments
    ///
    /// * `individuals`: The individuals in a non-dominated front.
    ///
    /// returns: `Result<(), OError>`
    pub fn set_crowding_distance(&self, mut individuals: &mut [Individual]) -> Result<(), OError> {
        match &self {
            CrowdingDistanceOperator::Default => {
                let inf = DataValue::Real(f64::MAX); // do not use INF because is not supported by serde
                let total_individuals = individuals.len();

                // if there are enough point set distance to + infinite
                if total_individuals < 3 {
                    for individual in individuals {
                        individual.set_data(CROWDING_DIST_KEY, inf.clone());
                    }
                    debug!("Setting crowding distance to Inf for all individuals. At least 3 individuals are needed");

                    return Ok(());
                }

                for individual in individuals.iter_mut() {
                    individual.set_data(CROWDING_DIST_KEY, DataValue::Real(0.0));
                }

                let problem = individuals.individual(0)?.problem();
                for obj_name in problem.objective_names() {
                    let mut obj_values = individuals.objective_values(&obj_name)?;
                    let delta_range = vector_max(&obj_values)? - vector_min(&obj_values)?;

                    // set all to infinite if distance is too small
                    if delta_range.abs() < f64::EPSILON {
                        for individual in &mut *individuals {
                            individual.set_data(CROWDING_DIST_KEY, inf.clone());
                        }
                        debug!("Setting crowding distance to Inf for all individuals. The min/max range is too small");
                        return Ok(());
                    }

                    // sort objectives and get indexes to map individuals to sorted objectives
                    let sorted_idx = argsort(&obj_values, Sort::Ascending);
                    obj_values.sort_by(|a, b| a.total_cmp(b));

                    // assign infinite distance to the boundary points
                    individuals
                        .individual_as_mut(sorted_idx[0])?
                        .set_data(CROWDING_DIST_KEY, inf.clone());
                    individuals
                        .individual_as_mut(sorted_idx[total_individuals - 1])?
                        .set_data(CROWDING_DIST_KEY, inf.clone());

                    for obj_i in 1..(total_individuals - 1) {
                        // get the corresponding individual to sorted objective
                        let ind_i = sorted_idx[obj_i];
                        let current_distance = individuals
                            .individual(ind_i)?
                            .get_data(CROWDING_DIST_KEY)
                            .unwrap_or(DataValue::Real(0.0));

                        if let DataValue::Real(current_distance) = current_distance {
                            let delta =
                                (obj_values[obj_i + 1] - obj_values[obj_i - 1]) / delta_range;
                            individuals.individual_as_mut(ind_i)?.set_data(
                                CROWDING_DIST_KEY,
                                DataValue::Real(current_distance + delta),
                            );
                        }
                    }
                }
            }
            CrowdingDistanceOperator::Rank(data) => {
                // get the objective min (f_i__min) and max (f_i__max) for Eq. 3
                let obj_max = individuals.get_max_objectives()?;
                let obj_min = individuals.get_min_objectives()?;

                // Section IV, Step 1 - calculate the distances from each reference point
                for (ri, ref_point) in data.reference_points.iter().enumerate() {
                    // this contains the distances, each index is the individual index
                    let mut distances = Vec::new();
                    for ind in individuals.iter_mut() {
                        let distance = ind
                            .get_objective_values()?
                            .iter()
                            .enumerate()
                            .map(|(ii, value)| {
                                data.weights[ii]
                                    * f64::powi(
                                        (value - ref_point[ii]) / (obj_max[ii] - obj_min[ii]),
                                        2,
                                    )
                            })
                            .sum();
                        let distance = f64::sqrt(distance);
                        ind.set_data(
                            format!("distance_from_ref_point_{}", ri).as_str(),
                            DataValue::Real(distance),
                        );
                        distances.push(distance);
                    }
                    // get the individuals index in ascending order
                    let individual_ranks = argsort(&distances, Sort::Ascending);

                    // assign rank for reference point
                    for (rank, ind_index) in individual_ranks.iter().enumerate() {
                        let ind = individuals.individual_as_mut(*ind_index)?;
                        ind.set_data(
                            format!("rank_for_ref_point_{}", ri).as_str(),
                            DataValue::USize(rank),
                        );
                    }
                }

                // Section IV, Step 2 - assign the minimum of the ranks as crowding distance
                for ind in individuals.iter_mut() {
                    let mut min_rank = usize::MAX;
                    for ri in 0..data.reference_points.len() {
                        let rank = ind
                            .get_data(format!("rank_for_ref_point_{}", ri).as_str())?
                            .as_usize()?;
                        if rank < min_rank {
                            min_rank = rank;
                        };
                    }
                    // set the rank as f64 for consistency with other approach. The distance
                    // is set as negative because NSGA2 favours largest distances by default, whereas
                    // this approach should favour individuals closer to reference points
                    ind.set_data(CROWDING_DIST_KEY, DataValue::Real(-(min_rank as f64)));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_default_crowding {
    use float_cmp::assert_approx_eq;

    use crate::algorithms::nsga2::crowding::CrowdingDistanceOperator;
    use crate::algorithms::nsga2::CROWDING_DIST_KEY;
    use crate::core::test_utils::individuals_from_obj_values_dummy;
    use crate::core::{DataValue, Individuals, ObjectiveDirection};

    fn operator() -> CrowdingDistanceOperator {
        CrowdingDistanceOperator::Default
    }

    #[test]
    /// Test the crowding distance algorithm (not enough points).
    fn test_crowding_distance_not_enough_points() {
        let objectives = vec![vec![0.0, 0.0], vec![50.0, 50.0]];
        let mut individuals = individuals_from_obj_values_dummy(
            &objectives,
            &[ObjectiveDirection::Minimise, ObjectiveDirection::Minimise],
            None,
        );
        operator().set_crowding_distance(&mut individuals).unwrap();
        for i in individuals {
            assert_eq!(
                i.get_data(CROWDING_DIST_KEY).unwrap(),
                DataValue::Real(f64::MAX)
            );
        }
    }

    #[test]
    /// Test the crowding distance algorithm (min and max of objective is equal).
    fn test_crowding_distance_min_max_range() {
        let objectives = vec![
            vec![10.0, 20.0],
            vec![10.0, 20.0],
            vec![10.0, 20.0],
            vec![10.0, 20.0],
        ];
        let mut individuals = individuals_from_obj_values_dummy(
            &objectives,
            &[ObjectiveDirection::Minimise, ObjectiveDirection::Minimise],
            None,
        );
        operator().set_crowding_distance(&mut individuals).unwrap();
        for i in individuals {
            assert_eq!(
                i.get_data(CROWDING_DIST_KEY).unwrap(),
                DataValue::Real(f64::MAX)
            );
        }
    }

    #[test]
    /// Test the crowding distance algorithm (3 points).
    fn test_crowding_distance_3_points() {
        // 3 points
        let scenarios = vec![
            vec![vec![0.0, 0.0], vec![-100.0, 100.0], vec![200.0, -200.0]],
            vec![vec![25.0, 25.0], vec![-100.0, 100.0], vec![200.0, -200.0]],
        ];
        for objectives in scenarios {
            let mut individuals = individuals_from_obj_values_dummy(
                &objectives,
                &[ObjectiveDirection::Minimise, ObjectiveDirection::Minimise],
                None,
            );
            operator().set_crowding_distance(&mut individuals).unwrap();

            assert_eq!(
                individuals
                    .as_mut_slice()
                    .individual(0)
                    .unwrap()
                    .get_data(CROWDING_DIST_KEY)
                    .unwrap(),
                DataValue::Real(2.0)
            );
            // boundaries
            assert_eq!(
                individuals
                    .as_mut_slice()
                    .individual(1)
                    .unwrap()
                    .get_data(CROWDING_DIST_KEY)
                    .unwrap(),
                DataValue::Real(f64::MAX)
            );
            assert_eq!(
                individuals
                    .as_mut_slice()
                    .individual(2)
                    .unwrap()
                    .get_data(CROWDING_DIST_KEY)
                    .unwrap(),
                DataValue::Real(f64::MAX)
            );
        }
    }

    #[test]
    /// Test the crowding distance algorithm (3 objectives).
    fn test_crowding_distance_3_obj() {
        let objectives = vec![
            vec![0.0, 0.0, 0.0],
            vec![-1.0, 1.0, 2.0],
            vec![2.0, -2.0, -2.0],
        ];
        let mut individuals = individuals_from_obj_values_dummy(
            &objectives,
            &[
                ObjectiveDirection::Minimise,
                ObjectiveDirection::Minimise,
                ObjectiveDirection::Minimise,
            ],
            None,
        );
        operator().set_crowding_distance(&mut individuals).unwrap();

        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(0)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(3.0)
        );
        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(1)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(f64::MAX)
        );
        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(2)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(f64::MAX)
        );
    }

    #[test]
    /// Test the crowding distance algorithm (4 points).
    fn test_crowding_distance_4points() {
        let objectives = vec![
            vec![0.0, 0.0],
            vec![100.0, -100.0],
            vec![200.0, -200.0],
            vec![400.0, -400.0],
        ];
        let mut individuals = individuals_from_obj_values_dummy(
            &objectives,
            &[ObjectiveDirection::Minimise, ObjectiveDirection::Minimise],
            None,
        );
        operator().set_crowding_distance(&mut individuals).unwrap();

        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(0)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(f64::MAX)
        );
        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(1)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(1.0)
        );
        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(2)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(1.5)
        );
        assert_eq!(
            individuals
                .as_mut_slice()
                .individual(3)
                .unwrap()
                .get_data(CROWDING_DIST_KEY)
                .unwrap(),
            DataValue::Real(f64::MAX)
        );
    }

    #[test]
    /// Test the crowding distance algorithm (6 points).
    fn test_crowding_distance_6points() {
        let objectives = vec![
            vec![1.1, 8.1],
            vec![2.1, 6.1],
            vec![3.1, 4.1],
            vec![5.1, 3.1],
            vec![8.1, 2.1],
            vec![11.1, 1.1],
        ];
        let mut individuals = individuals_from_obj_values_dummy(
            &objectives,
            &[ObjectiveDirection::Minimise, ObjectiveDirection::Minimise],
            None,
        );
        operator().set_crowding_distance(&mut individuals).unwrap();

        let expected = [
            f64::MAX,
            0.7714285714285714,
            0.728571429,
            0.785714286,
            0.885714286,
            f64::MAX,
        ];
        for (idx, value) in expected.into_iter().enumerate() {
            assert_approx_eq!(
                f64,
                individuals
                    .as_mut_slice()
                    .individual(idx)
                    .unwrap()
                    .get_data(CROWDING_DIST_KEY)
                    .unwrap()
                    .as_real()
                    .unwrap(),
                DataValue::Real(value).as_real().unwrap(),
                epsilon = 0.001
            );
        }
    }
}
