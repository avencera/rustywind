use crate::BumpSpec;
use color_eyre::{Result, eyre::Context};
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const PREPARED_BINARIES_MANIFEST: &str = ".prepared-binaries.json";

/// Mapping from Rust target to npm package directory
fn target_to_package() -> BTreeMap<&'static str, &'static str> {
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

fn prepared_binaries_manifest_path(packages_dir: &Path) -> PathBuf {
    packages_dir.join(PREPARED_BINARIES_MANIFEST)
}

#[derive(Serialize, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<BTreeMap<String, String>>,
    #[serde(flatten)]
    rest: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct PreparedBinariesManifest {
    version: String,
    tag: String,
    binaries: Vec<PreparedBinary>,
}

#[derive(Serialize, Deserialize)]
struct PreparedBinary {
    target: String,
    package_dir: String,
    package_name: String,
    asset_name: String,
    binary_name: String,
    archive_sha256: String,
    binary_sha256: String,
}

impl PreparedBinariesManifest {
    fn binary_for_target(&self, target: &str) -> Option<&PreparedBinary> {
        self.binaries.iter().find(|binary| binary.target == target)
    }
}

/// Update version across all npm packages
pub fn update_version(version: &str) -> Result<()> {
    let package_version = npm_package_version(version)?;
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

        pkg.version = package_version.clone();

        // Update optionalDependencies versions
        if let Some(ref mut deps) = pkg.optional_dependencies {
            for (name, dep_version) in deps.iter_mut() {
                if name.starts_with("rustywind-") {
                    *dep_version = package_version.clone();
                }
            }
        }

        let output = serde_json::to_string_pretty(&pkg)?;
        fs::write(&pkg_json_path, output + "\n")?;

        println!(
            "Updated {} to {}",
            path.file_name().unwrap().to_string_lossy(),
            package_version
        );
    }

    println!("\nAll packages updated to version {}", package_version);
    Ok(())
}

fn normalize_release(version: &str) -> Result<(String, String)> {
    let package_version = npm_package_version(version)?;
    let tag = format!("v{}", package_version);
    Ok((package_version, tag))
}

fn binary_name_for_target(target: &str) -> &'static str {
    if target.contains("windows") {
        "rustywind.exe"
    } else {
        "rustywind"
    }
}

fn archive_extension_for_target(target: &str) -> &'static str {
    if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    }
}

fn asset_name_for_target(tag: &str, target: &str) -> String {
    format!(
        "rustywind-{}-{}.{}",
        tag,
        target,
        archive_extension_for_target(target)
    )
}

fn sha256_bytes(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex_digest(&digest)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex_digest(&hasher.finalize()))
}

fn hex_digest(digest: &[u8]) -> String {
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(output, "{byte:02x}").expect("writing to a string should not fail");
    }
    output
}

fn read_package_json(path: &Path) -> Result<PackageJson> {
    let content =
        fs::read_to_string(path).wrap_err_with(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&content).wrap_err_with(|| format!("Failed to parse {}", path.display()))
}

fn validate_package_versions(
    packages_dir: &Path,
    target_map: &BTreeMap<&str, &str>,
    version: &str,
) -> Result<()> {
    let main_pkg_path = packages_dir.join("rustywind").join("package.json");
    let main_pkg = read_package_json(&main_pkg_path)?;

    if main_pkg.version != version {
        color_eyre::eyre::bail!(
            "Main npm package version is {}, expected {}",
            main_pkg.version,
            version
        );
    }

    let optional_dependencies = main_pkg.optional_dependencies.as_ref().ok_or_else(|| {
        color_eyre::eyre::eyre!("Main npm package is missing optionalDependencies")
    })?;

    for pkg_dir in target_map.values() {
        let pkg_json_path = packages_dir.join(pkg_dir).join("package.json");
        let pkg = read_package_json(&pkg_json_path)?;

        if pkg.version != version {
            color_eyre::eyre::bail!(
                "{} version is {}, expected {}",
                pkg.name,
                pkg.version,
                version
            );
        }

        let Some(dep_version) = optional_dependencies.get(&pkg.name) else {
            color_eyre::eyre::bail!(
                "Main npm package is missing optional dependency {}",
                pkg.name
            );
        };

        if dep_version != version {
            color_eyre::eyre::bail!(
                "Main npm package depends on {}@{}, expected {}",
                pkg.name,
                dep_version,
                version
            );
        }
    }

    Ok(())
}

