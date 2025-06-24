pub mod config;
pub mod install;
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
    config::register_command(&mut app);

    app
}

pub fn handle_command(matches: &ArgMatches) -> Result<()> {
    // 让各个模块自己处理命令
    version::handle_command(matches)?;
    install::handle_command(matches)?;
    config::handle_command(matches)?;

    // 如果没有匹配的子命令，显示默认信息
    if matches.subcommand().is_none() {
        println!("{}", t!("general.default_message"));
    }

    Ok(())
}
