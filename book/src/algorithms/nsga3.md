# NSGA3

The reference-point-based NSGA for many-objective problems, implemented from:

> K. Deb and H. Jain, "An Evolutionary Many-Objective Optimization Algorithm Using
> Reference-Point-Based Non-dominated Sorting Approach, Part I: Solving Problems With Box
> Constraints," IEEE Transactions on Evolutionary Computation, vol. 18, no. 4, pp. 577-601,
> 2014. <https://doi.org/10.1109/TEVC.2013.2281535>

With many objectives almost every solution is non-dominated and the crowding distance used by
NSGA2 stops working. NSGA3 keeps the non-dominated sort but replaces the crowding distance with a
set of well-spread reference points on the normalised hyper-plane. In the survival step it:

1. normalises the objectives using the ideal point and the extreme points (intercepts);
2. associates each individual with the closest reference line (perpendicular distance);
3. runs a niching procedure that picks, from the last front, the individuals linked to the
   reference points with the fewest associated individuals.

This keeps one solution close to each reference direction and gives an evenly spread front.

## Arguments

`NSGA3Arg` has two specific fields plus the [common arguments](index.md#common-arguments):

| Field | Description |
| --- | --- |
| `number_of_individuals` | `Nsga3NumberOfIndividuals::EqualToReferencePointCount`, or `Custom(n)` with `n` at least the number of reference points. |
| `number_of_partitions` | How the reference points are generated (see below). |

The algorithm is created with `NSGA3::new(problem, args, adaptive)`; set `adaptive` to `false`
for plain NSGA3 (or use [`AdaptiveNSGA3`](adaptive_nsga3.md)).

> With many objectives, two distant parents are likely to produce children far from
> both of them. Use a large SBX `distribution_index` (the paper uses `30`) to keep children close
> to their parents.

## Reference points

Reference points are generated with the Das & Dennis (1998) approach in
[`DasDarren1998`](https://docs.rs/nsga_rs/latest/nsga_rs/utils/struct.DasDarren1998.html). You choose
the number of uniform gaps along each objective axis with `NumberOfPartitions`:

- `NumberOfPartitions::OneLayer(p)`: one layer with `p` gaps. With \\( M \\) objectives this
  creates \\( \binom{M + p - 1}{p} \\) points.
- `NumberOfPartitions::TwoLayers(TwoLayerPartitions { boundary_layer, inner_layer, scaling })`:
  a boundary layer plus an inner layer scaled by `scaling` (default `0.5`). Use it with many
  objectives, where one layer would need too many points or would put all points on the boundary.

```rust,ignore
{{#include ../../../examples/reference_points_2layers.rs:layers}}
println!("Total points = {}", m.number_of_points());
let weights = m.get_weights();
```

| One layer, 5 gaps | Two layers (5 and 4 gaps) |
| :---: | :---: |
| ![One layer](../images/ref_points_1layer_3obj_5gaps.png) | ![Two layers](../images/ref_points_2layers_3obj_5gaps.png) |

Sources: [`examples/reference_points_1layer.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/reference_points_1layer.rs)
and [`examples/reference_points_2layers.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/reference_points_2layers.rs).

## Example: DTLZ1 with 3 objectives

This replicates the first test problem of the paper. The optimal solutions lie on the
hyper-plane crossing each objective axis at `0.5`.

```rust,ignore
{{#include ../../../examples/nsga3_dtlz1.rs:setup}}
```

After the run, the exported data gives access to the reference points and to the normalised
objectives with `get_nsga3_reference_points` and `get_nsga3_normalised_objectives`:

| Pareto front | Normalised objectives vs reference points |
| :---: | :---: |
| ![DTLZ1 Pareto front](../images/DTLZ1_3obj_NSGA3_gen400_Pareto_front.png) | ![DTLZ1 reference points](../images/DTLZ1_3obj_NSGA3_gen400_obj_vs_ref_points.png) |

Source: [`examples/nsga3_dtlz1.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/nsga3_dtlz1.rs).
See also `nsga3_dtlz1_8obj.rs` (8 objectives, two layers) and `nsga3_dtlz2.rs`.
