mod commands;

use clap::Command;
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
        .get_matches();

    let result = match matches.subcommand() {
        Some(("version", _)) => commands::handle_version(),
        Some(("install", _)) => commands::handle_install(),
        Some(("uninstall", _)) => commands::handle_uninstall(),
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
