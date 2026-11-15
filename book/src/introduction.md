# Introduction

NSGA RS is a Rust framework for solving multi-objective optimisation problems with the NSGA
family of multi-objective evolutionary algorithms (MOEAs).

| NSGA2: 2-objective Schaffer | NSGA3: 3-objective DTLZ1 |
| :---: | :---: |
| ![SCH Pareto front](images/SCH_2obj_NSGA2_Pareto_front.png) | ![DTLZ1 Pareto front](images/DTLZ1_3obj_NSGA3_gen400_Pareto_front.png) |

This guide shows how to use the library, from defining a problem to inspecting the results.
The full API reference is on [docs.rs](https://docs.rs/nsga_rs).

## Features

- Four built-in algorithms: [`NSGA2`](algorithms/nsga2.md), [`NSGA3`](algorithms/nsga3.md),
  [`AdaptiveNSGA3`](algorithms/adaptive_nsga3.md) and [`EpsNSGA2`](algorithms/eps_nsga2.md).
- Flexible problem definition: minimise or maximise any number of objectives, with or
  without constraints.
- Rich variable types: real, integer, boolean and categorical (choice) variables.
- Parallel evaluation: multi-threaded objective and constraint evaluation via Rayon.
- Resumable runs: export the population history as JSON and resume from any checkpoint.
- Hyper-volume metric: calculated from individuals, values or exported JSON files.
- Flexible stopping conditions: by generation, function evaluations, elapsed time, or a
  combination of them.
- Python bindings: a PyO3 interface you can wrap in your own Python package.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
nsga_rs = "1"
```

The `plotting` feature is enabled by default and requires [gnuplot](http://www.gnuplot.info/)
to be installed on your system to generate charts. Disable it if you do not need charts:

```toml
[dependencies]
nsga_rs = { version = "1", default-features = false }
```

| Feature | Description |
| --- | --- |
| `plotting` (default) | Pareto-front and convergence charts via `gnuplot`. |
| `python` | Expose the PyO3 classes (see [Python bindings](python.md)). |
| `python-module` | `python` plus `pyo3/extension-module`, to build a Python extension module. |

## Running the examples

All examples live in the
[`examples/`](https://github.com/s-simoncelli/nsga_rs/tree/main/examples) folder. Run them in
release mode, from the repository root, to speed up the calculation:

```sh
cargo run --example nsga2_sch --release
```

Results and charts are written to `examples/results`. The figures in this guide come from there.
