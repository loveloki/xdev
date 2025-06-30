pub mod config;
pub mod dns;
pub mod draft;
pub mod hosts;
pub mod install;
pub mod uninstall;
pub mod version;

use crate::core::i18n::t;
use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn register_command() -> Command {
    let mut app = Command::new("xdev")
        .version("0.1.0")
        .author("xleine@qq.com")
        .about(t!("general.app_description").to_string())
        .disable_version_flag(true); // 禁用自动生成的--version参数

    // 各模块注册自己的命令
    version::register_command(&mut app);
    install::register_command(&mut app);
    uninstall::register_command(&mut app);
    config::register_command(&mut app);
    draft::register_command(&mut app);
    dns::register_command(&mut app);
    hosts::register_command(&mut app);

    app
}

pub fn handle_command(app: &mut Command, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("version", _)) => version::execute(),
        Some(("install", _)) => install::execute(),
        Some(("uninstall", _)) => uninstall::execute(),
        Some(("config", config_matches)) => config::execute(config_matches),
        Some(("draft", _)) => draft::execute(),
        Some(("dns", dns_matches)) => dns::execute(dns_matches),
        Some(("hosts", hosts_matches)) => hosts::execute(hosts_matches),
        _ => {
            // 如果没有匹配的子命令，显示帮助信息
            app.print_help()?;
            println!(); // 添加一个换行
            Ok(())
        }
    }
}
