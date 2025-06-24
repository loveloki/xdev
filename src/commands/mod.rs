pub mod version;
pub mod install;
pub mod config;

use anyhow::Result;
use clap::{Command, ArgMatches};

pub fn register_command() -> Command {
    let mut app = Command::new("xdev")
        .version("0.1.0")
        .author("xdev CLI Tool")
        .about("A development CLI tool")
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
        println!("xdev CLI tool - use --help for more information");
    }
    
    Ok(())
} 
