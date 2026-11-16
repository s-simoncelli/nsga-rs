use crate::core::{OError, Population};
use crate::metrics::HyperVolume;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Mutex;

/// Stop when the relative change of the hyper-volume between the current generation and
///`generations` is below `threshold_percent`. The hyper-volume is only calculated every
/// `generations` to limit its cost.
#[cfg_attr(feature = "python", pyclass(from_py_object))]
#[derive(Serialize, Deserialize, Debug)]
pub struct HyperVolumeChange {
    /// The reference point used to calculate the hyper-volume.
    reference_point: Vec<f64>,
    /// The relative hyper-volume change (as percentage) below which the evolution stops.
    threshold_percent: f64,
    /// The number of generations between the two hyper-volume values being compared.
    generations: u32,
    /// The generation and hyper-volume of the last check. This uses interior mutability because
    /// the stopping condition is checked via `&self`; a `Mutex` keeps the struct `Sync` for Python.
    #[serde(skip)]
    last: Mutex<Option<(u32, f64)>>,
}

impl Clone for HyperVolumeChange {
    fn clone(&self) -> Self {
        Self {
            reference_point: self.reference_point.clone(),
            threshold_percent: self.threshold_percent,
            generations: self.generations,
            last: Mutex::new(*self.last.lock().unwrap()),
        }
    }
}

impl HyperVolumeChange {
    /// Create the stopping condition.
    ///
    /// # Arguments
    ///
    /// * `reference_point`: The reference point to calculate the hyper-volume with.
    /// * `threshold_percent`: The relative hyper-volume change (as percentage, for example `0.1`
    ///   for 0.1%) below which the evolution stops.
    /// * `generations`: The number of generations between the two hyper-volume values to compare.
    ///
    /// returns: `Result<HyperVolumeChange, OError>`
    pub fn new(
        reference_point: Vec<f64>,
        threshold_percent: f64,
        generations: u32,
    ) -> Result<Self, OError> {
        let error = |msg: &str| Err(OError::Generic(format!("HyperVolumeChange: {msg}")));
        if reference_point.is_empty() {
            return error("the reference point cannot be empty");
        }
        if generations == 0 {
            return error("the number of generations must be larger than 0");
        }
        if !(threshold_percent >= 0.0) {
            return error("the threshold must be a positive number");
        }
        Ok(Self {
            reference_point,
            threshold_percent,
            generations,
            last: Mutex::new(None),
        })
    }

    /// Check whether the relative hyper-volume change since the last check is below the
    /// threshold. To limit the cost, the hyper-volume is only calculated every `generations`.
    ///
    /// # Arguments
    ///
    /// * `generation`: The current generation.
    /// * `population`: The current population.
    ///
    /// returns: `Result<bool, OError>`
    pub fn is_met(&self, generation: u32, population: &Population) -> Result<bool, OError> {
        let mut last = self.last.lock().unwrap();
        if let Some((last_gen, _)) = *last {
            if generation < last_gen + self.generations {
                return Ok(false);
            }
        }

        // points not better than the reference point add no volume, and would make the metric
        // fail and stop the run with an error: drop them
        let mut individuals = Vec::new();
        for ind in population.individuals() {
            let problem = ind.problem();
            // stored objectives are negated when maximised: compare with the negated coordinate
            let internal_ref = problem
                .objective_names()
                .iter()
                .zip(&self.reference_point)
                .map(|(name, r)| {
                    Ok(if problem.is_objective_minimised(name)? {
                        *r
                    } else {
                        -r
                    })
                })
                .collect::<Result<Vec<f64>, OError>>()?;
            let objectives = ind.get_objective_values()?;
            if objectives.len() != internal_ref.len()
                || objectives.iter().zip(&internal_ref).all(|(v, r)| v < r)
            {
                individuals.push(ind.clone());
            }
        }
        let hv = match individuals.len() {
            0 => 0.0,
            _ => {
                // the metrics need 2 points: a duplicate point adds no volume
                if individuals.len() == 1 {
                    individuals.push(individuals[0].clone());
                }
                HyperVolume::from_individuals(&mut individuals, &self.reference_point)?
            }
        };
        let past = last.replace((generation, hv));
        Ok(match past {
            Some((_, past_hv)) if past_hv != 0.0 => {
                (hv - past_hv).abs() / past_hv * 100.0 < self.threshold_percent
            }
            _ => false,
        })
    }
}

