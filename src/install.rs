use crate::paths::kome_home;

use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub const DEFAULT_SDK_VERSION: &str = "27.0-dp.1";

const GITHUB_API: &str = "https://api.github.com/repos/mochiOS";

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
    destination: Destination,
}

#[derive(Clone, Copy)]
enum Destination {
    Home,
    Bin,
    StandardLibrary,
    ViewKit,
    AppCore,
}

impl Destination {
    fn path(self, home: &Path) -> PathBuf {
        match self {
            Self::Home => home.to_path_buf(),
            Self::Bin => home.join("bin"),
            Self::StandardLibrary => home.join("stdlib"),
            Self::ViewKit => home.join("viewkit"),
            Self::AppCore => home.join("appcore"),
        }
    }
}

pub fn install(version: &str) -> Result<(), String> {
    validate_version(version)?;
    let arch = std::env::consts::ARCH;

    println!("Kome SDK {version}をインストールしています...");

    let client = Client::builder()
        .user_agent("komeup")
        .build()
        .map_err(|error| error.to_string())?;

    let komec_release = fetch_release(&client, "komec", version)?;
    let komeup_release = fetch_release(&client, "komeup", version)?;
    let toolchains_release = fetch_release(&client, "toolchains", version)?;
    let devkit_release = fetch_release(&client, "devkit", version)?;
    let viewkit_release = fetch_release(&client, "ViewKit", version)?;

    let toolchain_name = find_toolchain_asset(&toolchains_release, arch, version)?;
    let requested = [
        (
            &komec_release,
            format!("{arch}-kome-{version}.tar.zst"),
            Destination::Bin,
        ),
        (
            &komec_release,
            format!("{arch}-komec-{version}.tar.zst"),
            Destination::Bin,
        ),
        (
            &komec_release,
            format!("{arch}-kome-std-{version}.tar.zst"),
            Destination::StandardLibrary,
        ),
        (
            &komeup_release,
            format!("{arch}-komeup-{version}.tar.zst"),
            Destination::Bin,
        ),
        (&toolchains_release, toolchain_name, Destination::Home),
        (
            &devkit_release,
            format!("{arch}-appcore-{version}.tar.zst"),
            Destination::AppCore,
        ),
        (
            &viewkit_release,
            format!("{arch}-viewkit-{version}.tar.zst"),
            Destination::ViewKit,
        ),
    ];

    let mut archives = Vec::with_capacity(requested.len());
    for (release, name, destination) in requested {
        let mut archive = download_and_verify(&client, release, &name)?;
        archive.destination = destination;
        archives.push(archive);
    }

    let home = kome_home()?;
    install_archives(&home, version, &archives)?;
    println!(
        "Kome SDK {version}を{}へインストールしました。",
        home.display()
    );
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let home = kome_home()?;
    if !home.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&home)
        .map_err(|error| format!("{}を削除できませんでした: {error}", home.display()))?;
    println!("{}を削除しました。", home.display());
    Ok(())
}

fn validate_version(version: &str) -> Result<(), String> {
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(format!("不正なSDK版です: `{version}`"));
    }
    Ok(())
}

fn release_api(repository: &str, version: &str) -> String {
    format!("{GITHUB_API}/{repository}/releases/tags/{version}")
}

fn fetch_release(client: &Client, repository: &str, version: &str) -> Result<Release, String> {
    client
        .get(release_api(repository, version))
        .send()
        .map_err(|error| {
            format!("{repository} {version}のリリース情報を取得できませんでした: {error}")
        })?
        .error_for_status()
        .map_err(|error| {
            format!("{repository} {version}のリリース情報を取得できませんでした: {error}")
        })?
        .json()
        .map_err(|error| format!("{repository}のリリース情報が不正です: {error}"))
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
        name: asset_name.to_owned(),
        data,
        destination: Destination::Home,
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
        .map_err(|error| format!("{asset_name}をダウンロードできませんでした: {error}"))?
        .error_for_status()
        .map_err(|error| format!("{asset_name}をダウンロードできませんでした: {error}"))?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| format!("{asset_name}を読み込めませんでした: {error}"))
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
        .map_err(|error| format!("{asset_name}をダウンロードできませんでした: {error}"))?
        .error_for_status()
        .map_err(|error| format!("{asset_name}をダウンロードできませんでした: {error}"))?
        .text()
        .map_err(|error| format!("{asset_name}を読み込めませんでした: {error}"))
}

