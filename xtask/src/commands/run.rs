use crate::utils::{categories::ClassCategory, parser};
use color_eyre::{Result, eyre::Context};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
struct SimpleFailure {
    prettier: String,
    rustywind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DetailedFailure {
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    test: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prettier: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rustywind: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<usize>,
    #[serde(skip)]
    hash: u64,
    count: usize,
}

impl DetailedFailure {
    fn compute_hash(input: &[String]) -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug)]
struct TestResult {
    passed: Option<usize>,
    detailed_failures: Vec<DetailedFailure>,
    success: bool,
    error: Option<String>,
}

fn check_prerequisites() -> Result<()> {
    let mut errors = Vec::new();

    // check RustyWind binary
    let binary_path = Path::new("target/release/rustywind");
    if !binary_path.exists() {
        errors.push(format!(
            "RustyWind binary not found at: {}\n   Build it with: cargo build --release",
            binary_path.display()
        ));
    } else {
        // check if executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = binary_path.metadata()?;
            let mode = metadata.permissions().mode();
            if mode & 0o111 == 0 {
                errors.push(format!(
                    "RustyWind binary exists but is not executable: {}",
                    binary_path.display()
                ));
            }
        }
    }

    // check npm
    if which::which("npm").is_err() {
        errors.push(
            "npm not found in PATH\n   Install Node.js from: https://nodejs.org/".to_string(),
        );
    }

    // check node_modules
    let node_modules = Path::new("tests/fuzz/node_modules");
    if !node_modules.exists() {
        errors.push(
            "npm dependencies not installed\n   Run: cd tests/fuzz && npm install".to_string(),
        );
    }

    // check package.json
    let package_json = Path::new("tests/fuzz/package.json");
    if !package_json.exists() {
        errors.push(format!(
            "package.json not found at: {}",
            package_json.display()
        ));
    }

    if !errors.is_empty() {
        eprintln!("Pre-flight check failed:\n");
        for error in errors {
            eprintln!("  {}\n", error);
        }
        std::process::exit(1);
    }

    println!("All prerequisites present\n");
    Ok(())
}

fn get_default_workers() -> usize {
    std::env::var("FUZZ_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            let cpu_count = num_cpus::get();
            cpu_count.clamp(2, 8)
        })
}

fn run_single_test(seed: &str) -> TestResult {
    let result = Command::new("npm")
        .arg("test")
        .current_dir("tests/fuzz")
        .env("FUZZ_SEED", seed)
        .env("DETAILED_OUTPUT", "1")
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // parse pass count
            let passed = parser::parse_pass_count(&stdout);

            // parse detailed JSON failures
            let mut detailed_failures = Vec::new();
            if let Some(json_start) = combined.find("__DETAILED_FAILURES_JSON__")
                && let Some(json_end) = combined.find("__END_DETAILED_FAILURES_JSON__")
            {
                let json_str = &combined[json_start + "__DETAILED_FAILURES_JSON__".len()..json_end];

                // define temporary struct for parsing
                #[derive(Deserialize)]
                struct JsFailure {
                    test: Option<usize>,
                    error: Option<String>,
                    position: Option<usize>,
                    original: Vec<String>,
                    prettier: Option<Vec<String>>,
                    rustywind: Option<Vec<String>>,
                }

                if let Ok(js_failures) = serde_json::from_str::<Vec<JsFailure>>(json_str) {
                    detailed_failures = js_failures
                        .into_iter()
                        .map(|f| {
                            let hash = DetailedFailure::compute_hash(&f.original);
                            DetailedFailure {
                                test: f.test,
                                error: f.error,
                                input: f.original,
                                prettier: f.prettier,
                                rustywind: f.rustywind,
                                position: f.position,
                                hash,
                                count: 1,
                            }
                        })
                        .collect();
                }
            }

            if passed.is_some() {
                TestResult {
                    passed,
                    detailed_failures,
                    success: true,
                    error: None,
                }
            } else {
                TestResult {
                    passed: None,
                    detailed_failures: Vec::new(),
                    success: false,
                    error: Some("parse_failed".to_string()),
                }
            }
        }
        Err(e) => TestResult {
            passed: None,
            detailed_failures: Vec::new(),
            success: false,
            error: Some(e.to_string()),
        },
    }
}

