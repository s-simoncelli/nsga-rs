use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Whether the objective should be minimised or maximised. Default is minimise.
#[derive(Default, Clone, Copy, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
pub enum ObjectiveDirection {
    #[default]
    /// Minimise an objective.
    Minimise,
    /// Maximise an objective.
    Maximise,
}

impl Display for ObjectiveDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectiveDirection::Minimise => f.write_str("minimised"),
            ObjectiveDirection::Maximise => f.write_str("maximised"),
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl ObjectiveDirection {
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ObjectiveDirection({self})",))
    }

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

/// Define a problem objective to minimise or maximise.
///
/// # Example
/// ```
///  use nsga_rs::core::{Objective, ObjectiveDirection};
///
///  let o = Objective::new("Reduce cost", ObjectiveDirection::Minimise);
///  println!("{}", o);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct Objective {
    /// The objective name.
    name: String,
    /// Whether the objective should be minimised or maximised.
    direction: ObjectiveDirection,
}

impl Objective {
    /// Create a new objective.
    ///
    /// # Arguments
    ///
    /// * `name`: The objective name.
    /// * `direction`:  Whether the objective should be minimised or maximised.
    ///
    /// returns: `Objective`
    pub fn new(name: &str, direction: ObjectiveDirection) -> Self {
        Self {
            name: name.to_string(),
            direction,
        }
    }

    /// Get the objective name.
    ///
    /// return: `String`
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get the objective direction.
    ///
    /// return: `ObjectiveDirection`
    pub fn direction(&self) -> ObjectiveDirection {
        self.direction
    }
}

impl Display for Objective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Objective '{}' is {}", self.name, self.direction)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Objective {
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Objective(name='{}', direction='{}')",
            self.name, self.direction
        ))
    }

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}
