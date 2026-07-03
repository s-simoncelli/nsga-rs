use rand::distr::uniform::SampleUniform;
use rand::prelude::{IndexedRandom, IteratorRandom};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::core::{OError, Problem};

/// A trait to define a decision variable.
pub trait Variable<T>: Display {
    /// Generate a new random value for the variable.
    fn generate(&self) -> T;
    /// Get the variable name
    fn name(&self) -> String;
}

pub trait BoundedNumberTrait: SampleUniform + PartialOrd + Display + Clone {}
impl<T: SampleUniform + PartialOrd + Display + Clone> BoundedNumberTrait for T {}

/// A variable between a lower and upper bound.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(IntoPyObject))]
pub struct BoundedNumber<N: BoundedNumberTrait> {
    /// The variable name
    name: String,
    /// The minimum value bound.
    min_value: N,
    /// The maximum value bound.
    max_value: N,
}

impl<N: BoundedNumberTrait> BoundedNumber<N> {
    /// Create a new decision variable using a number bounded between a lower and upper bound.
    /// When a new value `N` for this variable is generated, the new value will be picked such that
    /// `min_value` <= `N` <= `max_value`.
    ///
    /// # Arguments
    ///
    /// * `name`: The variable name.
    /// * `min_value`: The lower bound.
    /// * `max_value`: The upper bound.
    ///
    /// returns: `Result<BoundedNumber, OError>`
    pub fn new(name: &str, min_value: N, max_value: N) -> Result<Self, OError> {
        if min_value >= max_value {
            return Err(OError::TooLargeLowerBound(
                min_value.to_string(),
                max_value.to_string(),
            ));
        }
        Ok(Self {
            name: name.to_string(),
            min_value,
            max_value,
        })
    }

    /// The variable lower bound.
    ///
    /// return: `N`
    pub fn min_value(&self) -> N {
        self.min_value.clone()
    }

    /// The variable upper bound.
    ///
    /// return: `N`
    pub fn max_value(&self) -> N {
        self.max_value.clone()
    }

    /// The variable upper and lower bound.
    ///
    /// return: `N`
    pub fn bounds(&self) -> (N, N) {
        (self.min_value.clone(), self.max_value.clone())
    }
}

impl<N: BoundedNumberTrait + Copy> Variable<N> for BoundedNumber<N> {
    /// Randomly generate a new bounded number.
    fn generate(&self) -> N {
        let mut rng = rand::rng();
        rng.random_range(self.min_value..=self.max_value)
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<N: BoundedNumberTrait> Display for BoundedNumber<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundedNumber '{}' to [{}; {}]",
            self.name, self.min_value, self.max_value
        )
    }
}

/// A boolean variable.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(IntoPyObject))]
pub struct Boolean {
    /// The variable name.
    name: String,
}

impl Boolean {
    /// Create a new boolean variable.
    ///
    /// # Arguments
    ///
    /// * `name`: The variable name.
    ///
    /// returns: `Boolean`
    pub fn new(name: &str) -> Self {
        Boolean {
            name: name.to_string(),
        }
    }
}
impl Display for Boolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Boolean '{}'", self.name)
    }
}

impl Variable<bool> for Boolean {
    /// Randomly generate a boolean value.
    ///
    /// return: `bool`
    fn generate(&self) -> bool {
        let mut rng = rand::rng();
        !matches!([0, 1].choose(&mut rng).unwrap(), 0)
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

/// A variable choice.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(IntoPyObject))]
pub struct Choice {
    /// The variable name.
    name: String,
    /// The list of choices.
    choices: Vec<String>,
}

impl Choice {
    /// Create a new list of choices.
    ///
    /// # Arguments
    ///
    /// * `name`: The variable name.
    /// * `choices`: The list of choices.
    ///
    /// returns: `Choice`
    pub fn new(name: &str, choices: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            choices,
        }
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Choice '{}': {}", self.name, self.choices.join(", "))
    }
}

impl Variable<String> for Choice {
    /// Randomly pick a choice.
    fn generate(&self) -> String {
        let mut rng = rand::rng();
        let choice_index = (0..self.choices.len()).choose(&mut rng).unwrap();
        self.choices[choice_index].clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

/// The types of variables to set on a problem.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "python", derive(IntoPyObject))]
pub enum VariableType {
    /// A continuous bounded variable (f64)
    Real(BoundedNumber<f64>),
    /// A discrete bounded variable (i64)
    Integer(BoundedNumber<i64>),
    /// A boolean variable
    Boolean(Boolean),
    /// A variable representing a choice (as string)
    Choice(Choice),
}

impl VariableType {
    /// Generate a new random variable value based on its type.
    ///
    /// returns: `VariableValue`
    pub fn generate_random_value(&self) -> VariableValue {
        match &self {
            VariableType::Real(v) => VariableValue::Real(v.generate()),
            VariableType::Integer(v) => VariableValue::Integer(v.generate()),
            VariableType::Boolean(v) => VariableValue::Boolean(v.generate()),
            VariableType::Choice(v) => VariableValue::Choice(v.generate()),
        }
    }

    /// Get the variable name.
    ///
    /// return: `String`
    pub fn name(&self) -> String {
        match self {
            VariableType::Real(t) => t.name.clone(),
            VariableType::Integer(t) => t.name.clone(),
            VariableType::Boolean(t) => t.name.clone(),
            VariableType::Choice(t) => t.name.clone(),
        }
    }

    pub fn label(&self) -> String {
        let label = match &self {
            VariableType::Real(_) => "real",
            VariableType::Integer(_) => "integer",
            VariableType::Boolean(_) => "boolean",
            VariableType::Choice(_) => "choice",
        };
        label.into()
    }

    /// Check if the variable is a real number.
    ///
    /// return: `bool`
    pub(crate) fn is_real(&self) -> bool {
        matches!(self, VariableType::Real(_))
    }

    /// Check if the variable is an integer number.
    ///
    /// return: `bool`
    pub(crate) fn is_integer(&self) -> bool {
        matches!(self, VariableType::Integer(_))
    }
}

impl Display for VariableType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VariableType::Real(v) => write!(f, "{v}")?,
            VariableType::Integer(v) => write!(f, "{v}")?,
            VariableType::Boolean(v) => write!(f, "{v}")?,
            VariableType::Choice(v) => write!(f, "{v}")?,
        };
        Ok(())
    }
}

