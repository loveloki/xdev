use crate::commands::config::Config;
use crate::commands::hosts::helpers::{
    generate_backup_filename, print_content_preview, validate_hosts_content,
};
use crate::commands::hosts::structure::HostsFileStructure;
use crate::core::filesystem::{FileManager, StructuredFileManager};
use crate::core::http::HttpClient;
use crate::core::i18n::t;
use crate::core::permission::ensure_sudo_privileges;
use crate::core::validation::validate_url;
use anyhow::Result;

/// 处理订阅命令
pub fn handle_subscribe(url: &str) -> Result<()> {
    println!("{}", t!("command.hosts.subscribe.starting", url = url));

    // 权限检查
    ensure_sudo_privileges()?;

    // URL 验证
    validate_url(url)?;

    // 检查是否已经订阅过
    let mut config = Config::load()?;
    let current_subscriptions = config.get_hosts_subscriptions();
    if current_subscriptions.contains(&url.to_string()) {
        println!(
            "{}",
            t!("command.hosts.subscribe.already_exists", url = url)
        );
    }

    // 下载并验证内容
    println!("{}", t!("command.hosts.subscribe.downloading"));
    let content = download_and_validate_hosts(url)?;

    // 显示内容预览
    print_content_preview(&content);

    // 更新 hosts 文件（会自动备份）
    println!("{}", t!("command.hosts.subscribe.updating_hosts"));

    // 在执行危险操作前先保存当前配置状态
    let _original_subscriptions = config.get_hosts_subscriptions();

    match add_or_update_subscription(url, &content) {
        Ok(()) => {
            // hosts 文件更新成功，继续更新配置
            println!("{}", t!("command.hosts.subscribe.updating_config"));
            match config.add_hosts_subscription(url) {
                Ok(added) => {
                    match config.save() {
                        Ok(()) => {
                            // 配置保存成功，操作完成
                            if added {
                                println!("{}", t!("command.hosts.subscribe.success", url = url));
                            } else {
                                println!("{}", t!("command.hosts.subscribe.updated", url = url));
                            }

                            // 显示订阅统计
                            let subscriptions = config.get_hosts_subscriptions();
                            println!(
                                "{}",
                                t!(
                                    "command.hosts.subscribe.statistics",
                                    count = subscriptions.len()
                                )
                            );
                        }
                        Err(config_err) => {
                            // 配置保存失败，但 hosts 文件已经更新
                            println!(
                                "{}",
                                t!("command.hosts.config_save_failed", error = config_err)
                            );
                            println!("{}", t!("command.hosts.config_sync_warning"));

                            // 尝试从最新备份恢复 hosts 文件
                            println!("{}", t!("command.hosts.rollback_attempt"));
                            if let Err(restore_err) =
                                crate::commands::hosts::backup::attempt_hosts_rollback()
                            {
                                println!(
                                    "{}",
                                    t!("command.hosts.auto_rollback_failed", error = restore_err)
                                );
                                println!("{}", t!("command.hosts.system_inconsistent_check"));
                            } else {
                                println!("{}", t!("command.hosts.rollback_success"));
                            }
                            return Err(config_err);
                        }
                    }
                }
                Err(config_err) => {
                    // 配置更新失败，需要回滚 hosts 文件
                    println!(
                        "{}",
                        t!("command.hosts.config_update_failed", error = config_err)
                    );
                    println!("{}", t!("command.hosts.rollback_in_progress"));

                    if let Err(rollback_err) =
                        crate::commands::hosts::backup::attempt_hosts_rollback()
                    {
                        println!(
                            "{}",
                            t!("command.hosts.auto_rollback_failed", error = rollback_err)
                        );
                        println!("{}", t!("command.hosts.system_inconsistent"));
                        println!("{}", t!("command.hosts.manual_restore_suggestion"));
                    } else {
                        println!("{}", t!("command.hosts.rollback_success_complete"));
                    }
                    return Err(config_err);
                }
            }
        }
        Err(hosts_err) => {
            // hosts 文件更新失败
            println!(
                "{}",
                t!("command.hosts.hosts_file_update_failed", error = hosts_err)
            );
            return Err(hosts_err);
        }
    }

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
pub fn add_or_update_subscription(url: &str, content: &str) -> Result<()> {
    let hosts_manager = create_hosts_manager()?;
    let mut structure: HostsFileStructure = hosts_manager.parse_file()?;

    structure.add_or_update_subscription(url, content);
    let backup_filename = generate_backup_filename();
    hosts_manager.update_structure_with_backup(&structure, &backup_filename)?;

    println!("{}", t!("command.hosts.hosts_file_updated"));
    Ok(())
}
