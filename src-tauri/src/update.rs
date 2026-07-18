//! Optional online check against GitHub releases (does not run at build time).
//! Offline failure is silent: app works fully without network.

use crate::models::UpdateCheck;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const REPO: &str = "codemonkey2k5/BudgetMaster9000";
const USER_AGENT: &str = "BudgetMaster9000-update-check";

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Compare dotted semver-ish strings (1.0.0 vs 1.1.0). Non-numeric tails ignored.
pub fn version_is_newer(latest: &str, current: &str) -> bool {
    let lp = parse_ver(latest);
    let cp = parse_ver(current);
    lp > cp
}

fn parse_ver(v: &str) -> (u64, u64, u64) {
    let v = v.trim().trim_start_matches('v');
    let mut parts = v.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty());
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

fn http_get_json(url: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(12))
        .user_agent(USER_AGENT)
        .build();
    agent
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent(USER_AGENT)
        .build();
    let mut reader = agent
        .get(url)
        .call()
        .map_err(|e| e.to_string())?
        .into_reader();
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

pub fn check_for_update() -> UpdateCheck {
    let current = current_version();
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    match http_get_json(&url) {
        Ok(body) => match serde_json::from_str::<GhRelease>(&body) {
            Ok(rel) => {
                let latest = rel.tag_name.trim_start_matches('v').to_string();
                let zip = rel
                    .assets
                    .iter()
                    .find(|a| a.name.to_lowercase().ends_with(".zip"))
                    .map(|a| a.name.clone());
                UpdateCheck {
                    update_available: version_is_newer(&latest, &current),
                    current_version: current,
                    latest_version: latest,
                    release_url: rel.html_url,
                    zip_asset_name: zip,
                    checked: true,
                    error: None,
                }
            }
            Err(e) => UpdateCheck {
                current_version: current.clone(),
                latest_version: current,
                update_available: false,
                release_url: format!("https://github.com/{REPO}/releases"),
                zip_asset_name: None,
                checked: true,
                error: Some(format!("Could not parse GitHub response: {e}")),
            },
        },
        Err(e) => UpdateCheck {
            current_version: current.clone(),
            latest_version: current,
            update_available: false,
            release_url: format!("https://github.com/{REPO}/releases"),
            zip_asset_name: None,
            checked: true,
            error: Some(format!("Offline or unreachable: {e}")),
        },
    }
}

fn upgrade_instructions(latest: &str, current: &str) -> String {
    format!(
        r#"Budget Master 9000 — Upgrade instructions
========================================

You are updating from {current} to {latest}.

Pick ONE of the two distribution choices (same as a new install):

1) INSTALLER (recommended if you used the setup before)
   - Run BudgetMaster9000-Setup-*-x64.exe (or the .msi if present).
   - Follow the prompts. Your budget data stays in AppData automatically.
   - No database files to copy. No settings to edit.

2) PORTABLE
   - Close Budget Master 9000 if it is running.
   - Copy BudgetMaster9000.exe over the old one in the SAME folder.
   - Leave bm9000.db (your data) where it is — do not move or rename it.
   - Start the new BudgetMaster9000.exe.

After upgrade
-------------
- Open the app. Schema upgrades run automatically on first launch.
- Your Plan, months, and history should be unchanged.

If anything looks wrong, import a JSON backup from Settings (export first next time).

Download source: https://github.com/{REPO}/releases
"#
    )
}

fn downloads_dir() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Download the release zip if present; otherwise build a zip with installer/portable
/// assets from the release plus UPGRADE.txt.
pub fn download_update_package() -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = http_get_json(&url)?;
    let rel: GhRelease = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let latest = rel.tag_name.trim_start_matches('v').to_string();
    let current = current_version();
    if !version_is_newer(&latest, &current) {
        return Err("You already have the latest version.".into());
    }

    let dest_dir = downloads_dir();
    let _ = fs::create_dir_all(&dest_dir);
    let zip_name = format!("BudgetMaster9000-{latest}-Windows.zip");
    let dest = dest_dir.join(&zip_name);

    // Prefer an official zip asset that already includes instructions.
    if let Some(asset) = rel
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".zip"))
    {
        let bytes = http_get_bytes(&asset.browser_download_url)?;
        fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        return Ok(dest.display().to_string());
    }

    // No zip on the release: assemble one from available EXEs + UPGRADE.txt
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    files.push((
        "UPGRADE.txt".into(),
        upgrade_instructions(&latest, &current).into_bytes(),
    ));

    for asset in &rel.assets {
        let lower = asset.name.to_lowercase();
        let keep = lower.ends_with(".exe")
            || lower.ends_with(".msi")
            || lower.contains("setup")
            || lower == "budgetmaster9000.exe"
            || lower.ends_with("sums.txt");
        if !keep {
            continue;
        }
        // Skip huge accidental assets
        if asset.size > 80_000_000 {
            continue;
        }
        match http_get_bytes(&asset.browser_download_url) {
            Ok(bytes) => files.push((asset.name.clone(), bytes)),
            Err(e) => {
                files.push((
                    format!("FAILED_{}.txt", asset.name),
                    format!("Could not download {}: {e}", asset.name).into_bytes(),
                ));
            }
        }
    }

    if files.len() <= 1 {
        return Err(format!(
            "No installer/portable files found on the release. Open: {}",
            rel.html_url
        ));
    }

    write_zip(&dest, &files)?;
    Ok(dest.display().to_string())
}

fn write_zip(path: &Path, files: &[(String, Vec<u8>)]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in files {
        zip.start_file(name.as_str(), opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_compare() {
        assert!(version_is_newer("1.1.0", "1.0.0"));
        assert!(version_is_newer("v1.2.0", "1.1.9"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.1.0"));
    }

    #[test]
    fn upgrade_text_mentions_both_choices() {
        let t = upgrade_instructions("1.2.0", "1.1.0");
        assert!(t.contains("INSTALLER"));
        assert!(t.contains("PORTABLE"));
        assert!(t.contains("bm9000.db"));
    }
}