pub fn run(num_rounds: usize, workers: Option<usize>, seed: Option<String>) -> Result<()> {
    // ensure setup is done
    crate::commands::setup::ensure_setup()?;

    // pre-flight checks
    check_prerequisites()?;

    let num_workers = workers.unwrap_or_else(get_default_workers);

    // generate or use provided seed
    let base_seed = seed.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        // generate a random 8-character alphanumeric seed
        std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
            .map(char::from)
            .take(8)
            .collect()
    });

    println!(
        "Running {} rounds with {} parallel workers...",
        num_rounds, num_workers
    );
    println!("Base seed: {}\n", base_seed);
    println!("{}", "=".repeat(80));

    // configure rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_workers)
        .build_global()
        .context("Failed to configure thread pool")?;

    let progress = ProgressBar::new(num_rounds as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let completed = AtomicUsize::new(0);

    // run tests in parallel
    let results: Vec<TestResult> = (0..num_rounds)
        .into_par_iter()
        .map(|test_index| {
            let round_num = test_index + 1;
            let test_seed = format!("{}{}", base_seed, test_index);
            let result = run_single_test(&test_seed);

            let count = completed.fetch_add(1, Ordering::SeqCst) + 1;
            progress.set_position(count as u64);

            if result.success {
                if let Some(passed) = result.passed {
                    progress.set_message(format!("Round {}: {} passed", round_num, passed));
                }
            } else {
                progress.set_message(format!(
                    "Round {}: FAILED ({})",
                    round_num,
                    result.error.as_deref().unwrap_or("unknown")
                ));
            }

            result
        })
        .collect();

    progress.finish_with_message("Complete");
    println!("{}", "=".repeat(80));

    // separate successful and failed results
    let successful_results: Vec<&TestResult> = results.iter().filter(|r| r.success).collect();
    let failed_results: Vec<&TestResult> = results.iter().filter(|r| !r.success).collect();

    if !successful_results.is_empty() {
        let passed_list: Vec<usize> = successful_results.iter().filter_map(|r| r.passed).collect();
        let total_passed: usize = passed_list.iter().sum();
        let total_tests = passed_list.len() * 100;
        let pass_rate = (total_passed as f64 / total_tests as f64) * 100.0;

        let min = *passed_list.iter().min().unwrap();
        let max = *passed_list.iter().max().unwrap();
        let avg = passed_list.iter().sum::<usize>() as f64 / passed_list.len() as f64;

        println!("\nAGGREGATE RESULTS");
        println!("{}", "─".repeat(80));
        println!(
            "Completed:       {}/{}",
            successful_results.len(),
            num_rounds
        );
        println!("Total Tests:     {}", total_tests);
        println!("Total Passed:    {}", total_passed);
        println!("Total Failed:    {}", total_tests - total_passed);
        println!("Pass Rate:       {:.2}%", pass_rate);
        println!("{}", "─".repeat(80));
        println!("Min Pass:        {}/100 ({}%)", min, min);
        println!("Max Pass:        {}/100 ({}%)", max, max);
        println!("Avg Pass:        {:.1}/100 ({:.1}%)", avg, avg);
        println!("{}", "─".repeat(80));

        // distribution
        let ranges = [
            ("90-100%", 90..=100),
            ("80-89%", 80..=89),
            ("70-79%", 70..=79),
            ("60-69%", 60..=69),
            ("50-59%", 50..=59),
            ("<50%", 0..=49),
        ];

        println!("\nDISTRIBUTION");
        println!("{}", "─".repeat(80));
        for (label, range) in ranges {
            let count = passed_list.iter().filter(|&&p| range.contains(&p)).count();
            if count > 0 {
                let bar = "█".repeat(count);
                println!("{:10} {} ({} rounds)", label, bar, count);
            }
        }
        println!("{}", "─".repeat(80));
    } else {
        println!("\nNo successful test runs");
    }

    // report failures
    if !failed_results.is_empty() {
        println!("\nFAILED ROUNDS: {}", failed_results.len());
        let mut error_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for r in &failed_results {
            let error = r.error.clone().unwrap_or_else(|| "unknown".to_string());
            *error_counts.entry(error).or_insert(0) += 1;
        }
        for (error_type, count) in error_counts {
            println!("  {}: {}", error_type, count);
        }
    }

    // save detailed failures
    save_detailed_failures(&results, &base_seed)?;

    // analyze failures
    analyze_failures(&results, &base_seed)?;

    // print seed at the end for easy reference
    println!("\nBase seed used: {}", base_seed);

    Ok(())
}

