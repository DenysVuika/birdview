# BirdView

Command-line utilities to gather statistics for the Angular projects.

## Installing

Install Rust and Cargo  
https://doc.rust-lang.org/cargo/getting-started/installation.html

```shell
# with Cargo
cargo install birdview

# and then
birdview --help
```

## Basic Usage

```shell
cd <path-to-project>
birdview inspect . --all
```

Gives an output similar to the following:

```text
package.json files: 8 (35 deps, 67 dev deps)
unit test files (.spec.ts): 109 (906 cases))
e2e tests: 928 (168 files)
Components: 415
Directives: 58
Services: 181
Pipes: 23
Dialogs: 8
Inspection complete
```

## Code Inspection

```shell
birdview inspect --help
```

### Available Inspectors

- `package.json` files (`--packages`)
- unit and e2e tests (`--tests`)
- angular elements (`--angular`)

### Examples:

```shell
# generic inspection
birdview inspect <project>

# inspect tests
birdview inspect --tests <project>

# inspect packages
birdview inspect --packages <project>

# inspect tests and packages
birdview inspect --tests --packages <project>

# run all available inspections
birdview inspect --all <project>

# run all available inspections with detailed output
birdview inspect --all --verbose <project>
```

### Generating Reports

For additional processing or visualisation, you can generate full reports in the `JSON` format by using `--output` flag:

```shell
 birdview inspect <project> --all --output output.json
```

The format of the output is similar to the following example:

```json
{
  "project_name": "<package.json>/name",
  "project_version": "<package.json>/version",
  
  "total_package_files": 8,
  "total_package_deps": 35,
  "total_package_dev_deps": 67,
  "total_unit_test_files": 109,
  "total_unit_test_cases": 906,
  
  "unit_tests": [
    {
      "path": "<workspace>/<path>.spec.ts",
      "cases": [
        "case name 1",
        "case name 2"
      ]
    }
  ],
  "e2e_tests": [
    {
      "path": "<workspace>/<path>.e2e.ts",
      "cases": [
        "case name 1",
        "case name 2"
      ]
    }
  ],
  "packages": [
    {
      "path": "<workspace>/<path>/package.json",
      "dependencies": [
        {
          "name": "tslib",
          "version": "^2.0.0",
          "dev": false
        },
        {
          "name": "typescript",
          "version": "4.7.4",
          "dev": true
        }
      ]
    }
  ],
  "angular": {
    "components": [],
    "directives": [],
    "services": [],
    "pipes": [],
    "dialogs": []
  }
}
```
