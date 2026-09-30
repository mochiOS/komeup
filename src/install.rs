use crate::paths::kome_home;

use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::io::Cursor;
use std::path::Path;

const RELEASE_API: &str = "https://api.github.com/repos/mochiOS/toolchains/releases/latest";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn install() -> Result<(), String> {
    let platform = platform_name()?;
    let asset_name = format!("kome-{platform}.tar.zst");

    println!("Installing Kome...");

    let client = Client::builder()
        .user_agent("komeup")
        .build()
        .map_err(|error| error.to_string())?;

    let release: Release = client
        .get(RELEASE_API)
        .send()
        .map_err(|error| format!("failed to query releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to query releases: {error}"))?
        .json()
        .map_err(|error| format!("invalid release metadata: {error}"))?;

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| {
            format!(
                "release {} does not contain `{asset_name}`",
                release.tag_name
            )
        })?;

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .bytes()
        .map_err(|error| format!("failed to read {asset_name}: {error}"))?;

    let home = kome_home()?;

    fs::create_dir_all(&home)
        .map_err(|error| format!("failed to create {}: {error}", home.display()))?;

    extract_archive(&bytes, &home)?;

    println!("Installed Kome {} to {}", release.tag_name, home.display());

    Ok(())
}

fn platform_name() -> Result<&'static str, String> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok("x86_64-linux"),
        ("aarch64", "linux") => Ok("aarch64-linux"),

        (arch, os) => Err(format!("unsupported platform: {arch}-{os}")),
    }
}

fn extract_archive(data: &[u8], destination: &Path) -> Result<(), String> {
    let decoder = zstd::stream::read::Decoder::new(Cursor::new(data))
        .map_err(|error| format!("failed to decompress release: {error}"))?;

    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(destination)
        .map_err(|error| format!("failed to extract release: {error}"))
}
