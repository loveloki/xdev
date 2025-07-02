//! 验证功能模块
//!
//! 提供通用的 URL、IP 地址和其他数据验证功能，
//! 可被多个命令模块复用。

use crate::core::globals::{HTTP_PROTOCOL, HTTPS_PROTOCOL};
use crate::core::i18n::t;
use anyhow::Result;

/// 验证 URL 格式和协议
pub fn validate_url(url: &str) -> Result<()> {
    // 检查 URL 是否为空
    if url.trim().is_empty() {
        anyhow::bail!("{}", t!("error.hosts_url_empty"));
    }

    // 简单的 URL 格式检查
    let url = url.trim();

    // 检查协议
    if !url.starts_with(HTTP_PROTOCOL) && !url.starts_with(HTTPS_PROTOCOL) {
        anyhow::bail!("{}", t!("error.hosts_url_invalid_protocol"));
    }

    // 提取域名部分进行验证
    let without_protocol = if let Some(stripped) = url.strip_prefix(HTTPS_PROTOCOL) {
        stripped
    } else if let Some(stripped) = url.strip_prefix(HTTP_PROTOCOL) {
        stripped
    } else {
        anyhow::bail!("{}", t!("error.hosts_url_invalid_protocol"));
    };

    // 检查是否有域名部分
    if without_protocol.is_empty() {
        anyhow::bail!("{}", t!("error.hosts_url_missing_domain"));
    }

    // 提取域名（去掉路径部分）
    let domain_part = without_protocol.split('/').next().unwrap_or("");

    // 去掉端口号
    let domain = domain_part.split(':').next().unwrap_or("");

    if domain.is_empty() {
        anyhow::bail!("{}", t!("error.hosts_url_invalid_domain"));
    }

    // 检查域名格式（简单验证）
    if !is_valid_domain_simple(domain) {
        anyhow::bail!(
            "{}",
            t!("error.hosts_url_invalid_domain_format", domain = domain)
        );
    }

    Ok(())
}

/// 简单的域名格式验证（内部使用）
fn is_valid_domain_simple(domain: &str) -> bool {
    // 检查域名长度
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    // 检查是否包含有效字符
    let valid_chars = domain
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '.');

    if !valid_chars {
        return false;
    }

    // 检查是否以点开始或结束
    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }

    // 检查是否包含连续的点
    if domain.contains("..") {
        return false;
    }

    // 检查每个标签的长度（标签是点之间的部分）
    let labels: Vec<&str> = domain.split('.').collect();
    for label in labels {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
    }

    true
}
