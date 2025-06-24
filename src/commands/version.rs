use anyhow::Result;
use clap::{Command, ArgMatches};

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(
            Command::new("version")
                .about("Show version information")
        );
}

pub fn handle_command(matches: &ArgMatches) -> Result<()> {
    if matches.subcommand_matches("version").is_some() {
        execute()
    } else {
        Ok(())
    }
}

pub fn execute() -> Result<()> {
    println!("xdev {}", env!("CARGO_PKG_VERSION"));
    Ok(())
} 
