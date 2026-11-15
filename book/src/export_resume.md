# Exporting and resuming runs

## Final results

After `run()`, get the results in memory with `algo.get_results()`, or write them to a JSON file:

```rust,ignore
let file = algo.save_to_json(&destination_folder, Some("SCH_2obj"))?;
// -> <destination_folder>/SCH_2obj_NSGA2_gen250.json
```

The file is named `<prefix>_<algorithm>_gen<generation>.json` and contains:

- the problem (objectives, variables, constraints);
- the algorithm name and options (the `*Arg` struct);
- the generation, the number of function evaluations, the elapsed time and the export time;
- every individual with its variables, objectives, constraint values and algorithm data (for
  example the crowding distance or the NSGA3 reference point association).

## Reading exported data

Read a file back with the `read_json_file` of the algorithm that produced it:

```rust,ignore
use nsga_rs::algorithms::{Algorithm, AlgorithmExport, NSGA2};

let data = NSGA2::read_json_file(&file)?;
data.plot_front(&out_path.join("front.png"), None)?; // needs the `plotting` feature

let export: AlgorithmExport = data.try_into()?;
let x = export.get_real_variables("x")?;
let f1 = export.get_objective("x^2")?;
```

`NSGA2::read_json_files(&folder)` reads all the history files in a folder (see below).

## History

Set `export_history` to save the population every `generation_step` generations. This lets you
inspect the evolution and [track convergence](metrics_plotting.md), and it is a checkpoint if a
long run crashes.

```rust,ignore
use nsga_rs::algorithms::ExportHistory;

// export a JSON file every 100 generations; the folder must exist
let export_history = ExportHistory::new(100, &out_path)?;
let args = NSGA2Arg {
    export_history: Some(export_history),
    // ...
};
```

The algorithm writes:

- `Init_<algorithm>_gen1.json` with the initial population;
- `History_<algorithm>_gen<N>.json` every `generation_step` generations;
- `Final_<algorithm>_gen<N>.json` when the stopping condition is met.

For example, the [convergence example](metrics_plotting.md#tracking-convergence) produces
`History_NSGA2_gen100.json` … `History_NSGA2_gen900.json` and `Final_NSGA2_gen1000.json`.

## Resuming a run

Set `resume_from_file` to start from the population stored in an exported file (a history file
or a final file), instead of a random population:

```rust,ignore
let args = NSGA2Arg {
    resume_from_file: Some(PathBuf::from("results/History_NSGA2_gen500.json")),
    // ...
};
```

The file must come from the same problem: the algorithm returns an error if the number of
variables differs, or if the number of individuals in the file differs from the population
size. `EpsNSGA2` takes the population size from the file.
