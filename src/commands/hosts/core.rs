use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::collections::HashMap;

use crate::commands::hosts::{
    handle_backup, handle_list, handle_restore, handle_subscribe, handle_unsubscribe, handle_update,
};
use crate::core::filesystem::FileStructure;
use crate::core::i18n::t;

/// 注册 hosts 命令及其所有子命令
pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("hosts")
            .about(t!("command.hosts.description").to_string())
            .subcommand(
                Command::new("subscribe")
                    .about(t!("command.hosts.subscribe.description").to_string())
                    .arg(
                        Arg::new("url")
                            .help(t!("help.hosts_url").to_string())
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("unsubscribe")
                    .about(t!("command.hosts.unsubscribe.description").to_string())
                    .arg(
                        Arg::new("url")
                            .help(t!("help.hosts_url").to_string())
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("list").about(t!("command.hosts.list.description").to_string()),
            )
            .subcommand(
                Command::new("update").about(t!("command.hosts.update.description").to_string()),
            )
            .subcommand(
                Command::new("backup").about(t!("command.hosts.backup.description").to_string()),
            )
            .subcommand(
                Command::new("restore")
                    .about(t!("command.hosts.restore.description").to_string())
                    .arg(
                        Arg::new("backup_file")
                            .help(t!("help.backup_file").to_string())
                            .required(false)
                            .index(1),
                    ),
            ),
    );
}

/// 执行 hosts 命令
pub fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("subscribe", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").unwrap();
            handle_subscribe(url)
        }
        Some(("unsubscribe", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").unwrap();
            handle_unsubscribe(url)
        }
        Some(("list", _)) => handle_list(),
        Some(("update", _)) => handle_update(),
        Some(("backup", _)) => handle_backup(),
        Some(("restore", sub_matches)) => {
            let backup_file = sub_matches.get_one::<String>("backup_file");
            handle_restore(backup_file.map(|s| s.as_str()))
        }
        _ => {
            println!("{}", t!("command.hosts.help_message"));
            Ok(())
        }
    }
}

/// Hosts 文件结构
#[derive(Debug, Clone)]
pub struct HostsFileStructure {
    pub other_content: Vec<String>,
    pub subscription_blocks: HashMap<String, Vec<String>>,
}

impl FileStructure for HostsFileStructure {
    fn parse(content: &str) -> Self {
        let mut structure = HostsFileStructure::new();
        let mut current_block: Option<String> = None;
        let mut current_block_content = Vec::new();

        for line in content.lines() {
            let line = line.to_string();

            // 检查是否是订阅块的开始
            if let Some(url) = extract_subscription_url_from_start_marker(&line) {
                // 如果之前有未结束的块，保存到 other_content
                if let Some(block_url) = current_block.take() {
                    structure
                        .subscription_blocks
                        .insert(block_url, current_block_content.clone());
                    current_block_content.clear();
                }
                current_block = Some(url);
                current_block_content.push(line);
                continue;
            }

            // 检查是否是订阅块的结束
            if is_subscription_end_marker(&line) {
                if let Some(block_url) = current_block.take() {
                    current_block_content.push(line);
                    structure
                        .subscription_blocks
                        .insert(block_url, current_block_content.clone());
                    current_block_content.clear();
                } else {
                    // 没有匹配的开始标记，当作普通内容
                    structure.other_content.push(line);
                }
                continue;
            }

            // 处理普通行
            if current_block.is_some() {
                current_block_content.push(line);
            } else {
                structure.other_content.push(line);
            }
        }

        // 处理未结束的块
        if let Some(block_url) = current_block {
            structure
                .subscription_blocks
                .insert(block_url, current_block_content);
        }

        structure
    }

    fn reconstruct(&self) -> String {
        let mut content = Vec::new();

        // 添加其他内容
        content.extend(self.other_content.iter().cloned());

        // 添加所有订阅块
        for block_lines in self.subscription_blocks.values() {
            if !content.is_empty() && !content.last().unwrap().is_empty() {
                content.push(String::new()); // 添加空行分隔
            }
            content.extend(block_lines.iter().cloned());
        }

        content.join("\n")
    }
}

impl HostsFileStructure {
    pub fn new() -> Self {
        Self {
            other_content: Vec::new(),
            subscription_blocks: HashMap::new(),
        }
    }

    /// 添加或更新订阅块
    pub fn add_or_update_subscription(&mut self, url: &str, content: &str) {
        let block_content = create_subscription_block(url, content);
        let block_lines: Vec<String> = block_content.lines().map(|s| s.to_string()).collect();
        self.subscription_blocks
            .insert(url.to_string(), block_lines);
    }

    /// 删除订阅块
    pub fn remove_subscription(&mut self, url: &str) -> bool {
        self.subscription_blocks.remove(url).is_some()
    }

    /// 获取所有订阅的 URL
    pub fn get_all_subscriptions(&self) -> Vec<String> {
        self.subscription_blocks.keys().cloned().collect()
    }
}

/// 从开始标记中提取订阅 URL
fn extract_subscription_url_from_start_marker(line: &str) -> Option<String> {
    if line.starts_with("# === xdev hosts subscription: ") && line.ends_with(" ===") {
        let start = "# === xdev hosts subscription: ".len();
        let end = line.len() - " ===".len();
        Some(line[start..end].to_string())
    } else {
        None
    }
}

/// 检查是否是订阅结束标记
fn is_subscription_end_marker(line: &str) -> bool {
    line.starts_with("# === 结束 xdev hosts subscription: ") && line.ends_with(" ===")
}

/// 创建订阅块内容
fn create_subscription_block(url: &str, hosts_content: &str) -> String {
    let mut block = Vec::new();

    // 开始标记
    block.push(format!("# === xdev hosts subscription: {url} ==="));

    // 使用标准库格式化时间
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    block.push(format!("# 订阅时间: {timestamp} (UTC timestamp)"));
    block.push(String::new());

    // 添加 hosts 内容（过滤掉空行和注释）
    for line in hosts_content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            block.push(line.to_string());
        }
    }

    block.push(String::new());
    block.push(format!("# === 结束 xdev hosts subscription: {url} ==="));

    block.join("\n")
}
