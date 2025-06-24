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
    let _ = commands::config::Config::load();

    let app = commands::register_command();
    let matches = app.get_matches();

    commands::handle_command(&matches)
}
