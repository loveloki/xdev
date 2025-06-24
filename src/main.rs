use clap::Command;

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
        .get_matches();

    match matches.subcommand() {
        Some(("version", _)) => {
            println!("xdev {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            println!("xdev CLI tool - use --help for more information");
        }
    }
}
