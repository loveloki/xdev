pub mod file;
pub mod interactive;
pub mod model;

use crate::commands::config::{file::show, interactive::set_item};
use crate::core::i18n::t;
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("config")
            .about(t!("command.config.description").to_string())
            .subcommand(
                Command::new("show").about(t!("command.config.show.description").to_string()),
            )
            .subcommand(
                Command::new("set")
                    .about(t!("command.config.set.description").to_string())
                    .arg(
                        Arg::new("field")
                            .help(t!("help.config_field").to_string())
                            .required(false)
                            .index(1),
                    )
                    .arg(
                        Arg::new("value")
                            .help(t!("help.config_value").to_string())
                            .required(false)
                            .index(2),
                    ),
            ),
    );
}

pub fn execute(config_matches: &ArgMatches) -> Result<()> {
    match config_matches.subcommand() {
        Some(("show", _)) => show(),
        Some(("set", set_matches)) => {
            let field = set_matches.get_one::<String>("field").map(|s| s.as_str());
            let value = set_matches.get_one::<String>("value").map(|s| s.as_str());
            set_item(field, value)
        }
        _ => {
            show() // 默认显示配置
        }
    }
}

// Re-export Config for external use
pub use model::Config;
