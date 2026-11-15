# Algorithms

| Algorithm | Arguments | Best for |
| --- | --- | --- |
| [`NSGA2`](nsga2.md) | `NSGA2Arg` | 2 or 3 objectives |
| [`NSGA3`](nsga3.md) | `NSGA3Arg` | many objectives (4+) |
| [`AdaptiveNSGA3`](adaptive_nsga3.md) | `NSGA3Arg` | many objectives where some reference points do not intersect the front |
| [`EpsNSGA2`](eps_nsga2.md) | `EpsNSGA2Arg` | a front with a chosen resolution, with an automatically sized population |

All algorithms implement the
[`Algorithm`](https://docs.rs/nsga_rs/latest/nsga_rs/algorithms/trait.Algorithm.html) trait,
so they are used the same way:

```rust,ignore
use nsga_rs::algorithms::Algorithm;

let mut algo = NSGA2::new(Arc::new(problem), args)?;
algo.run()?;                                  // evolve until the stopping condition is met
let results = algo.get_results();             // AlgorithmExport with the final individuals
let file = algo.save_to_json(&dir, Some("my_run"))?; // export to JSON
```

## Common arguments

Each `*Arg` struct has algorithm-specific fields plus these common fields:

| Field | Type | Description |
| --- | --- | --- |
| `stopping_condition` | `StoppingCondition` | When to stop. See [Stopping conditions](../stopping_conditions.md). |
| `crossover_operator_options` | `Option<SimulatedBinaryCrossoverArgs>` | SBX options; `None` for defaults. See [Genetic operators](../operators.md). |
| `mutation_operator_options` | `Option<PolynomialMutationArgs>` | PM options; `None` for defaults. |
| `threads` | `NumThreads` | Parallel evaluation. See [Parallel evaluation](../parallelism.md). |
| `export_history` | `Option<ExportHistory>` | Export the population every N generations. See [Exporting and resuming](../export_resume.md). |
| `resume_from_file` | `Option<PathBuf>` | Seed the initial population from an exported JSON file. |
| `seed` | `Option<u64>` | Seed of the random number generator, to reproduce results. |

The argument structs are serialisable and are stored in every exported JSON file.
