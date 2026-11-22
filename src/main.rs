use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use argh::FromArgs;
use semver::Version;
use sendoff::changelog;

/// GitHub-native release tool.
#[derive(FromArgs, Debug, PartialEq)]
struct Args {
    /// print version
    #[argh(switch)]
    version: bool,

    #[argh(subcommand)]
    command: Option<Command>,
}

#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum Command {
    Changelog(ChangelogCommand),
}

/// Extract or update changelog entries.
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "changelog")]
struct ChangelogCommand {
    #[argh(subcommand)]
    action: ChangelogAction,
}

#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand)]
enum ChangelogAction {
    Extract(ExtractCommand),
    Update(UpdateCommand),
}

/// Extract the release notes section for a version.
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "extract")]
struct ExtractCommand {
    /// version to extract (for example, 0.0.1)
    #[argh(positional)]
    version: Version,

    /// path to the changelog file (default: CHANGELOG.md)
    #[argh(option, default = "PathBuf::from(\"CHANGELOG.md\")")]
    file: PathBuf,

    /// heading prefix for version entries (default: ##)
    #[argh(option, default = "String::from(\"##\")")]
    heading_level: String,
}

/// Insert or update a version section in the changelog.
#[derive(FromArgs, Debug, PartialEq)]
#[argh(subcommand, name = "update")]
struct UpdateCommand {
    /// version to insert or update (for example, 0.1.0)
    #[argh(positional)]
    version: Version,

    /// release notes body
    #[argh(option)]
    body: String,

    /// release date (YYYY-MM-DD)
    #[argh(option)]
    date: Option<String>,

    /// path to the changelog file (default: CHANGELOG.md)
    #[argh(option, default = "PathBuf::from(\"CHANGELOG.md\")")]
    file: PathBuf,

    /// heading prefix for version entries (default: ##)
    #[argh(option, default = "String::from(\"##\")")]
    heading_level: String,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let output = run(args)?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
}

fn run(args: Args) -> Result<String> {
    if args.version {
        return Ok(format!("sendoff {}", env!("CARGO_PKG_VERSION")));
    }

    let Some(command) = args.command else {
        bail!("missing subcommand (try --help)");
    };

    match command {
        Command::Changelog(cmd) => match cmd.action {
            ChangelogAction::Extract(extract) => {
                let content = fs::read_to_string(&extract.file)
                    .with_context(|| format!("failed to read {}", extract.file.display()))?;
                Ok(changelog::extract(
                    &content,
                    &extract.version.to_string(),
                    &extract.heading_level,
                )?)
            }
            ChangelogAction::Update(update) => {
                let content = fs::read_to_string(&update.file)
                    .with_context(|| format!("failed to read {}", update.file.display()))?;
                let updated = changelog::update(
                    &content,
                    &update.version.to_string(),
                    update.date.as_deref(),
                    &update.body,
                    &update.heading_level,
                )?;
                fs::write(&update.file, updated)
                    .with_context(|| format!("failed to write {}", update.file.display()))?;
                Ok(String::new())
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_flag() {
        let args = Args::from_args(&["sendoff"], &["--version"]);
        assert_eq!(
            args,
            Ok(Args {
                version: true,
                command: None,
            })
        );
        if let Ok(parsed) = args {
            assert_eq!(
                run(parsed).map_err(|e| e.to_string()),
                Ok(String::from("sendoff 0.0.1"))
            );
        }
    }

    #[test]
    fn parses_changelog_extract_defaults() {
        let args = Args::from_args(&["sendoff"], &["changelog", "extract", "0.0.1"]);
        assert_eq!(
            args,
            Ok(Args {
                version: false,
                command: Some(Command::Changelog(ChangelogCommand {
                    action: ChangelogAction::Extract(ExtractCommand {
                        version: Version::new(0, 0, 1),
                        file: PathBuf::from("CHANGELOG.md"),
                        heading_level: String::from("##"),
                    }),
                })),
            })
        );
    }

    #[test]
    fn parses_changelog_update_options() {
        let args = Args::from_args(
            &["sendoff"],
            &[
                "changelog",
                "update",
                "0.1.0",
                "--body",
                "- Shipped.",
                "--date",
                "2026-11-22",
                "--file",
                "README.md",
                "--heading-level",
                "###",
            ],
        );
        assert_eq!(
            args,
            Ok(Args {
                version: false,
                command: Some(Command::Changelog(ChangelogCommand {
                    action: ChangelogAction::Update(UpdateCommand {
                        version: Version::new(0, 1, 0),
                        body: String::from("- Shipped."),
                        date: Some(String::from("2026-11-22")),
                        file: PathBuf::from("README.md"),
                        heading_level: String::from("###"),
                    }),
                })),
            })
        );
    }
}
