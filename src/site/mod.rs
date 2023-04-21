use std::path::Path;

use axoasset::LocalAsset;
use linked_hash_map::LinkedHashMap;

use crate::config::Config;
use crate::data::{github::GithubRepo, Releases};
use crate::errors::*;
use crate::message::{Message, MessageType};

pub mod artifacts;
use artifacts::ArtifactsPage;
pub mod icons;
pub mod layout;
use layout::{css, javascript};
pub mod link;
pub mod markdown;
pub mod page;
use page::Page;
pub mod changelog;

#[derive(Debug)]
pub struct Site {
    pages: Vec<Page>,
    context: Option<Context>,
}

pub struct Context {
    path_prefix: Option<String>,
    changelog: bool,
    package_managers: Option<LinkedHashMap<String, String>>,
    repo: GithubRepo,
    syntax_theme: markdown::SyntaxTheme,
    releases: Releases,
}

impl Site {
    pub fn build(config: &Config) -> Result<Site> {
        Self::clean_dist_dir(&config.dist_dir)?;
        let index = Page::new_from_file(config, &config.readme_path, true)?;
        let mut pages = vec![index];
        if let Some(files) = &config.additional_pages {
            for file_path in files.values() {
                if page::source::is_markdown(file_path) {
                    let additional_page = Page::new_from_file(config, file_path, false)?;
                    pages.push(additional_page);
                } else {
                    let msg = format!(
                        "File {} in additional pages is not markdown and will be skipped",
                        file_path
                    );
                    Message::new(MessageType::Warning, &msg).print();
                }
            }
        }

        let mut context = None;
        if config.artifacts.is_some() || config.changelog {
            context = Some(Self::context(config)?);
            if let Some(latest_release) = context.releases.latest() {
                let artifacts_html = ArtifactsPage::new(&context)?.build()?;
                let artifacts_page =
                    Page::new_from_contents(artifacts_html, "artifacts.html", true);
                pages.push(artifacts_page);
            }
            if config.changelog {
                let changelog_html = changelog::build(&context)?;
                let changelog_page =
                    Page::new_from_contents(changelog_html, "changelog.html", true);
                pages.push(changelog_page);
            }
        }

        Ok(Site { pages, context })
    }

    fn context(config: &Config) -> Result<Context> {
        if let Some(repo_url) = config.repository {
            let repo = GithubRepo::from(&repo_url)?;
            let releases = Releases::fetch(&repo)?;
            if let Some(artifacts) = config.artifacts {
                Ok(Context {
                    path_prefix: config.path_prefix,
                    changelog: config.changelog,
                    package_managers: artifacts.package_managers,
                    repo,
                    syntax_theme: config.syntax_theme,
                    releases,
                })
            } else {
                Ok(Context {
                    path_prefix: config.path_prefix,
                    changelog: config.changelog,
                    package_managers: None,
                    repo,
                    syntax_theme: config.syntax_theme,
                    releases,
                })
            }
        } else {
            Err(OrandaError::Other(
                "repo is required for current feature set".to_string(),
            ))
        }
    }

    pub fn copy_static(dist_path: &String, static_path: &String) -> Result<()> {
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        fs_extra::copy_items(&[static_path], dist_path, &options)?;

        Ok(())
    }

    pub fn write(self, config: &Config) -> Result<()> {
        let dist = &config.dist_dir;
        for page in self.pages {
            let contents = page.build(&self.context, config)?;
            LocalAsset::write_new(&contents, &page.filename, dist)?;
        }
        if let Some(book_path) = &config.md_book {
            Self::copy_static(dist, book_path)?;
        }
        if Path::new(&config.static_dir).exists() {
            Self::copy_static(dist, &config.static_dir)?;
        }
        javascript::write_os_script(&config.dist_dir)?;
        if !config.additional_css.is_empty() {
            css::write_additional(&config.additional_css, &config.dist_dir)?;
        }

        Ok(())
    }

    pub fn clean_dist_dir(dist_path: &str) -> Result<()> {
        if Path::new(dist_path).exists() {
            std::fs::remove_dir_all(dist_path)?;
        }
        match std::fs::create_dir_all(dist_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(OrandaError::DistDirCreationError {
                dist_path: dist_path.to_string(),
                details: e,
            }),
        }
    }
}
