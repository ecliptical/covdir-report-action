use config::Config;
use config::Environment;
use gumdrop::Options;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use tracing::*;
use tracing_subscriber::EnvFilter;

mod node;

use crate::node::Node;

type ExplicitBool = bool;

#[derive(Debug, Options)]
struct Cmd {
    /// Path to input file.
    #[options(required)]
    file: PathBuf,

    /// Path to output file.
    out: PathBuf,

    /// Write job summary.
    #[options(no_short)]
    summary: ExplicitBool,

    /// Report tile.
    #[options(default = "Line coverage")]
    title: String,

    /// Display help.
    help: bool,
}

#[derive(Debug, Deserialize)]
struct Env {
    output: PathBuf,
    step_summary: PathBuf,
}

fn write_output(file: impl Write, root: &Node) -> std::io::Result<()> {
    let mut out = BufWriter::new(file);
    writeln!(&mut out, "lines_covered={}", root.lines_covered)?;
    writeln!(&mut out, "lines_missed={}", root.lines_missed)?;
    writeln!(&mut out, "lines_total={}", root.lines_total)?;
    writeln!(&mut out, "coverage_percent={}", root.coverage_percent)?;
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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .init();

    let opt = Cmd::parse_args_default_or_exit();

    let config = Config::builder()
        .add_source(Environment::with_prefix("GITHUB"))
        .build()?;

    let env: Env = config.try_deserialize()?;

    debug!(env = ?env, opt = ?opt);

    let file = File::open(opt.file)?;
    let reader = BufReader::new(file);

    let root: Node = serde_json::from_reader(reader)?;

    debug!(root = ?root);

    let out = File::create(env.output)?;
    write_output(out, &root)?;

    if !opt.out.as_os_str().is_empty() {
        let file = File::create(opt.out)?;
        write_summary(file, &root, &opt.title)?;
    }

    if opt.summary {
        let file = File::options()
            .create(true)
            .append(true)
            .open(env.step_summary)?;
        write_summary(file, &root, &opt.title)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
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
