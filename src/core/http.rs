//! HTTP 客户端和下载功能模块
//!
//! 提供通用的 HTTP 下载和网络请求功能，
//! 可被多个命令模块复用。

use crate::core::i18n::t;
use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

/// HTTP 客户端配置和下载功能
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30)) // 30秒超时
            .user_agent("xdev/1.0")
            .build()
            .map_err(|e| anyhow::anyhow!("{}", t!("error.http_client_failed", error = e)))?;

        Ok(Self { client })
    }

    /// 下载内容
    pub fn download(&self, url: &str) -> Result<String> {
        println!("{}", t!("command.hosts.download.downloading", url = url));

        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_download_failed", error = e)))?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            anyhow::bail!(
                "{}",
                t!(
                    "error.hosts_download_http_error",
                    status = response.status()
                )
            );
        }

        // 获取响应内容
        let content = response
            .text()
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_download_failed", error = e)))?;

        println!(
            "{}",
            t!("command.hosts.download.content_size", size = content.len())
        );

        Ok(content)
    }

    /// 测试 URL 的可达性
    pub fn test_url_accessibility(&self, url: &str) -> Result<()> {
        let response = self
            .client
            .head(url) // 使用 HEAD 请求只获取头部信息
            .send()
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_connection_failed", error = e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!(
                "{}",
                t!("error.hosts_url_not_accessible", status = response.status())
            )
        }
    }
}
