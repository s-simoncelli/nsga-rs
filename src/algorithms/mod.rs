pub use a_nsga3::AdaptiveNSGA3;
#[cfg(feature = "python")]
pub use algorithm::PyAlgorithm;
pub use algorithm::{
    Algorithm, AlgorithmExport, AlgorithmSerialisedExport, ExportHistory, ExportVecGroupBy,
};
pub use nsga2::{NSGA2Arg, CROWDING_DIST_KEY, NSGA2};
pub use nsga3::{NSGA3Arg, Nsga3NumberOfIndividuals, NSGA3};
#[cfg(feature = "python")]
pub use stopping_condition::py::PyStoppingConditionValue;
pub use stopping_condition::StoppingCondition;

mod a_nsga3;
mod algorithm;
mod nsga2;
mod nsga3;
mod stopping_condition;
