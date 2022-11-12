# Covdir Report Action

A [GitHub Action](https://docs.github.com/en/actions) for generating simple code coverage reports from [grcov-generated covdir files](https://github.com/mozilla/grcov#alternative-reports).

## Usage

Structure your workflow to include the following steps:

1. Checkout your code using the [checkout action](https://github.com/actions/checkout).
1. Install your nightly [Rust toolchain](https://github.com/actions-rs/toolchain). 
1. Build and test your code using the [cargo action](https://github.com/actions-rs/cargo) with some special compiler flags:
    ```yaml
        env:
            CARGO_INCREMENTAL: '0'
            RUSTFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
            RUSTDOCFLAGS: '-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
    ```
1. Run the [grcov action](https://github.com/actions-rs/grcov#usage) with `covdir` as its output-type:
    ```yaml
    output-type: covdir
    output-path: ./covdir.json
    ```
1. Run this action, passing the previously generated covdir.json file as its input:
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

TBD