/// Download binaries from GitHub release and prepare packages
pub fn prepare_binaries(version: &str, token: Option<&str>) -> Result<()> {
    let packages_dir = npm_packages_dir();
    let target_map = target_to_package();
    let (package_version, tag) = normalize_release(version)?;

    validate_package_versions(&packages_dir, &target_map, &package_version)?;

    let mut manifest = PreparedBinariesManifest {
        version: package_version.clone(),
        tag: tag.clone(),
        binaries: Vec::new(),
    };

    for (target, pkg_dir) in &target_map {
        let pkg_path = packages_dir.join(pkg_dir);
        let pkg = read_package_json(&pkg_path.join("package.json"))?;

        println!("Downloading binary for {} -> {}", target, pkg_dir);

        let is_windows = target.contains("windows");
        let binary_name = binary_name_for_target(target);
        let asset_name = asset_name_for_target(&tag, target);
        let download_url = format!(
            "https://github.com/avencera/rustywind/releases/download/{}/{}",
            tag, asset_name
        );

        // Download the asset
        let mut request = ureq::get(&download_url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {t}"));
        }

        let response = request
            .call()
            .wrap_err_with(|| format!("Failed to download {} from {}", asset_name, download_url))?;

        let mut data = Vec::new();
        response
            .into_parts()
            .1
            .into_reader()
            .read_to_end(&mut data)?;
        let archive_sha256 = sha256_bytes(&data);

        // Extract the binary
        let binary_path = pkg_path.join(binary_name);
        if binary_path.exists() {
            fs::remove_file(&binary_path)?;
        }

        let mut extracted = false;

        if is_windows {
            // Extract from zip
            let cursor = std::io::Cursor::new(data);
            let mut archive = zip::ZipArchive::new(cursor)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                if file.name().ends_with(binary_name) {
                    let mut outfile = fs::File::create(&binary_path)?;
                    std::io::copy(&mut file, &mut outfile)?;
                    extracted = true;
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
                    extracted = true;
                    break;
                }
            }
        }

        if !extracted {
            color_eyre::eyre::bail!("Failed to extract {} from {}", binary_name, asset_name);
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

        let binary_sha256 = sha256_file(&binary_path)?;
        manifest.binaries.push(PreparedBinary {
            target: target.to_string(),
            package_dir: pkg_dir.to_string(),
            package_name: pkg.name,
            asset_name,
            binary_name: binary_name.to_string(),
            archive_sha256,
            binary_sha256,
        });
    }

    manifest
        .binaries
        .sort_by(|left, right| left.package_dir.cmp(&right.package_dir));

    let output = serde_json::to_string_pretty(&manifest)?;
    fs::write(
        prepared_binaries_manifest_path(&packages_dir),
        output + "\n",
    )?;

    verify_prepared_binaries_for(&packages_dir, &target_map, &package_version, true)?;

    println!("\nAll binaries prepared successfully");
    Ok(())
}

fn host_release_target() -> Option<&'static str> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        return Some("aarch64-apple-darwin");
    }

    if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        return Some("x86_64-apple-darwin");
    }

    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        return Some("x86_64-unknown-linux-musl");
    }

    if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        return Some("aarch64-unknown-linux-gnu");
    }

    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        return Some("x86_64-pc-windows-msvc");
    }

    if cfg!(target_os = "windows") && cfg!(target_arch = "x86") {
        return Some("i686-pc-windows-msvc");
    }

    None
}

