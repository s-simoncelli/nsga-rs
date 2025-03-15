use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// The type of stopping condition. Pick one type to inform the algorithm how/when it should
/// terminate the population evolution.
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub enum StoppingCondition {
    /// Set a maximum duration (as number of seconds).
    MaxDuration(u32),
    /// Set a maximum number of generations.
    MaxGeneration(u32),
    /// Set a maximum number of function evaluations.
    MaxFunctionEvaluations(u32),
    /// Stop when at least on condition is met (this acts as an OR operator).
    Any(Vec<StoppingCondition>),
    /// Stop when all conditions are met (this acts as an AND operator).
    All(Vec<StoppingCondition>),
}

impl StoppingCondition {
    /// A name describing the stopping condition.
    ///
    /// returns: `String`
    pub fn name(&self) -> String {
        match self {
            StoppingCondition::MaxDuration(v) => format!("maximum duration={v} seconds"),
            StoppingCondition::MaxGeneration(v) => format!("maximum number of generations={v}"),
            StoppingCondition::MaxFunctionEvaluations(v) => {
                format!("maximum number of function evaluations={v}")
            }
            StoppingCondition::Any(s) => s
                .iter()
                .map(|cond| cond.name())
                .collect::<Vec<String>>()
                .join(" OR "),
            StoppingCondition::All(s) => s
                .iter()
                .map(|cond| cond.name())
                .collect::<Vec<String>>()
                .join(" AND "),
        }
    }

    /// Check whether the stopping condition is a vector and has nested vector in it.
    ///
    /// # Arguments
    ///
    /// * `conditions`: A vector of stopping conditions.
    ///
    /// returns: `bool`
    pub fn has_nested_vector(conditions: &[StoppingCondition]) -> bool {
        conditions.iter().any(|c| match c {
            StoppingCondition::Any(_) | StoppingCondition::All(_) => true,
            _ => false,
        })
    }
}

impl Display for StoppingCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            StoppingCondition::MaxDuration(duration) => write!(f, "{duration} seconds"),
            StoppingCondition::MaxGeneration(generation) => write!(f, "{generation} generations"),
            StoppingCondition::MaxFunctionEvaluations(nfe) => write!(f, "{nfe} evaluations"),
            StoppingCondition::Any(values) => {
                let values: Vec<String> = values.iter().map(|c| format!("{c}")).collect();
                write!(f, "{}", values.join(" OR "))
            }
            StoppingCondition::All(values) => {
                let values: Vec<String> = values.iter().map(|c| format!("{c}")).collect();
                write!(f, "{}", values.join(" AND "))
            }
        }
    }
}

#[cfg(feature = "python")]
pub mod py {
    use crate::algorithms::StoppingCondition;
    use pyo3::prelude::*;

    /// The stopping condition class in Python. Each enum item is a Python function of the
    /// StoppingConditionValue class. Items are lower-case to be PEP compliant.
    #[pyclass(name = "StoppingConditionValue")]
    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub enum PyStoppingConditionValue {
        max_duration(u32),
        max_generation(u32),
        max_function_evaluations(u32),
    }

    #[pymethods]
    impl PyStoppingConditionValue {
        fn value(&self) -> u32 {
            match self {
                PyStoppingConditionValue::max_duration(v) => *v,
                PyStoppingConditionValue::max_generation(v) => *v,
                PyStoppingConditionValue::max_function_evaluations(v) => *v,
            }
        }

        fn __repr__(&self) -> PyResult<String> {
            let attr = match &self {
                PyStoppingConditionValue::max_duration(duration) => {
                    format!("duration={duration} seconds")
                }
                PyStoppingConditionValue::max_generation(generation) => {
                    format!("generation={generation} generations")
                }
                PyStoppingConditionValue::max_function_evaluations(nfe) => {
                    format!("NFE={nfe} evaluations")
                }
            };
            Ok(format!("StoppingConditionValue({attr})"))
        }

        fn __str__(&self) -> String {
            self.__repr__().unwrap()
        }
    }

    impl From<PyStoppingConditionValue> for StoppingCondition {
        fn from(cond: PyStoppingConditionValue) -> Self {
            match cond {
                PyStoppingConditionValue::max_duration(duration) => {
                    StoppingCondition::MaxDuration(duration)
                }
                PyStoppingConditionValue::max_generation(generation) => {
                    StoppingCondition::MaxGeneration(generation)
                }
                PyStoppingConditionValue::max_function_evaluations(nfe) => {
                    StoppingCondition::MaxFunctionEvaluations(nfe)
                }
            }
        }
    }

    #[derive(FromPyObject)]
    enum PyStoppingConditionMap {
        #[pyo3(transparent, annotation = "condition")]
        Condition(PyStoppingConditionValue),
        #[pyo3(transparent, annotation = "list of conditions")]
        Vector(Vec<PyStoppingConditionValue>),
    }

    #[pymethods]
    impl StoppingCondition {
        #[new]
        fn new(condition: PyStoppingConditionMap) -> Self {
            match condition {
                PyStoppingConditionMap::Condition(cond) => cond.into(),
                // handle any only
                PyStoppingConditionMap::Vector(conds) => {
                    StoppingCondition::Any(conds.into_iter().map(|c| c.into()).collect())
                }
            }
        }

        fn conditions(&self) -> Vec<PyStoppingConditionValue> {
            match self {
                StoppingCondition::MaxDuration(d) => {
                    vec![PyStoppingConditionValue::max_duration(*d)]
                }
                StoppingCondition::MaxGeneration(g) => {
                    vec![PyStoppingConditionValue::max_generation(*g)]
                }
                StoppingCondition::MaxFunctionEvaluations(nfe) => {
                    vec![PyStoppingConditionValue::max_function_evaluations(*nfe)]
                }
                StoppingCondition::Any(conds) => {
                    let mut vec = vec![];
                    for cond in conds {
                        vec.extend(cond.conditions());
                    }
                    vec
                }
                StoppingCondition::All(_) => panic!("Not supported"),
            }
        }

        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("StoppingCondition({})", self.name()))
        }

        fn __str__(&self) -> String {
            self.__repr__().unwrap()
        }
    }
}

#[cfg(all(feature = "python", test))]
mod tests {
    use crate::algorithms::StoppingCondition;
    use pyo3::ffi::c_str;
    use pyo3::prelude::*;

    #[test]
    fn test_stopping_condition_value_init() {
        #[pymodule]
        fn py_optirustic(m: &Bound<'_, PyModule>) -> PyResult<()> {
            m.add_class::<StoppingCondition>()?;

            Ok(())
        }
        pyo3_build_config::add_extension_module_link_args();

        pyo3::append_to_inittab!(py_optirustic);
        Python::with_gil(|py| {
            Python::run(
                py,
                c_str!(
                    "\
                import StoppingConditionValue;\
                StoppingConditionValue.max_duration(3)\
            "
                ),
                None,
                None,
            )
        })
        .unwrap();

        // let instance = Python::with_gil(|py| {
        //     py_run!(
        //         py,
        //         (),
        //         r#"
        //
        //         "#
        //     )
        //
        // });
    }
}
