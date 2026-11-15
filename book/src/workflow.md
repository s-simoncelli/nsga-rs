# Workflow

Every optimisation follows the same steps, whichever algorithm you use:

1. Define the problem: objectives, variables, optional constraints, and an `Evaluator` that
   calculates them for a candidate solution. See [Defining a problem](problem.md).
2. Pick an algorithm: `NSGA2` for 2 or 3 objectives, `NSGA3` or `AdaptiveNSGA3` for many
   objectives, `EpsNSGA2` when you want a front with a user-defined resolution and no population
   size to tune. See [Algorithms](algorithms/index.md).
3. Configure the algorithm arguments (`NSGA2Arg`, `NSGA3Arg`, `EpsNSGA2Arg`): population
   size, [stopping condition](stopping_conditions.md), [operators](operators.md),
   [threads and seed](parallelism.md), [history export](export_resume.md).
4. Run it with `algo.run()?`.
5. Get the results with `algo.get_results()` or export them with `algo.save_to_json(..)`.
6. Inspect the results: plot the Pareto front and track convergence with the
   [hyper-volume](metrics_plotting.md).
7. Optionally, resume the run from an exported file with `resume_from_file`.

## A complete example

The example below solves the Schaffer problem (SCH), where the following two objectives are
minimised:

$$ f_1(x) = x^2 \qquad f_2(x) = (x - 2)^2 $$

with \\( -1000 \leq x \leq 1000 \\). The optimal solutions lie in the \\( [0, 2] \\) range.

### 1. Define the problem

The problem is available as `nsga_rs::core::builtin_problems::SCHProblem` and is implemented as:

```rust,ignore
{{#include ../../src/core/problem.rs:sch_problem}}
```

### 2. Configure and run the algorithm

```rust,ignore
{{#include ../../examples/nsga2_sch.rs:args}}
```

### 3. Export and plot the results

```rust,ignore
{{#include ../../examples/nsga2_sch.rs:export}}
```

`save_to_json` writes `SCH_2obj_NSGA2_gen250.json`, which contains the problem, the algorithm
options and all individuals (variables, objectives, constraints). `plot_front` reads it back and
draws the Pareto front:

![SCH Pareto front](images/SCH_2obj_NSGA2_Pareto_front.png)

The exported data can be converted into an `AlgorithmExport` to access variables and objectives
by name (`get_real_variables("x")`, `get_objective("x^2")`) and draw your own charts. The example
plots the solutions over the two objective functions:

![SCH solutions](images/SCH_2obj_NSGA2_solutions.png)

The full source is in
[`examples/nsga2_sch.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/nsga2_sch.rs).
Run it with:

```sh
cargo run --example nsga2_sch --release
```
