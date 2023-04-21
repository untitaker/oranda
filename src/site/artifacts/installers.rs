use axohtml::elements::{div, li, span};
use axohtml::{html, text, unsafe_text};
use cargo_dist_schema::Artifact as DistArtifact;
use cargo_dist_schema::ArtifactKind as DistArtifactKind;
use cargo_dist_schema::DistManifest;
use cargo_dist_schema::Release as DistRelease;
use chrono::DateTime;

use crate::data::cargo_dist;
use crate::errors::*;
use crate::site::{icons, link, markdown, Context};

fn build_detect_html(targets: &str) -> Box<span<String>> {
    match cargo_dist::get_os(targets) {
        Some(os) => {
            html!(
                <span class="detect">{text!("We have detected you are on ")}
                    <span class="detected-os">{text!(os)}</span>
                    {text!(", are we wrong?")}
                </span>)
        }
        None => {
            html!(<span class="detect">{text!("We couldn't detect the system you are using.")}</span>)
        }
    }
}

pub fn build_header(manifest: DistManifest, context: &Context) -> Result<Box<div<String>>> {
    let downloads_href = link::generate(&context.path_prefix, "artifacts.html");

    let mut html: Vec<Box<div<String>>> = vec![];
    for app in manifest.releases.iter() {
        for artifact_id in app.artifacts.iter() {
            let artifact = manifest.artifacts[artifact_id];
            if let DistArtifactKind::ExecutableZip = artifact.kind {
                let mut targets = String::new();
                for targ in artifact.target_triples.iter() {
                    targets.push_str(format!("{} ", targ).as_str());
                }
                let install_code_block = build_install_block(&manifest, app, &artifact, context);
                let title = format!("Install v{}", app.app_version);
                let publish_date = app.source.published_at;
                let formatted_date = match DateTime::parse_from_rfc3339(&publish_date) {
                    Ok(date) => date.format("%b %e %Y at %R UTC").to_string(),
                    Err(_) => publish_date,
                };
                let date_published = format!("Published at {}", formatted_date);
                html.extend(html!(
                    <div class="hidden target artifact-header" data-targets=&targets>
                        <h4>{text!(title)}</h4>
                        <div>
                            <small class="published-date">{text!(date_published)}</small>
                        </div>
                        {install_code_block}
                        <div>
                            {build_detect_html(&targets)}
                            <a href=&downloads_href>{text!("View all installation options")}</a>
                        </div>
                    </div>
                ));
            }
        }
    }
    Ok(html!(
    <div class="artifacts">
        {html}
        <a href=&downloads_href class="hidden backup-download business-button primary">{text!("View installation options")}</a>
    </div>
    ))
}

fn build_install_block(
    manifest: DistManifest,
    release: DistRelease,
    artifact: DistArtifact,
) -> Result<Box<div<String>>> {
    // If there's an installer that covers that, prefer it
    if let Ok(val) = build_install_block_for_installer(manifest, release, artifact) {
        return Ok(val);
    }

    // Otherwise, just link the artifact
    let url = asset_url(artifact.name)?;
    Ok(html!(
        <div class="install-code-wrapper">
            <a href=url>{text!("Download {}", artifact.name.as_ref().unwrap())}</a>
        </div>
    ))
}

/// Tries to recommend an installer that installs the given artifact
fn build_install_block_for_installer(
    manifest: &DistManifest,
    release: &DistRelease,
    artifact: &DistArtifact,
) -> Result<Box<div<String>>> {
    let install_code = build_install_hint_code(manifest, release, &artifact.target_triples)?;

    let copy_icon = icons::copy();
    let hint = cargo_dist::get_install_hint(manifest, release, &artifact.target_triples)?;

    Ok(html!(
        <div class="install-code-wrapper">
            {unsafe_text!(install_code)}
            <button
                data-copy={hint.0}
                class="button primary copy-clipboard-button button">
                {copy_icon}
            </button>
            <a class="button primary button" href=(hint.1)>
                {text!("Source")}
            </a>
        </div>
    ))
}

pub struct InstallerArtifact {
    title: String,
    install_code_block: (),
}

impl InstallerArtifact {
    fn new(artifact: DistArtifact) -> Self {
        Self {
            title: Self::title(artifact),
            install_code_block: Self::install_block(),
        }
    }

    fn title(artifact: DistArtifact) -> String {
        let mut targets = String::new();
        for targ in artifact.target_triples.iter() {
            targets.push_str(format!("{} ", targ).as_str());
        }

        match artifact.description.clone() {
            Some(desc) => desc,
            None => match cargo_dist::get_os(targets.as_str()) {
                Some(os) => String::from(os),
                None => targets,
            },
        }
    }

    fn build(&self) -> Box<li<String>> {
        html!(
            <li class="list-none">
                <h5 class="capitalize">{text!(self.title)}</h5>
                {self.install_code_block}
            </li>
        )
    }
}

// False positive duplicate allocation warning
// https://github.com/rust-lang/rust-clippy/issues?q=is%3Aissue+redundant_allocation+sort%3Aupdated-desc
#[allow(clippy::vec_box)]
pub fn build_list(manifest: DistManifest, context: &Context) -> Result<Box<div<String>>> {
    let mut list = vec![];
    for app in manifest.releases.iter() {
        for artifact_id in app.artifacts.iter() {
            let artifact = manifest.artifacts[artifact_id];
            if let DistArtifactKind::ExecutableZip = artifact.kind {
                list.push(InstallerArtifact::new().build())
            }
        }
    }

    Ok(html!(
    <div>
        <h3>{text!("Install via script")}</h3>
        <ul>
            {list}
        </ul>
    </div>
    ))
}

pub fn build_install_hint_code(
    manifest: &DistManifest,
    app: &DistRelease,
    target_triples: &[String],
    context: &Context,
) -> Result<String> {
    let install_hint =
        cargo_dist::get_install_hint(manifest, app, target_triples, context.dist_dir)?;

    let highlighted_code =
        markdown::syntax_highlight(Some("sh"), &install_hint.0, &context.syntax_theme);
    match highlighted_code {
        Ok(code) => Ok(code),
        Err(_) => Ok(format!(
            "<code class='inline-code'>{}</code>",
            install_hint.0
        )),
    }
}
