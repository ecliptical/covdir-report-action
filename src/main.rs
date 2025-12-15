use anyhow::bail;
use gha::*;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

mod node;

use crate::node::Node;

/// grcov configuration options
#[derive(Debug, Default)]
struct GrcovConfig {
    /// Skip bundled grcov (use pre-generated covdir.json instead)
    skip_grcov: bool,
    /// Path to coverage data for grcov integration
    coverage_path: PathBuf,
    /// Source directory for grcov (-s flag)
    source_dir: String,
    /// Binary path for grcov (--binary-path flag)
    binary_path: String,
    /// Keep only files matching pattern (--keep-only flag)
    keep_only: String,
    /// Exclude start pattern (--excl-start flag)
    excl_start: String,
    /// Include branch coverage (--branch flag)
    branch: bool,
    /// Output path for generated covdir.json
    covdir_output: String,
    /// Additional arguments to pass to grcov
    grcov_args: String,
}

#[derive(Debug, Default)]
struct Cmd {
    /// Path to input file.
    file: PathBuf,

    /// Path to output file.
    out: PathBuf,

    /// Write job summary.
    summary: bool,

    /// Report title.
    title: String,

    /// grcov configuration (if using integrated grcov)
    grcov: GrcovConfig,
}

/// Run grcov to generate covdir.json and return the output path
fn run_grcov(config: &GrcovConfig) -> anyhow::Result<PathBuf> {
    println!("::group::Running grcov");

    let mut cmd = Command::new("/usr/local/bin/grcov");
    cmd.arg(&config.coverage_path);
    cmd.args(["-s", &config.source_dir]);
    cmd.args(["--binary-path", &config.binary_path]);
    cmd.args(["-t", "covdir"]);
    cmd.arg("--ignore-not-existing");

    if !config.keep_only.is_empty() {
        cmd.args(["--keep-only", &config.keep_only]);
    }

    if !config.excl_start.is_empty() {
        cmd.args(["--excl-start", &config.excl_start]);
    }

    if config.branch {
        cmd.arg("--branch");
    }

    // Parse additional arguments if provided
    if !config.grcov_args.is_empty() {
        cmd.args(config.grcov_args.split_whitespace());
    }

    cmd.args(["-o", &config.covdir_output]);

    debug!("Running grcov: {cmd:?}");

    let status = cmd.status()?;
    println!("::endgroup::");

    if !status.success() {
        bail!("grcov failed with exit code: {}", status);
    }

    Ok(PathBuf::from(&config.covdir_output))
}

fn write_output(file: impl Write, root: &Node) -> std::io::Result<()> {
    let mut out = BufWriter::new(file);
    append_name_value(&mut out, "lines_covered", root.lines_covered)?;
    append_name_value(&mut out, "lines_missed", root.lines_missed)?;
    append_name_value(&mut out, "lines_total", root.lines_total)?;
    append_name_value(&mut out, "coverage_percent", root.coverage_percent)?;
    Ok(())
}

fn write_summary(file: impl Write, root: &Node, title: &str) -> std::io::Result<()> {
    let mut out = BufWriter::new(file);
    writeln!(
        &mut out,
        "| {title}: | {:.0}% |",
        root.coverage_percent.round()
    )?;
    writeln!(&mut out, "|:---|:---|")?;
    writeln!(
        &mut out,
        "| Lines covered: | {} |",
        fmt_number(root.lines_covered)
    )?;
    writeln!(
        &mut out,
        "| Lines missed: | {} |",
        fmt_number(root.lines_missed)
    )?;
    writeln!(
        &mut out,
        "| Total lines: | {} |",
        fmt_number(root.lines_total)
    )?;
    Ok(())
}

fn fmt_number(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .map(String::from_utf8_lossy)
        .rev()
        .collect::<Vec<_>>()
        .as_slice()
        .join(",")
}

fn main() -> anyhow::Result<()> {
    let mut opt = Cmd::default();
    for arg in env::args().skip(1) {
        let Some((name, value)) = arg.split_once('=') else {
            bail!("invalid argument: {arg}");
        };

        match name {
            "--file" => opt.file = value.into(),
            "--summary" => opt.summary = value == "true",
            "--title" => opt.title = value.to_string(),
            "--out" => opt.out = value.into(),
            // grcov integration options
            "--skip-grcov" => opt.grcov.skip_grcov = value == "true",
            "--coverage-path" => opt.grcov.coverage_path = value.into(),
            "--source-dir" => opt.grcov.source_dir = value.to_string(),
            "--binary-path" => opt.grcov.binary_path = value.to_string(),
            "--keep-only" => opt.grcov.keep_only = value.to_string(),
            "--excl-start" => opt.grcov.excl_start = value.to_string(),
            "--branch" => opt.grcov.branch = value == "true",
            "--covdir-output" => opt.grcov.covdir_output = value.to_string(),
            "--grcov-args" => opt.grcov.grcov_args = value.to_string(),
            _ => bail!("unknown argument: {name}"),
        }
    }

    debug!("opt = {opt:#?}");

    // If skip_grcov is false and coverage_path is provided, run grcov first
    let input_file = if !opt.grcov.skip_grcov && !opt.grcov.coverage_path.as_os_str().is_empty() {
        run_grcov(&opt.grcov)?
    } else {
        opt.file
    };

    if input_file.as_os_str().is_empty() {
        bail!("either --file or --coverage-path must be provided");
    }

    let file = File::open(&input_file)?;
    let reader = BufReader::new(file);

    let root: Node = serde_json::from_reader(reader)?;

    debug!("root = {root:#?}");

    let out = File::create(github_output())?;
    write_output(out, &root)?;

    if !opt.out.as_os_str().is_empty() {
        let file = File::create(opt.out)?;
        write_summary(file, &root, &opt.title)?;
    }

    if opt.summary {
        let file = File::options()
            .create(true)
            .append(true)
            .open(github_step_summary())?;
        write_summary(file, &root, &opt.title)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn fmt_number_short() {
        assert_eq!(fmt_number(123), "123");
    }

    #[test]
    fn fmt_number_medium() {
        assert_eq!(fmt_number(1234), "1,234");
    }

    #[test]
    fn fmt_number_long() {
        assert_eq!(fmt_number(1234567890), "1,234,567,890");
    }

    #[test]
    fn write_output_ok() {
        let root = Node {
            name: String::default(),
            lines_covered: 12,
            lines_missed: 34,
            lines_total: 46,
            coverage_percent: 12.34,
            children: Vec::default(),
        };

        let mut buf = Vec::default();

        let res = write_output(&mut buf, &root);

        assert!(res.is_ok());
        assert_snapshot!(String::from_utf8(buf).unwrap_or_default());
    }

    #[test]
    fn write_summary_ok() {
        let root = Node {
            name: String::default(),
            lines_covered: 12,
            lines_missed: 34,
            lines_total: 46,
            coverage_percent: 12.34,
            children: Vec::default(),
        };

        let mut buf = Vec::default();

        let res = write_summary(&mut buf, &root, "Line coverage");

        assert!(res.is_ok());
        assert_snapshot!(String::from_utf8(buf).unwrap_or_default());
    }
}
