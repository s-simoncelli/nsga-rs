# Hyper-volume and plotting

## Hyper-volume

The hyper-volume is the volume of the objective space dominated by the Pareto front and bounded
by a reference (anti-optimal) point. It grows as the front gets closer to the optimum and better
spread, so it measures both convergence and diversity.
[`HyperVolume`](https://docs.rs/nsga_rs/latest/nsga_rs/metrics/struct.HyperVolume.html) picks an
exact method from the number of objectives:

| Objectives | Method |
| --- | --- |
| 2 | Sum of the rectangle areas between each point and the reference point. |
| 3 | [Fonseca et al. (2006)](http://dx.doi.org/10.1109/CEC.2006.1688440) (`HyperVolumeFonseca2006`). |
| 4+ | [While et al. (2012)](http://dx.doi.org/10.1109/TEVC.2010.2077298) (`HyperVolumeWhile2012`, WFG). |

The reference point must be dominated by every point of the front. Both minimised and maximised
objectives are handled.

You can calculate it from:

| Source | Function |
| --- | --- |
| individuals | `HyperVolume::from_individuals(&mut individuals, &ref_point)` |
| objective values | `HyperVolume::from_values(..)` |
| an exported JSON file | `HyperVolume::from_file(&data, &ref_point)` |
| a folder of JSON files | `HyperVolume::from_files(&all_data, &ref_point)` |

If you do not know which reference point to use, estimate it from the worst value of each
objective, plus an optional offset, with `HyperVolume::estimate_reference_point_from_file` or
`HyperVolume::estimate_reference_point_from_files`. Use the same reference point for all
generations, otherwise the values cannot be compared.

## Tracking convergence

Export the history (see [Exporting and resuming runs](export_resume.md)), calculate the
hyper-volume of each file and plot it. When it stops growing, the algorithm has converged.

```rust,ignore
{{#include ../../examples/convergence.rs:28:}}
```

![SCH convergence chart](images/SCH_2obj_NSGA2_convergence_chart.png)

Source: [`examples/convergence.rs`](https://github.com/s-simoncelli/nsga_rs/blob/main/examples/convergence.rs).

```sh
cargo run --example convergence --release
```

## Plotting

With the `plotting` feature (on by default, needs `gnuplot` installed):

- `plot_front(&path, None)` on data read with `read_json_file` draws the Pareto front for problems
  with 2 or 3 objectives.
- `HyperVolume::plot_from_files(&data, &ref_point, &path)` draws the convergence chart above.

For other charts, convert the exported data into an `AlgorithmExport` and use its getters:

| Getter | Returns |
| --- | --- |
| `get_real_variables(name)` | the values of a real variable for all individuals |
| `get_objective(name)` / `get_objectives()` | the values of one or all objectives |
| `get_nsga3_reference_points(ExportVecGroupBy::Objective)` | the NSGA3 reference points |
| `get_nsga3_normalised_objectives(ExportVecGroupBy::Objective)` | the NSGA3 normalised objectives |
| `get_adaptive_nsga3_reference_points(..)` | the original and new A-NSGA3 reference points |

The examples use the [`gnuplot`](https://docs.rs/gnuplot) crate to draw the solution and reference
point charts in this guide, but any plotting library works with these vectors.
