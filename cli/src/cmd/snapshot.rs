//! Snapshot command

use crate::cmd::{
    test,
    test::{Test, TestOutcome},
    Cmd,
};
use ansi_term::Colour;
use eyre::Context;
use forge::{TestKind, TestResult};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Formatter, Write},
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

/// A regex that matches a basic snapshot entry like
/// `testDeposit() (gas: 58804)`
pub static RE_BASIC_SNAPSHOT_ENTRY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<sig>(\w+)\s*\((.*?)\))\s*\(((gas:)?\s*(?P<gas>\d+)|(μ:\s*(?P<avg>\d+),\s*~:\s*(?P<med>\d+)))\)").unwrap()
});

#[derive(Debug, Clone, StructOpt)]
pub struct SnapshotArgs {
    /// All test arguments are supported
    #[structopt(flatten)]
    test: test::TestArgs,
    /// Additional configs for test results
    #[structopt(flatten)]
    config: SnapshotConfig,
    #[structopt(
        help = "Compare against a snapshot and display changes from the snapshot. Takes an optional snapshot file, [default: .gas-snapshot]",
        conflicts_with = "snap",
        long
    )]
    diff: Option<Option<PathBuf>>,
    #[structopt(
        help = "Run snapshot in 'check' mode and compares against an existing snapshot file, [default: .gas-snapshot]. Exits with 0 if snapshots match. Exits with 1 and prints a diff otherwise",
        conflicts_with = "diff",
        long
    )]
    check: Option<Option<PathBuf>>,
    #[structopt(help = "How to format the output.", long)]
    format: Option<Format>,
    #[structopt(help = "Output file for the snapshot.", default_value = ".gas-snapshot", long)]
    snap: PathBuf,
}

impl Cmd for SnapshotArgs {
    type Output = ();

    fn run(self) -> eyre::Result<()> {
        let outcome = self.test.run()?;
        outcome.ensure_ok()?;
        let tests = self.config.apply(outcome);

        if let Some(path) = self.diff {
            let snap = path.as_ref().unwrap_or(&self.snap);
            let snaps = read_snapshot(snap)?;
            diff(tests, snaps)?;
        } else if let Some(path) = self.check {
            let snap = path.as_ref().unwrap_or(&self.snap);
            let snaps = read_snapshot(snap)?;
            if check(tests, snaps) {
                std::process::exit(0)
            } else {
                std::process::exit(1)
            }
        } else {
            write_to_snapshot_file(&tests, self.snap, self.format)?;
        }
        Ok(())
    }
}

// TODO implement pretty tables
#[derive(Debug, Clone)]
pub enum Format {
    Table,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" | "table" => Ok(Format::Table),
            _ => Err(format!("Unrecognized format `{}`", s)),
        }
    }
}

/// Additional filters that can be applied on the test results
#[derive(Debug, Clone, StructOpt, Default)]
struct SnapshotConfig {
    #[structopt(help = "sort results by ascending gas used.", long)]
    asc: bool,
    #[structopt(help = "sort results by descending gas used.", conflicts_with = "asc", long)]
    desc: bool,
    #[structopt(help = "Only include tests that used more gas that the given amount.", long)]
    min: Option<u64>,
    #[structopt(help = "Only include tests that used less gas that the given amount.", long)]
    max: Option<u64>,
}

impl SnapshotConfig {
    fn is_in_gas_range(&self, gas_used: u64) -> bool {
        if let Some(min) = self.min {
            if gas_used < min {
                return false
            }
        }
        if let Some(max) = self.max {
            if gas_used > max {
                return false
            }
        }
        true
    }

    fn apply(&self, outcome: TestOutcome) -> Vec<Test> {
        let mut tests = outcome
            .into_tests()
            .filter_map(|test| test.gas_used().map(|gas| (test, gas)))
            .filter(|(_test, gas)| self.is_in_gas_range(*gas))
            .map(|(test, _)| test)
            .collect::<Vec<_>>();

        if self.asc {
            tests.sort_by_key(|a| a.gas_used());
        } else if self.desc {
            tests.sort_by_key(|b| std::cmp::Reverse(b.gas_used()))
        }

        tests
    }
}

