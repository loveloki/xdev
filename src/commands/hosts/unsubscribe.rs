use crate::commands::config::Config;
use crate::commands::hosts::helpers::generate_backup_filename;
use crate::commands::hosts::structure::HostsFileStructure;
use crate::core::filesystem::{FileManager, StructuredFileManager};
use crate::core::i18n::t;
use crate::core::permission::ensure_sudo_privileges;
use anyhow::Result;

/// 处理取消订阅命令
pub fn handle_unsubscribe(url: &str) -> Result<()> {
    println!("{}", t!("command.hosts.unsubscribe.starting", url = url));

    // 权限检查
    ensure_sudo_privileges()?;

    // 检查配置文件中是否存在该订阅
    let mut config = Config::load()?;
    let current_subscriptions = config.get_hosts_subscriptions();

    if !current_subscriptions.contains(&url.to_string()) {
        println!("{}", t!("command.hosts.unsubscribe.not_found", url = url));
        println!("{}", t!("command.hosts.unsubscribe.current_list"));
        if current_subscriptions.is_empty() {
            println!("{}", t!("command.hosts.unsubscribe.no_subscriptions"));
        } else {
            for (i, sub_url) in current_subscriptions.iter().enumerate() {
                println!("   {}. {}", i + 1, sub_url);
            }
        }
        return Ok(());
    }

    // 从 hosts 文件中移除订阅块
    println!("{}", t!("command.hosts.unsubscribe.removing_from_hosts"));
    let removed_from_hosts = remove_subscription(url)?;

    // 从配置文件中移除订阅
    println!("{}", t!("command.hosts.unsubscribe.updating_config"));
    let removed_from_config = config.remove_hosts_subscription(url)?;
    config.save()?;

    // 显示结果
    if removed_from_hosts && removed_from_config {
        println!("{}", t!("command.hosts.unsubscribe.success", url = url));
    } else if removed_from_config {
        println!(
            "{}",
            t!("command.hosts.unsubscribe.config_removed", url = url)
        );
        if !removed_from_hosts {
            println!("{}", t!("command.hosts.unsubscribe.hosts_not_found"));
        }
    } else {
        println!("{}", t!("command.hosts.unsubscribe.process_issue"));
    }

    // 显示剩余订阅统计
    let remaining_subscriptions = config.get_hosts_subscriptions();
    println!(
        "{}",
        t!(
            "command.hosts.unsubscribe.remaining_count",
            count = remaining_subscriptions.len()
        )
    );

    if remaining_subscriptions.is_empty() {
        println!("{}", t!("command.hosts.unsubscribe.all_cleared"));
    }

    Ok(())
}

/// 创建 hosts 文件管理器
fn create_hosts_manager() -> Result<StructuredFileManager> {
    let file_manager =
        FileManager::with_typed_backup(std::path::PathBuf::from("/etc/hosts"), "hosts")?;
    Ok(StructuredFileManager::new(file_manager))
}

/// 移除订阅
pub fn remove_subscription(url: &str) -> Result<bool> {
    let hosts_manager = create_hosts_manager()?;
    let mut structure: HostsFileStructure = hosts_manager.parse_file()?;

    let removed = structure.remove_subscription(url);
    if removed {
        let backup_filename = generate_backup_filename();
        hosts_manager.update_structure_with_backup(&structure, &backup_filename)?;
        println!("{}", t!("command.hosts.subscription_removed", url = url));
    } else {
        println!(
            "{}",
            t!("command.hosts.subscription_not_found_hosts", url = url)
        );
    }

    Ok(removed)
}
