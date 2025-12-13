use crate::BumpSpec;
use color_eyre::{Result, eyre::Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Mapping from Rust target to npm package directory
fn target_to_package() -> HashMap<&'static str, &'static str> {
    [
        ("aarch64-apple-darwin", "darwin-arm64"),
        ("x86_64-apple-darwin", "darwin-x64"),
        ("aarch64-unknown-linux-gnu", "linux-arm64-gnu"),
        ("aarch64-unknown-linux-musl", "linux-arm64-musl"),
        ("arm-unknown-linux-gnueabihf", "linux-arm-gnueabihf"),
        ("x86_64-unknown-linux-musl", "linux-x64-musl"),
        ("i686-pc-windows-msvc", "win32-ia32-msvc"),
        ("x86_64-pc-windows-msvc", "win32-x64-msvc"),
    ]
    .into_iter()
    .collect()
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn npm_packages_dir() -> PathBuf {
    project_root().join("npm").join("packages")
}

#[derive(Serialize, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<HashMap<String, String>>,
    #[serde(flatten)]
    rest: HashMap<String, serde_json::Value>,
}

/// Update version across all npm packages
pub fn update_version(version: &str) -> Result<()> {
    let packages_dir = npm_packages_dir();

    // Get all package directories
    let entries = fs::read_dir(&packages_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let pkg_json_path = path.join("package.json");
        if !pkg_json_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&pkg_json_path)?;
        let mut pkg: PackageJson = serde_json::from_str(&content)?;

        pkg.version = version.to_string();

        // Update optionalDependencies versions
        if let Some(ref mut deps) = pkg.optional_dependencies {
            for (name, dep_version) in deps.iter_mut() {
                if name.starts_with("rustywind-") {
                    *dep_version = version.to_string();
                }
            }
        }

        let output = serde_json::to_string_pretty(&pkg)?;
        fs::write(&pkg_json_path, output + "\n")?;

        println!(
            "Updated {} to {}",
            path.file_name().unwrap().to_string_lossy(),
            version
        );
    }

    println!("\nAll packages updated to version {}", version);
    Ok(())
}

/// Download binaries from GitHub release and prepare packages
pub fn prepare_binaries(version: &str, token: Option<&str>) -> Result<()> {
    let packages_dir = npm_packages_dir();
    let target_map = target_to_package();

    // Ensure version starts with 'v'
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };

    for (target, pkg_dir) in &target_map {
        let pkg_path = packages_dir.join(pkg_dir);

        println!("Downloading binary for {} -> {}", target, pkg_dir);

        let is_windows = target.contains("windows");
        let ext = if is_windows { "zip" } else { "tar.gz" };
        let binary_name = if is_windows {
            "rustywind.exe"
        } else {
            "rustywind"
        };

        let asset_name = format!("rustywind-{}-{}.{}", tag, target, ext);
        let download_url = format!(
            "https://github.com/avencera/rustywind/releases/download/{}/{}",
            tag, asset_name
        );

        // Download the asset
        let mut request = ureq::get(&download_url);
        if let Some(t) = token {
            request = request.set("Authorization", &format!("token {}", t));
        }

        let response = request
            .call()
            .wrap_err_with(|| format!("Failed to download {} from {}", asset_name, download_url))?;

        let mut data = Vec::new();
        response.into_reader().read_to_end(&mut data)?;

        // Extract the binary
        let binary_path = pkg_path.join(binary_name);

        if is_windows {
            // Extract from zip
            let cursor = std::io::Cursor::new(data);
            let mut archive = zip::ZipArchive::new(cursor)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                if file.name().ends_with(binary_name) {
                    let mut outfile = fs::File::create(&binary_path)?;
                    std::io::copy(&mut file, &mut outfile)?;
                    break;
                }
            }
        } else {
            // Extract from tar.gz
            let cursor = std::io::Cursor::new(data);
            let gz = flate2::read::GzDecoder::new(cursor);
            let mut archive = tar::Archive::new(gz);

            for entry in archive.entries()? {
                let mut entry = entry?;
                let path = entry.path()?;
                if path.file_name().map(|n| n == binary_name).unwrap_or(false) {
                    let mut outfile = fs::File::create(&binary_path)?;
                    std::io::copy(&mut entry, &mut outfile)?;
                    break;
                }
            }
        }

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if !is_windows {
                fs::set_permissions(&binary_path, fs::Permissions::from_mode(0o755))?;
            }
        }

        // Verify binary exists
        if binary_path.exists() {
            println!("  ✓ {}", binary_path.display());
        } else {
            color_eyre::eyre::bail!("Failed to extract binary to {}", binary_path.display());
        }
    }

    println!("\nAll binaries prepared successfully");
    Ok(())
}

