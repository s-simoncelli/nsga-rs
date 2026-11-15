use crate::core::{Individual, OError};
use crate::metrics::hv_wfg::wfg::{Optimisation, Wfg};
use crate::metrics::hypervolume::prepare_front;

mod wfg;

/// Calculate the hyper-volume using the WFG algorithm proposed by [While et al. (2012)](http://dx.doi.org/10.1109/TEVC.2010.2077298)
/// for a problem with `d` objectives and `n` individuals.
///
/// **IMPLEMENTATION NOTES**:
/// 1) Dominated and unfeasible solutions are excluded using the NSGA2 [`crate::utils::fast_non_dominated_sort()`]
///    algorithm in order to get the Pareto front. As assumed in the paper, non-dominated points do
///    not contribute do the metric.
/// 2) The coordinates of maximised objectives of the reference point are multiplied by -1 as the
///    algorithm assumes all objectives are maximised.
#[derive(Debug)]
pub struct HyperVolumeWhile2012 {
    /// The individuals to use. The size of this vector corresponds to the individual size and the
    /// size of the nested vector corresponds to the number of problem objectives.
    individuals: Vec<Vec<f64>>,
    /// The reference point.
    reference_point: Vec<f64>,
    /// The name of this metric
    metric_name: String,
}

impl HyperVolumeWhile2012 {
    /// Calculate the hyper-volume using the WFG algorithm proposed by [While et al. (2012)](http://dx.doi.org/10.1109/TEVC.2010.2077298)
    /// for a problem with `d` objectives and `n` individuals.
    ///
    /// # Arguments
    ///
    /// * `individuals`: The list of individuals.
    /// * `reference_point`: The reference point.
    ///
    /// returns: `Result<HyperVolumeWhile2012, OError>`
    pub fn new(individuals: &mut [Individual], reference_point: &[f64]) -> Result<Self, OError> {
        let metric_name = "Hyper-volume While et al. (2012)".to_string();
        let (individuals, reference_point) =
            prepare_front(individuals, reference_point, &metric_name)?;
        Ok(Self {
            individuals,
            reference_point,
            metric_name,
        })
    }

    /// Calculate the hyper-volume.
    ///
    /// return: `Result<f64, OError>`
    pub fn compute(&mut self) -> Result<f64, OError> {
        let wfg = Wfg::new(&self.individuals, &self.reference_point, Optimisation::O2);
        wfg.calculate()
            .map_err(|e| OError::Metric(self.metric_name.clone(), e))
    }
}

#[cfg(test)]
mod test {
    use float_cmp::approx_eq;

    use crate::core::test_utils::individuals_from_obj_values_dummy;
    use crate::core::ObjectiveDirection;
    use crate::metrics::test_utils::parse_pagmo_test_data_file;
    use crate::metrics::HyperVolumeWhile2012;

    /// Run a test using a Pagmo file.
    ///
    /// # Arguments
    ///
    /// * `file`: The file name in the `test_data` folder.
    ///
    /// returns: ()
    fn assert_test_file(file: &str) {
        let all_test_data = parse_pagmo_test_data_file(file).unwrap();
        let obj_count = all_test_data.first().unwrap().reference_point.len();
        let objective_direction = vec![ObjectiveDirection::Minimise; obj_count];

        for (ti, test_data) in all_test_data.iter().enumerate() {
            let mut individuals = individuals_from_obj_values_dummy(
                &test_data.objective_values,
                &objective_direction,
                None,
            );
            let mut hv =
                HyperVolumeWhile2012::new(&mut individuals, &test_data.reference_point).unwrap();

            let calculated = hv.compute().unwrap();
            let expected = test_data.hyper_volume;
            if !approx_eq!(f64, calculated, expected, epsilon = 0.001) {
                panic!(
                    r#"assertion failed for test #{}: `(left approx_eq right)` left: `{:?}`, right: `{:?}`"#,
                    ti + 1,
                    calculated,
                    expected,
                )
            }
        }
    }

    #[test]
    // /// Test the `HyperVolumeWhile2012` struct using Pagmo c_max_t1_d5_n1024 test data.
    // /// See https://github.com/esa/pagmo2/tree/master/tests/hypervolume_test_data
    fn test_c_max_t1_d5_n1024() {
        assert_test_file("c_max_t1_d5_n1024");
    }

    #[test]
    /// Test the `HyperVolumeWhile2012` struct using Pagmo c_max_t100_d3_n128 test data.
    /// See https://github.com/esa/pagmo2/tree/master/tests/hypervolume_test_data
    fn test_c_max_t100_d3_n128() {
        assert_test_file("c_max_t100_d3_n128");
    }

    #[test]
    /// Test the `HyperVolumeWhile2012` struct using Pagmo c_max_t1_d3_n2048 test data.
    /// See https://github.com/esa/pagmo2/tree/master/tests/hypervolume_test_data
    fn test_c_max_t1_d3_n2048() {
        assert_test_file("c_max_t1_d3_n2048");
    }
}