/// A general entry in a snapshot file
///
/// Has the form `<signature>(gas:? 40181)`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnapshotEntry {
    pub signature: String,
    pub gas_used: SnapshotGas,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SnapshotGas {
    Standard(u64),
    Fuzz { mean: u64, median: u64 },
}

impl SnapshotGas {
    /// Returns the gas to compare against
    fn gas(&self) -> u64 {
        match self {
            SnapshotGas::Standard(gas) => *gas,
            SnapshotGas::Fuzz { median, .. } => *median,
        }
    }
}

impl<'a> From<&'a TestResult> for SnapshotGas {
    fn from(test: &'a TestResult) -> Self {
        match &test.kind {
            TestKind::Standard => SnapshotGas::Standard(test.gas_used.unwrap_or_default()),
            TestKind::Fuzz(fuzzed) => {
                SnapshotGas::Fuzz { median: fuzzed.median_gas(), mean: fuzzed.mean_gas() }
            }
        }
    }
}

impl fmt::Display for SnapshotGas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotGas::Standard(gas) => gas.fmt(f),
            SnapshotGas::Fuzz { median, mean } => {
                write!(f, "(μ: {}, ~: {})", median, mean)
            }
        }
    }
}

impl FromStr for SnapshotEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RE_BASIC_SNAPSHOT_ENTRY
            .captures(s)
            .and_then(|cap| {
                cap.name("sig").and_then(|sig| {
                    if let Some(gas) = cap.name("gas") {
                        Some(SnapshotEntry {
                            signature: sig.as_str().to_string(),
                            gas_used: SnapshotGas::Standard(gas.as_str().parse().unwrap()),
                        })
                    } else {
                        cap.name("avg").and_then(|avg| cap.name("med").map(|med| (avg, med))).map(
                            |(avg, med)| SnapshotEntry {
                                signature: sig.as_str().to_string(),
                                gas_used: SnapshotGas::Fuzz {
                                    median: med.as_str().parse().unwrap(),
                                    mean: avg.as_str().parse().unwrap(),
                                },
                            },
                        )
                    }
                })
            })
            .ok_or_else(|| format!("Could not extract Snapshot Entry for {}", s))
    }
}

/// Reads a list of snapshot entries from a snapshot file
fn read_snapshot(path: impl AsRef<Path>) -> eyre::Result<Vec<SnapshotEntry>> {
    let path = path.as_ref();
    let mut entries = Vec::new();
    for line in io::BufReader::new(
        fs::File::open(path)
            .wrap_err(format!("failed to read snapshot file \"{}\"", path.display()))?,
    )
    .lines()
    {
        entries
            .push(SnapshotEntry::from_str(line?.as_str()).map_err(|err| eyre::eyre!("{}", err))?);
    }
    Ok(entries)
}

/// Writes a series of tests to a snapshot file
fn write_to_snapshot_file(
    tests: &[Test],
    path: impl AsRef<Path>,
    _format: Option<Format>,
) -> eyre::Result<()> {
    let mut out = String::new();
    for test in tests {
        match &test.result.kind {
            TestKind::Standard => {
                if let Some(gas) = test.gas_used() {
                    writeln!(out, "{} (gas: {})", test.signature, gas)?;
                }
            }
            TestKind::Fuzz(fuzzed) => {
                let mean = fuzzed.mean_gas();
                let median = fuzzed.median_gas();
                writeln!(out, "{} (μ: {}, ~: {})", test.signature, mean, median)?;
            }
        }
    }
    Ok(fs::write(path, out)?)
}

/// A Snapshot entry diff
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnapshotDiff {
    pub signature: String,
    pub source_gas_used: SnapshotGas,
    pub target_gas_used: SnapshotGas,
}

impl SnapshotDiff {
    /// Returns the gas diff
    ///
    /// `> 0` if the source used more gas
    /// `< 0` if the source used more gas
    fn gas_change(&self) -> i128 {
        self.source_gas_used.gas() as i128 - self.target_gas_used.gas() as i128
    }

