use crate::commands::config::Config;
use crate::commands::hosts::{core::HostsFileStructure, create_hosts_manager};
use crate::core::i18n::t;
use anyhow::Result;

/// 处理列表命令
pub fn handle_list() -> Result<()> {
    println!("{}", t!("command.hosts.list.title"));
    println!();

    // 从配置文件获取订阅列表
    let config = Config::load()?;
    let subscriptions = config.get_hosts_subscriptions();

    if subscriptions.is_empty() {
        println!("{}", t!("command.hosts.list.empty"));
        println!("{}", t!("command.hosts.list.empty_hint"));
        return Ok(());
    }

    // 获取 hosts 文件中的实际订阅状态
    let hosts_subscriptions = match get_all_subscriptions() {
        Ok(subs) => subs,
        Err(e) => {
            println!("{}", t!("command.hosts.list.read_error", error = e));
            Vec::new()
        }
    };

    // 显示表格头部
    println!("{}", t!("command.hosts.table.border_top"));
    println!("{}", t!("command.hosts.table.header"));
    println!("{}", t!("command.hosts.table.separator"));

    // 显示每个订阅
    for (index, url) in subscriptions.iter().enumerate() {
        let status = if hosts_subscriptions.contains(url) {
            t!("command.hosts.list.status_applied")
        } else {
            t!("command.hosts.list.status_not_synced")
        };

        // 截取过长的 URL
        let display_url = if url.len() > 50 {
            format!("{}...", &url[..47])
        } else {
            url.clone()
        };

        println!("│ {:3} │ {:<51} │ {:<10} │", index + 1, display_url, status);
    }

    println!("{}", t!("command.hosts.table.border_bottom"));
    println!();

    // 显示统计信息
    let total_count = subscriptions.len();
    let active_count = subscriptions
        .iter()
        .filter(|url| hosts_subscriptions.contains(url))
        .count();
    let inactive_count = total_count - active_count;

    println!("{}", t!("command.hosts.list.statistics_title"));
    println!(
        "{}",
        t!("command.hosts.list.total_count", count = total_count)
    );
    println!(
        "{}",
        t!("command.hosts.list.applied_count", count = active_count)
    );
    if inactive_count > 0 {
        println!(
            "{}",
            t!(
                "command.hosts.list.not_synced_count",
                count = inactive_count
            )
        );
        println!();
        println!("{}", t!("command.hosts.list.update_suggestion"));
    }

    // 如果有未同步的订阅，显示详细信息
    if inactive_count > 0 {
        println!();
        println!("{}", t!("command.hosts.list.not_synced_list"));
        for url in &subscriptions {
            if !hosts_subscriptions.contains(url) {
                println!("   • {url}");
            }
        }
    }

    Ok(())
}

/// 获取所有订阅
pub fn get_all_subscriptions() -> Result<Vec<String>> {
    let hosts_manager = create_hosts_manager()?;
    let structure: HostsFileStructure = hosts_manager.parse_file()?;
    Ok(structure.get_all_subscriptions())
}
