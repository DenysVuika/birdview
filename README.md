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
workspace: .
unit test files (.spec.ts): 109 (906 cases)
e2e test files (.test.ts, .e2e.ts): 74 (768 cases)
Found package.json files: 8
Found root package.json file
  ├── scripts: 27
  ├── dependencies: 29
  ├── devDependencies: 67
Inspection complete
```

## Code Inspection

```shell
birdview inspect --help
```

### Examples:

```shell
# generic inspection
birdview inspect <project>

# inspect tests
birdview inspect --tests <project>

# inspect dependencies
birdview inspect --deps <project>

# inspect tests and dependencies
birdview inspect --tests --deps <project>

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

The format of the output is as follows:

```json
{
  "project_name": "<package.json>/name",
  "project_version": "<package.json>/version",
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
  ]
}
```
