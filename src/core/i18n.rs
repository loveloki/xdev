use crate::core::globals::{CURRENT_LANGUAGE, DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES};
use anyhow::Result;

// 重新导出 t! 宏供其他模块使用
pub use rust_i18n::t;

/// 初始化 i18n 系统
pub fn init_i18n() -> Result<()> {
    // 设置默认语言
    set_language(DEFAULT_LANGUAGE)?;
    Ok(())
}

/// 设置当前语言
pub fn set_language(lang: &str) -> Result<()> {
    if !crate::core::globals::validate_language(lang) {
        anyhow::bail!("Unsupported language: {}", lang);
    }

    // 设置 rust-i18n 的语言
    rust_i18n::set_locale(lang);

    // 更新内部语言状态
    let _ = CURRENT_LANGUAGE.set(lang.to_string());

    Ok(())
}

/// 获取支持的语言列表
pub fn get_supported_languages() -> Vec<&'static str> {
    SUPPORTED_LANGUAGES.to_vec()
}

// 重新导出 globals 模块中的函数，保持向后兼容性
pub use crate::core::globals::{get_language_display_name, validate_language};
