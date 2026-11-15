# Defining a problem

A problem is described by a [`Problem`](https://docs.rs/nsga_rs/latest/nsga_rs/core/struct.Problem.html):

$$ \min/\max \\; \big(f_1(x), f_2(x), \ldots, f_M(x)\big) $$

where \\( M \geq 1 \\) is the number of objectives and \\( x \\) the \\( N \\)-variable solution
vector bounded to \\( x_i^{(L)} \leq x_i \leq x_i^{(U)} \\). The problem can also be subject to
inequality constraints \\( g_j(x) \geq 0 \\) and equality constraints \\( h_k(x) = 0 \\).

You build it with:

```rust,ignore
Problem::new(objectives, variables, constraints, evaluator)
```

All names (objectives, variables, constraints) must be unique; `Problem::new` returns an error
otherwise, or when there are no objectives or no variables.

## Objectives

```rust,ignore
use nsga_rs::core::{Objective, ObjectiveDirection};

let objectives = vec![
    Objective::new("cost", ObjectiveDirection::Minimise),
    Objective::new("reliability", ObjectiveDirection::Maximise),
];
```

Internally every objective is minimised: maximised objectives are stored with the opposite sign
and converted back when results are exported.

## Variables

Each variable is a [`VariableType`](https://docs.rs/nsga_rs/latest/nsga_rs/core/enum.VariableType.html):

| Type | Constructor | Value accessor in the evaluator |
| --- | --- | --- |
| Real | `VariableType::Real(BoundedNumber::new("x", -1.0, 1.0)?)` | `.as_real()?` |
| Integer | `VariableType::Integer(BoundedNumber::new("n", 0, 10)?)` | `.as_integer()?` |
| Boolean | `VariableType::Boolean(Boolean::new("flag"))` | match `VariableValue::Boolean(b)` |
| Choice | `VariableType::Choice(Choice::new("colour", vec!["red".into(), "blue".into()]))` | match `VariableValue::Choice(s)` |

`BoundedNumber::new` returns an error when the minimum is not smaller than the maximum.

## Constraints

A [`Constraint`](https://docs.rs/nsga_rs/latest/nsga_rs/core/struct.Constraint.html) compares a
value calculated by the evaluator against a target using a
[`RelationalOperator`](https://docs.rs/nsga_rs/latest/nsga_rs/core/enum.RelationalOperator.html):
`EqualTo`, `NotEqualTo`, `LessOrEqualTo`, `LessThan`, `GreaterOrEqualTo` or `GreaterThan`.

```rust,ignore
use nsga_rs::core::{Constraint, RelationalOperator};

// g(x) >= 0
let constraints = vec![Constraint::new("g", RelationalOperator::GreaterOrEqualTo, 0.0)];
```

`Constraint::new_with_modifiers(name, operator, target, scale, offset)` scales and then offsets
the target.

When a constraint is not met, its violation is the absolute distance between the value and the
target (`NotEqualTo` counts as `1`). The violations of an individual are summed in
`Individual::constraint_violation()`; a solution is feasible when this is `0`.

The algorithms rank solutions with the constrained-dominance operator (Deb & Agrawal, 2002).
A solution \\( S_1 \\) is preferred to \\( S_2 \\) when:

1. \\( S_1 \\) is feasible and \\( S_2 \\) is not;
2. both are infeasible and \\( S_1 \\) has the smaller constraint violation; or
3. both are feasible and \\( S_1 \\) Pareto-dominates \\( S_2 \\).

## The evaluator

The `Evaluator` trait is where your model runs. Each time the algorithm creates new individuals,
it calls `evaluate` with an [`Individual`](https://docs.rs/nsga_rs/latest/nsga_rs/core/struct.Individual.html)
holding the proposed variables. The function must return the value of every objective and
constraint in an `EvaluationResult`, keyed by name; the algorithm returns an error otherwise.

```rust,ignore
use std::collections::HashMap;
use std::error::Error;
use nsga_rs::core::{EvaluationResult, Evaluator, Individual};

#[derive(Debug)]
struct MyModel;

impl Evaluator for MyModel {
    fn evaluate(&self, i: &Individual) -> Result<EvaluationResult, Box<dyn Error>> {
        let x = i.get_variable_value("x")?.as_real()?;

        let mut objectives = HashMap::new();
        objectives.insert("cost".to_string(), x.powi(2));
        objectives.insert("reliability".to_string(), 1.0 - (x - 2.0).abs());

        let mut constraints = HashMap::new();
        constraints.insert("g".to_string(), x + 1.0);

        Ok(EvaluationResult {
            objectives,
            constraints: Some(constraints), // `None` for unconstrained problems
        })
    }
}
```

The trait requires `Sync + Send + Debug`, because individuals can be
[evaluated in parallel](parallelism.md). Any error you return stops the algorithm.

Finally wrap the problem in an `Arc` and pass it to the algorithm:

```rust,ignore
let problem = Problem::new(objectives, variables, Some(constraints), Box::new(MyModel))?;
let mut algo = NSGA2::new(Arc::new(problem), args)?;
```

## Built-in test problems

`nsga_rs::core::builtin_problems` contains the classic benchmarks used in the examples and tests:

| Problem | Objectives | Notes |
| --- | --- | --- |
| `SCHProblem` | 2 | Schaffer, 1 variable |
| `FonProblem` | 2 | Fonseca and Fleming |
| `ZTD1Problem`, `ZTD2Problem`, `ZTD3Problem`, `ZTD4Problem`, `ZTD6Problem` | 2 | Zitzler et al. (2000) |
| `DTLZ1Problem` | any | Deb et al.; constrained, optionally inverted |
| `DTLZ2Problem` | any | Deb et al. |

Their source in [`src/core/problem.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/src/core/problem.rs)
is a good template for your own problems. For example, `DTLZ1Problem` shows a problem with a
variable number of objectives and a constraint.
