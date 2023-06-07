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
Packages
 ├── Files: 32
 ├── Dependencies: 145
 └── Dev dependencies: 104
Unit Tests
 ├── Cases: 5635
 └── Files: 452
E2E Tests
 ├── Cases: 928
 └── Files: 168
Angular
 ├── Module: 149
 ├── Component: 415 (standalone: 0)
 ├── Directive: 58
 ├── Service: 181
 ├── Pipe: 23
 └── Dialog: 8
Project Files
 ├── HTML: 379
 ├── SCSS: 536
 ├── CSS: 33
 ├── TypeScript: 5125
 ├── JavaScript: 301
 ├── JSON: 548
 └── Markdown: 497
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
- markdown files (`--markdown`)

### Examples:

```shell
# run all available inspections
birdview inspect --all <project>

# inspect tests
birdview inspect --tests <project>

# inspect packages
birdview inspect --packages <project>

# inspect tests and packages
birdview inspect --tests --packages <project>
```

### Generating Reports

```shell
birdview inspect <project> --all --output <path>
```

You can generate reports using multiple templates, based on the output extension:

- `.html`: single-page HTML report
- `.json`: raw JSON report

#### HTML Report

```shell
# generate report as output.html
birdview inspect <project> --all --output output.html

# generate report as output.html and open with the default browser
birdview inspect <project> --all --output output.html --open
```

Provides an output that is similar to the following one:

![html report](docs/html-report.png)

#### JSON Report

```shell
birdview inspect <project> --all --output output.json
```

The format of the output is similar to the following example:

```json
{
  "project_name": "<package.json>/name",
  "project_version": "<package.json>/version",
  "report_date": "<date/time UTC>",

  "stats": {
    "package": {
      "files": 32,
      "prod_deps": 145,
      "dev_deps": 104
    },

    "tests": {
      "unit_test": 452,
      "unit_test_case": 5635,
      "e2e_test": 168,
      "e2e_test_case": 928
    },

    "angular": {
      "module": 149,
      "component": 415,
      "component_standalone": 23,
      "directive": 58,
      "service": 181,
      "pipe": 23,
      "dialog": 8
    },

    "types": {
      "html": 379,
      "scss": 536,
      "css": 33,
      "ts": 5125,
      "js": 301,
      "md": 497,
      "json": 548
    }
  },

  "angular": {
    "components": [
      {
        "path": "<workspace>/<path>.component.ts",
        "standalone": false
      }
    ],
    "directives": [],
    "services": [],
    "pipes": [],
    "dialogs": []
  },
  
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