/// The value of a variable to set on an individual.
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(from_py_object), derive(IntoPyObjectRef))]
#[serde(untagged)]
pub enum VariableValue {
    /// The value for a floating-point number. This is a f64.
    Real(f64),
    /// The value for an integer number. This is an i64.
    Integer(i64),
    /// The value for a boolean variable.
    Boolean(bool),
    /// The value for a choice variable.
    Choice(String),
}

impl PartialEq for VariableValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VariableValue::Real(s), VariableValue::Real(o)) => {
                (s.is_nan() && o.is_nan()) || (*s == *o)
            }
            (VariableValue::Integer(s), VariableValue::Integer(o)) => *s == *o,
            (VariableValue::Boolean(s), VariableValue::Boolean(o)) => s == o,
            (VariableValue::Choice(s), VariableValue::Choice(o)) => s == o,
            _ => false,
        }
    }
}

impl VariableValue {
    /// Check if the variable value matches the variable type set on the problem. This return an
    /// error if the variable name does not exist in the problem.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the variable in the problem.
    /// * `problem`: The problem being solved.
    ///
    /// returns: `Result<bool, OError>`
    pub fn match_type(&self, name: &str, problem: Arc<Problem>) -> Result<bool, OError> {
        let value = match problem.get_variable(name)? {
            VariableType::Real(_) => matches!(self, VariableValue::Real(_)),
            VariableType::Integer(_) => matches!(self, VariableValue::Integer(_)),
            VariableType::Boolean(_) => matches!(self, VariableValue::Boolean(_)),
            VariableType::Choice(_) => matches!(self, VariableValue::Choice(_)),
        };
        Ok(value)
    }

    /// Get the value if the variable is of real type. This returns an error if the variable is not
    /// real.
    ///
    /// returns: `Result<f64, OError>`
    pub fn as_real(&self) -> Result<f64, OError> {
        if let VariableValue::Real(v) = self {
            Ok(*v)
        } else {
            Err(OError::WrongVariableType("real".to_string()))
        }
    }

    /// Get the value if the variable is of discrete type. This returns an error if the variable is
    /// not an integer.
    ///
    /// returns: `Result<f64, OError>`
    pub fn as_integer(&self) -> Result<i64, OError> {
        if let VariableValue::Integer(v) = self {
            Ok(*v)
        } else {
            Err(OError::WrongVariableType("integer".to_string()))
        }
    }
}

impl Debug for VariableValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VariableValue::Real(v) => write!(f, "{v}")?,
            VariableValue::Integer(v) => write!(f, "{v}")?,
            VariableValue::Boolean(v) => write!(f, "{v}")?,
            VariableValue::Choice(v) => write!(f, "{v}")?,
        };
        Ok(())
    }
}

/// Python classes to handle Rust variables with generics.
#[cfg(feature = "python")]
#[pyclass(name = "VariableType", eq, eq_int, from_py_object)]
#[derive(Clone, PartialEq)]
pub enum PyVariableType {
    Real,
    Integer,
    Boolean,
    Choice,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyVariableType {
    pub fn __repr__(&self) -> String {
        let label = match &self {
            PyVariableType::Real => "real",
            PyVariableType::Integer => "integer",
            PyVariableType::Boolean => "boolean",
            PyVariableType::Choice => "choice",
        };
        label.to_string()
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python class holding the data for a problem variable.
#[cfg(feature = "python")]
#[pyclass(get_all, name = "Variable")]
pub struct PyVariable {
    /// The variable name,
    name: String,
    /// The type of variable.
    var_type: PyVariableType,
    /// The optional lower bound.
    min_value: Option<f64>,
    /// The optional upper bound.
    max_value: Option<f64>,
}

/// Convert `&VariableType` to `PyVariable`
#[cfg(feature = "python")]
impl From<&VariableType> for PyVariable {
    fn from(value: &VariableType) -> Self {
        let var_type = match value {
            VariableType::Real(_) => PyVariableType::Real,
            VariableType::Integer(_) => PyVariableType::Integer,
            VariableType::Boolean(_) => PyVariableType::Boolean,
            VariableType::Choice(_) => PyVariableType::Choice,
        };
        let (min_value, max_value) = match value {
            VariableType::Real(n) => (Some(n.min_value()), Some(n.max_value())),
            VariableType::Integer(n) => (Some(n.min_value() as f64), Some(n.max_value() as f64)),
            VariableType::Boolean(_) => (None, None),
            VariableType::Choice(_) => (None, None),
        };

        PyVariable {
            name: value.name(),
            var_type,
            min_value,
            max_value,
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyVariable {
    pub fn __repr__(&self) -> PyResult<String> {
        let args = if let (Some(min_value), Some(max_value)) = (self.min_value, self.max_value) {
            format!(", min-value={min_value}, max_value={max_value}")
        } else {
            String::from("")
        };

        Ok(format!(
            "Variable(name='{}', type='{}'{args})",
            self.name,
            self.var_type.__repr__()
        ))
    }

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}
