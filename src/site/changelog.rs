use axohtml::elements::{div, li, section};
use axohtml::html;
use axohtml::{text, unsafe_text};
use chrono::DateTime;

use crate::data::{Release, Releases};
use crate::errors::*;
use crate::site::{icons, markdown, markdown::SyntaxTheme, Context};

pub fn build(context: &Context) -> Result<String> {
    let releases = Releases::fetch(&context.repo)?;
    let mut releases_html: Vec<Box<section<String>>> = vec![];
    let mut releases_nav: Vec<Box<li<String>>> = vec![];
    for release in releases.all.iter() {
        let classnames = if release.source.prerelease {
            "pre-release hidden"
        } else {
            ""
        };

        let link = format!("#{}", &release.source.tag_name);

        releases_html.extend(build_page_preview(release, &context.syntax_theme)?);
        releases_nav.extend(
            html!(<li class=classnames><a href=link>{text!(&release.source.tag_name)}</a></li>),
        )
    }

    Ok(html!(
        <div>
            <h1>{text!("Releases")}</h1>
            <div class="releases-wrapper">
                <nav class="releases-nav">
                    {build_prerelease_toggle(releases)}
                    <ul>
                        {releases_nav}
                    </ul>
                </nav>
                <div class="releases-list">{releases_html}</div>
            </div>
        </div>
    )
    .to_string())
}

pub fn build_page_preview(
    release: &Release,
    syntax_theme: &SyntaxTheme,
) -> Result<Box<section<String>>> {
    let tag_name = release.source.tag_name.clone();
    let title = release.source.name.clone().unwrap_or(tag_name.clone());

    let id: axohtml::types::Id = axohtml::types::Id::new(tag_name.clone());
    let formatted_date = match DateTime::parse_from_rfc3339(&release.source.published_at) {
        Ok(date) => date.format("%b %e %Y at %R UTC").to_string(),
        Err(_) => release.source.published_at.to_owned(),
    };

    let classnames = if release.source.prerelease {
        "release pre-release hidden"
    } else {
        "release"
    };
    let link = format!("#{}", &tag_name);
    let body = build_release_body(release, syntax_theme)?;

    Ok(html!(
    <section class=classnames>
        <h2 id=id><a href=link>{text!(title)}</a></h2>
        <div class="release-info">
            <span class="flex items-center gap-2">
                {icons::tag()}{text!(tag_name)}
            </span>
            <span class="flex items-center gap-2">
                {icons::date()}{text!(&formatted_date)}
            </span>
        </div>
        <div class="release-body mb-6">
            {unsafe_text!(body)}
        </div>
    </section>
    ))
}

fn build_release_body(release: &Release, syntax_theme: &SyntaxTheme) -> Result<String> {
    let contents = if let Some(manifest) = release.manifest {
        manifest.announcement_changelog.unwrap_or(String::new())
    } else {
        release.source.body.clone().unwrap_or(String::new())
    };

    markdown::to_html(&contents, &syntax_theme)
}

fn build_prerelease_toggle(releases: Releases) -> Option<Box<div<String>>> {
    if releases.has_prereleases {
        Some(html!(
    <div class="prereleases-toggle">
        <div class="flex h-6 items-center">
            <input id="show-prereleases" type="checkbox" />
        </div>
        <div class="ml-3">
            <label for="show-prereleases">{text!("Show prereleases")}</label>
        </div>
    </div>))
    } else {
        None
    }
}