fn save_detailed_failures(results: &[TestResult], seed: &str) -> Result<()> {
    // collect all detailed failures
    let all_detailed: Vec<&DetailedFailure> = results
        .iter()
        .filter(|r| r.success)
        .flat_map(|r| &r.detailed_failures)
        .collect();

    if all_detailed.is_empty() {
        return Ok(());
    }

    // deduplicate by hash
    let mut deduped: HashMap<u64, DetailedFailure> = HashMap::new();
    for failure in &all_detailed {
        deduped
            .entry(failure.hash)
            .and_modify(|e| e.count += 1)
            .or_insert_with(|| (*failure).clone());
    }

    // convert to sorted vector
    let mut failures_vec: Vec<DetailedFailure> = deduped.into_values().collect();
    failures_vec.sort_by_key(|failure| Reverse(failure.count));

    // create output structure
    #[derive(Serialize)]
    struct DetailedOutput {
        run_metadata: RunMetadata,
        failures: Vec<DetailedFailure>,
    }

    #[derive(Serialize)]
    struct RunMetadata {
        total_failures: usize,
        unique_failures: usize,
        timestamp: String,
    }

    let output = DetailedOutput {
        run_metadata: RunMetadata {
            total_failures: all_detailed.len(),
            unique_failures: failures_vec.len(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
        failures: failures_vec,
    };

    // save to file
    let output_dir = Path::new("tests/fuzz/tools/output");
    std::fs::create_dir_all(output_dir)?;
    let output_file = output_dir.join(format!("detailed_failures_{}.json", seed));

    let json = serde_json::to_string_pretty(&output)?;
    std::fs::write(&output_file, json)?;

    println!(
        "\nDetailed failures saved to {} ({} unique failures from {} total)",
        output_file.display(),
        output.run_metadata.unique_failures,
        output.run_metadata.total_failures
    );

    Ok(())
}

fn analyze_failures(results: &[TestResult], seed: &str) -> Result<()> {
    // collect all failures from successful runs, extracting from detailed_failures
    let all_failures: Vec<SimpleFailure> = results
        .iter()
        .filter(|r| r.success)
        .flat_map(|r| &r.detailed_failures)
        .filter_map(|df| {
            // extract the mismatched classes at the position
            match (&df.prettier, &df.rustywind, df.position) {
                (Some(prettier), Some(rustywind), Some(pos)) => {
                    if pos < prettier.len() && pos < rustywind.len() {
                        Some(SimpleFailure {
                            prettier: prettier[pos].clone(),
                            rustywind: rustywind[pos].clone(),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect();

    if all_failures.is_empty() {
        println!("\nNo failures to analyze!");
        return Ok(());
    }

    let successful_runs = results.iter().filter(|r| r.success).count();
    let failed_runs = results.iter().filter(|r| !r.success).count();

    println!(
        "\nFAILURE ANALYSIS FROM {} SUCCESSFUL RUNS",
        successful_runs
    );
    println!("{}", "=".repeat(80));
    println!("Total Failures: {}\n", all_failures.len());

    // analyze category pairs
    let mut category_pairs: HashMap<String, usize> = HashMap::new();
    let mut specific_pairs: HashMap<String, usize> = HashMap::new();

    for failure in &all_failures {
        let p_cat = ClassCategory::categorize(&failure.prettier);
        let r_cat = ClassCategory::categorize(&failure.rustywind);

        let cat_pair = format!("{} before {}", p_cat, r_cat);
        *category_pairs.entry(cat_pair).or_insert(0) += 1;

        let class_pair = format!("{} vs {}", failure.prettier, failure.rustywind);
        *specific_pairs.entry(class_pair).or_insert(0) += 1;
    }

    // sort and print category pairs
    let mut category_pairs_vec: Vec<_> = category_pairs.into_iter().collect();
    category_pairs_vec.sort_by_key(|pair| Reverse(pair.1));

    println!("TOP CATEGORY MISMATCHES");
    println!("{}", "-".repeat(80));
    for (cat_pair, count) in category_pairs_vec.iter().take(20) {
        let pct = (*count as f64 / all_failures.len() as f64) * 100.0;
        println!("{:50} {:4} ({:5.1}%)", cat_pair, count, pct);
    }

    // sort and print specific pairs
    let mut specific_pairs_vec: Vec<_> = specific_pairs.into_iter().collect();
    specific_pairs_vec.sort_by_key(|pair| Reverse(pair.1));

    println!("\nTOP SPECIFIC CLASS PAIRS (appearing 3+ times)");
    println!("{}", "-".repeat(80));
    for (class_pair, count) in specific_pairs_vec.iter().take(30) {
        if *count >= 3 {
            println!("{:65} {:4}", class_pair, count);
        }
    }

    // save detailed results
    let output_dir = Path::new("tests/fuzz/tools/output");
    std::fs::create_dir_all(output_dir)?;
    let output_file = output_dir.join(format!("failure_analysis_{}.txt", seed));

    let mut content = String::new();
    content.push_str(&format!(
        "FAILURE ANALYSIS FROM {} SUCCESSFUL RUNS\n",
        successful_runs
    ));
    content.push_str(&format!("{}\n", "=".repeat(80)));
    content.push_str(&format!("Total Failures: {}\n", all_failures.len()));
    if failed_runs > 0 {
        content.push_str(&format!("Failed Runs: {}\n", failed_runs));
    }
    content.push('\n');

    content.push_str("CATEGORY PAIRS:\n");
    content.push_str(&format!("{}\n", "-".repeat(80)));
    for (cat_pair, count) in &category_pairs_vec {
        let pct = (*count as f64 / all_failures.len() as f64) * 100.0;
        content.push_str(&format!("{:50} {:4} ({:5.1}%)\n", cat_pair, count, pct));
    }

    content.push_str("\n\nSPECIFIC PAIRS:\n");
    content.push_str(&format!("{}\n", "-".repeat(80)));
    for (class_pair, count) in specific_pairs_vec {
        if count >= 2 {
            content.push_str(&format!("{:65} {:4}\n", class_pair, count));
        }
    }

    std::fs::write(&output_file, content)?;
    println!("\nDetailed results saved to {}", output_file.display());

    Ok(())
}
