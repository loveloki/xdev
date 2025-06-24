mod commands;

use clap::{Command, Arg};
use std::process;

fn main() {
    let matches = Command::new("xdev")
        .version("0.1.0")
        .author("xdev CLI Tool")
        .about("A development CLI tool")
        .disable_version_flag(true) // 禁用自动生成的--version参数
        .subcommand(
            Command::new("version")
                .about("Show version information")
        )
        .subcommand(
            Command::new("install")
                .about("Install xdev binary to system")
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall xdev binary from system")
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration")
                .subcommand(
                    Command::new("show")
                        .about("Show current configuration")
                )
                .subcommand(
                    Command::new("set")
                        .about("Set configuration values")
                        .arg(
                            Arg::new("field")
                                .help("Configuration field to set")
                                .required(false)
                                .index(1)
                        )
                        .arg(
                            Arg::new("value")
                                .help("Value to set")
                                .required(false)
                                .index(2)
                        )
                )
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("version", _)) => commands::handle_version(),
        Some(("install", _)) => commands::handle_install(),
        Some(("uninstall", _)) => commands::handle_uninstall(),
        Some(("config", config_matches)) => {
            match config_matches.subcommand() {
                Some(("show", _)) => commands::handle_config_show(),
                Some(("set", set_matches)) => {
                    let field = set_matches.get_one::<String>("field").map(|s| s.as_str());
                    let value = set_matches.get_one::<String>("value").map(|s| s.as_str());
                    commands::handle_config_set(field, value)
                }
                _ => {
                    commands::handle_config_show() // 默认显示配置
                }
            }
        }
        _ => {
            println!("xdev CLI tool - use --help for more information");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
