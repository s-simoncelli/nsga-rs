# NSGA2

The Non-dominated Sorting Genetic Algorithm II, implemented from:

> K. Deb, A. Pratap, S. Agarwal and T. Meyarivan, "A fast and elitist multi-objective genetic
> algorithm: NSGA-II," IEEE Transactions on Evolutionary Computation, vol. 6, no. 2,
> pp. 182-197, 2002. <https://doi.org/10.1109/4235.996017>

At each generation NSGA2:

1. selects parents with a binary tournament using the crowded-comparison operator;
2. creates the offspring with [SBX crossover and polynomial mutation](../operators.md) and
   evaluates them;
3. merges parents and offspring, sorts them into non-dominated fronts (fast non-dominated sort,
   with constrained dominance);
4. fills the next population front by front, breaking ties in the last front with the
   crowding distance, so that less crowded solutions are preferred.

The crowded-comparison operator prefers a solution with a lower rank or, with equal rank, a larger
crowding distance. The distance of each individual is stored in its data under the
`CROWDING_DIST_KEY` key.

## Arguments

`NSGA2Arg` has one specific field plus the [common arguments](index.md#common-arguments):

| Field | Description |
| --- | --- |
| `number_of_individuals` | Population size. Must be a multiple of 2. |

## Example: SCH problem

```rust,ignore
{{#include ../../../examples/nsga2_sch.rs:args}}
```

| Pareto front | Solutions |
| :---: | :---: |
| ![SCH Pareto front](../images/SCH_2obj_NSGA2_Pareto_front.png) | ![SCH solutions](../images/SCH_2obj_NSGA2_solutions.png) |

Source: [`examples/nsga2_sch.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/nsga2_sch.rs).

## Example: ZDT1 problem

[`examples/nsga2_zdt1.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/nsga2_zdt1.rs)
solves the ZDT1 problem with 30 variables in \\( [0, 1] \\):

$$ f_1(x) = x_1 \qquad f_2(x) = g(x) \left[ 1 - \sqrt{x_1 / g(x)} \right] \qquad g(x) = 1 + \frac{9}{n - 1} \sum_{i=2}^{n} x_i $$

```sh
cargo run --example nsga2_zdt1 --release
```
