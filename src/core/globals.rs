//! 全局配置和常量模块
//!
//! 集中管理应用程序的全局常量、配置和状态。
//! 按功能分类组织，便于维护和扩展。

use std::sync::OnceLock;

// ============================================================================
// 国际化配置
// ============================================================================

/// 支持的语言列表
pub const SUPPORTED_LANGUAGES: &[&str] = &["zh-Hans", "en"];

/// 当前语言设置
pub static CURRENT_LANGUAGE: OnceLock<String> = OnceLock::new();

/// 默认语言
pub const DEFAULT_LANGUAGE: &str = "zh-Hans";

// ============================================================================
// 文件系统配置
// ============================================================================

/// 默认的 zdocs 路径
pub const ZDOCS_PATH: &str = "/tmp/zdocs";

/// 必需的 zdocs 文件列表
pub const REQUIRED_ZDOCS_FILES: &[&str] = &[
    "content.json",
    "meta.json",
    "numbering.json",
    "relations.json",
    "settings.json",
    "styles.json",
];

// ============================================================================
// 用户界面配置
// ============================================================================

/// 内容预览的最大行数
pub const MAX_PREVIEW_LINES: usize = 8;

/// 预览行最大显示长度
pub const MAX_PREVIEW_LINE_LENGTH: usize = 60;

/// 预览行截断后的显示长度
pub const PREVIEW_LINE_DISPLAY_LENGTH: usize = 57;

// ============================================================================
// 网络配置
// ============================================================================

/// HTTP 客户端超时时间（秒）
pub const HTTP_TIMEOUT_SECONDS: u64 = 30;

/// HTTP 用户代理
pub const HTTP_USER_AGENT: &str = "xdev/1.0";

/// 支持的 URL 协议
pub const HTTP_PROTOCOL: &str = "http://";
pub const HTTPS_PROTOCOL: &str = "https://";

// ============================================================================
// 配置管理
// ============================================================================

/// 备份文件前缀
pub const BACKUP_FILE_PREFIX: &str = "hosts_backup_";

/// 备份文件后缀
pub const BACKUP_FILE_SUFFIX: &str = ".txt";

// ============================================================================
// 应用程序配置
// ============================================================================

/// 应用程序名称
pub const APP_NAME: &str = "xdev";

/// Hosts 订阅标记
pub const HOSTS_SUBSCRIPTION_START_MARKER: &str = "# === xdev hosts subscription: ";
pub const HOSTS_SUBSCRIPTION_END_MARKER: &str = "# === 结束 xdev hosts subscription: ";
pub const HOSTS_SUBSCRIPTION_MARKER_SUFFIX: &str = " ===";

// ============================================================================
// 语言显示名称映射
// ============================================================================

/// 获取语言的显示名称
pub fn get_language_display_name(lang: &str) -> &'static str {
    match lang {
        "zh-Hans" => "简体中文",
        "en" => "English",
        _ => "Unknown",
    }
}

// ============================================================================
// 验证函数
// ============================================================================

/// 验证语言代码是否支持
pub fn validate_language(lang: &str) -> bool {
    SUPPORTED_LANGUAGES.contains(&lang)
}
