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
## [1.2.2] - 2025-03-22

### 🚀 Features

- Added StoppingCondition::MaxDurationAsMinutes StoppingCondition::MaxDurationAsHours to stop the algorithm after a specified number of minutes or hours

### ⚙️ Miscellaneous Tasks

- Updated changelog for version 1.2.1
## [1.2.0] - 2025-03-16

### ⚙️ Miscellaneous Tasks

- Updated changelog file to include changes up to version 1.2.0
## [1.2.1] - 2025-03-16

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
- Bumped optirustic-macro version
- Made pyo3-build-config dependency optional
- Use new macro dep version from crate.io
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