fn verify_binary_version(binary_path: &Path, version: &str) -> Result<()> {
    let output = Command::new(binary_path).arg("-V").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if !output.status.success() {
        color_eyre::eyre::bail!(
            "{} -V failed with status {}",
            binary_path.display(),
            output.status
        );
    }

    if !combined.contains(version) {
        color_eyre::eyre::bail!(
            "{} reports {:?}, expected version {}",
            binary_path.display(),
            combined.trim(),
            version
        );
    }

    Ok(())
}

fn verify_prepared_binaries_for(
    packages_dir: &Path,
    target_map: &BTreeMap<&str, &str>,
    version: &str,
    verify_host_binary: bool,
) -> Result<()> {
    validate_package_versions(packages_dir, target_map, version)?;

    let manifest_path = prepared_binaries_manifest_path(packages_dir);
    let manifest_content = fs::read_to_string(&manifest_path).wrap_err_with(|| {
        format!(
            "Prepared binaries manifest is missing at {}; run `cargo xtask npm bump {}` or `cargo xtask npm prepare-binaries {}` first",
            manifest_path.display(),
            version,
            version
        )
    })?;
    let manifest: PreparedBinariesManifest = serde_json::from_str(&manifest_content)?;
    let expected_tag = format!("v{}", version);

    if manifest.version != version {
        color_eyre::eyre::bail!(
            "Prepared binaries are for {}, expected {}",
            manifest.version,
            version
        );
    }

    if manifest.tag != expected_tag {
        color_eyre::eyre::bail!(
            "Prepared binaries tag is {}, expected {}",
            manifest.tag,
            expected_tag
        );
    }

    for (target, pkg_dir) in target_map {
        let Some(prepared) = manifest.binary_for_target(target) else {
            color_eyre::eyre::bail!("Prepared binaries manifest is missing target {}", target);
        };

        let pkg = read_package_json(&packages_dir.join(pkg_dir).join("package.json"))?;
        let expected_asset_name = asset_name_for_target(&expected_tag, target);
        let expected_binary_name = binary_name_for_target(target);

        if prepared.package_dir != *pkg_dir {
            color_eyre::eyre::bail!(
                "Prepared target {} points at package dir {}, expected {}",
                target,
                prepared.package_dir,
                pkg_dir
            );
        }

        if prepared.package_name != pkg.name {
            color_eyre::eyre::bail!(
                "Prepared target {} package is {}, expected {}",
                target,
                prepared.package_name,
                pkg.name
            );
        }

        if prepared.asset_name != expected_asset_name {
            color_eyre::eyre::bail!(
                "Prepared target {} asset is {}, expected {}",
                target,
                prepared.asset_name,
                expected_asset_name
            );
        }

        if prepared.binary_name != expected_binary_name {
            color_eyre::eyre::bail!(
                "Prepared target {} binary is {}, expected {}",
                target,
                prepared.binary_name,
                expected_binary_name
            );
        }

        let binary_path = packages_dir.join(pkg_dir).join(expected_binary_name);
        let actual_sha256 = sha256_file(&binary_path).wrap_err_with(|| {
            format!(
                "Failed to hash prepared binary {}; run `cargo xtask npm prepare-binaries {}` again",
                binary_path.display(),
                version
            )
        })?;

        if actual_sha256 != prepared.binary_sha256 {
            color_eyre::eyre::bail!(
                "Prepared binary hash mismatch for {}: got {}, expected {}",
                binary_path.display(),
                actual_sha256,
                prepared.binary_sha256
            );
        }

        if verify_host_binary && host_release_target() == Some(*target) {
            verify_binary_version(&binary_path, version)?;
        }
    }

    Ok(())
}

fn verify_prepared_binaries(version: &str) -> Result<()> {
    let packages_dir = npm_packages_dir();
    let target_map = target_to_package();
    verify_prepared_binaries_for(&packages_dir, &target_map, version, true)
}

