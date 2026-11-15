use hv_fonseca_et_al_2006_sys::calculate_hv;

use crate::core::{Individual, OError};
use crate::metrics::hypervolume::prepare_front;

/// Calculate the hyper-volume using the algorithm proposed by [Fonseca et al. (2006)](http://dx.doi.org/10.1109/CEC.2006.1688440)
/// for a problem with `d` objectives and `n` individuals. The function calls version 4 of the
/// algorithm, therefore its complexity is O(`n^(d-2)*log n`).
///
/// **IMPLEMENTATION NOTES**:
/// 1) Points dominated by the reference point are removed from the calculation.
/// 2) Dominated and unfeasible solutions are excluded using the NSGA2 [`crate::utils::fast_non_dominated_sort()`]
///    algorithm in order to get the Pareto front. As assumed in the paper, non-dominated points do
///    not contribute do the metric.
/// 3) The coordinates of maximised objectives of the reference point are multiplied by -1 as the
///    algorithm assumes all objectives are maximised.
#[derive(Debug)]
pub struct HyperVolumeFonseca2006 {
    /// The individuals to use. The size of this vector corresponds to the individual size and the
    /// size of the nested vector corresponds to the number of problem objectives.
    individuals: Vec<Vec<f64>>,
    /// The reference point.
    reference_point: Vec<f64>,
}

impl HyperVolumeFonseca2006 {
    /// Calculate the hyper-volume using the algorithm proposed by [Fonseca et al. (2006)](http://dx.doi.org/10.1109/CEC.2006.1688440)
    /// for a problem with `d` objectives and `n` individuals. The function calls version 4 of the
    /// algorithm, therefore its complexity is O(`n^(d-2)*log n`).
    ///
    /// # Arguments
    ///
    /// * `individuals`: The list of individuals.
    /// * `reference_point`: The reference point.
    ///
    /// returns: `Result<HyperVolumeFonseca2006, OError>`
    pub fn new(individuals: &mut [Individual], reference_point: &[f64]) -> Result<Self, OError> {
        let metric_name = "Hyper-volume Fonseca et al. (2006)";
        if reference_point.len() != 3 {
            return Err(OError::Metric(
                metric_name.to_string(),
                "This can only be used on a 3-objective problem.".to_string(),
            ));
        }
        let (individuals, reference_point) =
            prepare_front(individuals, reference_point, metric_name)?;
        Ok(Self {
            individuals,
            reference_point,
        })
    }

    /// Calculate the hyper-volume.
    ///
    /// return: `f64`
    pub fn compute(&self) -> f64 {
        calculate_hv(&self.individuals, &self.reference_point)
    }
}

#[cfg(test)]
mod test {
    use float_cmp::approx_eq;

    use crate::core::test_utils::individuals_from_obj_values_dummy;
    use crate::core::ObjectiveDirection;
    use crate::metrics::test_utils::parse_pagmo_test_data_file;
    use crate::metrics::HyperVolumeFonseca2006;

    #[test]
    /// Reference point must be strictly larger than any objective
    fn test_wrong_ref_point() {
        let objective_values = vec![vec![1.0, 2.0, 1.0], vec![2.0, 1.0, 1.0]];
        let objective_direction = [ObjectiveDirection::Minimise; 3];

        let mut individuals =
            individuals_from_obj_values_dummy(&objective_values, &objective_direction, None);
        let ref_point = vec![2.0, 2.0, 2.0];
        let hv = HyperVolumeFonseca2006::new(&mut individuals, &ref_point);
        assert!(hv
            .unwrap_err()
            .to_string()
            .contains("The coordinate (2) of the reference point #1 must be strictly larger"));
    }

    #[test]
    /// Test avery simple front
    fn test_simple_front() {
        let objective_values = vec![vec![1.0, 1.0, 1.0], vec![2.0, 2.0, 2.0]];
        let objective_direction = vec![ObjectiveDirection::Minimise; 3];
        let ref_point = [3.0, 3.0, 3.0];

        let mut individuals = individuals_from_obj_values_dummy(
            &objective_values,
            &objective_direction.clone(),
            None,
        );
        let hv = HyperVolumeFonseca2006::new(&mut individuals, &ref_point).unwrap();
        assert_eq!(hv.compute(), 8.0);
    }

    /// Run a test using a Pagmo file.
    ///
    /// # Arguments
    ///
    /// * `file`: The file name in the `test_data` folder.
    ///
    /// returns: ()
    pub(crate) fn assert_test_file(file: &str) {
        let all_test_data = parse_pagmo_test_data_file(file).unwrap();
        let obj_count = all_test_data.first().unwrap().reference_point.len();
        let objective_direction = vec![ObjectiveDirection::Minimise; obj_count];

        for (ti, test_data) in all_test_data.iter().enumerate() {
            let mut individuals = individuals_from_obj_values_dummy(
                &test_data.objective_values,
                &objective_direction,
                None,
            );
            let hv =
                HyperVolumeFonseca2006::new(&mut individuals, &test_data.reference_point).unwrap();

            let calculated = hv.compute();
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
    /// Test the `HyperVolumeFonseca2006` struct using Pagmo c_max_t100_d3_n128 test data.
    /// See https://github.com/esa/pagmo2/tree/master/tests/hypervolume_test_data
    fn test_c_max_t100_d3_n128() {
        assert_test_file("c_max_t100_d3_n128");
    }

    #[test]
    /// Test the `HyperVolumeFonseca2006` struct using Pagmo c_max_t1_d3_n2048 test data.
    /// See https://github.com/esa/pagmo2/tree/master/tests/hypervolume_test_data
    fn test_c_max_t1_d3_n2048() {
        assert_test_file("c_max_t1_d3_n2048");
    }
}
