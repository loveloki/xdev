use crate::commands::config::Config;
use crate::commands::hosts::helpers::{
    display_update_summary, generate_backup_filename, validate_hosts_content,
};
use crate::commands::hosts::structure::HostsFileStructure;
use crate::core::filesystem::{FileManager, StructuredFileManager};
use crate::core::http::HttpClient;
use crate::core::i18n::t;
use crate::core::permission::ensure_sudo_privileges;
use crate::core::validation::validate_url;
use anyhow::Result;

/// 处理更新命令
pub fn handle_update() -> Result<()> {
    println!("{}", t!("command.hosts.update.starting"));

    // 权限检查
    ensure_sudo_privileges()?;

    // 获取所有订阅
    let config = Config::load()?;
    let subscriptions = config.get_hosts_subscriptions();

    if subscriptions.is_empty() {
        println!("{}", t!("command.hosts.update.empty"));
        println!("{}", t!("command.hosts.update.empty_hint"));
        return Ok(());
    }

    println!(
        "{}",
        t!(
            "command.hosts.update.found_count",
            count = subscriptions.len()
        )
    );
    println!();

    let mut success_count = 0;
    let mut failed_urls = Vec::new();

    // 逐个更新订阅
    for (index, url) in subscriptions.iter().enumerate() {
        println!(
            "{}",
            t!(
                "command.hosts.update.progress",
                current = index + 1,
                total = subscriptions.len(),
                url = url
            )
        );

        match update_single_subscription(url) {
            Ok(()) => {
                success_count += 1;
                println!(
                    "{}",
                    t!(
                        "command.hosts.update.success_item",
                        current = index + 1,
                        total = subscriptions.len()
                    )
                );
            }
            Err(e) => {
                failed_urls.push(url.clone());
                println!(
                    "{}",
                    t!(
                        "command.hosts.update.failed_item",
                        current = index + 1,
                        total = subscriptions.len(),
                        error = e
                    )
                );
            }
        }

        // 添加短暂延迟，避免过于频繁的请求
        if index < subscriptions.len() - 1 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        println!();
    }

    // 显示更新结果摘要
    display_update_summary(success_count, &failed_urls, subscriptions.len());

    Ok(())
}

/// 更新单个订阅
pub fn update_single_subscription(url: &str) -> Result<()> {
    // URL 验证
    validate_url(url)?;

    // 下载内容
    let content = download_and_validate_hosts(url)?;

    // 更新 hosts 文件
    add_or_update_subscription(url, &content)?;

    Ok(())
}

/// 下载并验证 hosts 列表
fn download_and_validate_hosts(url: &str) -> Result<String> {
    let http_client = HttpClient::new()?;

    // 先测试 URL 可达性
    http_client.test_url_accessibility(url)?;

    // 下载内容
    let content = http_client.download(url)?;

    // 验证内容格式
    validate_hosts_content(&content)?;

    Ok(content)
}

/// 创建 hosts 文件管理器
fn create_hosts_manager() -> Result<StructuredFileManager> {
    let file_manager =
        FileManager::with_typed_backup(std::path::PathBuf::from("/etc/hosts"), "hosts")?;
    Ok(StructuredFileManager::new(file_manager))
}

/// 添加或更新订阅
fn add_or_update_subscription(url: &str, content: &str) -> Result<()> {
    let hosts_manager = create_hosts_manager()?;
    let mut structure: HostsFileStructure = hosts_manager.parse_file()?;

    structure.add_or_update_subscription(url, content);
    let backup_filename = generate_backup_filename();
    hosts_manager.update_structure_with_backup(&structure, &backup_filename)?;

    println!("{}", t!("command.hosts.hosts_file_updated"));
    Ok(())
}