    /// Determines the percentage change
    fn gas_diff(&self) -> f64 {
        self.gas_change() as f64 / self.target_gas_used.gas() as f64
    }
}

/// Compares the set of tests with an existing snapshot
///
/// Returns true all tests match
fn check(tests: Vec<Test>, snaps: Vec<SnapshotEntry>) -> bool {
    let snaps = snaps.into_iter().map(|s| (s.signature, s.gas_used)).collect::<HashMap<_, _>>();
    let mut has_diff = false;

    for test in tests.into_iter().filter(|t| t.gas_used().is_some()) {
        if let Some(target_gas) = snaps.get(&test.signature).cloned() {
            let source_gas = SnapshotGas::from(&test.result);
            if source_gas != target_gas {
                println!(
                    "Diff in \"{}\": consumed {} gas, expected {} gas ",
                    test.signature, source_gas, target_gas
                );
                has_diff = true;
            }
        } else {
            println!(
                "No matching snapshot entry found for \"{}\" in snapshot file",
                test.signature
            );
            has_diff = true;
        }
    }
    !has_diff
}

/// Compare the set of tests with an existing snapshot
fn diff(tests: Vec<Test>, snaps: Vec<SnapshotEntry>) -> eyre::Result<()> {
    let snaps = snaps.into_iter().map(|s| (s.signature, s.gas_used)).collect::<HashMap<_, _>>();
    let mut diffs = Vec::with_capacity(tests.len());
    for test in tests.into_iter().filter(|t| t.gas_used().is_some()) {
        let target_gas_used = snaps.get(&test.signature).cloned().ok_or_else(|| {
            eyre::eyre!(
                "No matching snapshot entry found for \"{}\" in snapshot file",
                test.signature
            )
        })?;

        diffs.push(SnapshotDiff {
            source_gas_used: SnapshotGas::from(&test.result),
            signature: test.signature,
            target_gas_used,
        });
    }
    let mut overall_gas_change = 0i128;
    let mut overall_gas_diff = 0f64;

    diffs.sort_by(|a, b| {
        a.gas_diff().abs().partial_cmp(&b.gas_diff().abs()).unwrap_or(Ordering::Equal)
    });

    for diff in diffs {
        let gas_change = diff.gas_change();
        overall_gas_change += gas_change;
        let gas_diff = diff.gas_diff();
        overall_gas_diff += gas_diff;
        println!(
            "{} (gas: {} ({})) ",
            diff.signature,
            fmt_change(gas_change),
            fmt_pct_change(gas_diff)
        );
    }

    println!(
        "Overall gas change: {} ({})",
        fmt_change(overall_gas_change),
        fmt_pct_change(overall_gas_diff)
    );
    Ok(())
}

fn fmt_pct_change(change: f64) -> String {
    match change.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
        Ordering::Less => Colour::Green.paint(format!("{:.3}%", change)).to_string(),
        Ordering::Equal => {
            format!("{:.3}%", change)
        }
        Ordering::Greater => Colour::Red.paint(format!("{:.3}%", change)).to_string(),
    }
}

fn fmt_change(change: i128) -> String {
    match change.cmp(&0) {
        Ordering::Less => Colour::Green.paint(format!("{}", change)).to_string(),
        Ordering::Equal => {
            format!("{}", change)
        }
        Ordering::Greater => Colour::Red.paint(format!("{}", change)).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_basic_snapshot_entry() {
        let s = "deposit() (gas: 7222)";
        let entry = SnapshotEntry::from_str(s).unwrap();
        assert_eq!(
            entry,
            SnapshotEntry {
                signature: "deposit()".to_string(),
                gas_used: SnapshotGas::Standard(7222)
            }
        );
    }

    #[test]
    fn can_parse_fuzz_snapshot_entry() {
        let s = "deposit() (μ: 100, ~:200)";
        let entry = SnapshotEntry::from_str(s).unwrap();
        assert_eq!(
            entry,
            SnapshotEntry {
                signature: "deposit()".to_string(),
                gas_used: SnapshotGas::Fuzz { median: 200, mean: 100 }
            }
        );
    }
}
