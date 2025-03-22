use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use pyo3::exceptions::PyValueError;
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use optirustic::algorithms::{
    Algorithm, AlgorithmExport, AlgorithmSerialisedExport, ExportHistory, NSGA2Arg, NSGA3Arg,
    PyAlgorithm, PyStoppingConditionValue, StoppingCondition, NSGA2, NSGA3,
};
use optirustic::core::{
    DataValue, Individual, OError, ObjectiveDirection, PyProblem, RelationalOperator,
};
use optirustic::metrics::HyperVolume;
use optirustic::operators::{PolynomialMutationArgs, SimulatedBinaryCrossoverArgs};
use optirustic::utils::{DasDarren1998, NumberOfPartitions, TwoLayerPartitions};

/// Get the python function from the utils.plot module
pub fn get_plot_fun(function: &str, py: Python<'_>) -> PyResult<PyObject> {
    let code = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/utils/plot.py"))?;
    let module = PyModule::from_code(
        py,
        CString::new(code)?.as_c_str(),
        c_str!("plot.py"),
        c_str!("utils.plot"),
    )?;
    let fun: Py<PyAny> = module.getattr(function)?.into();
    Ok(fun)
}

/// Macro to generate python class for an algorithm data reader
macro_rules! create_interface {
    ($name: ident, $type: ident, $ArgType: ident) => {
        #[pyclass]
        pub struct $name {
            export_data: AlgorithmExport,
            #[pyo3(get)]
            problem: PyProblem,
            #[pyo3(get)]
            individuals: Vec<Individual>,
            #[pyo3(get)]
            took: PyObject,
            #[pyo3(get)]
            objectives: HashMap<String, Vec<f64>>,
            #[pyo3(get)]
            additional_data: Option<HashMap<String, DataValue>>,
            #[pyo3(get)]
            exported_on: DateTime<Utc>,
        }

        #[pymethods]
        impl $name {
            #[new]
            /// Initialise the class
            pub fn new(file: PathBuf) -> PyResult<Self> {
                let path = PathBuf::from(file);
                let file_data: AlgorithmSerialisedExport<$ArgType> =
                    $type::read_json_file(&path)
                        .map_err(|e| PyValueError::new_err(e.to_string()))?;

                // Algorthm data
                let additional_data = file_data.additional_data.clone();
                let exported_on = file_data.exported_on.clone();

                // Convert export
                let export_data: AlgorithmExport = file_data
                    .try_into()
                    .map_err(|e: OError| PyValueError::new_err(e.to_string()))?;

                // Problem
                let problem: PyProblem = export_data.problem.as_ref().into();

                // Time taken
                let took = Python::with_gil(|py| {
                    let module = PyModule::import(py, "datetime")?;

                    let timedelta = module.getattr("timedelta")?;
                    let kwargs = PyDict::new(py);
                    kwargs.set_item("hours", export_data.took.hours)?;
                    kwargs.set_item("minutes", export_data.took.minutes)?;
                    kwargs.set_item("seconds", export_data.took.seconds)?;
                    let result = timedelta.call((), Some(&kwargs))?;
                    result.extract::<PyObject>()
                })?;

                // Individuals
                let individuals = export_data.individuals.clone();

                // All objective values by name
                let objectives = export_data
                    .get_objectives()
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;

                Ok(Self {
                    export_data,
                    problem,
                    took,
                    individuals,
                    objectives,
                    additional_data,
                    exported_on,
                })
            }

            #[getter]
            /// Get the generation number.
            pub fn generation(&self) -> u32 {
                self.export_data.generation
            }

            #[getter]
            /// Get the algorithm name.
            pub fn algorithm(&self) -> String {
                self.export_data.algorithm.clone()
            }

            /// Calculate the hyper-volume metric.
            pub fn hyper_volume(&mut self, reference_point: Vec<f64>) -> PyResult<f64> {
                let hv = HyperVolume::from_individual(
                    &mut self.export_data.individuals,
                    &reference_point,
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(hv)
            }

            /// Estimate the reference point from serialised data.
            #[pyo3(signature = (offset=None))]
            pub fn estimate_reference_point(&self, offset: Option<Vec<f64>>) -> PyResult<Vec<f64>> {
                let individuals = &self.export_data.individuals;
                let ref_point = HyperVolume::estimate_reference_point(individuals, offset)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(ref_point)
            }

            /// Estimate the reference point from files.
            #[staticmethod]
            #[pyo3(signature = (folder, offset=None))]
            pub fn estimate_reference_point_from_files(
                folder: PathBuf,
                offset: Option<Vec<f64>>,
            ) -> PyResult<Vec<f64>> {
                let all_serialise_data_vec = $type::read_json_files(&folder)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let ref_point = HyperVolume::estimate_reference_point_from_files(
                    &all_serialise_data_vec,
                    offset,
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(ref_point)
            }

            #[staticmethod]
            pub fn convergence_data(
                folder: String,
                reference_point: Vec<f64>,
            ) -> PyResult<(Vec<u32>, Vec<DateTime<Utc>>, Vec<f64>)> {
                let folder = PathBuf::from(folder);
                let all_serialise_data = $type::read_json_files(&folder)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let data = HyperVolume::from_files(&all_serialise_data, &reference_point)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;

                Ok((data.generations(), data.times(), data.values()))
            }

            /// Plot the Pareto front
            pub fn plot(&self) -> PyResult<PyObject> {
                Python::with_gil(|py| {
                    let obj_count = self.problem.number_of_objectives;
                    let fun_name = match obj_count {
                        2 => "plot_2d",
                        3 => "plot_3d",
                        _ => "plot_parallel",
                    };
                    let fun: Py<PyAny> = get_plot_fun(fun_name, py)?;
                    fun.call1(
                        py,
                        (
                            self.objectives.clone(),
                            self.algorithm(),
                            self.generation(),
                            self.export_data.individuals.len(),
                        ),
                    )
                })
            }

            #[staticmethod]
            pub fn plot_convergence(
                folder: String,
                reference_point: Vec<f64>,
            ) -> PyResult<PyObject> {
                let folder = PathBuf::from(folder);
                let all_serialise_data = $type::read_json_files(&folder)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let data = HyperVolume::from_files(&all_serialise_data, &reference_point)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;

                Python::with_gil(|py| {
                    let fun: Py<PyAny> = get_plot_fun("plot_convergence", py)?;
                    fun.call1(py, (data.generations(), data.values()))
                })
            }
        }
    };
}

// Register the python classes
create_interface!(NSGA2Data, NSGA2, NSGA2Arg);
create_interface!(NSGA3Data, NSGA3, NSGA3Arg);

#[pymethods]
impl NSGA3Data {
    /// Reference point plot using the points exported by the NSGA3 algorithm
    pub fn plot_reference_points(&self) -> PyResult<PyObject> {
        let algorithm_data = self.additional_data.as_ref().unwrap();
        Python::with_gil(|py| {
            let ref_points = algorithm_data.get("reference_points").unwrap();
            let fun: Py<PyAny> = get_plot_fun("plot_reference_points", py)?;
            fun.call1(py, (ref_points,))
        })
    }
}

/// Add new methods to `DasDarren1998`.
#[pyclass(name = "DasDarren1998")]
pub struct PyDasDarren1998(DasDarren1998);

#[pymethods]
impl PyDasDarren1998 {
    #[new]
    pub fn new(
        number_of_objectives: usize,
        number_of_partitions: NumberOfPartitions,
    ) -> PyResult<Self> {
        Ok(Self(DasDarren1998::py_new(
            number_of_objectives,
            number_of_partitions,
        )?))
    }

    pub fn calculate(&self) -> PyResult<Vec<Vec<f64>>> {
        self.0.calculate()
    }

    /// Reference point plot from vector.
    pub fn plot(&self, ref_points: Vec<Vec<f64>>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let fun: Py<PyAny> = get_plot_fun("plot_reference_points", py)?;
            fun.call1(py, (ref_points,))
        })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        self.0.__repr__()
    }

    pub fn __str__(&self) -> String {
        self.0.__str__()
    }
}

#[pymodule(name = "optirustic")]
fn optirustic_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StoppingCondition>()?;
    m.add_class::<PyStoppingConditionValue>()?;
    m.add_class::<SimulatedBinaryCrossoverArgs>()?;
    m.add_class::<PolynomialMutationArgs>()?;
    m.add_class::<ExportHistory>()?;

    m.add_class::<NSGA2Arg>()?;

    m.add_class::<PyDasDarren1998>()?;
    m.add_class::<TwoLayerPartitions>()?;
    m.add_class::<NSGA3Arg>()?;

    m.add_class::<PyAlgorithm>()?;

    m.add_class::<NSGA2Data>()?;
    m.add_class::<NSGA3Data>()?;
    m.add_class::<ObjectiveDirection>()?;
    m.add_class::<RelationalOperator>()?;

    Ok(())
}