impl Display for HyperVolumeChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "hyper-volume change < {}% over {} generations",
            self.threshold_percent, self.generations
        )
    }
}

/// The type of stopping condition. Pick one type to inform the algorithm how/when it should
/// terminate the population evolution.
#[derive(Serialize, Deserialize, Clone)]
pub enum StoppingCondition {
    /// Set a maximum duration (as number of minutes).
    MaxDurationAsMinutes(u32),
    /// Set a maximum duration (as number of hours).
    MaxDurationAsHours(u32),
    /// Set a maximum number of generations.
    MaxGeneration(u32),
    /// Set a maximum number of function evaluations.
    MaxFunctionEvaluations(u32),
    /// Stop when the relative hyper-volume change over a number of generations is below a
    /// threshold.
    HyperVolumeChange(HyperVolumeChange),
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
            StoppingCondition::MaxDurationAsMinutes(v) => format!("maximum duration={v} minutes"),
            StoppingCondition::MaxDurationAsHours(v) => format!("maximum duration={v} hours"),
            StoppingCondition::MaxGeneration(v) => format!("maximum number of generations={v}"),
            StoppingCondition::MaxFunctionEvaluations(v) => {
                format!("maximum number of function evaluations={v}")
            }
            StoppingCondition::HyperVolumeChange(c) => c.to_string(),
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
            StoppingCondition::MaxDurationAsMinutes(duration) => write!(f, "{duration} minutes"),
            StoppingCondition::MaxDurationAsHours(duration) => write!(f, "{duration} hours"),
            StoppingCondition::MaxGeneration(generation) => write!(f, "{generation} generations"),
            StoppingCondition::MaxFunctionEvaluations(nfe) => write!(f, "{nfe} evaluations"),
            StoppingCondition::HyperVolumeChange(c) => write!(f, "{c}"),
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
    use crate::algorithms::{HyperVolumeChange, StoppingCondition};
    use pyo3::{exceptions::PyValueError, prelude::*, types::PyList, IntoPyObjectExt};

    #[pymethods]
    impl HyperVolumeChange {
        #[new]
        fn py_new(
            reference_point: Vec<f64>,
            threshold_percent: f64,
            generations: u32,
        ) -> PyResult<Self> {
            HyperVolumeChange::new(reference_point, threshold_percent, generations)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        #[getter]
        fn reference_point(&self) -> Vec<f64> {
            self.reference_point.clone()
        }

        #[getter]
        fn threshold_percent(&self) -> f64 {
            self.threshold_percent
        }

        #[getter]
        fn generations(&self) -> u32 {
            self.generations
        }

        fn __repr__(&self) -> String {
            format!("HyperVolumeChange({self})")
        }
    }

    /// Handle conversion of `StoppingCondition` into Python object for `NSGA*Args` structs.
    impl<'py> IntoPyObject<'py> for StoppingCondition {
        type Target = PyAny;
        type Output = Bound<'py, Self::Target>;
        type Error = PyErr;

        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            match self {
                StoppingCondition::MaxDurationAsMinutes(d) => {
                    PyStoppingConditionValue::max_duration_as_minutes(d).into_bound_py_any(py)
                }
                StoppingCondition::MaxDurationAsHours(d) => {
                    PyStoppingConditionValue::max_duration_as_hours(d).into_bound_py_any(py)
                }
                StoppingCondition::MaxGeneration(g) => {
                    PyStoppingConditionValue::max_generation(g).into_bound_py_any(py)
                }
                StoppingCondition::MaxFunctionEvaluations(nfe) => {
                    PyStoppingConditionValue::max_function_evaluations(nfe).into_bound_py_any(py)
                }
                StoppingCondition::HyperVolumeChange(c) => {
                    PyStoppingConditionValue::hypervolume_change(c).into_bound_py_any(py)
                }
                StoppingCondition::Any(stopping_conditions)
                | StoppingCondition::All(stopping_conditions) => {
                    let items = stopping_conditions
                        .into_iter()
                        .map(|c| c.into_pyobject(py))
                        .collect::<Result<Vec<_>, _>>()?;
                    PyList::new(py, items)?.into_bound_py_any(py)
                }
            }
            // out.into_bound_py_any(py)
        }
    }

