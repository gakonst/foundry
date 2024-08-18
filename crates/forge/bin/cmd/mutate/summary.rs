use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, Attribute, Cell, CellAlignment, Color, Row, Table,
};
use core::fmt;
use eyre::{eyre, Result};
use foundry_common::shell::{self};
use foundry_evm_mutator::Mutant;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use similar::TextDiff;
use std::{collections::BTreeMap, ops::Add, time::Duration};
use yansi::Paint;

const MAX_SURVIVE_RESULT_LOG_SIZE: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutantTestStatus {
    Killed,
    Survived,
    #[default]
    Equivalent,
    Timeout,
}

impl fmt::Display for MutantTestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutantTestStatus::Killed => "KILLED".fmt(f),
            MutantTestStatus::Survived => "SURVIVED".fmt(f),
            MutantTestStatus::Equivalent => "EQUIVALENT".fmt(f),
            MutantTestStatus::Timeout => "TIMEOUT".fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MutantTestResult {
    pub duration: Duration,
    pub mutant: Mutant,
    status: MutantTestStatus,
}

impl MutantTestResult {
    pub fn new(duration: Duration, mutant: Mutant, status: MutantTestStatus) -> Self {
        Self { duration, mutant, status }
    }

    pub fn killed(&self) -> bool {
        matches!(self.status, MutantTestStatus::Killed)
    }

    pub fn survived(&self) -> bool {
        matches!(self.status, MutantTestStatus::Survived)
    }

    pub fn equivalent(&self) -> bool {
        matches!(self.status, MutantTestStatus::Equivalent)
    }

    pub fn timeout(&self) -> bool {
        matches!(self.status, MutantTestStatus::Timeout)
    }

    pub fn diff(&self) -> String {
        "".into()
    }
}

impl Serialize for MutantTestResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("MutantTestResult", 6)?;
        state.serialize_field("name", self.mutant.source.filename())?;
        state.serialize_field(
            "path",
            &self.mutant.source.sourceroot().join(&self.mutant.source.filename()).to_string_lossy(),
        )?;
        state.serialize_field("description", &self.mutant.op.to_string())?;

        let diff = mutant_diff(&self.mutant);

        state.serialize_field("diff", &diff)?;
        state.serialize_field("result", &self.status.to_string())?;

        state.end()
    }
}

/// Results and duration for mutation tests for a contract
#[derive(Debug, Clone)]
pub struct MutationTestSuiteResult {
    /// Total duration of the mutation tests run for this contract
    pub duration: Duration,
    /// Individual mutation test results. `file_name -> MutationTestResult`
    mutation_test_results: Vec<MutantTestResult>,
}

impl MutationTestSuiteResult {
    pub fn new(duration: Duration, results: Vec<MutantTestResult>) -> Self {
        Self { duration, mutation_test_results: results }
    }

    pub fn killed(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.mutation_test_results().filter(|result| result.killed())
    }

    pub fn survived(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.mutation_test_results().filter(|result| result.survived())
    }

    pub fn equivalent(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.mutation_test_results().filter(|result| result.equivalent())
    }

    pub fn timeout(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.mutation_test_results().filter(|result| result.timeout())
    }

    pub fn mutation_test_results(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.mutation_test_results.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.mutation_test_results.is_empty()
    }

    pub fn len(&self) -> usize {
        self.mutation_test_results.len()
    }
}

/// Represents the bundled results of all tests
#[derive(Clone, Debug)]
pub struct MutationTestOutcome {
    /// Whether failures are allowed
    /// This enables to exit early
    pub allow_failure: bool,

    // this would be Contract -> SuiteResult
    pub test_suite_result: BTreeMap<String, MutationTestSuiteResult>,
}

impl MutationTestOutcome {
    pub fn new(
        allow_failure: bool,
        test_suite_result: BTreeMap<String, MutationTestSuiteResult>,
    ) -> Self {
        Self { allow_failure, test_suite_result }
    }

    /// Total duration for tests
    pub fn duration(&self) -> Duration {
        self.test_suite_result
            .values()
            .map(|suite| suite.duration)
            .fold(Duration::from_secs(0), |acc, duration| acc + duration)
    }

    /// Iterator over all killed mutation tests
    pub fn killed(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.results().filter(|result| result.killed())
    }

