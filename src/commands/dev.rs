use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use axoproject::WorkspaceSearch;
use camino::Utf8PathBuf;
use clap::Parser;
use miette::Report;

use crate::{
    commands::{Build, Serve},
    message::{Message, MessageType},
};
use oranda::{
    config::Config,
    errors::*,
    site::mdbook::{custom_theme, load_mdbook, mdbook_dir},
};

#[derive(Clone, Debug, Parser)]
pub struct Dev {
    /// The port for the file server to be launched on
    #[arg(long)]
    port: Option<u16>,
    /// DO NOT USE: Path to the root dir of the project
    ///
    /// This flag exists for internal testing. It is incorrectly implemented for actual
    /// end-users and will make you very confused and sad.
    #[clap(hide = true)]
    #[arg(long)]
    project_root: Option<Utf8PathBuf>,
    /// DO NOT USE: Path to the oranda.json
    ///
    /// This flag exists for internal testing. It is incorrectly implemented for actual
    /// end-users and will make you very confused and sad.
    #[clap(hide = true)]
    #[arg(long)]
    config_path: Option<Utf8PathBuf>,
    /// Skip the first build before starting to watch for changes
    #[arg(long)]
    no_first_build: bool,
    /// List of extra paths to watch
    #[arg(short, long)]
    include_paths: Option<Vec<Utf8PathBuf>>,
}

impl Dev {
    pub fn run(self) -> Result<()> {
        Message::new(
            MessageType::Info,
            "Starting dev, looking for paths to watch...",
        )
        .print();
        tracing::info!("Starting dev, looking for paths to watch...");

        let config = Config::build(
            &self
                .config_path
                .clone()
                .unwrap_or(Utf8PathBuf::from("./oranda.json")),
        )?;
        let mut paths_to_watch = vec![];
        // Watch for the readme file
        paths_to_watch.push(config.project.readme_path);
        // Watch for the oranda config file
        paths_to_watch.push(
            self.config_path
                .clone()
                .unwrap_or(Utf8PathBuf::from("./oranda.json"))
                .into(),
        );

        // Watch for any user-provided paths
        if let Some(include_paths) = &self.include_paths {
            let mut include_paths: Vec<String> =
                include_paths.iter().map(|p| p.to_string()).collect();
            paths_to_watch.append(&mut include_paths);
        }

        // Watch for the funding.md page and the funding.yml file
        if let Some(funding) = &config.components.funding {
            if let Some(path) = &funding.yml_path {
                paths_to_watch.push(path.clone());
            }
            if let Some(path) = &funding.md_path {
                paths_to_watch.push(path.clone());
            }
        }

        // Watch for additional pages, if we have any
        if !config.build.additional_pages.is_empty() {
            let mut additional_pages: Vec<String> =
                config.build.additional_pages.values().cloned().collect();
            paths_to_watch.append(&mut additional_pages);
        }

        // Watch for the mdbook directory, if we have it
        if let Some(book_cfg) = &config.components.mdbook {
            let path = mdbook_dir(book_cfg)?;
            let md = load_mdbook(&path)?;
            // watch book.toml and /src/
            paths_to_watch.push(md.root.join("book.toml").display().to_string());
            paths_to_watch.push(md.source_dir().display().to_string());

            // If we're not clobbering the theme, also watch the theme dir
            // (note that this may not exist on the fs, mdbook reports the path regardless)
            if custom_theme(book_cfg, &config.styles.theme).is_none() {
                paths_to_watch.push(md.theme_dir().display().to_string());
            }
        }

        // Watch for any project manifest files
        let project = axoproject::get_workspaces("./".into(), None);
        if let WorkspaceSearch::Found(workspace) = project.rust {
            paths_to_watch.push(workspace.manifest_path.into());
        }
        if let WorkspaceSearch::Found(workspace) = project.javascript {
            paths_to_watch.push(workspace.manifest_path.into());
        }

        let (tx, rx) = std::sync::mpsc::channel();

        // We debounce events so that we don't end up building 5 times in a row because 5 different
        // filesystem events fired.
        let mut debouncer = notify_debouncer_mini::new_debouncer(Duration::from_secs(1), None, tx)?;
        let watcher = debouncer.watcher();
        let mut existing_paths = vec![];
        for path in paths_to_watch {
            let path = PathBuf::from(path);
            // If no path exists, oranda won't work anyways, so there's not much need to let the user know
            // (they'll know either way ;) )
            if path.exists() {
                watcher.watch(
                    path.as_path(),
                    notify_debouncer_mini::notify::RecursiveMode::Recursive,
                )?;
                existing_paths.push(path.clone());
            }
        }

        Message::new(
            MessageType::Info,
            &format!(
                "Found {} paths to watch, starting watch...",
                existing_paths.len()
            ),
        )
        .print();
        tracing::info!(
            "Found {} paths to watch, starting watch...",
            existing_paths.len()
        );
        Message::new(
            MessageType::Debug,
            &format!("Files watched: {:?}", existing_paths),
        )
        .print();

        if !self.no_first_build {
            Build::new(self.project_root.clone(), self.config_path.clone()).run()?;
        }

        // Spawn the serve process out into a separate thread so that we can loop through received events on this thread
        let _ = std::thread::spawn(move || Serve::new(self.port).run());
        loop {
            // Wait for all debounced events to arrive
            let first_event = rx.recv().unwrap();
            sleep(Duration::from_millis(50));
            let other_events = rx.try_iter();

            let all_events = std::iter::once(first_event).chain(other_events);
            // Unpack events into paths
            let paths: Vec<_> = all_events
                .filter_map(|event| match event {
                    Ok(events) => Some(events),
                    Err(errors) => {
                        for error in errors {
                            Message::new(
                                MessageType::Warning,
                                &format!("Error while watching for changes: {error}",),
                            )
                            .print();
                            tracing::warn!("Error while watching for changes: {error}",);
                        }
                        None
                    }
                })
                .flatten()
                .map(|event| event.path)
                .collect();

            if !paths.is_empty() {
                Message::new(
                    MessageType::Info,
                    &format!("Path(s) {:?} changed, rebuilding...", paths),
                )
                .print();

                if let Err(e) =
                    Build::new(self.project_root.clone(), self.config_path.clone()).run()
                {
                    eprintln!("{:?}", Report::new(e));
                    continue;
                }
            }
        }
    }
}
