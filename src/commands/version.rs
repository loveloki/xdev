use crate::core::i18n::t;
use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(Command::new("version").about(t!("command.version.description").to_string()));
}

pub fn handle_command(matches: &ArgMatches) -> Result<()> {
    if matches.subcommand_matches("version").is_some() {
        execute()
    } else {
        Ok(())
    }
}

pub fn execute() -> Result<()> {
    println!(
        "{}",
        t!(
            "command.version.output",
            version = env!("CARGO_PKG_VERSION")
        )
    );
    Ok(())
}
