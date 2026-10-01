use crate::paths::kome_home;

use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::Path;

const KOMEC_RELEASE_API: &str = "https://api.github.com/repos/mochiOS/komec/releases/latest";
const KOMEUP_RELEASE_API: &str = "https://api.github.com/repos/mochiOS/komeup/releases/latest";
const TOOLCHAINS_RELEASE_API: &str =
    "https://api.github.com/repos/mochiOS/toolchains/releases/latest";
const DEVKIT_RELEASE_API: &str = "https://api.github.com/repos/mochiOS/devkit/releases/latest";

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

struct DownloadedArchive {
    name: String,
    data: Vec<u8>,
}

pub fn install() -> Result<(), String> {
    let arch = std::env::consts::ARCH;
    let platform = platform_name()?;

    println!("Installing Kome...");

    let client = Client::builder()
        .user_agent("komeup")
        .build()
        .map_err(|error| error.to_string())?;

    let komec_release = fetch_release(&client, KOMEC_RELEASE_API)?;
    let komeup_release = fetch_release(&client, KOMEUP_RELEASE_API)?;
    let toolchains_release = fetch_release(&client, TOOLCHAINS_RELEASE_API)?;
    let devkit_release = fetch_release(&client, DEVKIT_RELEASE_API)?;

    let mut archives = vec![
        download_and_verify(
            &client,
            &komec_release,
            &format!("{arch}-kome-{platform}.tar.zst"),
        )?,
        download_and_verify(
            &client,
            &komec_release,
            &format!("{arch}-kome-std-{platform}.tar.zst"),
        )?,
        download_and_verify(
            &client,
            &komeup_release,
            &format!("{arch}-komeup-{platform}.tar.zst"),
        )?,
    ];

    let toolchain_name = find_toolchain_asset(&toolchains_release, arch, platform)?;
    archives.push(download_and_verify(
        &client,
        &toolchains_release,
        &toolchain_name,
    )?);

    let appcore_version = devkit_release.tag_name.trim_start_matches('v');
    let appcore = download_and_verify(
        &client,
        &devkit_release,
        &format!("{arch}-appcore-{appcore_version}.zst"),
    )?;

    let home = kome_home()?;

    fs::create_dir_all(&home)
        .map_err(|error| format!("failed to create {}: {error}", home.display()))?;

    for archive in archives.drain(..) {
        extract_archive(&archive.data, &home)
            .map_err(|error| format!("failed to install {}: {error}", archive.name))?;
    }

    let appcore_home = home.join("appcore");
    fs::create_dir_all(&appcore_home)
        .map_err(|error| format!("failed to create {}: {error}", appcore_home.display()))?;

    extract_archive(&appcore.data, &appcore_home)
        .map_err(|error| format!("failed to install {}: {error}", appcore.name))?;

    println!("Installed Kome to {}", home.display());

    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home = kome_home()?;

    if !home.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&home)
        .map_err(|error| format!("failed to remove {}: {error}", home.display()))?;

    println!("Removed {}", home.display());

    Ok(())
}

fn fetch_release(client: &Client, api: &str) -> Result<Release, String> {
    client
        .get(api)
        .send()
        .map_err(|error| format!("failed to query release: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to query release: {error}"))?
        .json()
        .map_err(|error| format!("invalid release metadata: {error}"))
}

fn download_and_verify(
    client: &Client,
    release: &Release,
    asset_name: &str,
) -> Result<DownloadedArchive, String> {
    let checksums = download_text_asset(client, release, "SHA256SUMS")?;
    let data = download_binary_asset(client, release, asset_name)?;

    verify_checksum(asset_name, &data, &checksums)?;

    Ok(DownloadedArchive {
        name: asset_name.to_string(),
        data,
    })
}

fn download_binary_asset(
    client: &Client,
    release: &Release,
    asset_name: &str,
) -> Result<Vec<u8>, String> {
    let asset = find_asset(release, asset_name)?;

    client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| format!("failed to read {asset_name}: {error}"))
}

fn download_text_asset(
    client: &Client,
    release: &Release,
    asset_name: &str,
) -> Result<String, String> {
    let asset = find_asset(release, asset_name)?;

    client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("failed to download {asset_name}: {error}"))?
        .text()
        .map_err(|error| format!("failed to read {asset_name}: {error}"))
}

fn find_asset<'a>(release: &'a Release, name: &str) -> Result<&'a Asset, String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| {
            format!(
                "release {} does not contain `{name}`",
                release.tag_name
            )
        })
}

fn find_toolchain_asset(
    release: &Release,
    arch: &str,
    platform: &str,
) -> Result<String, String> {
    let version = release.tag_name.trim_start_matches('v');
    let candidates = [
        format!("{arch}-mochios-toolchain-{version}.tar.zst"),
        format!("{arch}-mochios-toolchain-{version}.zst"),
        format!("{arch}-mochios-toolchain-{platform}.tar.zst"),
    ];

    candidates
        .into_iter()
        .find(|candidate| release.assets.iter().any(|asset| asset.name == *candidate))
        .ok_or_else(|| {
            format!(
                "release {} does not contain a toolchain archive for {arch}",
                release.tag_name
            )
        })
}

fn verify_checksum(name: &str, data: &[u8], checksums: &str) -> Result<(), String> {
    let expected = checksums
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');

            (file == name).then_some(hash)
        })
        .ok_or_else(|| format!("checksum for `{name}` was not found"))?;

    let actual = format!("{:x}", Sha256::digest(data));

    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!("checksum mismatch for `{name}`"));
    }

    Ok(())
}

fn platform_name() -> Result<&'static str, String> {
    match std::env::consts::OS {
        "linux" => Ok("linux"),
        "windows" => Ok("windows"),
        "macos" => Ok("macos"),
        os => Err(format!(
            "unsupported platform: {}-{os}",
            std::env::consts::ARCH
        )),
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
