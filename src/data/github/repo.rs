use crate::errors::*;

use miette::{miette, IntoDiagnostic};
use url::Url;

/// Represents a GitHub repository that we can query things about.
#[derive(Debug, Clone)]
pub struct GithubRepo {
    /// The repository owner.
    pub owner: String,
    /// The repository name.
    pub name: String,
}

#[derive(Debug)]
enum GithubRepoInput {
    Url(String),
    Ssh(String),
}

impl GithubRepoInput {
    fn new(repo_string: String) -> Self {
        if repo_string.starts_with("https") {
            Self::Url(repo_string)
        } else if repo_string.starts_with("git@") {
            Self::Ssh(repo_string)
        } else {
            todo!()
        }
    }

    fn parse(self) -> Result<GithubRepo> {
        match self {
            Self::Url(s) => Ok(Self::parse_url(s)?),
            Self::Ssh(s) => Ok(Self::parse_ssh(s)?),
        }
    }

    fn parse_url(repo_string: String) -> Result<GithubRepo> {
        let binding = Url::parse(&repo_string).into_diagnostic().map_err(|e| {
            OrandaError::RepoParseError {
                repo: repo_string.to_string(),
                details: e,
            }
        })?;
        let segment_list = binding.path_segments().map(|c| c.collect::<Vec<_>>());
        if let Some(segments) = segment_list {
            if segments.len() >= 2 {
                let owner = segments[0].to_string();
                let name = Self::remove_git_suffix(segments[1].to_string());
                let rest_is_empty = segments.iter().skip(2).all(|s| s.trim().is_empty());
                if rest_is_empty {
                    return Ok(GithubRepo { owner, name });
                } else {
                    return Err(OrandaError::RepoParseError {
                        repo: binding.to_string(),
                        details: miette!("This URL has more parts than we expected"),
                    });
                }
            }
        }
        Err(OrandaError::RepoParseError {
                    repo: repo_string,
                    details: miette!("We found a repo url but we had trouble parsing it. Please make sure it's entered correctly. This may be an error, and if so you should file an issue."),
                })
    }

    fn parse_ssh(repo_string: String) -> Result<GithubRepo> {
        todo!();
    }

    fn remove_git_suffix(s: String) -> String {
        if s.ends_with(".git") {
            s.replace(".git", "")
        } else {
            s
        }
    }
}

impl GithubRepo {
    /// Constructs a new Github repository from a "owner/name" string. Notably, this does not check
    /// whether the repo actually exists.
    pub fn from_url(repo_url: &str) -> Result<Self> {
        GithubRepoInput::new(repo_url.to_string()).parse()
    }
}
