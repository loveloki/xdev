use crate::core::i18n::t;
use anyhow::Result;
use clap::Command;

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(Command::new("version").about(t!("command.version.description").to_string()));
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
