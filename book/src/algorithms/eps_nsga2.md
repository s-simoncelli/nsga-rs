# ε-NSGA2

`EpsNSGA2` is the ε-dominance NSGA2 with population doubling, 25% injection and adaptive
population sizing, implemented from:

> J. B. Kollat and P. M. Reed, "The value of online adaptive search: a performance comparison of
> NSGAII, ε-NSGAII and εMOEA," EMO 2005, pp. 386-398.
> <https://doi.org/10.1007/978-3-540-31880-4_27>

> J. B. Kollat and P. M. Reed, "Comparing state-of-the-art evolutionary multi-objective
> algorithms for long-term groundwater monitoring design," Advances in Water Resources, vol. 29,
> no. 6, pp. 792-807, 2006. <https://doi.org/10.1016/j.advwatres.2005.07.010>

Dynamic population sizing for NSGA2 was first presented in Reed, Minsker and Goldberg (2003),
<https://doi.org/10.1029/2002WR001483>.

## How it works

### ε-dominance archive

The objective space is split into hyper-boxes of size
\\( \epsilon_j \\) for each objective \\( j \\). Each solution is placed in the box with
coordinates \\( B_j = \lfloor f_j / \epsilon_j \rfloor \\). The archive (the algorithm's
population) keeps at most one solution per box. Each new offspring is compared with the archive:

- if an archived solution ε-dominates it, the offspring is rejected;
- if it ε-dominates archived solutions, those are removed and the offspring is accepted;
- if it falls in the same box as an archived solution, the one closest to the box corner is kept;
- otherwise it lies in a new non-dominated box and is accepted.

Smaller ε values give a finer grid and more solutions on the front. You set the resolution of the
front instead of a population size.

### Intra-runs and inter-runs

The algorithm runs a sequence of NSGA2 runs (intra-runs). It
starts with a small population (`10` by default). Every time an intra-run stops, the population
is resized to four times the archive size (at least `10` individuals): the archive makes up 25%
of the new population (25% injection), the remaining 75% are new random individuals, and a new
NSGA2 run starts.

## Arguments

`EpsNSGA2Arg` has these specific fields plus the [common arguments](index.md#common-arguments):

| Field | Description |
| --- | --- |
| `eps` | Hyper-box size for each objective. One value per objective. |
| `number_of_initial_individuals` | Size of the first population. Defaults to `10`. |
| `intra_run_stop_cond` | When an NSGA2 intra-run stops. Defaults to `IntraRunStoppingCondition::default()`. |
| `inter_run_stopping_condition` | Optional early stop of the whole algorithm when the archive stops growing. |

The overall `stopping_condition` also applies within an intra-run: the algorithm stops as soon as
it is met.

### Intra-run stopping condition

An intra-run stops when the archive does not grow enough over a lag window:

$$ \frac{|K - A|}{A} \leq \text{threshold} $$

where \\( A \\) is the archive size when the lag window is reached and \\( K \\) the archive size
one lag window before. The paper uses a longer window for the first run, because the archive grows
more slowly from the small initial population.

```rust,ignore
use nsga_rs::algorithms::IntraRunStoppingCondition;

let intra = IntraRunStoppingCondition::new(
    50,   // lag window for the first intra-run
    10,   // lag window for the next intra-runs
    0.1,  // threshold
    250,  // maximum generations of an intra-run, regardless of archive gains
)?;
```

These are also the defaults.

### Inter-run stopping condition

`InterRunStoppingCondition::Threshold(t)` stops the algorithm when the archive size changes by no
more than `t` between two consecutive intra-runs (Eq. 1 in Kollat & Reed (2005), which uses `0.1`).
It is off by default (`None` or `InterRunStoppingCondition::Off()`), because it can stop the run
early with fewer solutions than you would like.

## Example: SCH with two resolutions

```rust,ignore
{{#include ../../../examples/eps_nsga2_sch.rs:args}}
```

The example solves the problem with ε = 0.1 and ε = 0.3. A larger box gives fewer, more spaced
solutions:

| | Pareto front | Solutions |
| :---: | :---: | :---: |
| ε = 0.1 | ![Front 0.1](../images/SCH_2obj_0.1_EpsNSGA2_Pareto_front.png) | ![Solutions 0.1](../images/SCH_2obj_0.1_EpsNSGA2_solutions.png) |
| ε = 0.3 | ![Front 0.3](../images/SCH_2obj_0.3_EpsNSGA2_Pareto_front.png) | ![Solutions 0.3](../images/SCH_2obj_0.3_EpsNSGA2_solutions.png) |

Source: [`examples/eps_nsga2_sch.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/eps_nsga2_sch.rs).

## Example: DTLZ1 with 3 objectives

[`examples/eps_nsga2_dtlz1.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/eps_nsga2_dtlz1.rs)
solves DTLZ1 with `eps: [0.05; 3].to_vec()`:

![DTLZ1 Eps-NSGA2 Pareto front](../images/DTLZ1_3obj_EpsNSGA2_Pareto_front.png)
