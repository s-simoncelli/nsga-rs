# Adaptive NSGA3

`AdaptiveNSGA3` is NSGA3 with adaptive reference points, implemented from:

> H. Jain and K. Deb, "An Evolutionary Many-Objective Optimization Algorithm Using
> Reference-Point Based Non dominated Sorting Approach, Part II: Handling Constraints and
> Extending to an Adaptive Approach," IEEE Transactions on Evolutionary Computation, vol. 18,
> pp. 602-622, 2014. <https://doi.org/10.1109/TEVC.2013.2281534>

## When to use it

NSGA3 tries to associate one individual with each reference point. On some problems (for
example the inverted DTLZ1) many reference lines never intersect the Pareto front: those points
stay empty, while the points that do intersect it get crowded. The adaptive approach
(Section VII of the paper) moves the reference points towards the front in two steps:

1. Addition: around each crowded reference point (niche count \\( \rho_j \geq 2 \\)), three
   new points are added with the same gap. A new point is kept only if it lies in the first
   quadrant (all coordinates between 0 and 1) and does not overlap an existing point.
2. Deletion: when every reference point has a niche count of 1 (the ideal case), the added
   points with no associated individual are removed. Original points are always kept.

## Usage

It takes the same [`NSGA3Arg`](nsga3.md#arguments) arguments:

```rust,ignore
{{#include ../../../examples/adaptive_nsga3_inverted_dtlz1.rs:init}}
```

`AdaptiveNSGA3::new(problem, args)` is the same as `NSGA3::new(problem, args, true)`.

After the run, `get_adaptive_nsga3_reference_points` on the exported data splits the points into
the original ones and the new adaptive ones:

```rust,ignore
{{#include ../../../examples/adaptive_nsga3_inverted_dtlz1.rs:ref_points}}
// ref_points.original and ref_points.new
```

## Example: inverted DTLZ1

| Pareto front | Original (red) and adaptive (blue) reference points |
| :---: | :---: |
| ![Inverted DTLZ1 Pareto front](../images/DTLZ1_3obj_Adaptive_NSGA3_gen400_Pareto_front.png) | ![Adaptive reference points](../images/DTLZ1_3obj_Adaptive_NSGA3_gen400_obj_vs_ref_points.png) |

Source: [`examples/adaptive_nsga3_inverted_dtlz1.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/adaptive_nsga3_inverted_dtlz1.rs).

```sh
cargo run --example adaptive_nsga3_inverted_dtlz1 --release
```
