# BirdView

Command-line utilities to gather statistics for the Angular projects.

## Getting Help

```shell
birdview --help
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

### Generating reports

```shell
 birdview inspect <project> --all --output output.json
```
