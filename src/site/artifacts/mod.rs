use axohtml::elements::div;
use axohtml::html;

use crate::errors::*;
use crate::site::Context;

mod installers;
mod package_managers;
mod table;

pub struct ArtifactsPage {
    installer_list: Option<Box<div<String>>>,
    package_manager_list: Option<Box<div<String>>>,
    artifacts_table: Option<Box<div<String>>>,
}

impl ArtifactsPage {
    pub fn new(context: &Context) -> Result<Self> {
        let (installer_list, artifacts_table) =
            if let Some(latest_release) = context.releases.latest() {
                if let Some(manifest) = latest_release.manifest {
                    (
                        Some(installers::build_list(latest_release)?),
                        Some(table::build(latest_release)?),
                    )
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

        let package_manager_list = if let Some(package_managers) = context.package_managers {
            Some(package_managers::build_list(&package_managers, context))
        } else {
            None
        };

        Ok(Self {
            installer_list,
            package_manager_list,
            artifacts_table,
        })
    }

    pub fn build(&self) -> Result<String> {
        Ok(html!(
            <div><div class="package-managers-downloads">
            {self.installer_list}
            {self.package_manager_list}
            </div>
            {self.artifacts_table}</div>
        ))
    }
}

pub fn header(context: &Context) -> Result<Option<Box<div<String>>>> {
    if let Some(latest_release) = context.releases.latest() {
        if let Some(manifest) = latest_release.manifest {
            let header = installers::build_header(manifest, context)?;
            return Ok(Some(header));
        }
    }
    if let Some(package_managers) = &context.package_managers {
        let header = package_managers::build_header(package_managers, context.syntax_theme)?;
        return Ok(Some(header));
    }
    Ok(None)
}
