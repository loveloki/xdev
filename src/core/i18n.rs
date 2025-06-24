use anyhow::Result;
use std::sync::OnceLock;

// 重新导出 t! 宏供其他模块使用
pub use rust_i18n::t;

// 支持的语言列表
pub const SUPPORTED_LANGUAGES: &[&str] = &["zh-Hans", "en"];

// 当前语言设置
static CURRENT_LANGUAGE: OnceLock<String> = OnceLock::new();

/// 初始化 i18n 系统
pub fn init_i18n() -> Result<()> {
    // 设置默认语言
    set_language("zh-Hans")?;
    Ok(())
}

/// 设置当前语言
pub fn set_language(lang: &str) -> Result<()> {
    if !validate_language(lang) {
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

/// 验证语言代码是否支持
pub fn validate_language(lang: &str) -> bool {
    SUPPORTED_LANGUAGES.contains(&lang)
}

/// 获取语言的显示名称
pub fn get_language_display_name(lang: &str) -> &'static str {
    match lang {
        "zh-Hans" => "简体中文",
        "en" => "English",
        _ => "Unknown",
    }
}