    /// Iterator over all surviving mutation tests
    pub fn survived(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.results().filter(|result| result.survived())
    }

    /// Iterator over all equivalent mutation tests
    pub fn equivalent(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.results().filter(|result| result.equivalent())
    }

    /// Iterator over all timeout mutation tests
    pub fn timeout(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.results().filter(|result| result.timeout())
    }

    /// Iterator over all mutation tests and their names
    pub fn results(&self) -> impl Iterator<Item = &MutantTestResult> {
        self.test_suite_result.values().flat_map(|suite| suite.mutation_test_results())
    }

    pub fn summary(&self) -> String {
        let survived = self.survived().count();
        let result = if survived == 0 { Paint::green("ok") } else { Paint::red("FAILED") };
        format!(
            "Mutation Test result: {}. {} killed; {} survived; {} equivalent; {} timeout, finished in {:.2?}",
            result,
            Paint::green(self.killed().count()),
            Paint::red(survived),
            Paint::yellow(self.equivalent().count()),
            Paint::cyan(self.timeout().count()),
            self.duration()
        )
    }

    /// Checks if there is any surviving mutations and failures are disallowed
    pub fn ensure_ok(&self) -> eyre::Result<()> {
        let survived = self.survived().count();

        if self.allow_failure || survived == 0 {
            return Ok(());
        }

        if !shell::verbosity().is_normal() {
            // skip printing and exit early
            std::process::exit(1);
        }

        println!();
        println!("Surviving Mutations:");

        for (contract_name, suite_result) in self.test_suite_result.iter() {
            let survived = suite_result.survived().count();
            if survived == 0 {
                continue;
            }

            let term = if survived > 1 { "mutations" } else { "mutation" };
            println!("Encountered {} surviving {term} in {}", survived, contract_name);

            for survive_result in suite_result.survived().take(MAX_SURVIVE_RESULT_LOG_SIZE) {
                let description = survive_result.mutant.op.to_string();
                let (line, _) = survive_result
                    .mutant
                    .get_line_column()
                    .map_err(|x| eyre!(format!("{:?}", x)))?;
                println!(
                    "\t Location: {}:{}, MutationType: {}",
                    survive_result.mutant.source.filename_as_str(),
                    line,
                    description
                );
            }

            if survived > MAX_SURVIVE_RESULT_LOG_SIZE {
                println!("\t More..");
            }
        }

        println!();
        println!(
            "Encountered a total of {} surviving mutations, {} mutations killed",
            Paint::red(survived.to_string()),
            Paint::green(self.killed().count().to_string())
        );
        std::process::exit(1);
    }
}

pub struct MutationTestSummaryReporter {
    /// The mutation test summary table.
    pub(crate) table: Table,
    pub(crate) is_detailed: bool,
}

impl MutationTestSummaryReporter {
    pub(crate) fn new(is_detailed: bool) -> Self {
        let mut table = Table::new();
        table.apply_modifier(UTF8_ROUND_CORNERS);
        let mut row = Row::from(vec![
            Cell::new("File").set_alignment(CellAlignment::Left).add_attribute(Attribute::Bold),
            Cell::new("Killed")
                .set_alignment(CellAlignment::Center)
                .add_attribute(Attribute::Bold)
                .fg(Color::White),
            Cell::new("Survived")
                .set_alignment(CellAlignment::Center)
                .add_attribute(Attribute::Bold)
                .fg(Color::White),
            Cell::new("Equivalent")
                .set_alignment(CellAlignment::Center)
                .add_attribute(Attribute::Bold)
                .fg(Color::White),
            Cell::new("Timeout")
                .set_alignment(CellAlignment::Center)
                .add_attribute(Attribute::Bold)
                .fg(Color::White),
        ]);

        if is_detailed {
            row.add_cell(
                Cell::new("Duration")
                    .set_alignment(CellAlignment::Center)
                    .add_attribute(Attribute::Bold),
            );
        }

        row.add_cell(
            Cell::new("% Score")
                .set_alignment(CellAlignment::Center)
                .add_attribute(Attribute::Bold)
                .fg(Color::White),
        );

        table.set_header(row);
        Self { table, is_detailed }
    }

