use color_eyre::{Result, eyre::Context};
use std::path::Path;
use std::process::Command;

/// Run setup: build RustyWind release binary and install npm dependencies
pub fn run() -> Result<()> {
    println!("Setting up fuzz test environment...\n");

    // build RustyWind release binary
    println!("Building RustyWind release binary...");
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .context("Failed to run cargo build")?;

    if !build_status.success() {
        eprintln!("Failed to build RustyWind");
        std::process::exit(1);
    }
    println!("RustyWind binary built successfully\n");

    // install npm dependencies
    let fuzz_dir = Path::new("tests/fuzz");
    if !fuzz_dir.exists() {
        eprintln!("Fuzz test directory not found: {}", fuzz_dir.display());
        std::process::exit(1);
    }

    println!("Installing npm dependencies in {}...", fuzz_dir.display());
    let npm_status = Command::new("npm")
        .arg("install")
        .current_dir(fuzz_dir)
        .status()
        .context("Failed to run npm install")?;

    if !npm_status.success() {
        eprintln!("Failed to install npm dependencies");
        std::process::exit(1);
    }
    println!("npm dependencies installed successfully\n");

    println!("Setup complete!");
    Ok(())
}

/// Check if setup is needed and run it automatically
pub fn ensure_setup() -> Result<()> {
    let mut needs_setup = false;

    // check if RustyWind binary exists
    let binary_path = Path::new("target/release/rustywind");
    if !binary_path.exists() {
        println!("RustyWind release binary not found");
        needs_setup = true;
    }

    // check if node_modules exists
    let node_modules = Path::new("tests/fuzz/node_modules");
    if !node_modules.exists() {
        println!("npm dependencies not installed");
        needs_setup = true;
    }

    if needs_setup {
        println!("\nRunning automatic setup...\n");
        run()?;
        println!();
    }

    Ok(())
}