fn find_asset<'a>(release: &'a Release, name: &str) -> Result<&'a Asset, String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| format!("リリース{}に`{name}`がありません", release.tag_name))
}

fn find_toolchain_asset(release: &Release, arch: &str, version: &str) -> Result<String, String> {
    let candidates = [
        format!("{arch}-mochios-toolchain-{version}.tar.zst"),
        format!("{arch}-mochios-toolchain-{version}.zst"),
    ];
    candidates
        .into_iter()
        .find(|candidate| release.assets.iter().any(|asset| asset.name == *candidate))
        .ok_or_else(|| {
            format!(
                "リリース{}に{arch}用ツールチェーンがありません",
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
        .ok_or_else(|| format!("`{name}`のチェックサムがありません"))?;
    let actual = format!("{:x}", Sha256::digest(data));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!("`{name}`のチェックサムが一致しません"));
    }
    Ok(())
}

fn install_archives(
    home: &Path,
    version: &str,
    archives: &[DownloadedArchive],
) -> Result<(), String> {
    let parent = home.parent().ok_or_else(|| {
        format!(
            "インストール先{}に親ディレクトリがありません",
            home.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("{}を作成できませんでした: {error}", parent.display()))?;
    if home.exists() {
        let metadata = fs::symlink_metadata(home)
            .map_err(|error| format!("{}を確認できませんでした: {error}", home.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "インストール先{}は通常のディレクトリではありません",
                home.display()
            ));
        }
    }

    let staging = tempfile::Builder::new()
        .prefix(".kome-install-")
        .tempdir_in(parent)
        .map_err(|error| format!("一時ディレクトリを作成できませんでした: {error}"))?;
    let staged_home = staging.path().join("home");
    fs::create_dir_all(&staged_home)
        .map_err(|error| format!("{}を作成できませんでした: {error}", staged_home.display()))?;

    for archive in archives {
        let destination = archive.destination.path(&staged_home);
        fs::create_dir_all(&destination)
            .map_err(|error| format!("{}を作成できませんでした: {error}", destination.display()))?;
        extract_archive(&archive.data, &destination)
            .map_err(|error| format!("{}を展開できませんでした: {error}", archive.name))?;
    }
    fs::write(staged_home.join("sdk-version"), format!("{version}\n"))
        .map_err(|error| format!("SDK版を記録できませんでした: {error}"))?;

    if !home.exists() {
        return fs::rename(&staged_home, home)
            .map_err(|error| format!("{}を配置できませんでした: {error}", home.display()));
    }

    let backup = parent.join(format!(".kome-backup-{}", std::process::id()));
    if backup.exists() {
        return Err(format!(
            "一時バックアップ{}が既に存在します",
            backup.display()
        ));
    }
    fs::rename(home, &backup).map_err(|error| format!("既存SDKを退避できませんでした: {error}"))?;
    if let Err(error) = fs::rename(&staged_home, home) {
        let _ = fs::rename(&backup, home);
        return Err(format!("新しいSDKを配置できませんでした: {error}"));
    }
    fs::remove_dir_all(&backup)
        .map_err(|error| format!("古いSDKを削除できませんでした: {error}"))?;
    Ok(())
}

fn extract_archive(data: &[u8], destination: &Path) -> Result<(), String> {
    let decoder = zstd::stream::read::Decoder::new(Cursor::new(data))
        .map_err(|error| format!("圧縮データを開けませんでした: {error}"))?;
    tar::Archive::new(decoder)
        .unpack(destination)
        .map_err(|error| format!("アーカイブを展開できませんでした: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_tagged_sdk_releases() {
        assert_eq!(
            release_api("komec", "27.0-dp.1"),
            "https://api.github.com/repos/mochiOS/komec/releases/tags/27.0-dp.1"
        );
    }

    #[test]
    fn rejects_unsafe_sdk_versions() {
        assert!(validate_version("27.0-dp.1").is_ok());
        assert!(validate_version("../latest").is_err());
    }
}
