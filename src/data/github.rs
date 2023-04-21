use crate::errors::*;

use miette::{miette, IntoDiagnostic};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug)]
pub struct GithubRepo {
    owner: String,
    name: String,
}

impl GithubRepo {
    pub fn from(repo_url: &str) -> Result<Self> {
        let repo_parsed = match Url::parse(repo_url).into_diagnostic() {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(OrandaError::RepoParseError {
                repo: repo_url.to_string(),
                details: e,
            }),
        };
        let binding = repo_parsed?;
        let segment_list = binding.path_segments().map(|c| c.collect::<Vec<_>>());
        if let Some(segments) = segment_list {
            if segments.len() == 2 {
                return Ok(Self {
                    owner: segments[0].to_string(),
                    name: segments[1].to_string(),
                });
            }
        }
        Err(OrandaError::RepoParseError {
            repo: binding.to_string(),
            details: miette!("This URL is not structured the expected way, expected more segments"),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubRelease {
    pub url: String,
    pub assets_url: String,
    pub html_url: String,
    pub id: i64,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<GithubReleaseAsset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubReleaseAsset {
    pub url: String,
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub label: String,
    pub content_type: String,
    pub state: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

impl GithubRelease {
    pub fn fetch_all(repo: &GithubRepo) -> Result<Vec<GithubRelease>> {
        let url = format!(
            "https://octolotl.axodotdev.host/releases/{}/{}",
            repo.owner, repo.name
        );
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let header = format!("oranda-{}", VERSION);

        let response = reqwest::blocking::Client::new()
            .get(url)
            .header(USER_AGENT, header)
            .send()?;

        match response.error_for_status() {
            Ok(r) => match r.json() {
                Ok(releases) => Ok(releases),
                Err(e) => Err(OrandaError::GithubReleaseParseError { details: e }),
            },
            Err(e) => Err(OrandaError::GithubReleasesFetchError { details: e }),
        }
    }

    pub fn asset_url(&self, asset_name: &str) -> Option<String> {
        for asset in &self.assets {
            if asset.name == asset_name {
                return Some(asset.browser_download_url);
            }
        }
        None
    }
}
