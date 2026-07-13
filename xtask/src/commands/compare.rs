use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use clap::ValueEnum;
use color_eyre::{
    Result,
    eyre::{Context, bail, eyre},
};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

const REPORT_SCHEMA_VERSION: u32 = 1;
const MINIMUM_NODE_VERSION: &str = ">=20.19.0";

/// A pinned real-world Tailwind corpus.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub(crate) enum CorpusName {
    #[value(name = "shadcn-ui")]
    ShadcnUi,
    #[value(name = "shadcn-svelte")]
    ShadcnSvelte,
    Astrowind,
}

impl CorpusName {
    fn as_str(self) -> &'static str {
        match self {
            Self::ShadcnUi => "shadcn-ui",
            Self::ShadcnSvelte => "shadcn-svelte",
            Self::Astrowind => "astrowind",
        }
    }
}

impl fmt::Display for CorpusName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CorpusKind {
    Tsx,
    Svelte,
    Astro,
}

impl CorpusKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Tsx => "tsx",
            Self::Svelte => "svelte",
            Self::Astro => "astro",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExpectedCorpus {
    files: u64,
    attributes: u64,
    fingerprint: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CorpusSpec {
    name: CorpusName,
    repository: &'static str,
    revision: &'static str,
    scope: &'static str,
    kind: CorpusKind,
    limit: u64,
    expected: ExpectedCorpus,
}

impl CorpusSpec {
    const fn new(
        name: CorpusName,
        repository: &'static str,
        revision: &'static str,
        scope: &'static str,
        kind: CorpusKind,
        limit: u64,
        expected: ExpectedCorpus,
    ) -> Self {
        Self {
            name,
            repository,
            revision,
            scope,
            kind,
            limit,
            expected,
        }
    }

    fn repository_url(self) -> String {
        format!("https://github.com/{}.git", self.repository)
    }
}

// update these fingerprints after intentionally refreshing the pinned corpus reports
const CORPORA: [CorpusSpec; 3] = [
    CorpusSpec::new(
        CorpusName::ShadcnUi,
        "shadcn-ui/ui",
        "bb8ba0daceb717eefc3a6ecb3457f28ca770d138",
        "apps/v4/app",
        CorpusKind::Tsx,
        40,
        ExpectedCorpus {
            files: 40,
            attributes: 834,
            fingerprint: "bf5e7fd497e6cfef0cccbc852d281a53b5d60a6f0e7ffd966ab7984b0259d0da",
        },
    ),
    CorpusSpec::new(
        CorpusName::ShadcnSvelte,
        "huntabyte/shadcn-svelte",
        "efcf8a4ef2c6a3a21ee2fd4db905519f8d4c8e63",
        "docs/src/lib/registry/examples",
        CorpusKind::Svelte,
        40,
        ExpectedCorpus {
            files: 40,
            attributes: 717,
            fingerprint: "2b1e23a82fff982fe3f270a2cefaffeac3290c2fc640a656dd77a176bf13d80e",
        },
    ),
    CorpusSpec::new(
        CorpusName::Astrowind,
        "onwidget/astrowind",
        "522530a242f5855f22a44a001f7b2e199669073d",
        "src",
        CorpusKind::Astro,
        53,
        ExpectedCorpus {
            files: 53,
            attributes: 379,
            fingerprint: "7045db395ba1d01792b2731e78c94744745a9e6da56a6920fdd71efde752ac3e",
        },
    ),
];

pub(crate) fn run(requested: &[CorpusName], offline: bool) -> Result<()> {
    CompareRunner::new(offline)?.run(requested)
}

struct CompareRunner {
    workspace: PathBuf,
    harness: PathBuf,
    output_directory: PathBuf,
    repository_directory: PathBuf,
    offline: bool,
}

struct RustywindBinary(PathBuf);

impl RustywindBinary {
    fn build(workspace: &Path, offline: bool) -> Result<Self> {
        let mut build = Command::new("cargo");
        build.current_dir(workspace).args([
            "build",
            "--release",
            "--package",
            "rustywind",
            "--bin",
            "rustywind",
            "--message-format=json-render-diagnostics",
        ]);
        if offline {
            build.arg("--offline");
        }

        let output = build
            .output()
            .wrap_err("failed to build the RustyWind release binary")?;
        let output = checked_output(output, "build the RustyWind release binary")?;
        let binary = Self(resolve_rustywind_binary(&output.stdout)?);
        if !binary.0.is_file() {
            bail!(
                "RustyWind release build reported a missing executable at {}",
                binary.0.display()
            );
        }
        Ok(binary)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl CompareRunner {
    fn new(offline: bool) -> Result<Self> {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| eyre!("xtask manifest directory has no workspace parent"))?
            .to_path_buf();
        let harness = workspace.join("tests/tailwind-compare");
        let target = workspace.join("target/tailwind-compare");

        Ok(Self {
            repository_directory: target.join("repos"),
            output_directory: target.join("results"),
            workspace,
            harness,
            offline,
        })
    }

    fn run(&self, requested: &[CorpusName]) -> Result<()> {
        let selected = select_corpora(requested);
        fs::create_dir_all(&self.repository_directory).wrap_err_with(|| {
            format!(
                "failed to create repository cache {}",
                self.repository_directory.display()
            )
        })?;
        fs::create_dir_all(&self.output_directory).wrap_err_with(|| {
            format!(
                "failed to create result directory {}",
                self.output_directory.display()
            )
        })?;

        let rustywind = match self.prepare_environment() {
            Ok(rustywind) => rustywind,
            Err(error) => {
                eprintln!("Comparison setup failed: {error:#}");
                let results = selected
                    .into_iter()
                    .map(|spec| CorpusRunResult::failed(spec, FailureReason::Setup))
                    .collect();
                let summary = ComparisonSummary::new(results);
                self.write_summary(&summary)?;
                self.print_summary(&summary);
                bail!(
                    "Tailwind comparison setup failed; see target/tailwind-compare/results/report.md"
                );
            }
        };

        let results = selected
            .into_iter()
            .map(|spec| self.run_corpus(spec, &rustywind))
            .collect::<Vec<_>>();
        let summary = ComparisonSummary::new(results);
        self.write_summary(&summary)?;
        self.print_summary(&summary);

        if summary.status == AggregateStatus::Failed {
            bail!("Tailwind comparison failed; see target/tailwind-compare/results/report.md");
        }

        Ok(())
    }

    fn prepare_environment(&self) -> Result<RustywindBinary> {
        validate_prerequisites()?;
        validate_harness(&self.harness)?;

        let build_result = RustywindBinary::build(&self.workspace, self.offline);

        let mut npm = Command::new("npm");
        npm.current_dir(&self.harness).arg("ci");
        if self.offline {
            npm.arg("--offline").env("npm_config_offline", "true");
        }
        let npm_result = run_checked(&mut npm, "install comparison harness dependencies");

        if let Err(error) = &build_result {
            eprintln!("RustyWind build failed: {error:#}");
        }
        if let Err(error) = &npm_result {
            eprintln!("Comparison dependency installation failed: {error:#}");
        }
        if build_result.is_err() || npm_result.is_err() {
            bail!("comparison environment preparation failed");
        }

        build_result
    }

    fn run_corpus(&self, spec: CorpusSpec, rustywind: &RustywindBinary) -> CorpusRunResult {
        println!("Comparing {} at {}", spec.name, spec.revision);

        let repository = match self.prepare_repository(spec) {
            Ok(repository) => repository,
            Err(error) => {
                return failed_corpus(spec, FailureReason::RepositoryUnavailable, error);
            }
        };
        let report = match self.invoke_comparison(spec, &repository, rustywind) {
            Ok(report) => report,
            Err(error) => return failed_corpus(spec, FailureReason::InvocationFailed, error),
        };
        let report = match validate_report(spec, report) {
            Ok(report) => report,
            Err(error) => return failed_corpus(spec, FailureReason::InvalidReport, error),
        };
        CorpusRunResult::from_report(spec, &report)
    }

    fn prepare_repository(&self, spec: CorpusSpec) -> Result<PathBuf> {
        let repository = self.repository_directory.join(spec.name.as_str());
        if !repository.exists() {
            if self.offline {
                bail!(
                    "{} is not cached; rerun without --offline to fetch it",
                    spec.name
                );
            }
            let mut init = Command::new("git");
            init.args(["init", "--quiet"]).arg(&repository);
            run_checked(&mut init, "initialize corpus repository")?;
        }

        validate_git_repository(&repository)?;
        ensure_clean_repository(&repository)?;

        if !self.offline {
            self.configure_remote(&repository, spec)?;
            let mut fetch = Command::new("git");
            fetch.current_dir(&repository).args([
                "fetch",
                "--quiet",
                "--depth=1",
                "origin",
                spec.revision,
            ]);
            run_checked(&mut fetch, "fetch pinned corpus revision")?;
        }

        let mut checkout = Command::new("git");
        checkout
            .current_dir(&repository)
            .args(["checkout", "--quiet", "--detach", spec.revision]);
        run_checked(&mut checkout, "check out pinned corpus revision")?;

        let actual_revision = command_stdout(
            Command::new("git")
                .current_dir(&repository)
                .args(["rev-parse", "HEAD"]),
            "read corpus revision",
        )?;
        validate_exact_revision(spec.revision, actual_revision.trim())?;

        let branch = command_stdout(
            Command::new("git").current_dir(&repository).args([
                "rev-parse",
                "--abbrev-ref",
                "HEAD",
            ]),
            "verify detached corpus checkout",
        )?;
        if branch.trim() != "HEAD" {
            bail!("{} checkout is attached to branch {branch}", spec.name);
        }

        Ok(repository)
    }

    fn configure_remote(&self, repository: &Path, spec: CorpusSpec) -> Result<()> {
        let expected_url = spec.repository_url();
        let output = Command::new("git")
            .current_dir(repository)
            .args(["remote", "get-url", "origin"])
            .output()
            .wrap_err("failed to inspect corpus remote")?;

        if output.status.success() {
            let actual_url = String::from_utf8(output.stdout)
                .wrap_err("corpus remote URL was not valid UTF-8")?;
            if actual_url.trim() != expected_url {
                bail!(
                    "{} cache has unexpected origin URL {}; expected {expected_url}",
                    spec.name,
                    actual_url.trim()
                );
            }
            return Ok(());
        }

        let mut add_remote = Command::new("git");
        add_remote
            .current_dir(repository)
            .args(["remote", "add", "origin", &expected_url]);
        run_checked(&mut add_remote, "configure corpus remote")
    }

    fn invoke_comparison(
        &self,
        spec: CorpusSpec,
        repository: &Path,
        rustywind: &RustywindBinary,
    ) -> Result<NodeReport> {
        let output_path = self
            .output_directory
            .join(format!("{}.json", spec.name.as_str()));
        if output_path.exists() {
            fs::remove_file(&output_path).wrap_err_with(|| {
                format!("failed to remove stale report {}", output_path.display())
            })?;
        }

        let mut node = Command::new("node");
        node.current_dir(&self.harness)
            .arg("compare.mjs")
            .args(["--corpus", spec.name.as_str()])
            .args(["--repo"])
            .arg(repository)
            .args(["--revision", spec.revision])
            .args(["--scope", spec.scope])
            .args(["--kind", spec.kind.as_str()])
            .args(["--limit", &spec.limit.to_string()])
            .args(["--rustywind"])
            .arg(rustywind.path())
            .args(["--stylesheet"])
            .arg(self.harness.join("tailwind.css"))
            .args(["--output"])
            .arg(&output_path);
        run_checked(&mut node, "run corpus comparison")?;

        let report = fs::read_to_string(&output_path)
            .wrap_err_with(|| format!("failed to read report {}", output_path.display()))?;
        serde_json::from_str(&report)
            .wrap_err_with(|| format!("failed to parse report {}", output_path.display()))
    }

    fn write_summary(&self, summary: &ComparisonSummary) -> Result<()> {
        let json = serde_json::to_string_pretty(summary)
            .wrap_err("failed to serialize comparison summary")?;
        fs::write(
            self.output_directory.join("summary.json"),
            format!("{json}\n"),
        )
        .wrap_err("failed to write comparison summary JSON")?;
        fs::write(self.output_directory.join("report.md"), summary.markdown())
            .wrap_err("failed to write comparison Markdown report")
    }

    fn print_summary(&self, summary: &ComparisonSummary) {
        println!();
        println!("Tailwind comparison: {}", summary.status);
        for corpus in &summary.corpora {
            println!("  {} {}: {}", corpus.name, corpus.revision, corpus.status);
        }
        println!("Results: target/tailwind-compare/results/report.md");
    }
}

fn failed_corpus(
    spec: CorpusSpec,
    reason: FailureReason,
    error: color_eyre::Report,
) -> CorpusRunResult {
    eprintln!("{} failed: {error:#}", spec.name);
    CorpusRunResult::failed(spec, reason)
}

fn select_corpora(requested: &[CorpusName]) -> Vec<CorpusSpec> {
    if requested.is_empty() {
        return CORPORA.to_vec();
    }

    let requested = requested.iter().copied().collect::<HashSet<_>>();
    CORPORA
        .iter()
        .copied()
        .filter(|spec| requested.contains(&spec.name))
        .collect()
}

fn validate_prerequisites() -> Result<()> {
    for executable in ["cargo", "git", "node", "npm"] {
        which::which(executable)
            .wrap_err_with(|| format!("required executable `{executable}` was not found"))?;
    }

    let version = command_stdout(
        Command::new("node").arg("--version"),
        "read Node.js version",
    )?;
    let version = Version::parse(version.trim().trim_start_matches('v'))
        .wrap_err_with(|| format!("Node.js returned an invalid version: {version:?}"))?;
    let requirement = VersionReq::parse(MINIMUM_NODE_VERSION)
        .wrap_err("invalid built-in Node.js version requirement")?;
    if !requirement.matches(&version) {
        bail!("Node.js {MINIMUM_NODE_VERSION} is required; found {version}");
    }

    Ok(())
}

fn validate_harness(harness: &Path) -> Result<()> {
    for relative in [
        "compare.mjs",
        "package.json",
        "package-lock.json",
        "tailwind.css",
    ] {
        let path = harness.join(relative);
        if !path.is_file() {
            bail!("comparison harness is missing {relative}");
        }
    }
    Ok(())
}

fn validate_git_repository(repository: &Path) -> Result<()> {
    let inside_work_tree = command_stdout(
        Command::new("git")
            .current_dir(repository)
            .args(["rev-parse", "--is-inside-work-tree"]),
        "validate corpus repository",
    )?;
    if inside_work_tree.trim() != "true" {
        bail!("corpus cache is not a Git work tree");
    }
    Ok(())
}

fn ensure_clean_repository(repository: &Path) -> Result<()> {
    let status = command_stdout(
        Command::new("git")
            .current_dir(repository)
            .args(["status", "--porcelain"]),
        "inspect corpus repository status",
    )?;
    if !status.trim().is_empty() {
        bail!("corpus cache has local changes");
    }
    Ok(())
}

fn validate_exact_revision(expected: &str, actual: &str) -> Result<()> {
    if actual != expected {
        bail!("corpus revision mismatch: expected {expected}, found {actual}");
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
enum CargoBuildMessage {
    CompilerArtifact {
        target: CargoTarget,
        executable: Option<PathBuf>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<CargoTargetKind>,
}

#[derive(Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CargoTargetKind {
    Bin,
    #[serde(other)]
    Other,
}

fn resolve_rustywind_binary(build_output: &[u8]) -> Result<PathBuf> {
    let build_output =
        std::str::from_utf8(build_output).wrap_err("Cargo build output was not valid UTF-8")?;
    let mut executables = build_output
        .lines()
        .map(serde_json::from_str::<CargoBuildMessage>)
        .collect::<serde_json::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|message| match message {
            CargoBuildMessage::CompilerArtifact { target, executable }
                if target.name == "rustywind" && target.kind.contains(&CargoTargetKind::Bin) =>
            {
                executable
            }
            CargoBuildMessage::CompilerArtifact { .. } | CargoBuildMessage::Other => None,
        });
    let executable = executables
        .next()
        .ok_or_else(|| eyre!("Cargo did not report the RustyWind executable artifact"))?;
    if let Some(other) = executables.next() {
        bail!(
            "Cargo reported multiple RustyWind executable artifacts: {} and {}",
            executable.display(),
            other.display()
        );
    }
    Ok(executable)
}

fn command_stdout(command: &mut Command, description: &str) -> Result<String> {
    let output = command
        .output()
        .wrap_err_with(|| format!("failed to {description}"))?;
    checked_output(output, description).and_then(|output| {
        String::from_utf8(output.stdout)
            .wrap_err_with(|| format!("{description} returned non-UTF-8 output"))
    })
}

fn run_checked(command: &mut Command, description: &str) -> Result<()> {
    let output = command
        .output()
        .wrap_err_with(|| format!("failed to {description}"))?;
    checked_output(output, description).map(|_| ())
}

fn checked_output(output: Output, description: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    bail!("failed to {description}: {detail}");
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeReport {
    schema_version: u32,
    corpus: ReportCorpus,
    versions: ToolVersions,
    summary: ReportSummary,
    candidates: Vec<Candidate>,
    details: Vec<ComparisonDetail>,
    failures: Vec<ReportFailure>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportCorpus {
    name: String,
    revision: String,
    scope: String,
    kind: String,
    limit: u64,
    files: u64,
    attributes: u64,
    fingerprint: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolVersions {
    prettier: String,
    prettier_plugin_tailwindcss: String,
    tailwindcss: String,
    prettier_plugin_astro: String,
    prettier_plugin_svelte: String,
}

impl ToolVersions {
    fn validate(&self) -> Result<()> {
        for (name, version) in [
            ("prettier", self.prettier.as_str()),
            (
                "prettierPluginTailwindcss",
                self.prettier_plugin_tailwindcss.as_str(),
            ),
            ("tailwindcss", self.tailwindcss.as_str()),
            ("prettierPluginAstro", self.prettier_plugin_astro.as_str()),
            ("prettierPluginSvelte", self.prettier_plugin_svelte.as_str()),
        ] {
            if version.trim().is_empty() {
                bail!("report has an empty {name} version");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportSummary {
    prettier_changed: u64,
    rustywind_changed: u64,
    exact: u64,
    custom_only: u64,
    known_order_mismatch: u64,
    extraction_mismatch: u64,
    failures: u64,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    path: String,
    attributes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
enum ComparisonDetail {
    AttributeCount {
        path: String,
        expected: u64,
        original: u64,
        prettier: u64,
        rustywind: u64,
    },
    TokenMultiset {
        path: String,
        attribute: u64,
        original: Vec<String>,
        prettier: Vec<String>,
        rustywind: Vec<String>,
    },
    CustomOnly {
        path: String,
        attribute: u64,
        original: Vec<String>,
        prettier: Vec<String>,
        rustywind: Vec<String>,
        prettier_unknown: Vec<String>,
    },
    KnownOrder {
        path: String,
        attribute: u64,
        original: Vec<String>,
        prettier: Vec<String>,
        rustywind: Vec<String>,
        prettier_unknown: Vec<String>,
    },
}

impl ComparisonDetail {
    fn path(&self) -> &str {
        match self {
            Self::AttributeCount { path, .. }
            | Self::TokenMultiset { path, .. }
            | Self::CustomOnly { path, .. }
            | Self::KnownOrder { path, .. } => path,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReportFailure {
    path: String,
    stage: String,
    error: String,
}

fn validate_report(spec: CorpusSpec, report: NodeReport) -> Result<NodeReport> {
    if report.schema_version != REPORT_SCHEMA_VERSION {
        bail!(
            "unsupported report schema version {}; expected {REPORT_SCHEMA_VERSION}",
            report.schema_version
        );
    }
    validate_report_metadata(spec, &report.corpus)?;
    report.versions.validate()?;
    validate_candidates(&report)?;
    validate_details(&report)?;
    validate_failures(&report)?;
    validate_fingerprint(spec.expected.fingerprint, &report.corpus.fingerprint)?;
    Ok(report)
}

fn validate_report_metadata(spec: CorpusSpec, actual: &ReportCorpus) -> Result<()> {
    let expected = [
        ("name", spec.name.as_str(), actual.name.as_str()),
        ("revision", spec.revision, actual.revision.as_str()),
        ("scope", spec.scope, actual.scope.as_str()),
        ("kind", spec.kind.as_str(), actual.kind.as_str()),
    ];
    for (field, expected, actual) in expected {
        if actual != expected {
            bail!("report {field} mismatch: expected {expected}, found {actual}");
        }
    }
    if actual.limit != spec.limit {
        bail!(
            "report limit mismatch: expected {}, found {}",
            spec.limit,
            actual.limit
        );
    }
    if actual.files != spec.expected.files {
        bail!(
            "report file count mismatch: expected {}, found {}",
            spec.expected.files,
            actual.files
        );
    }
    if actual.attributes != spec.expected.attributes {
        bail!(
            "report attribute count mismatch: expected {}, found {}",
            spec.expected.attributes,
            actual.attributes
        );
    }
    Ok(())
}

fn validate_candidates(report: &NodeReport) -> Result<()> {
    if report.candidates.len() as u64 != report.corpus.files {
        bail!(
            "candidate count mismatch: metadata says {}, report contains {}",
            report.corpus.files,
            report.candidates.len()
        );
    }

    let mut paths = HashSet::new();
    let mut attributes = 0_u64;
    for candidate in &report.candidates {
        let path = Path::new(&candidate.path);
        if path.is_absolute() || candidate.path.trim().is_empty() {
            bail!("candidate path must be a non-empty repository-relative path");
        }
        if !paths.insert(candidate.path.as_str()) {
            bail!("candidate path appears more than once: {}", candidate.path);
        }
        attributes = attributes
            .checked_add(candidate.attributes)
            .ok_or_else(|| eyre!("candidate attribute count overflow"))?;
    }
    if attributes != report.corpus.attributes {
        bail!(
            "candidate attribute count mismatch: metadata says {}, candidates contain {attributes}",
            report.corpus.attributes
        );
    }

    if report.summary.prettier_changed > report.corpus.attributes
        || report.summary.rustywind_changed > report.corpus.attributes
    {
        bail!("summary changed count exceeds corpus attributes");
    }
    Ok(())
}

fn validate_details(report: &NodeReport) -> Result<()> {
    let candidate_attributes = report
        .candidates
        .iter()
        .map(|candidate| (candidate.path.as_str(), candidate.attributes))
        .collect::<HashMap<_, _>>();
    let mut attribute_details = HashSet::new();
    let mut whole_file_details = HashSet::new();
    let mut custom_only = 0_u64;
    let mut known_order = 0_u64;
    let mut extraction = 0_u64;

    for detail in &report.details {
        let path = detail.path();
        if Path::new(path).is_absolute() || path.trim().is_empty() {
            bail!("report contains an invalid detail path");
        }
        let candidate_attributes = *candidate_attributes
            .get(path)
            .ok_or_else(|| eyre!("detail refers to a file outside the candidate set"))?;

        match detail {
            ComparisonDetail::AttributeCount {
                expected,
                original,
                prettier,
                rustywind,
                ..
            } => {
                if *expected != candidate_attributes {
                    bail!("attribute-count detail does not match candidate metadata");
                }
                if original == expected && prettier == expected && rustywind == expected {
                    bail!("attribute-count detail does not describe a mismatch");
                }
                if !whole_file_details.insert(path) {
                    bail!("candidate has more than one attribute-count detail");
                }
                extraction = extraction
                    .checked_add(*expected)
                    .ok_or_else(|| eyre!("detail extraction count overflow"))?;
            }
            ComparisonDetail::TokenMultiset {
                attribute,
                original,
                prettier,
                rustywind,
                ..
            } => {
                validate_attribute_detail(
                    path,
                    *attribute,
                    candidate_attributes,
                    original,
                    prettier,
                    rustywind,
                    &mut attribute_details,
                )?;
                extraction = extraction
                    .checked_add(1)
                    .ok_or_else(|| eyre!("detail extraction count overflow"))?;
            }
            ComparisonDetail::CustomOnly {
                attribute,
                original,
                prettier,
                rustywind,
                prettier_unknown,
                ..
            } => {
                validate_attribute_detail(
                    path,
                    *attribute,
                    candidate_attributes,
                    original,
                    prettier,
                    rustywind,
                    &mut attribute_details,
                )?;
                validate_unknown_tokens(prettier_unknown, rustywind)?;
                custom_only = custom_only
                    .checked_add(1)
                    .ok_or_else(|| eyre!("custom-only detail count overflow"))?;
            }
            ComparisonDetail::KnownOrder {
                attribute,
                original,
                prettier,
                rustywind,
                prettier_unknown,
                ..
            } => {
                validate_attribute_detail(
                    path,
                    *attribute,
                    candidate_attributes,
                    original,
                    prettier,
                    rustywind,
                    &mut attribute_details,
                )?;
                validate_unknown_tokens(prettier_unknown, rustywind)?;
                known_order = known_order
                    .checked_add(1)
                    .ok_or_else(|| eyre!("known-order detail count overflow"))?;
            }
        }
    }

    if attribute_details
        .iter()
        .any(|(path, _)| whole_file_details.contains(path.as_str()))
    {
        bail!("attribute-count detail overlaps per-attribute details for the same candidate");
    }
    if custom_only != report.summary.custom_only
        || known_order != report.summary.known_order_mismatch
        || extraction != report.summary.extraction_mismatch
    {
        bail!("detail classifications do not match report summary");
    }
    Ok(())
}

fn validate_attribute_detail(
    path: &str,
    attribute: u64,
    candidate_attributes: u64,
    original: &[String],
    prettier: &[String],
    rustywind: &[String],
    seen: &mut HashSet<(String, u64)>,
) -> Result<()> {
    if attribute >= candidate_attributes {
        bail!("detail attribute index is outside its candidate");
    }
    if prettier == rustywind {
        bail!("comparison detail does not describe a mismatch");
    }
    if original.iter().any(|token| token.contains('\0'))
        || prettier.iter().any(|token| token.contains('\0'))
        || rustywind.iter().any(|token| token.contains('\0'))
    {
        bail!("comparison detail contains an invalid class token");
    }
    if !seen.insert((path.to_string(), attribute)) {
        bail!("candidate attribute appears in more than one detail");
    }
    Ok(())
}

fn validate_unknown_tokens(unknown: &[String], rustywind: &[String]) -> Result<()> {
    if unknown.iter().any(|token| !rustywind.contains(token)) {
        bail!("detail unknown token is absent from the RustyWind tokens");
    }
    Ok(())
}

fn validate_failures(report: &NodeReport) -> Result<()> {
    if report.failures.len() as u64 != report.summary.failures {
        bail!(
            "failure count mismatch: summary says {}, report contains {}",
            report.summary.failures,
            report.failures.len()
        );
    }
    let candidate_attributes = report
        .candidates
        .iter()
        .map(|candidate| (candidate.path.as_str(), candidate.attributes))
        .collect::<HashMap<_, _>>();
    let mut failed_paths = HashSet::new();
    let mut failed_attributes = 0_u64;
    for failure in &report.failures {
        if Path::new(&failure.path).is_absolute()
            || failure.path.trim().is_empty()
            || failure.stage.trim().is_empty()
            || failure.error.trim().is_empty()
        {
            bail!("report contains an invalid failure record");
        }
        if !failed_paths.insert(failure.path.as_str()) {
            bail!("failure path appears more than once: {}", failure.path);
        }
        let attributes = candidate_attributes
            .get(failure.path.as_str())
            .ok_or_else(|| eyre!("failure refers to a file outside the candidate set"))?;
        failed_attributes = failed_attributes
            .checked_add(*attributes)
            .ok_or_else(|| eyre!("failed candidate attribute count overflow"))?;
    }
    if report
        .details
        .iter()
        .any(|detail| failed_paths.contains(detail.path()))
    {
        bail!("failed candidate also contains comparison details");
    }

    let classified_attributes = report
        .summary
        .exact
        .checked_add(report.summary.custom_only)
        .and_then(|count| count.checked_add(report.summary.known_order_mismatch))
        .and_then(|count| count.checked_add(report.summary.extraction_mismatch))
        .ok_or_else(|| eyre!("summary classification count overflow"))?;
    if classified_attributes.checked_add(failed_attributes) != Some(report.corpus.attributes) {
        bail!("summary classifications and failed files do not account for all corpus attributes");
    }
    Ok(())
}

fn validate_fingerprint(expected: &str, actual: &str) -> Result<()> {
    validate_sha256("expected", expected)?;
    validate_sha256("reported", actual)?;
    if actual != expected {
        bail!("corpus fingerprint mismatch: expected {expected}, found {actual}");
    }
    Ok(())
}

fn validate_sha256(label: &str, value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} fingerprint is not a lowercase SHA-256 digest");
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum FailureReason {
    Setup,
    RepositoryUnavailable,
    InvocationFailed,
    InvalidReport,
    KnownOrderMismatch,
    ExtractionMismatch,
    HarnessFailure,
}

impl FailureReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::RepositoryUnavailable => "repository unavailable",
            Self::InvocationFailed => "invocation failed",
            Self::InvalidReport => "invalid report",
            Self::KnownOrderMismatch => "known-order mismatch",
            Self::ExtractionMismatch => "extraction mismatch",
            Self::HarnessFailure => "harness failure",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CorpusStatus {
    Passed,
    Failed,
}

impl fmt::Display for CorpusStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Passed => formatter.write_str("passed"),
            Self::Failed => formatter.write_str("failed"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusRunResult {
    name: &'static str,
    repository: &'static str,
    revision: &'static str,
    status: CorpusStatus,
    expected: ExpectedCounts,
    actual: Option<ActualCounts>,
    reasons: Vec<FailureReason>,
}

impl CorpusRunResult {
    fn failed(spec: CorpusSpec, reason: FailureReason) -> Self {
        Self {
            name: spec.name.as_str(),
            repository: spec.repository,
            revision: spec.revision,
            status: CorpusStatus::Failed,
            expected: ExpectedCounts::from(spec.expected),
            actual: None,
            reasons: vec![reason],
        }
    }

    fn from_report(spec: CorpusSpec, report: &NodeReport) -> Self {
        let mut reasons = Vec::new();
        if report.summary.known_order_mismatch > 0 {
            reasons.push(FailureReason::KnownOrderMismatch);
        }
        if report.summary.extraction_mismatch > 0 {
            reasons.push(FailureReason::ExtractionMismatch);
        }
        if report.summary.failures > 0 {
            reasons.push(FailureReason::HarnessFailure);
        }

        Self {
            name: spec.name.as_str(),
            repository: spec.repository,
            revision: spec.revision,
            status: if reasons.is_empty() {
                CorpusStatus::Passed
            } else {
                CorpusStatus::Failed
            },
            expected: ExpectedCounts::from(spec.expected),
            actual: Some(ActualCounts {
                files: report.corpus.files,
                attributes: report.corpus.attributes,
                fingerprint: report.corpus.fingerprint.clone(),
                summary: report.summary,
            }),
            reasons,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedCounts {
    files: u64,
    attributes: u64,
    fingerprint: &'static str,
}

impl From<ExpectedCorpus> for ExpectedCounts {
    fn from(expected: ExpectedCorpus) -> Self {
        Self {
            files: expected.files,
            attributes: expected.attributes,
            fingerprint: expected.fingerprint,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActualCounts {
    files: u64,
    attributes: u64,
    fingerprint: String,
    summary: ReportSummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum AggregateStatus {
    Passed,
    Failed,
}

impl fmt::Display for AggregateStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Passed => formatter.write_str("passed"),
            Self::Failed => formatter.write_str("failed"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ComparisonSummary {
    schema_version: u32,
    status: AggregateStatus,
    corpora: Vec<CorpusRunResult>,
}

impl ComparisonSummary {
    fn new(corpora: Vec<CorpusRunResult>) -> Self {
        let status = if corpora
            .iter()
            .all(|corpus| corpus.status == CorpusStatus::Passed)
        {
            AggregateStatus::Passed
        } else {
            AggregateStatus::Failed
        };
        Self {
            schema_version: REPORT_SCHEMA_VERSION,
            status,
            corpora,
        }
    }

    fn markdown(&self) -> String {
        let mut markdown = format!(
            "# Tailwind comparison\n\nOverall status: **{}**\n\n| Corpus | Revision | Files | Attributes | Exact | Custom only | Known-order mismatch | Extraction mismatch | Failures | Status | Reason |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |\n",
            self.status
        );
        for corpus in &self.corpora {
            let actual = corpus.actual.as_ref();
            let files = actual
                .map(|actual| actual.files.to_string())
                .unwrap_or_else(|| "—".to_string());
            let attributes = actual
                .map(|actual| actual.attributes.to_string())
                .unwrap_or_else(|| "—".to_string());
            let metric = |select: fn(&ReportSummary) -> u64| {
                actual
                    .map(|actual| select(&actual.summary).to_string())
                    .unwrap_or_else(|| "—".to_string())
            };
            let reasons = if corpus.reasons.is_empty() {
                "—".to_string()
            } else {
                corpus
                    .reasons
                    .iter()
                    .map(|reason| reason.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            markdown.push_str(&format!(
                "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                corpus.name,
                corpus.revision,
                files,
                attributes,
                metric(|summary| summary.exact),
                metric(|summary| summary.custom_only),
                metric(|summary| summary.known_order_mismatch),
                metric(|summary| summary.extraction_mismatch),
                metric(|summary| summary.failures),
                corpus.status,
                reasons
            ));
        }
        markdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FINGERPRINT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn binary_resolution_uses_cargo_reported_executable() {
        let expected = PathBuf::from(
            "/custom-target/aarch64-unknown-linux-gnu/release/rustywind-custom-suffix",
        );
        let output = format!(
            "{{\"reason\":\"compiler-artifact\",\"target\":{{\"name\":\"rustywind_core\",\"kind\":[\"lib\"]}},\"executable\":null}}\n{{\"reason\":\"compiler-artifact\",\"target\":{{\"name\":\"rustywind\",\"kind\":[\"bin\"]}},\"executable\":{}}}\n{{\"reason\":\"build-finished\",\"success\":true}}\n",
            serde_json::to_string(&expected).unwrap()
        );

        assert_eq!(
            resolve_rustywind_binary(output.as_bytes()).unwrap(),
            expected
        );
    }

    #[test]
    fn empty_selection_uses_every_corpus_in_canonical_order() {
        let selected = select_corpora(&[]);

        assert_eq!(
            selected.iter().map(|spec| spec.name).collect::<Vec<_>>(),
            vec![
                CorpusName::ShadcnUi,
                CorpusName::ShadcnSvelte,
                CorpusName::Astrowind
            ]
        );
    }

    #[test]
    fn explicit_selection_is_deduplicated_and_uses_canonical_order() {
        let selected = select_corpora(&[
            CorpusName::Astrowind,
            CorpusName::ShadcnUi,
            CorpusName::Astrowind,
        ]);

        assert_eq!(
            selected.iter().map(|spec| spec.name).collect::<Vec<_>>(),
            vec![CorpusName::ShadcnUi, CorpusName::Astrowind]
        );
    }

    #[test]
    fn exact_revision_validation_rejects_abbreviated_and_different_revisions() {
        let revision = CORPORA[0].revision;

        assert!(validate_exact_revision(revision, revision).is_ok());
        assert!(validate_exact_revision(revision, &revision[..12]).is_err());
        assert!(
            validate_exact_revision(revision, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err()
        );
    }

    #[test]
    fn fingerprint_validation_requires_the_exact_expected_value() {
        assert!(validate_fingerprint(FINGERPRINT, FINGERPRINT).is_ok());
        assert!(
            validate_fingerprint(
                FINGERPRINT,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            )
            .is_err()
        );
        assert!(validate_fingerprint("not-a-fingerprint", FINGERPRINT).is_err());
    }

    #[test]
    fn custom_only_does_not_fail_aggregate_status() {
        let report = report_with_summary(ReportSummary {
            prettier_changed: 1,
            rustywind_changed: 1,
            exact: 0,
            custom_only: 1,
            known_order_mismatch: 0,
            extraction_mismatch: 0,
            failures: 0,
        });
        let result = CorpusRunResult::from_report(test_spec(), &report);

        assert_eq!(result.status, CorpusStatus::Passed);
        assert_eq!(
            ComparisonSummary::new(vec![result]).status,
            AggregateStatus::Passed
        );
    }

    #[test]
    fn any_actionable_mismatch_fails_aggregate_status() {
        for summary in [
            ReportSummary {
                prettier_changed: 1,
                rustywind_changed: 1,
                exact: 0,
                custom_only: 0,
                known_order_mismatch: 1,
                extraction_mismatch: 0,
                failures: 0,
            },
            ReportSummary {
                prettier_changed: 1,
                rustywind_changed: 1,
                exact: 0,
                custom_only: 0,
                known_order_mismatch: 0,
                extraction_mismatch: 1,
                failures: 0,
            },
            ReportSummary {
                prettier_changed: 0,
                rustywind_changed: 0,
                exact: 0,
                custom_only: 0,
                known_order_mismatch: 0,
                extraction_mismatch: 0,
                failures: 1,
            },
        ] {
            let result = CorpusRunResult::from_report(test_spec(), &report_with_summary(summary));
            assert_eq!(
                ComparisonSummary::new(vec![result]).status,
                AggregateStatus::Failed
            );
        }
    }

    #[test]
    fn detail_classifications_must_match_summary_counts() {
        let mut report = report_with_summary(ReportSummary {
            prettier_changed: 1,
            rustywind_changed: 1,
            exact: 1,
            custom_only: 0,
            known_order_mismatch: 0,
            extraction_mismatch: 0,
            failures: 0,
        });
        report.details.push(ComparisonDetail::KnownOrder {
            path: "src/file.tsx".to_string(),
            attribute: 0,
            original: vec!["px-2".to_string(), "flex".to_string()],
            prettier: vec!["flex".to_string(), "px-2".to_string()],
            rustywind: vec!["px-2".to_string(), "flex".to_string()],
            prettier_unknown: Vec::new(),
        });

        assert!(validate_details(&report).is_err());
        report.summary.exact = 0;
        report.summary.known_order_mismatch = 1;
        assert!(validate_details(&report).is_ok());
    }

    #[test]
    fn deserializes_camel_case_detail_fields() {
        let detail = serde_json::from_str::<ComparisonDetail>(
            r#"{
                "kind": "custom-only",
                "path": "src/file.tsx",
                "attribute": 0,
                "original": ["brand-card", "flex"],
                "prettier": ["brand-card", "flex"],
                "rustywind": ["flex", "brand-card"],
                "prettierUnknown": ["brand-card"]
            }"#,
        )
        .unwrap();

        assert!(matches!(detail, ComparisonDetail::CustomOnly { .. }));
    }

    fn test_spec() -> CorpusSpec {
        CorpusSpec::new(
            CorpusName::ShadcnUi,
            "example/repository",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "src",
            CorpusKind::Tsx,
            1,
            ExpectedCorpus {
                files: 1,
                attributes: 1,
                fingerprint: FINGERPRINT,
            },
        )
    }

    fn report_with_summary(summary: ReportSummary) -> NodeReport {
        NodeReport {
            schema_version: REPORT_SCHEMA_VERSION,
            corpus: ReportCorpus {
                name: "shadcn-ui".to_string(),
                revision: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                scope: "src".to_string(),
                kind: "tsx".to_string(),
                limit: 1,
                files: 1,
                attributes: 1,
                fingerprint: FINGERPRINT.to_string(),
            },
            versions: ToolVersions {
                prettier: "3.0.0".to_string(),
                prettier_plugin_tailwindcss: "0.7.0".to_string(),
                tailwindcss: "4.0.0".to_string(),
                prettier_plugin_astro: "0.14.0".to_string(),
                prettier_plugin_svelte: "3.0.0".to_string(),
            },
            summary,
            candidates: vec![Candidate {
                path: "src/file.tsx".to_string(),
                attributes: 1,
            }],
            details: Vec::new(),
            failures: Vec::new(),
        }
    }
}
