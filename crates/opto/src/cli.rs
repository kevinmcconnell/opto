use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::range::{PRESETS, preset_named};

#[derive(Parser)]
#[command(
    name = "opto",
    version,
    about = "See how busy your machine has been, and why",
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[command(flatten)]
    pub view: View,
}

#[derive(Subcommand)]
pub enum Command {
    /// Open the viewer (the default when no command is given)
    View(View),
    /// Start the collector and connect Claude Code, or set up just one of them
    Setup { tool: Option<Tool> },
    /// Undo what `setup` did; the recorded data is kept
    Remove { tool: Option<Tool> },
}

#[derive(Args)]
pub struct View {
    /// Database to read
    #[arg(long, value_name = "PATH", default_value_os_t = opto_core::db::default_db_path())]
    pub db: PathBuf,
    /// Time range to show: 5m, 10m, 15m, 30m, 1h, 6h, 24h or 5d
    #[arg(long, value_name = "RANGE", default_value = PRESETS[0].0, value_parser = parse_range)]
    pub range: i64,
    /// Another machine to read over ssh
    #[arg(value_name = "USER@HOST")]
    pub machine: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Tool {
    /// The collector's systemd user service
    Service,
    /// Claude Code's status line, which feeds the Claude card
    Claude,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Setup,
    Remove,
}

fn parse_range(name: &str) -> Result<i64, String> {
    preset_named(name).ok_or_else(|| format!("unknown range {name:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("opto").chain(args.iter().copied())).unwrap()
    }

    #[test]
    fn bare_opto_opens_the_viewer() {
        let cli = parse(&["--range", "6h", "me@host"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.view.range, 6 * 3600);
        assert_eq!(cli.view.machine.as_deref(), Some("me@host"));
    }

    #[test]
    fn view_takes_the_same_options() {
        let Some(Command::View(view)) = parse(&["view", "--range", "5m"]).command else {
            panic!("expected view");
        };
        assert_eq!(view.range, 300);
        assert_eq!(view.machine, None);
    }

    #[test]
    fn setup_and_remove_take_an_optional_tool() {
        assert!(matches!(
            parse(&["setup"]).command,
            Some(Command::Setup { tool: None })
        ));
        assert!(matches!(
            parse(&["setup", "service"]).command,
            Some(Command::Setup {
                tool: Some(Tool::Service)
            })
        ));
        assert!(matches!(
            parse(&["remove"]).command,
            Some(Command::Remove { tool: None })
        ));
        assert!(matches!(
            parse(&["remove", "claude"]).command,
            Some(Command::Remove {
                tool: Some(Tool::Claude)
            })
        ));
        assert!(Cli::try_parse_from(["opto", "setup", "vim"]).is_err());
    }

    #[test]
    fn rejects_unknown_ranges() {
        assert!(Cli::try_parse_from(["opto", "--range", "2h"]).is_err());
    }
}
