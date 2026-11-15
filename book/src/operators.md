# Genetic operators

All algorithms create new individuals with the same operators. Pass `None` in the algorithm
arguments to use the defaults.

## Selection

Parents are picked with a binary tournament
([`TournamentSelector`](https://docs.rs/nsga_rs/latest/nsga_rs/operators/struct.TournamentSelector.html)).
NSGA2 uses the crowded-comparison operator (rank first, then crowding distance); NSGA3 uses the
constrained Pareto dominance (`ParetoConstrainedDominance`).

## Simulated Binary Crossover (SBX)

SBX recombines the variables of two parents into two children. Set it with
`crossover_operator_options: Some(SimulatedBinaryCrossoverArgs { .. })`.

| Field | Default | Description |
| --- | --- | --- |
| `distribution_index` | `15.0` | \\( \eta_c \\). Large values give children close to their parents; small values give distant children. |
| `crossover_probability` | `1.0` | Probability that the parents take part in the crossover. |
| `variable_probability` | `0.5` | Probability that a variable is crossed over. |

```rust,ignore
use nsga_rs::operators::SimulatedBinaryCrossoverArgs;

let crossover_operator_options = SimulatedBinaryCrossoverArgs {
    distribution_index: 30.0,
    crossover_probability: 1.0,
    ..SimulatedBinaryCrossoverArgs::default()
};
```

## Polynomial Mutation (PM)

PM perturbs the variables of a child. Set it with
`mutation_operator_options: Some(PolynomialMutationArgs { .. })`.

| Field | Default | Description |
| --- | --- | --- |
| `index_parameter` | `20.0` | \\( \eta_m \\). Values in `[20, 100]` are suggested. |
| `variable_probability` | `1 / n` | Probability of mutating a variable, with `n` the number of real and integer variables. |

The default depends on the problem, so it is built with `PolynomialMutationArgs::default(&problem)`.
