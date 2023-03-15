mod icons;

use crate::errors::*;
use axohtml::{dom::UnsafeTextNode, elements::li, html, types::SpacedList};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contents {
    pub name: String,
    pub content: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Funding {
    github: Option<Vec<String>>,
    patreon: Option<String>,
    open_collective: Option<String>,
    ko_fi: Option<String>,
    tidelift: Option<String>,
    community_bridge: Option<String>,
    liberapay: Option<String>,
    issuehunt: Option<String>,
    lfx_crowdfunding: String,
    custom: Option<Vec<String>>,
}

fn create_link(link: &str, icon: Box<UnsafeTextNode<String>>) -> Box<axohtml::elements::a<String>> {
    let mut rels = SpacedList::new();
    rels.add("noopener");
    rels.add("noreferrer");
    html!(<a href=link target="_blank" rel=rels>{icon}</a>)
}

pub fn build_funding_html(funding_links: Funding) -> Vec<Box<li<String>>> {
    let mut html = vec![];
    if let Some(github) = funding_links.github {
        for link in github {
            let gh_link = format!("https://github.com/sponsors/{}", link);

            html.extend(html!(<li>{create_link(&gh_link,icons::get_github_icon())}</li>))
        }
    }

    if let Some(patreon) = funding_links.patreon {
        let patreon_link = format!("https://www.patreon.com/{}", patreon);

        html.extend(html!(<li>{create_link(&patreon_link,icons::get_patreon_icon())}</li>))
    }

    if let Some(open_collective) = funding_links.open_collective {
        let open_collective_link = format!("https://opencollective.com/{}", open_collective);

        html.extend(
            html!(<li>{create_link(&open_collective_link,icons::get_open_collective_icon())}</li>),
        )
    }

    if let Some(ko_fi) = funding_links.ko_fi {
        let ko_fi_link = format!("https://ko-fi.com/{}", ko_fi);

        html.extend(html!(<li>{create_link(&ko_fi_link,icons::get_kofi_icon())}</li>))
    }

    if let Some(tidelift) = funding_links.tidelift {
        let tidelift_link = format!("https://tidelift.com/subscription/pkg/{}", tidelift);

        html.extend(html!(<li>{create_link(&tidelift_link,icons::get_tidelif_icon())}</li>))
    }

    if let Some(liberapay) = funding_links.liberapay {
        let liberapay_link = format!("https://liberapay.com/{}", liberapay);

        html.extend(html!(<li>{create_link(&liberapay_link,icons::get_liberapay_icon())}</li>))
    }

    if let Some(issue_hunt) = funding_links.issuehunt {
        // their logo SUCKS need to find a way to minify that
        let issue_hunt_link = format!("https://issuehunt.io/r/{}", issue_hunt);

        html.extend(html!(<li>{create_link(&issue_hunt_link,icons::get_liberapay_icon())}</li>))
    }

    html
}

pub fn fetch_funding_info(repo: &str) -> Result<String> {
    let repo_parsed = match Url::parse(repo) {
        Ok(parsed) => Ok(parsed),
        Err(parse_error) => Err(OrandaError::RepoParseError {
            repo: repo.to_string(),
            details: parse_error.to_string(),
        }),
    };
    let binding = repo_parsed?;
    let parts = binding.path_segments().map(|c| c.collect::<Vec<_>>());
    if let Some(url_parts) = parts {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/.github/FUNDING.yml",
            url_parts[0], url_parts[1]
        );
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let header = format!("oranda-{}", VERSION);

        let response = reqwest::blocking::Client::new()
            .get(url)
            .header(USER_AGENT, header)
            .send()?;

        match response.error_for_status() {
            Ok(r) => match r.json::<Contents>() {
                Ok(contents) => {
                    let string_yaml = &general_purpose::STANDARD
                        .decode(contents.content.replace('\n', ""))
                        .unwrap();
                    let yaml_contents = match std::str::from_utf8(string_yaml) {
                        Ok(y) => {
                            let deserialized_map: Funding = serde_yaml::from_str(y).unwrap();
                            let funding_html = build_funding_html(deserialized_map);
                            Ok(html!(<ul class="funding-list">{funding_html}</ul>).to_string())
                        }
                        Err(e) => Err(OrandaError::GithubFundingParseError {
                            details: e.to_string(),
                        }),
                    }?;

                    Ok(yaml_contents.to_string())
                }
                Err(e) => Err(OrandaError::GithubFundingParseError {
                    details: e.to_string(),
                }),
            },
            Err(e) => Err(OrandaError::GithubFundingFetchError {
                details: e.to_string(),
            }),
        }
    } else {
        Err(OrandaError::RepoParseError {
            repo: binding.to_string(),
            details: "This URL is not structured the expected way, expected more segments-"
                .to_owned(),
        })
    }
}