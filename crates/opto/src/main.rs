mod askpass;
mod axis;
mod cards;
mod chart;
mod cli;
mod config;
mod integrate;
mod machine;
mod palette;
mod procs;
mod range;
mod service;
mod window;
mod worker;

use adw::prelude::*;
use clap::Parser;
use gtk::glib;

use crate::cli::{Action, Cli, Command, Tool};

const APP_ID: &str = "io.github.kevinmcconnell.Opto";

fn main() -> glib::ExitCode {
    if let Some(code) = askpass::answer_for_ssh() {
        return code;
    }
    let cli = Cli::parse();
    let view = match cli.command {
        None => cli.view,
        Some(Command::View(view)) => view,
        Some(Command::Setup { tool }) => std::process::exit(run(Action::Setup, tool)),
        Some(Command::Remove { tool }) => std::process::exit(run(Action::Remove, tool)),
    };
    let cli::View { db, range, machine } = view;
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| range::load_style());
    app.connect_activate(move |app| {
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }
        window::build(
            app,
            db.clone(),
            machine.clone(),
            range,
            omarchy_theme::follow(),
        );
    });
    app.run_with_args::<&str>(&[])
}

/// Carries on past a failing tool, and returns the first failure's code.
fn run(action: Action, tool: Option<Tool>) -> i32 {
    let tools = match (action, tool) {
        (_, Some(tool)) => vec![tool],
        (Action::Setup, None) if !integrate::claude_installed() => {
            println!("opto: Claude Code not found; skipping its status line");
            vec![Tool::Service]
        }
        (Action::Setup, None) => vec![Tool::Service, Tool::Claude],
        (Action::Remove, None) => vec![Tool::Claude, Tool::Service],
    };
    tools
        .into_iter()
        .map(|tool| match tool {
            Tool::Service => service::service(action),
            Tool::Claude => integrate::claude(action),
        })
        .fold(0, |first, code| if first != 0 { first } else { code })
}