/// Determine the npm dist-tag for a given version string.
///
/// Prerelease versions (e.g. "0.25.0-alpha.1") return a tag based on the
/// prerelease identifier ("alpha", "beta", "rc"). Stable versions return None
/// (npm defaults to "latest").
fn dist_tag_for_version(version: &str) -> Option<String> {
    let version = Version::parse(version.strip_prefix('v').unwrap_or(version)).ok()?;
    version
        .pre
        .as_str()
        .split('.')
        .next()
        .filter(|id| !id.is_empty())
        .map(|id| id.to_string())
}

/// Build the base `npm publish` command with optional dist-tag.
fn npm_publish_cmd(current_dir: &Path, tag: Option<&str>, dry_run: bool) -> Command {
    let mut cmd = Command::new("npm");
    cmd.arg("publish")
        .arg("--access")
        .arg("public")
        .current_dir(current_dir);

    if let Some(tag) = tag {
        cmd.arg("--tag").arg(tag);
    }

    if dry_run {
        cmd.arg("--dry-run");
    }

    cmd
}

/// Publish all npm packages
pub fn publish(dry_run: bool) -> Result<()> {
    let packages_dir = npm_packages_dir();
    let target_map = target_to_package();
    let current_version = get_current_version()?;
    let tag = dist_tag_for_version(&current_version);

    verify_prepared_binaries(&current_version)?;

    if let Some(ref tag) = tag {
        println!(
            "Detected prerelease {}, using --tag {}",
            current_version, tag
        );
    }

    // Platform packages first
    for pkg_dir in target_map.values() {
        let pkg_path = packages_dir.join(pkg_dir);
        println!("Publishing rustywind-{}...", pkg_dir);

        let status = npm_publish_cmd(&pkg_path, tag.as_deref(), dry_run).status()?;
        if !status.success() {
            color_eyre::eyre::bail!("Failed to publish rustywind-{}", pkg_dir);
        }
    }

    // Main package last
    println!("Publishing rustywind (main package)...");
    let main_pkg_path = packages_dir.join("rustywind");

    // Install dependencies first
    let status = Command::new("npm")
        .args([
            "install",
            "--ignore-scripts",
            "--package-lock=false",
            "--no-audit",
        ])
        .current_dir(&main_pkg_path)
        .status()?;

    if !status.success() {
        color_eyre::eyre::bail!("Failed to install dependencies for main package");
    }

    let status = npm_publish_cmd(&main_pkg_path, tag.as_deref(), dry_run).status()?;
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
    let current = current.strip_prefix('v').unwrap_or(current);
    let mut version = Version::parse(current)?;

    match position {
        0 => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        1 => {
            version.minor += 1;
            version.patch = 0;
        }
        2 => version.patch += 1,
        _ => color_eyre::eyre::bail!("Invalid version bump position: {}", position),
    }

    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;

    Ok(version.to_string())
}

