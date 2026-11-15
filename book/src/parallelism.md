# Parallel evaluation and reproducibility

## Threads

The `threads` argument sets how many threads evaluate the objectives and constraints of the new
individuals at each generation, using a [Rayon](https://docs.rs/rayon) thread pool:

| Value | Description |
| --- | --- |
| `NumThreads::Off` | Evaluate one individual at a time. |
| `NumThreads::Use(n)` | Use `n` threads. |
| `NumThreads::Max` (default) | Use all the logical CPUs available. |

Turn it on when your `Evaluator` is slow (for example, it runs a simulation model). For cheap
functions such as the test problems, the threading overhead can make the run slower, which is why
the examples use `NumThreads::Off`.

Because individuals are evaluated concurrently, your evaluator must be `Sync + Send`. Use
`Arc`/`Mutex` or per-thread copies for any shared state.

## Seed

The algorithms are stochastic: random numbers are used to create the first population, and during
selection, crossover and mutation. Set `seed: Some(n)` to reproduce a run exactly; with `None`
the seed is picked randomly.