    pub fn print_summary(&mut self, mutation_test_outcome: &MutationTestOutcome) {
        let mut total_killed = 0.0;
        let mut total_survived = 0.0;
        let mut total_equivalent = 0.0;
        let mut total_timeout = 0.0;
        let mut total_time_taken = Duration::ZERO;
        for (contract_name, suite_result) in mutation_test_outcome.test_suite_result.iter() {
            let mut row = Row::new();

            let contract_title: String;
            if let Some(result) = suite_result.mutation_test_results.first() {
                contract_title =
                    format!("{}:{}", result.mutant.source.filename_as_str(), contract_name);
            } else {
                contract_title = contract_name.to_string();
            }

            let file_cell = Cell::new(contract_title).set_alignment(CellAlignment::Left);
            row.add_cell(file_cell);

            let killed = suite_result.killed().count() as f64;
            total_killed += killed;
            let survived = suite_result.survived().count() as f64;
            total_survived += survived;
            let equivalent = suite_result.equivalent().count() as f64;
            total_equivalent += equivalent;
            let timeout = suite_result.timeout().count() as f64;
            total_timeout += timeout;

            let mut killed_cell = Cell::new(killed).set_alignment(CellAlignment::Center);
            let mut survived_cell = Cell::new(survived).set_alignment(CellAlignment::Center);
            let mut equivalent_cell = Cell::new(equivalent).set_alignment(CellAlignment::Center);
            let mut timeout_cell = Cell::new(timeout).set_alignment(CellAlignment::Center);

            if killed > 0.0 {
                killed_cell = killed_cell.fg(Color::Green);
            }
            row.add_cell(killed_cell);

            if survived > 0.0 {
                survived_cell = survived_cell.fg(Color::Red);
            }
            row.add_cell(survived_cell);

            if equivalent > 0.0 {
                equivalent_cell = equivalent_cell.fg(Color::Yellow);
            }
            row.add_cell(equivalent_cell);

            if timeout > 0.0 {
                timeout_cell = timeout_cell.fg(Color::Cyan);
            }
            row.add_cell(timeout_cell);

            if self.is_detailed {
                total_time_taken = total_time_taken.add(suite_result.duration);
                row.add_cell(Cell::new(format!("{:.2?}", suite_result.duration).to_string()));
            }

            let mut mutation_score: f64 = 0.0;
            if killed > 0.0 {
                mutation_score = ((killed / (killed + survived)) * 100.0) as f64;
            }
            let mut mutation_score_cell = Cell::new(format!("{:.2}", mutation_score).to_string())
                .set_alignment(CellAlignment::Center);

            mutation_score_cell = if mutation_score > 50.0 {
                mutation_score_cell.fg(Color::Green)
            } else {
                mutation_score_cell.fg(Color::Red)
            };

            row.add_cell(mutation_score_cell);

            self.table.add_row(row);
        }

        let mut footer = Row::from(vec![
            Cell::new("Total").set_alignment(CellAlignment::Center),
            Cell::new(total_killed).set_alignment(CellAlignment::Center),
            Cell::new(total_survived).set_alignment(CellAlignment::Center),
            Cell::new(total_equivalent).set_alignment(CellAlignment::Center),
            Cell::new(total_timeout).set_alignment(CellAlignment::Center),
        ]);

        if self.is_detailed {
            footer.add_cell(
                Cell::new(format!("{:.2?}", total_time_taken).to_string())
                    .set_alignment(CellAlignment::Left),
            );
        }

        let mut mutation_score: f64 = 0.0;
        if total_killed > 0.0 {
            mutation_score = ((total_killed / (total_killed + total_survived)) * 100.0) as f64;
        }
        let mut mutation_score_cell = Cell::new(format!("{:.2}", mutation_score).to_string())
            .set_alignment(CellAlignment::Center);
        mutation_score_cell = if mutation_score > 50.0 {
            mutation_score_cell.fg(Color::Green)
        } else {
            mutation_score_cell.fg(Color::Red)
        };

        footer.add_cell(mutation_score_cell);
        self.table.add_row(footer);

        println!("\n{}", self.table);
    }
}

pub fn mutant_diff(mutant: &Mutant) -> String {
    let orig_contents: String = String::from_utf8_lossy(mutant.source.contents()).into();
    let mutant_contents = mutant.as_source_string().unwrap();

    TextDiff::from_lines(&orig_contents, &mutant_contents)
        .unified_diff()
        .header("original", "mutant")
        .to_string()
}