    /// The stopping condition class in Python. Each enum item is a Python function of the
    /// [`StoppingConditionValue`] class. Items are lower-case to be PEP compliant.
    #[pyclass(name = "StoppingCondition", from_py_object)]
    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub enum PyStoppingConditionValue {
        max_duration_as_minutes(u32),
        max_duration_as_hours(u32),
        max_generation(u32),
        max_function_evaluations(u32),
        hypervolume_change(HyperVolumeChange),
    }

    #[pymethods]
    impl PyStoppingConditionValue {
        fn value(&self) -> PyResult<f64> {
            match self {
                PyStoppingConditionValue::max_duration_as_minutes(v) => Ok(*v as f64),
                PyStoppingConditionValue::max_duration_as_hours(v) => Ok(*v as f64),
                PyStoppingConditionValue::max_generation(v) => Ok(*v as f64),
                PyStoppingConditionValue::max_function_evaluations(v) => Ok(*v as f64),
                PyStoppingConditionValue::hypervolume_change(data) => Ok(data.threshold_percent()),
            }
        }

        fn __repr__(&self) -> PyResult<String> {
            let attr = match &self {
                PyStoppingConditionValue::max_duration_as_minutes(duration) => {
                    format!("duration={duration} minutes")
                }
                PyStoppingConditionValue::max_duration_as_hours(duration) => {
                    format!("duration={duration} hours")
                }
                PyStoppingConditionValue::max_generation(generation) => {
                    format!("generation={generation} generations")
                }
                PyStoppingConditionValue::max_function_evaluations(nfe) => {
                    format!("NFE={nfe} evaluations")
                }
                PyStoppingConditionValue::hypervolume_change(c) => c.to_string(),
            };
            Ok(format!("StoppingConditionValue({attr})"))
        }

        fn __str__(&self) -> String {
            self.__repr__().unwrap()
        }
    }

    /// Allow conversion to [`StoppingCondition`] from Python when an algorithm is initialised
    impl From<PyStoppingConditionValue> for StoppingCondition {
        fn from(cond: PyStoppingConditionValue) -> Self {
            match cond {
                PyStoppingConditionValue::max_duration_as_minutes(duration) => {
                    StoppingCondition::MaxDurationAsMinutes(duration)
                }
                PyStoppingConditionValue::max_duration_as_hours(duration) => {
                    StoppingCondition::MaxDurationAsHours(duration)
                }
                PyStoppingConditionValue::max_generation(generation) => {
                    StoppingCondition::MaxGeneration(generation)
                }
                PyStoppingConditionValue::max_function_evaluations(nfe) => {
                    StoppingCondition::MaxFunctionEvaluations(nfe)
                }
                PyStoppingConditionValue::hypervolume_change(c) => {
                    StoppingCondition::HyperVolumeChange(c)
                }
            }
        }
    }

    /// Python conversion of stopping condition or list of.
    #[derive(FromPyObject)]
    pub enum PyStoppingConditionMap {
        #[pyo3(transparent, annotation = "condition")]
        Condition(PyStoppingConditionValue),
        #[pyo3(transparent, annotation = "list of conditions")]
        Vector(Vec<PyStoppingConditionValue>),
    }

    /// Handle initialisation of condition(s) from Python to Rust/
    impl From<PyStoppingConditionMap> for StoppingCondition {
        fn from(value: PyStoppingConditionMap) -> Self {
            match value {
                PyStoppingConditionMap::Condition(condition) => condition.into(),
                PyStoppingConditionMap::Vector(conditions) => {
                    StoppingCondition::Any(conditions.into_iter().map(|c| c.into()).collect())
                }
            }
        }
    }
}
