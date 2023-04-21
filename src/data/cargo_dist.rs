use axoasset::{Asset, LocalAsset};
use cargo_dist_schema::Release as DistRelease;
use cargo_dist_schema::{ArtifactKind, DistManifest};

use crate::data::Release;
use crate::errors::*;

pub struct Installer {
    hint: String,
}

impl Installer {
    fn write_source(&self, dist_dir: &str) -> Result<String> {
        let installer_source_future = Asset::load_string(installer_url);
        let installer_source =
            tokio::runtime::Handle::current().block_on(installer_source_future)?;
        let file_path = format!("{}.txt", &installer_name);
        LocalAsset::write_new(&installer_source, &file_path, dist_dir)?;
        Ok(file_path)
    }

    pub fn new(release: Release) -> Result<Self> {
        let hint = "dummy".to_string();
        Ok(Self { hint })
    }
}

pub fn get_os(name: &str) -> Option<&str> {
    match name.trim() {
        "x86_64-unknown-linux-gnu" => Some("linux"),
        "x86_64-apple-darwin" => Some("mac"),
        "aarch64-apple-darwin" => Some("arm mac"),
        "x86_64-pc-windows-msvc" => Some("windows"),
        &_ => None,
    }
}

pub fn get_kind_string(kind: &ArtifactKind) -> String {
    match kind {
        ArtifactKind::ExecutableZip => String::from("Executable Zip"),
        ArtifactKind::Symbols => String::from("Symbols"),
        ArtifactKind::Installer => String::from("Installer"),
        _ => String::from("Unknown"),
    }
}