/// Publish all npm packages
pub fn publish(dry_run: bool) -> Result<()> {
    let packages_dir = npm_packages_dir();
    let target_map = target_to_package();

    // Platform packages first
    for pkg_dir in target_map.values() {
        let pkg_path = packages_dir.join(pkg_dir);
        println!("Publishing rustywind-{}...", pkg_dir);

        let mut cmd = Command::new("npm");
        cmd.arg("publish")
            .arg("--access")
            .arg("public")
            .current_dir(&pkg_path);

        if dry_run {
            cmd.arg("--dry-run");
        }

        let status = cmd.status()?;
        if !status.success() && !dry_run {
            eprintln!("Warning: Failed to publish {}, may already exist", pkg_dir);
        }
    }

    // Main package last
    println!("Publishing rustywind (main package)...");
    let main_pkg_path = packages_dir.join("rustywind");

    // Install dependencies first
    let status = Command::new("npm")
        .args(["install", "--ignore-scripts"])
        .current_dir(&main_pkg_path)
        .status()?;

    if !status.success() {
        color_eyre::eyre::bail!("Failed to install dependencies for main package");
    }

    let mut cmd = Command::new("npm");
    cmd.arg("publish")
        .arg("--access")
        .arg("public")
        .current_dir(&main_pkg_path);

    if dry_run {
        cmd.arg("--dry-run");
    }

    let status = cmd.status()?;
    if !status.success() {
        color_eyre::eyre::bail!("Failed to publish main package");
    }

    println!("\nAll packages published successfully");
    Ok(())
}

/// Get the current version from the main rustywind package.json
fn get_current_version() -> Result<String> {
    let pkg_json_path = npm_packages_dir().join("rustywind").join("package.json");
    let content =
        fs::read_to_string(&pkg_json_path).wrap_err("Failed to read main package.json")?;
    let pkg: PackageJson = serde_json::from_str(&content)?;
    Ok(pkg.version)
}

/// Bump version and run full release
pub fn bump(spec: BumpSpec, token: Option<&str>, dry_run: bool) -> Result<()> {
    let current_version = get_current_version()?;

    let (new_version, tag) = match spec {
        BumpSpec::Major => {
            let new = increment_version(&current_version, 0)?;
            println!("=== major bump: {} -> {} ===\n", current_version, new);
            (new.clone(), format!("v{}", new))
        }
        BumpSpec::Minor => {
            let new = increment_version(&current_version, 1)?;
            println!("=== minor bump: {} -> {} ===\n", current_version, new);
            (new.clone(), format!("v{}", new))
        }
        BumpSpec::Patch => {
            let new = increment_version(&current_version, 2)?;
            println!("=== patch bump: {} -> {} ===\n", current_version, new);
            (new.clone(), format!("v{}", new))
        }
        BumpSpec::Version(ver) => {
            let version_num = ver.strip_prefix('v').unwrap_or(&ver).to_string();
            let tag = if ver.starts_with('v') {
                ver
            } else {
                format!("v{}", ver)
            };
            println!("=== releasing version {} ===\n", version_num);
            (version_num, tag)
        }
    };

    println!("=== Updating versions to {} ===\n", new_version);
    update_version(&new_version)?;

    println!("\n=== Downloading binaries for {} ===\n", tag);
    prepare_binaries(&tag, token)?;

    println!("\n=== Publishing packages ===\n");
    publish(dry_run)?;

    Ok(())
}

/// Increment a semver version at the specified position (0=major, 1=minor, 2=patch)
fn increment_version(current: &str, position: usize) -> Result<String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        color_eyre::eyre::bail!("Invalid version format: {}", current);
    }

    let mut nums: Vec<u32> = parts
        .iter()
        .map(|p| p.parse().wrap_err("Invalid version number"))
        .collect::<Result<_>>()?;

    nums[position] += 1;
    for num in nums.iter_mut().skip(position + 1) {
        *num = 0;
    }

    Ok(format!("{}.{}.{}", nums[0], nums[1], nums[2]))
}
