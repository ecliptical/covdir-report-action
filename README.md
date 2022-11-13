# Covdir Report Action

A [GitHub Action](https://docs.github.com/en/actions) for generating simple code coverage reports from [grcov-generated covdir files](https://github.com/mozilla/grcov#alternative-reports).

## Usage

Structure your workflow to include the following steps:

1. Check out your code using the [checkout action](https://github.com/actions/checkout).
1. Install your nightly [Rust toolchain](https://github.com/actions-rs/toolchain). If you prefer to use stable, or anything other than nightly, then you must add `RUSTC_BOOTSTRAP: '1'` in the env section of the cargo steps below.
1. Build and test your code (in two separate steps) using the [cargo action](https://github.com/actions-rs/cargo) with some special compiler flags that are required for grcov to work:
    ```yaml
    env:
        CARGO_INCREMENTAL: '0'
        RUSTFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
        RUSTDOCFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
    ```
1. Install grcov using cargo; e.g.,
    ```yaml
      - uses: actions-rs/cargo@v1
        with:
          command: install
          args: grcov
    ```
1. Run grcov in the root of your workspace, specifying `covdir` as the output format and the path to the output file (e.g., ./target/covdir.json). Add any other arguments required for your particular project; e.g.,
    ```yaml
      - run: grcov . -s . --binary-path ./target/debug/ --excl-start '^mod\s+test(s)?\s*\{$' -t covdir --branch --ignore-not-existing --keep-only 'src/**' -o ./target/covdir.json
    ```
1. Finally, run this action, passing the path to the previously generated covdir.json file as the minimum input:
    ```yaml
        - uses: ecliptical/covdir-report-action@v1
        with:
            file: ./covdir.json
    ```

By default, the action only produces output variables with values from the root node of the covdir.json file:

- lines_covered
- lines_missed
- lines_total
- coverage_percent

## Inputs

| ID | Description | Required | Default |
| --- | --- | --- | --- |
| file | Path to covdir.json file | yes | |
| summary | Write report to step summary if `true` | no | `false` |
| out | Write report to the given file | no | |
| title | Report title | no | Line coverage |

## Outputs

| ID | Description |
| --- | --- |
| lines_covered | Number of lines covered |
| lines_missed | Number of lines missed |
| lines_total | Total number of lines |
| coverage_percent | Percentage of lines covered |

## Example

An example workflow that builds and runs unit tests, collects test coverage, generates a simple markdown report, outputs it as the job sunmmary and posts it as a PR comment:

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
        uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.65.0
          override: true
      - name: Build the binary
        uses: actions-rs/cargo@v1
        with:
          command: build
        env:
          CARGO_INCREMENTAL: '0'
          RUSTC_BOOTSTRAP: '1'
          RUSTFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
          RUSTDOCFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
      - name: Run unit tests
        uses: actions-rs/cargo@v1
        with:
          command: test
        env:
          CARGO_INCREMENTAL: '0'
          RUSTC_BOOTSTRAP: '1'
          RUSTFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
          RUSTDOCFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
      - name: Install grcov
        uses: actions-rs/cargo@v1
        with:
          command: install
          args: grcov
      - name: Run grcov
        run: grcov . -s . --binary-path ./target/debug/ --excl-start '^mod\s+test(s)?\s*\{$' -t covdir --branch --ignore-not-existing --keep-only 'src/**' -o ./target/covdir.json
      - name: Generate coverage report
        uses: ecliptical/covdir-report-action@v0.1
        with:
          file: ./target/covdir.json
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
