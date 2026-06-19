use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn main() {
    if let Err(error) = generate_third_party_licenses() {
        panic!("failed to generate third-party license notices: {error}");
    }
}

fn generate_third_party_licenses() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let metadata = cargo_metadata(&manifest_dir)?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("cargo metadata did not contain packages")?;
    let root_package_id = metadata["resolve"]["root"]
        .as_str()
        .ok_or("cargo metadata did not contain a root package")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("cargo metadata did not contain resolve nodes")?;

    let packages_by_id = packages
        .iter()
        .filter_map(|package| package["id"].as_str().map(|id| (id, package)))
        .collect::<BTreeMap<_, _>>();

    let mut third_party_ids = BTreeSet::new();
    for node in nodes {
        let Some(id) = node["id"].as_str() else {
            continue;
        };

        if id == root_package_id {
            continue;
        }

        let Some(package) = packages_by_id.get(id) else {
            continue;
        };

        if package["source"]
            .as_str()
            .is_some_and(|source| source.starts_with("registry+"))
        {
            third_party_ids.insert(id);
        }
    }

    let mut lines = vec![
        "# Third-Party Licenses".to_string(),
        String::new(),
        "This file contains third-party dependency license notices for pomoshumai.".to_string(),
        "It is generated from Cargo metadata and crate license files by examples/generate-third-party-licenses.rs."
            .to_string(),
        String::new(),
        "Generated from Cargo.lock.".to_string(),
        String::new(),
        "## Dependency Summary".to_string(),
        String::new(),
        "| Package | Version | License | Repository |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];

    for id in &third_party_ids {
        let package = packages_by_id[id];
        let name = markdown_cell(package["name"].as_str().unwrap_or_default());
        let version = markdown_cell(package["version"].as_str().unwrap_or_default());
        let license = markdown_cell(package_license(package));
        let repository = markdown_cell(package["repository"].as_str().unwrap_or_default());
        lines.push(format!("| {name} | {version} | {license} | {repository} |"));
    }

    lines.push(String::new());
    lines.push("## License Texts And Notices".to_string());
    lines.push(String::new());

    for id in &third_party_ids {
        let package = packages_by_id[id];
        let name = package["name"].as_str().unwrap_or_default();
        let version = package["version"].as_str().unwrap_or_default();
        let license = package_license(package);

        lines.push(format!("### {name} {version}"));
        lines.push(String::new());
        lines.push(format!("- License: {license}"));
        if let Some(repository) = package["repository"].as_str() {
            lines.push(format!("- Repository: {repository}"));
        }
        lines.push(String::new());

        let notice_files = notice_files(package);
        if notice_files.is_empty() {
            lines.push(
                "No license file was found in the local crate source. See the license expression above."
                    .to_string(),
            );
            lines.push(String::new());
            continue;
        }

        for file in notice_files {
            let filename = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("license file");
            lines.push(format!("#### {filename}"));
            lines.push(String::new());
            lines.push(format!("Source file: `{name}-{version}/{filename}`"));
            lines.push(String::new());
            lines.push("```text".to_string());
            lines.push(fs::read_to_string(&file)?.trim_end().replace("\r\n", "\n"));
            lines.push("```".to_string());
            lines.push(String::new());
        }
    }

    fs::write(
        manifest_dir.join("THIRD_PARTY_LICENSES.md"),
        lines.join("\n"),
    )?;

    Ok(())
}

fn cargo_metadata(manifest_dir: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(manifest_dir)
        .output()?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

fn package_license(package: &Value) -> &str {
    package["license"]
        .as_str()
        .or_else(|| package["license_file"].as_str().map(|_| "See license file"))
        .unwrap_or("UNKNOWN")
}

fn notice_files(package: &Value) -> Vec<PathBuf> {
    let Some(manifest_path) = package["manifest_path"].as_str() else {
        return Vec::new();
    };
    let Some(package_dir) = Path::new(manifest_path).parent() else {
        return Vec::new();
    };

    let mut files = BTreeSet::new();
    if let Some(license_file) = package["license_file"].as_str() {
        let license_path = package_dir.join(license_file);
        if license_path.is_file() {
            files.insert(license_path);
        }
    }

    if let Ok(entries) = fs::read_dir(package_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if is_notice_filename(filename) {
                files.insert(path);
            }
        }
    }

    files.into_iter().collect()
}

fn is_notice_filename(filename: &str) -> bool {
    let uppercase = filename.to_ascii_uppercase();
    ["LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE"]
        .iter()
        .any(|prefix| {
            uppercase == *prefix
                || uppercase.strip_prefix(prefix).is_some_and(|suffix| {
                    matches!(suffix.as_bytes().first(), Some(b'.' | b'-' | b'_'))
                })
        })
}

fn markdown_cell(value: &str) -> String {
    value.replace('\\', "\\\\").replace('|', "\\|")
}
