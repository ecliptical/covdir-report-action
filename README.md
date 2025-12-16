# Covdir Report Action

A [GitHub Action](https://docs.github.com/en/actions) for generating simple code coverage reports from [grcov-generated covdir files](https://github.com/mozilla/grcov#alternative-reports).

This action downloads [grcov](https://github.com/mozilla/grcov) automatically and uses your project's LLVM tools (from `rustup component add llvm-tools`) to process coverage data.

## Quick Start

The simplest way to use this action is with the bundled grcov (enabled by default):

```yaml
on:
  pull_request:
  push:
    branches:
      - main

name: Rust Coverage

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools
      - name: Test
        run: cargo test
        env:
          RUSTFLAGS: '-Cinstrument-coverage'
          LLVM_PROFILE_FILE: 'target/coverage/%p-%m.profraw'
      - name: Generate coverage report
        uses: ecliptical/covdir-report-action@v0.3
        with:
          summary: 'true'
```

## Usage with Pre-generated covdir.json

If you prefer to run grcov yourself (for more control over its options), you can disable the built-in grcov integration and pass a pre-generated covdir.json file:

```yaml
- uses: ecliptical/covdir-report-action@v0.3
  with:
    skip_grcov: 'true'
    file: ./target/covdir.json
    summary: 'true'
```

## Inputs

### Report Inputs

| ID | Description | Required | Default |
| --- | --- | --- | --- |
| file | Path to covdir.json file (not needed if using `coverage_path`) | no | |
| summary | Write report to step summary if `true` | no | `false` |
| out | Write report to the given file | no | |
| title | Report title | no | `Line coverage` |

### Bundled grcov Inputs

These inputs control the bundled grcov, which is enabled by default.

| ID | Description | Required | Default |
| --- | --- | --- | --- |
| skip_grcov | Skip bundled grcov and use a pre-generated covdir.json file | no | `false` |
| coverage_path | Path to coverage data | no | `./target/coverage` |
| source_dir | Source directory for grcov (`-s` flag) | no | `.` |
| binary_path | Binary path for grcov (`--binary-path` flag) | no | `./target/debug` |
| keep_only | Keep only files matching pattern (`--keep-only` flag) | no | `src/**` |
| excl_start | Exclude start pattern (`--excl-start` flag) | no | `^mod\s+tests?\s*\{$` |
| branch | Include branch coverage (`--branch` flag) | no | `true` |
| covdir_output | Output path for generated covdir.json | no | `./target/covdir.json` |
| grcov_args | Additional arguments to pass to grcov | no | |

## Outputs

| ID | Description |
| --- | --- |
| lines_covered | Number of lines covered |
| lines_missed | Number of lines missed |
| lines_total | Total number of lines |
| coverage_percent | Percentage of lines covered |

## Example

An example workflow that runs unit tests with coverage, generates a simple markdown report, outputs it as the job summary and posts it as a PR comment:

```yaml
on:
  pull_request:
  push:
    branches:
      - main

name: Rust Coverage

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - name: Check out the source code
        uses: actions/checkout@v6
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools
      - name: Run unit tests
        run: cargo test
        env:
          RUSTFLAGS: '-Cinstrument-coverage'
          LLVM_PROFILE_FILE: 'target/coverage/%p-%m.profraw'
      - name: Generate coverage report
        uses: ecliptical/covdir-report-action@v0.3
        with:
          summary: 'true'
          out: ./target/coverage.md
      - name: Add coverage comment to the pull request
        uses: marocchino/sticky-pull-request-comment@v2
        if: github.event_name == 'pull_request'
        with:
          hide_and_recreate: true
          hide_classify: "OUTDATED"
          path: ./target/coverage.md
```
## Third-Party Software

This action downloads and uses [grcov](https://github.com/mozilla/grcov), a code coverage tool by Mozilla, which is licensed under the [Mozilla Public License 2.0 (MPL-2.0)](https://github.com/mozilla/grcov/blob/master/LICENSE-MPL-2.0). The grcov source code is available at https://github.com/mozilla/grcov.
