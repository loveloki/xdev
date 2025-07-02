// 必须在 crate root 中调用 i18n 宏
rust_i18n::i18n!("locales", fallback = "zh-Hans");

mod commands;
mod core;

use anyhow::Result;
use core::i18n::init_i18n;

fn main() -> Result<()> {
    // 初始化 i18n 系统
    init_i18n()?;

    // 提前加载配置以设置正确的语言
    let config = commands::config::Config::load()?;
    // 确保语言设置生效
    crate::core::i18n::set_language(&config.lang)?;

    let mut app = commands::register_command();
    let matches = match app.clone().try_get_matches() {
        Ok(matches) => matches,
        Err(err) => {
            // 如果是未知子命令错误，显示帮助信息
            if err.kind() == clap::error::ErrorKind::UnknownArgument
                || err.kind() == clap::error::ErrorKind::InvalidSubcommand
            {
                app.print_help()?;
                println!();
                return Ok(());
            } else {
                // 其他错误直接退出
                err.exit();
            }
        }
    };

    commands::handle_command(&mut app, &matches)?;
    Ok(())
}
