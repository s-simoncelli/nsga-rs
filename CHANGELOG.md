## [1.5.0] - 2026-11-15

### 🚀 Features

- Added ability to specify custom function in StoppingCondition to stop the algorithm evolution. The Python API was also tweaked to allow conversion from PyStoppingConditionMap
- Moved common fields (crossover_operator_options, mutation_operator_options, resume_from_file and seed) used to initialise algorithms into as_algorithm_args proc macro
- All algorithms require Arc<Problem> for initialisation so that Problem can be cloned
- Added DataValue::IntVector to add support for i64 data for Individual
- Fast_non_dominated_sort can also now calculate and sort the Pareto fronts using custom data stored in Individual
- Added euclidean_distance distance in utils crate so that it can be reused
- Added support to generics in #[algorithm] proc macro
- Added after_evolve field, and set_after_evolve and set_stopping_condition_as_any method in NSGA2 so that the algorithm can be integrated within Esp-NSGA2 under development
- Algorithm::evolve now returns EvolveStatus which determine whether to stop the evolution
- Added ϵ-Domination NSGA2 algorithm with tests and examples
- Added a new stopping condition that stops an algorithm when the relative change of the hyper-volume between the current generation and delta generations is below a threshold

### 🐛 Bug Fixes

- Use Mutex in custom StoppingCondition to ensure support via Python
- A custom stopping condition function receives the generation and NFE numbers
- Fixed missing IntVector in IntoPYObject trait for DataValue
- Include all path segments in algorithm_trait_items
- Require published version of nsga-rs-macros
- Fixed StoppingCondition::MaxDurationAsMinutes and StoppingCondition::MaxDurationAsHours to use start time instead of current time to assess the duration

### 📚 Documentation

- Fixed commands in examples files

### ⚙️ Miscellaneous Tasks

- Renamed proc macros in nsga-rs-macros crate
- Moved algorithm_options_as_str to algorithm crate as function. Added Algorithm::generation_as_ref()
- Moved Pareto constraint dominance logic to utils module so that the function can be reused
- NSGA2.set_after_evolve now can return EvolveStatus. Also added set_opulation method
- StoppingCondition::Function was removed as this can be handled by Algorithm.after_evolve
- The population item can now be access at crate level
- Bumped nsga-rs-macros version
- NSGA2::set_crowding_distance is not public in crate
- Fixed publish pipeline tagging
- The publish CI now bumps the nsga_rs crate version
- Remove wrong code from eps-nsga2 DTLZ1 example
- Removed unused imports in eps_nsga2::mod
- Use github app to bypass PR restrictions when publishing
## [1.4.1] - 2026-07-18

### 🚀 Features

- Added Individual::get_feasible_objective_values to get vector with feasible objectives

### ⚙️ Miscellaneous Tasks

- Renamed HyperVolume::from_individual to HyperVolume::from_individuals
- *(release)* 1.4.1
## [1.4.0] - 2026-07-18

### 🚀 Features

- Added option to specify the number of threads for parallel evaluation using `NumThreads`. Previously it was only possible to use no threads or the available max
- Load pool thread only once and do not set max threads on unlimited threads
- Added estimation of time left to reach stopping condition with max duration, generation and function evaluations

### 🐛 Bug Fixes

- Fixed syntax error in NumThreads to py object
- Replaced "parallel" option with "threads" in example results
- Replaced "parallel" option with "threads" in example history results
- In NSGA summary print correct vaue of variable probability of mutation operator
- In mutation operator, calculate the variable probability by also including integer types

### 🧪 Testing

- Fixed tests when checking for ref point error string
- Fixed metrics::hypervolume_2d::test::test_non_dominated_solutions_min_objectives when checking for error

### ⚙️ Miscellaneous Tasks

- Fixed tagging
- Update pyo3 min version to 0.28.0
- Manually implemented IntoPyObject trait for DataValue as pyo3 derive macros do not work with self-referenced enums
- Improve convergence example to use offset to make sure it never return Err due to result randomness
- Increase offset in convergence example to avoid random errors in tests due to RNG
- *(release)* 1.4.0
## [1.3.0] - 2026-07-03

### 🚀 Features

- Moved plotting functionalities to rust with gnuplot
- Added get_min_objectives and get_max_objectives methods to Population and Individuals structs
- Added create_py_reader_interface macro to export Python reader and added tests

### 🐛 Bug Fixes

- Return Population after initialisation in new init method

### 🧪 Testing

- Fixed python test as new JSON files are generated after algorithms run
- Reset repo after rust tests
- Moved python test to separate job
- Fixed python test when checking for exported_on

### ⚙️ Miscellaneous Tasks

- Tagged repo with correct version
- Fix PYI file formatting
- Renamed project to nsga_rs
- Removed python project as this can be bundle using a custom pyo3 project with this lib
- Updated README with new name and made minor improvements to its structure
- Added documentation to Hypervolume::plot_from_files
- Fixed rust CI and disabled Python pipeline
- Required pyo3 0.27.0
- Bumped version to 1.2.3
- Always run tests with --all-features flag
- Install gnuplot on github CI
- Use sudo in CI to install gnuplot
- Do not hard-code rank and crowding distance keys in BinaryComparisonOperator
- Use from_py_object with pyclass
- Enable all features in VSCode rust analyser
- Added git-cliff to bump version and generate changelog file
- Added new publish CI to bump version, generate changelog and publish crate
- Fixed publish CI to calculate tag on bump
- Tag repo in publish CI
- Change Cargo.toml version after bumping
- *(release)* V1.3.0
## [1.2.2] - 2025-03-22

### 🚀 Features

- Added StoppingCondition::MaxDurationAsMinutes StoppingCondition::MaxDurationAsHours to stop the algorithm after a specified number of minutes or hours

### ⚙️ Miscellaneous Tasks

- Updated changelog for version 1.2.1
## [1.2.1] - 2025-03-16

### ⚙️ Miscellaneous Tasks

- Bumped optirustic-macro version
- Made pyo3-build-config dependency optional
- Use new macro dep version from crate.io
## [1.2.0] - 2025-03-16

### 🚀 Features

- Added "python" feature to enable Python API. Added new API in Python package

### 🐛 Bug Fixes

- Fixed NSGA3 example with new Rust API
- Fixed dev dependencies installation in pyproject.toml file
- Fixed package path in Python pipeline

### ⚙️ Miscellaneous Tasks

- Run Rust test with default features
- Fixed Python pipeline to run ruff
- Use optirustic-py path in Python CI
- Updated changelog file to include changes up to version 1.2.0
## [1.1.3] - 2025-02-26

### 🐛 Bug Fixes

- Fixed SBX test with integer values by changing the seed number after rnf update. The old seed truncates the integers.
- Ensure that with integer values the crossover and mutation operators do not exceed the var upper bound. This might have happened when the variable is selected close to its upper bound.

### ⚙️ Miscellaneous Tasks

- Use new clang in ubuntu workflow
## [1.1.2] - 2025-02-26

### 🚀 Features

- Added destination and generation_step methods in ExportHistory struct

### 🐛 Bug Fixes

- Import IndexedRandom in rand prelude

### ⚙️ Miscellaneous Tasks

- Updated dependencies in Cargo file
- Use new rand crate API
- Bumped crate version and updated optirustic-py pyo3
- Rever back to pyo3 0.22.2 due to API braking changes
## [0.3.3] - 2024-08-12
