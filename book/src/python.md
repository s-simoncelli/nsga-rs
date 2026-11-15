# Python bindings

The library includes a [PyO3](https://pyo3.rs/) interface so you can wrap it, together with your
optimisation problem, in your own Python package built with [Maturin](https://www.maturin.rs/).
There is no pre-built package on PyPI: the bindings are a starting point for you to build and
distribute.

## Features

| Feature | Use |
| --- | --- |
| `python` | Adds the `#[pyclass]` attributes to the public types. |
| `python-module` | `python` plus `pyo3/extension-module`. Use it when building the extension module. |

```toml
[dependencies]
nsga_rs = { version = "1", features = ["python-module"] }
```

## What is exposed

With the `python` feature the following types become Python classes, which you register in your
own `#[pymodule]`:

- the algorithm arguments: `NSGA2Arg`, `NSGA3Arg`, `EpsNSGA2Arg`, plus `StoppingCondition`
  (`max_generation`, `max_function_evaluations`, `max_duration_as_minutes`,
  `max_duration_as_hours`; a list of conditions is converted to `Any`), `NumThreads`,
  `ExportHistory`, `SimulatedBinaryCrossoverArgs`, `PolynomialMutationArgs`,
  `TwoLayerPartitions`, `DasDarren1998`, `IntraRunStoppingCondition`,
  `InterRunStoppingCondition`;
- the problem types: `Problem`, `Objective`, `ObjectiveDirection`, `Constraint`,
  `RelationalOperator`, `Variable` and `VariableType`;
- the readers of the exported JSON files, `NSGA2Data` and `NSGA3Data`, with the problem, the
  individuals, `generation`, `algorithm`, `exported_on`, `hyper_volume(reference_point)`,
  `estimate_reference_point(offset)` and the convergence data;
- `PyAlgorithm` (named `Algorithm` in Python, with the `nsga2`, `nsga3` and `adaptive_nsga3`
  variants), an enum to pass the chosen algorithm and its options from Python to your Rust code.

A typical package exposes these classes plus a function that receives the algorithm options from
Python, builds your `Problem` in Rust and runs the algorithm:

```python
args = NSGA2Arg(
    number_of_individuals=100,
    stopping_condition=StoppingCondition.max_generation(250),
)
algorithm = Algorithm.nsga2(args)  # PyAlgorithm
my_package.run(algorithm)          # your Rust function
```

## Testing

The bindings are tested with:

```sh
cargo test --package nsga_rs --lib --no-default-features --features python-test -- 'algorithms::algorithm::test_python_api'
```
