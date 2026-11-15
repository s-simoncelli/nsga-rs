# Stopping conditions

The `stopping_condition` field of the algorithm arguments tells the algorithm when to stop
evolving the population. It is a
[`StoppingCondition`](https://docs.rs/nsga_rs/latest/nsga_rs/algorithms/enum.StoppingCondition.html):

| Variant | Stops when |
| --- | --- |
| `MaxGeneration(n)` | the generation counter reaches `n`. |
| `MaxFunctionEvaluations(n)` | the evaluator has been called `n` times. |
| `MaxDurationAsMinutes(m)` | `m` minutes have elapsed since the run started. |
| `MaxDurationAsHours(h)` | `h` hours have elapsed since the run started. |
| `Any(vec![..])` | at least one of the conditions is met (OR). |
| `All(vec![..])` | all the conditions are met (AND). |

```rust,ignore
use nsga_rs::algorithms::StoppingCondition;

// stop at 250 generations
let stopping_condition = StoppingCondition::MaxGeneration(250);

// stop at 1000 generations or after 2 hours, whichever comes first
let stopping_condition = StoppingCondition::Any(vec![
    StoppingCondition::MaxGeneration(1000),
    StoppingCondition::MaxDurationAsHours(2),
]);
```

`Any` and `All` cannot be nested: a run with an `Any` or `All` inside another one returns an error.

The condition is checked after each generation. When it is met, the algorithm logs which
condition stopped it and, if [history export](export_resume.md) is on, saves a `Final_*` file.
With a single condition, the log also shows an estimate of the time left.

## Choosing a condition

- Use `MaxGeneration` to replicate a paper or compare algorithms on equal terms.
- Use `MaxFunctionEvaluations` when the evaluator is expensive and you have a budget of model
  runs. This is also fairer when comparing algorithms with different population sizes, such as
  [`EpsNSGA2`](algorithms/eps_nsga2.md), whose population changes size during the run.
- Use the duration variants to fit the run in a time slot; combine them with `Any` so the run also
  stops when the target generation is reached first.
- To stop when the front stops improving, export the history and track the
  [hyper-volume](metrics_plotting.md).

`EpsNSGA2` has its own [intra-run and inter-run conditions](algorithms/eps_nsga2.md#intra-run-stopping-condition)
on top of this one.