fn npm_package_version(version: &str) -> Result<String> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = Version::parse(version)?;
    Ok(version.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_ID: AtomicUsize = AtomicUsize::new(0);

    fn temp_packages_dir() -> PathBuf {
        let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "rustywind-xtask-npm-test-{}-{id}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_package_json(path: &Path, name: &str, version: &str, deps: Option<&[(&str, &str)]>) {
        fs::create_dir_all(path).unwrap();

        let optional_dependencies = deps.map(|deps| {
            deps.iter()
                .map(|(name, version)| (name.to_string(), version.to_string()))
                .collect::<BTreeMap<_, _>>()
        });

        let pkg = PackageJson {
            name: name.to_string(),
            version: version.to_string(),
            optional_dependencies,
            rest: BTreeMap::new(),
        };

        fs::write(
            path.join("package.json"),
            serde_json::to_string_pretty(&pkg).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn increments_prerelease_versions_from_core_numbers() {
        assert_eq!(increment_version("0.25.0-alpha.1", 0).unwrap(), "1.0.0");
        assert_eq!(increment_version("0.25.0-alpha.1", 1).unwrap(), "0.26.0");
        assert_eq!(increment_version("0.25.0-alpha.1", 2).unwrap(), "0.25.1");
    }

    #[test]
    fn normalizes_npm_package_versions() {
        assert_eq!(
            npm_package_version("v0.25.0-alpha.1").unwrap(),
            "0.25.0-alpha.1"
        );
        assert_eq!(
            npm_package_version("0.25.0-alpha.1").unwrap(),
            "0.25.0-alpha.1"
        );
        assert!(npm_package_version("v0.25").is_err());
    }

    #[test]
    fn normalizes_release_versions_to_package_version_and_tag() {
        assert_eq!(
            normalize_release("v0.25.1").unwrap(),
            ("0.25.1".to_string(), "v0.25.1".to_string())
        );
        assert_eq!(
            normalize_release("0.25.1").unwrap(),
            ("0.25.1".to_string(), "v0.25.1".to_string())
        );
    }

    #[test]
    fn verifies_prepared_binary_manifest() {
        let packages_dir = temp_packages_dir();
        let mut target_map = BTreeMap::new();
        target_map.insert("fake-target", "fake-package");

        write_package_json(
            &packages_dir.join("rustywind"),
            "rustywind",
            "0.25.1",
            Some(&[("rustywind-fake-package", "0.25.1")]),
        );
        write_package_json(
            &packages_dir.join("fake-package"),
            "rustywind-fake-package",
            "0.25.1",
            None,
        );

        let binary_path = packages_dir.join("fake-package").join("rustywind");
        fs::write(&binary_path, "fake binary").unwrap();
        let binary_sha256 = sha256_file(&binary_path).unwrap();

        let manifest = PreparedBinariesManifest {
            version: "0.25.1".to_string(),
            tag: "v0.25.1".to_string(),
            binaries: vec![PreparedBinary {
                target: "fake-target".to_string(),
                package_dir: "fake-package".to_string(),
                package_name: "rustywind-fake-package".to_string(),
                asset_name: "rustywind-v0.25.1-fake-target.tar.gz".to_string(),
                binary_name: "rustywind".to_string(),
                archive_sha256: "archive".to_string(),
                binary_sha256,
            }],
        };

        fs::write(
            prepared_binaries_manifest_path(&packages_dir),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        verify_prepared_binaries_for(&packages_dir, &target_map, "0.25.1", false).unwrap();

        let _ = fs::remove_dir_all(packages_dir);
    }

    #[test]
    fn rejects_stale_prepared_binary_manifest() {
        let packages_dir = temp_packages_dir();
        let mut target_map = BTreeMap::new();
        target_map.insert("fake-target", "fake-package");

        write_package_json(
            &packages_dir.join("rustywind"),
            "rustywind",
            "0.25.2",
            Some(&[("rustywind-fake-package", "0.25.2")]),
        );
        write_package_json(
            &packages_dir.join("fake-package"),
            "rustywind-fake-package",
            "0.25.2",
            None,
        );

        let manifest = PreparedBinariesManifest {
            version: "0.25.1".to_string(),
            tag: "v0.25.1".to_string(),
            binaries: Vec::new(),
        };

        fs::write(
            prepared_binaries_manifest_path(&packages_dir),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let err = verify_prepared_binaries_for(&packages_dir, &target_map, "0.25.2", false)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Prepared binaries are for 0.25.1"));

        let _ = fs::remove_dir_all(packages_dir);
    }

    #[test]
    fn stable_versions_use_default_npm_dist_tag() {
        assert_eq!(dist_tag_for_version("0.25.0"), None);
        assert_eq!(dist_tag_for_version("v0.25.0"), None);
    }

    #[test]
    fn prerelease_versions_use_prerelease_identifier_as_dist_tag() {
        assert_eq!(
            dist_tag_for_version("0.25.0-alpha.1").as_deref(),
            Some("alpha")
        );
        assert_eq!(dist_tag_for_version("v0.25.0-rc.1").as_deref(), Some("rc"));
    }
}
