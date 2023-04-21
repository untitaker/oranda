use cargo_dist_schema::DistManifest;

use crate::errors::*;

pub mod cargo_dist;
pub mod github;
use github::{GithubRelease, GithubRepo};

pub struct Release {
    pub manifest: Option<DistManifest>,
    pub source: GithubRelease,
}

pub struct Releases {
    pub all: Vec<Release>,
    pub has_prereleases: bool,
}

impl Release {
    fn new(gh_release: GithubRelease) -> Result<Self> {
        Ok(Self {
            manifest: Self::fetch_manifest(gh_release.dist_manifest_url())?,
            source: gh_release,
        })
    }

    fn fetch_manifest(manifest_url: &str) -> Result<Option<DistManifest>> {
        match reqwest::blocking::get(manifest_url)?.error_for_status() {
            Ok(resp) => match resp.json::<DistManifest>() {
                Ok(manifest) => Ok(Some(manifest)),
                Err(e) => Err(OrandaError::CargoDistManifestParseError {
                    url: manifest_url.to_string(),
                    details: e,
                }),
            },
            Err(e) => Err(OrandaError::CargoDistManifestFetchError {
                url: manifest_url.to_string(),
                status_code: e.status().unwrap_or(reqwest::StatusCode::BAD_REQUEST),
            }),
        }
        Ok(None)
    }
}

impl Releases {
    pub fn fetch(repo: &GithubRepo) -> Result<Self> {
        let mut has_prereleases = false;
        let gh_releases = GithubRelease::fetch_all(repo)?;
        let all = vec![];
        for gh_release in gh_releases {
            if gh_release.prerelease {
                has_prereleases = true;
            }
            all.push(Release::new(gh_release))
        }

        Ok(Releases {
            all,
            has_prereleases,
        })
    }

    pub fn latest(&self) -> Option<Release> {
        self.all.first()
    }

    fn latest_dist_release(&self) -> Option<Release> {
        for release in self.all {
            if release.manifest.is_some() {
                return Some(release);
            }
        }
        None
    }
}
