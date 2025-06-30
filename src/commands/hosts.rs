use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use nix::unistd::geteuid;
use reqwest::blocking::Client;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::commands::config::Config;
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
                            .help("hosts 列表的 URL")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("unsubscribe")
                    .about(t!("command.hosts.unsubscribe.description").to_string())
                    .arg(
                        Arg::new("url")
                            .help("要取消订阅的 URL")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("list")
                    .about(t!("command.hosts.list.description").to_string()),
            )
            .subcommand(
                Command::new("update")
                    .about(t!("command.hosts.update.description").to_string()),
            )
            .subcommand(
                Command::new("backup")
                    .about(t!("command.hosts.backup.description").to_string()),
            )
            .subcommand(
                Command::new("restore")
                    .about(t!("command.hosts.restore.description").to_string())
                    .arg(
                        Arg::new("backup_file")
                            .help("备份文件路径（可选）")
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

/// 处理订阅命令
fn handle_subscribe(url: &str) -> Result<()> {
    println!("{}", t!("command.hosts.subscribe.starting", url = url));
    
    // 权限检查
    ensure_root_privileges()?;
    
    // URL 验证
    validate_url(url)?;
    
    // 检查是否已经订阅过
    let mut config = Config::load()?;
    let current_subscriptions = config.get_hosts_subscriptions();
    if current_subscriptions.contains(&url.to_string()) {
        println!("{}", t!("command.hosts.subscribe.already_exists", url = url));
    }
    
    // 下载并验证内容
    println!("{}", t!("command.hosts.subscribe.downloading"));
    let content = download_and_validate_hosts(url)?;
    
    // 显示内容预览
    print_content_preview(&content);
    
    // 更新 hosts 文件（会自动备份）
    println!("{}", t!("command.hosts.subscribe.updating_hosts"));
    let hosts_manager = HostsFileManager::new()?;
    
    // 在执行危险操作前先保存当前配置状态
    let _original_subscriptions = config.get_hosts_subscriptions();
    
    match hosts_manager.add_or_update_subscription(url, &content) {
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
                             println!("{}", t!("command.hosts.subscribe.statistics", count = subscriptions.len()));
                        }
                        Err(config_err) => {
                            // 配置保存失败，但 hosts 文件已经更新
                            println!("{}", t!("command.hosts.config_save_failed", error = config_err));
                            println!("{}", t!("command.hosts.config_sync_warning"));
                            
                            // 尝试从最新备份恢复 hosts 文件
                            println!("{}", t!("command.hosts.rollback_attempt"));
                            if let Err(restore_err) = attempt_hosts_rollback(&hosts_manager) {
                                println!("{}", t!("command.hosts.auto_rollback_failed", error = restore_err));
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
                    println!("{}", t!("command.hosts.config_update_failed", error = config_err));
                    println!("{}", t!("command.hosts.rollback_in_progress"));
                    
                    if let Err(rollback_err) = attempt_hosts_rollback(&hosts_manager) {
                        println!("{}", t!("command.hosts.auto_rollback_failed", error = rollback_err));
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
            println!("{}", t!("command.hosts.hosts_file_update_failed", error = hosts_err));
            return Err(hosts_err);
        }
         }
    
    Ok(())
}

/// 处理取消订阅命令
fn handle_unsubscribe(url: &str) -> Result<()> {
    println!("{}", t!("command.hosts.unsubscribe.starting", url = url));
    
    // 权限检查
    ensure_root_privileges()?;
    
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
    let hosts_manager = HostsFileManager::new()?;
    let removed_from_hosts = hosts_manager.remove_subscription(url)?;
    
    // 从配置文件中移除订阅
    println!("{}", t!("command.hosts.unsubscribe.updating_config"));
    let removed_from_config = config.remove_hosts_subscription(url)?;
    config.save()?;
    
    // 显示结果
    if removed_from_hosts && removed_from_config {
        println!("{}", t!("command.hosts.unsubscribe.success", url = url));
    } else if removed_from_config {
        println!("{}", t!("command.hosts.unsubscribe.config_removed", url = url));
        if !removed_from_hosts {
            println!("{}", t!("command.hosts.unsubscribe.hosts_not_found"));
        }
    } else {
        println!("{}", t!("command.hosts.unsubscribe.process_issue"));
    }
    
    // 显示剩余订阅统计
    let remaining_subscriptions = config.get_hosts_subscriptions();
    println!("{}", t!("command.hosts.unsubscribe.remaining_count", count = remaining_subscriptions.len()));
    
    if remaining_subscriptions.is_empty() {
        println!("{}", t!("command.hosts.unsubscribe.all_cleared"));
    }
    
    Ok(())
}

/// 处理列表命令
fn handle_list() -> Result<()> {
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
    let hosts_manager = HostsFileManager::new()?;
    let hosts_subscriptions = match hosts_manager.get_all_subscriptions() {
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
        
        println!("│ {:3} │ {:<51} │ {:<10} │", 
                 index + 1, 
                 display_url, 
                 status);
    }
    
    println!("{}", t!("command.hosts.table.border_bottom"));
    println!();
    
    // 显示统计信息
    let total_count = subscriptions.len();
    let active_count = subscriptions.iter()
        .filter(|url| hosts_subscriptions.contains(url))
        .count();
    let inactive_count = total_count - active_count;
    
    println!("{}", t!("command.hosts.list.statistics_title"));
    println!("{}", t!("command.hosts.list.total_count", count = total_count));
    println!("{}", t!("command.hosts.list.applied_count", count = active_count));
    if inactive_count > 0 {
        println!("{}", t!("command.hosts.list.not_synced_count", count = inactive_count));
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

/// 处理更新命令
fn handle_update() -> Result<()> {
    println!("{}", t!("command.hosts.update.starting"));
    
    // 权限检查
    ensure_root_privileges()?;
    
    // 获取所有订阅
    let config = Config::load()?;
    let subscriptions = config.get_hosts_subscriptions();
    
    if subscriptions.is_empty() {
        println!("{}", t!("command.hosts.update.empty"));
        println!("{}", t!("command.hosts.update.empty_hint"));
        return Ok(());
    }
    
    println!("{}", t!("command.hosts.update.found_count", count = subscriptions.len()));
    println!();
    
    let hosts_manager = HostsFileManager::new()?;
    let mut success_count = 0;
    let mut failed_urls = Vec::new();
    
    // 逐个更新订阅
    for (index, url) in subscriptions.iter().enumerate() {
        println!("{}", t!("command.hosts.update.progress", current = index + 1, total = subscriptions.len(), url = url));
        
        match update_single_subscription(url, &hosts_manager) {
            Ok(()) => {
                success_count += 1;
                println!("{}", t!("command.hosts.update.success_item", current = index + 1, total = subscriptions.len()));
            }
            Err(e) => {
                failed_urls.push(url.clone());
                println!("{}", t!("command.hosts.update.failed_item", current = index + 1, total = subscriptions.len(), error = e));
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

/// 处理备份命令
fn handle_backup() -> Result<()> {
    println!("{}", t!("command.hosts.backup.starting"));
    
    // 权限检查（读取文件需要权限）
    let hosts_manager = HostsFileManager::new()?;
    
    // 执行备份
    match hosts_manager.backup_hosts_file() {
        Ok(backup_path) => {
            println!("{}", t!("command.hosts.backup.success"));
            println!("{}", t!("command.hosts.backup.file_path", path = backup_path.display()));
            
            // 显示备份文件大小
            if let Ok(metadata) = std::fs::metadata(&backup_path) {
                println!("{}", t!("command.hosts.backup.file_size", size = metadata.len()));
            }
            
            // 显示所有备份文件
            println!();
            display_available_backups(&hosts_manager)?;
        }
        Err(e) => {
            println!("{}", t!("command.hosts.backup.failed", error = e));
            return Err(e);
        }
    }
    
    Ok(())
}

/// 处理恢复命令
fn handle_restore(backup_file: Option<&str>) -> Result<()> {
    println!("{}", t!("command.hosts.restore.starting"));
    
    // 权限检查
    ensure_root_privileges()?;
    
    let hosts_manager = HostsFileManager::new()?;
    
    match backup_file {
        Some(file_path) => {
            // 使用指定的备份文件
            println!("{}", t!("command.hosts.restore.from_specified", path = file_path));
            
            // 检查文件是否存在
            if !std::path::Path::new(file_path).exists() {
                anyhow::bail!("{}", t!("error.hosts_backup_file_not_exist", path = file_path));
            }
            
            // 显示文件信息
            if let Ok(metadata) = std::fs::metadata(file_path) {
                println!("{}", t!("command.hosts.restore.file_size", size = metadata.len()));
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                        println!("{}", t!("command.hosts.restore.backup_time", timestamp = duration.as_secs()));
                    }
                }
            }
            
            // 执行恢复
            hosts_manager.restore_from_backup(Some(file_path))?;
        }
        None => {
            // 使用最新的备份文件
            println!("{}", t!("command.hosts.restore.finding_latest"));
            
            // 显示可用备份
            display_available_backups(&hosts_manager)?;
            
            // 恢复最新备份
            println!("{}", t!("command.hosts.restore.restoring_latest"));
            hosts_manager.restore_from_backup(None)?;
        }
    }
    
    println!("{}", t!("command.hosts.restore.success"));
    println!("{}", t!("command.hosts.restore.check_suggestion"));
    
    Ok(())
}

// ==================== 权限和安全检查 ====================

/// 确保当前用户具有 root 权限
fn ensure_root_privileges() -> Result<()> {
    if !geteuid().is_root() {
        anyhow::bail!(t!("error.hosts_root_required"));
    }
    Ok(())
}

/// 验证 URL 格式和安全性
fn validate_url(url: &str) -> Result<()> {
    // 基本 URL 格式检查
    if !url.starts_with("http://") && !url.starts_with("https://") {
        anyhow::bail!(t!("error.hosts_url_invalid_protocol"));
    }
    
    // 安全性：建议使用 HTTPS
    if url.starts_with("http://") && !url.starts_with("http://localhost") && !url.starts_with("http://127.0.0.1") {
        println!("{}", t!("command.hosts.http_warning"));
    }
    
    // 检查 URL 长度
    if url.len() > 1024 {
        anyhow::bail!(t!("error.hosts_url_too_long"));
    }
    
    // 基本的恶意 URL 检查
    if url.contains("..") || url.contains("file://") || url.contains("ftp://") {
        anyhow::bail!(t!("error.hosts_url_unsafe"));
    }
    
    Ok(())
}



/// 简单的 IP 地址验证
fn is_valid_ip(ip: &str) -> bool {
    // IPv4 简单验证
    if ip.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }
    
    // IPv6 简单验证
    if ip.parse::<std::net::Ipv6Addr>().is_ok() {
        return true;
    }
    
    false
}

// ==================== HTTP 客户端和下载功能 ====================

/// HTTP 客户端配置
struct HttpDownloader {
    client: Client,
}

impl HttpDownloader {
    /// 创建新的 HTTP 下载器
    fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))  // 30秒超时
            .user_agent("xdev-hosts-manager/1.0")
            .build()
            .map_err(|e| anyhow::anyhow!("创建 HTTP 客户端失败: {}", e))?;
        
        Ok(Self { client })
    }
    
    /// 下载 hosts 列表内容
    fn download_hosts_list(&self, url: &str) -> Result<String> {
        println!("{}", t!("command.hosts.download.downloading", url = url));
        
        let response = self.client
            .get(url)
            .send()
            .map_err(|e| anyhow::anyhow!("下载请求失败: {}", e))?;
        
        // 检查 HTTP 状态码
        if !response.status().is_success() {
            anyhow::bail!("下载失败，HTTP 状态码: {}", response.status());
        }
        
        // 检查 Content-Type
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            if !content_type_str.starts_with("text/") && !content_type_str.starts_with("application/") {
                println!("{}", t!("command.hosts.download.content_type_warning", content_type = content_type_str));
            }
        }
        
        // 检查内容大小
        if let Some(content_length) = response.content_length() {
            if content_length > 10 * 1024 * 1024 { // 10MB 限制
                anyhow::bail!("{}", t!("error.hosts_download_too_large", size = content_length));
            }
            println!("{}", t!("command.hosts.download.content_size", size = content_length));
        }
        
        // 获取响应内容
        let content = response
            .text()
            .map_err(|e| anyhow::anyhow!("读取响应内容失败: {}", e))?;
        
        // 检查内容大小（防止无限制下载）
        if content.len() > 10 * 1024 * 1024 {
            anyhow::bail!("{}", t!("error.hosts_download_too_large", size = content.len()));
        }
        
        // 验证下载的内容
        self.validate_downloaded_content(&content)?;
        
        println!("{}", t!("command.hosts.download.download_complete", size = content.len()));
        Ok(content)
    }
    
    /// 验证下载的内容是否为有效的 hosts 格式
    fn validate_downloaded_content(&self, content: &str) -> Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            anyhow::bail!("下载的内容为空");
        }
        
        let mut valid_lines = 0;
        let mut total_non_empty_lines = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            total_non_empty_lines += 1;
            
            // 检查是否为有效的 hosts 格式
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && is_valid_ip(parts[0]) {
                valid_lines += 1;
            } else {
                // 允许一些无效行，但不能太多
                if line_num < 10 {  // 只打印前10个错误
                    println!("{}", t!("command.hosts.download.validation_warning_line", line_num = line_num + 1, line = line));
                }
            }
        }
        
        if total_non_empty_lines == 0 {
            anyhow::bail!("下载的内容没有有效的 hosts 条目");
        }
        
        // 如果有效行数少于总行数的 50%，可能不是 hosts 文件
        if valid_lines < total_non_empty_lines / 2 {
            println!("{}", t!("command.hosts.download.validation_warning_format"));
            println!("{}", t!("command.hosts.download.validation_line_stats", valid = valid_lines, total = total_non_empty_lines));
        } else {
            println!("{}", t!("command.hosts.download.validation_success", valid = valid_lines, total = total_non_empty_lines));
        }
        
        Ok(())
    }
    
    /// 测试 URL 的可达性
    fn test_url_accessibility(&self, url: &str) -> Result<()> {
        println!("{}", t!("command.hosts.download.url_accessibility_test", url = url));
        
        let response = self.client
            .head(url)  // 使用 HEAD 请求只获取头部信息
            .send()
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_connection_failed", error = e)))?;
        
        if response.status().is_success() {
            println!("{}", t!("command.hosts.download.url_accessible"));
            Ok(())
        } else {
            anyhow::bail!("URL 不可访问，HTTP 状态码: {}", response.status())
        }
    }
}

/// 下载并验证 hosts 列表
fn download_and_validate_hosts(url: &str) -> Result<String> {
    let downloader = HttpDownloader::new()?;
    
    // 先测试 URL 可达性
    downloader.test_url_accessibility(url)?;
    
    // 下载内容
    let content = downloader.download_hosts_list(url)?;
    
    Ok(content)
}

// ==================== Hosts 文件管理 ====================

/// Hosts 文件管理器
struct HostsFileManager {
    hosts_path: PathBuf,
    pub backup_dir: PathBuf,
}

impl HostsFileManager {
    /// 创建新的 hosts 文件管理器
    fn new() -> Result<Self> {
        let hosts_path = PathBuf::from("/etc/hosts");
        
        // 创建备份目录
        let backup_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("xdev").join("hosts_backups")
        } else {
            PathBuf::from("/tmp/xdev_hosts_backups")
        };
        
        // 确保备份目录存在
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            hosts_path,
            backup_dir,
        })
    }
    
    /// 读取 hosts 文件内容
    fn read_hosts_file(&self) -> Result<String> {
        fs::read_to_string(&self.hosts_path)
            .map_err(|e| anyhow::anyhow!("无法读取 hosts 文件: {}", e))
    }
    
    /// 备份 hosts 文件
    fn backup_hosts_file(&self) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_filename = format!("hosts_backup_{timestamp}.txt");
        let backup_path = self.backup_dir.join(backup_filename);
        
        let content = self.read_hosts_file()?;
        fs::write(&backup_path, content)?;
        
        println!("{}", t!("command.hosts.hosts_file_backup_saved", path = backup_path.display()));
        Ok(backup_path)
    }
    
    /// 原子性写入 hosts 文件
    fn write_hosts_file_atomic(&self, content: &str) -> Result<()> {
        // 使用临时文件进行原子性写入
        let temp_path = self.hosts_path.with_extension("tmp");
        
        // 写入临时文件
        {
            let mut temp_file = fs::File::create(&temp_path)?;
            temp_file.write_all(content.as_bytes())?;
            temp_file.sync_all()?; // 确保数据写入磁盘
        }
        
        // 原子性重命名
        fs::rename(&temp_path, &self.hosts_path)
            .map_err(|e| anyhow::anyhow!("写入 hosts 文件失败: {}", e))?;
        
        println!("{}", t!("command.hosts.hosts_file_updated"));
        Ok(())
    }
    
    /// 解析 hosts 文件，提取订阅块
    fn parse_hosts_file(&self, content: &str) -> HostsFileStructure {
        let mut structure = HostsFileStructure::new();
        let mut current_block: Option<String> = None;
        let mut current_block_content = Vec::new();
        
        for line in content.lines() {
            let line = line.to_string();
            
            // 检查是否是订阅块的开始
            if let Some(url) = extract_subscription_url_from_start_marker(&line) {
                // 如果之前有未结束的块，保存到 other_content
                if let Some(block_url) = current_block.take() {
                    structure.subscription_blocks.insert(block_url, current_block_content.clone());
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
                    structure.subscription_blocks.insert(block_url, current_block_content.clone());
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
            structure.subscription_blocks.insert(block_url, current_block_content);
        }
        
        structure
    }
    
    /// 添加或更新订阅块（支持事务性操作）
    fn add_or_update_subscription(&self, url: &str, content: &str) -> Result<()> {
        // 创建事务上下文
        let mut transaction = HostsTransaction::new(self)?;
        
        // 执行更新操作
        match transaction.execute_update(url, content) {
            Ok(()) => {
                transaction.commit()?;
                println!("{}", t!("command.hosts.hosts_file_updated"));
                Ok(())
            }
            Err(e) => {
                // 自动回滚
                if let Err(rollback_err) = transaction.rollback() {
                    println!("{}", t!("command.hosts.auto_rollback_failed_simple", error = rollback_err));
                }
                Err(e)
            }
        }
    }
    
    /// 移除订阅块（支持事务性操作）
    fn remove_subscription(&self, url: &str) -> Result<bool> {
        // 创建事务上下文
        let mut transaction = HostsTransaction::new(self)?;
        
        // 执行删除操作
        match transaction.execute_removal(url) {
            Ok(removed) => {
                if removed {
                    transaction.commit()?;
                    println!("{}", t!("command.hosts.subscription_removed", url = url));
                } else {
                    println!("{}", t!("command.hosts.subscription_not_found_hosts", url = url));
                }
                Ok(removed)
            }
            Err(e) => {
                // 自动回滚
                if let Err(rollback_err) = transaction.rollback() {
                    println!("{}", t!("command.hosts.auto_rollback_failed_simple", error = rollback_err));
                }
                Err(e)
            }
        }
    }
    
    /// 获取所有订阅的 URL
    fn get_all_subscriptions(&self) -> Result<Vec<String>> {
        let content = self.read_hosts_file()?;
        let structure = self.parse_hosts_file(&content);
        Ok(structure.subscription_blocks.keys().cloned().collect())
    }
    
    /// 恢复备份文件
    fn restore_from_backup(&self, backup_path: Option<&str>) -> Result<()> {
        let backup_file = if let Some(path) = backup_path {
            PathBuf::from(path)
        } else {
            // 找到最新的备份文件
            self.find_latest_backup()?
        };
        
        if !backup_file.exists() {
            anyhow::bail!("备份文件不存在: {}", backup_file.display());
        }
        
        let backup_content = fs::read_to_string(&backup_file)?;
        self.write_hosts_file_atomic(&backup_content)?;
        
        println!("{}", t!("command.hosts.hosts_file_backup_restored", path = backup_file.display()));
        Ok(())
    }
    
    /// 查找最新的备份文件
    fn find_latest_backup(&self) -> Result<PathBuf> {
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_str()
                    .map(|name| name.starts_with("hosts_backup_") && name.ends_with(".txt"))
                    .unwrap_or(false)
            })
            .collect();
        
        if backups.is_empty() {
            anyhow::bail!("未找到备份文件");
        }
        
        // 按文件名排序（时间戳）
        backups.sort_by_key(|entry| entry.file_name());
        
        Ok(backups.last().unwrap().path())
    }
}

/// Hosts 文件结构
#[derive(Debug)]
struct HostsFileStructure {
    other_content: Vec<String>,
    subscription_blocks: std::collections::HashMap<String, Vec<String>>,
}

/// Hosts 文件事务管理器
struct HostsTransaction<'a> {
    hosts_manager: &'a HostsFileManager,
    backup_path: Option<PathBuf>,
    temp_content: Option<String>,
}

impl<'a> HostsTransaction<'a> {
    /// 创建新的事务
    fn new(hosts_manager: &'a HostsFileManager) -> Result<Self> {
        // 创建备份
        let backup_path = hosts_manager.backup_hosts_file().ok();
        
        Ok(Self {
            hosts_manager,
            backup_path,
            temp_content: None,
        })
    }
    
    /// 执行更新操作
    fn execute_update(&mut self, url: &str, content: &str) -> Result<()> {
        // 读取并解析当前 hosts 文件
        let current_content = self.hosts_manager.read_hosts_file()?;
        let mut structure = self.hosts_manager.parse_hosts_file(&current_content);
        
        // 创建订阅块内容
        let block_content = create_subscription_block(url, content);
        
        // 更新或添加订阅块
        structure.subscription_blocks.insert(url.to_string(), 
            block_content.lines().map(|s| s.to_string()).collect());
        
        // 重新组装文件内容
        let new_content = structure.reconstruct();
        self.temp_content = Some(new_content);
        
        Ok(())
    }
    
    /// 执行删除操作
    fn execute_removal(&mut self, url: &str) -> Result<bool> {
        // 读取并解析当前 hosts 文件
        let current_content = self.hosts_manager.read_hosts_file()?;
        let mut structure = self.hosts_manager.parse_hosts_file(&current_content);
        
        // 移除订阅块
        let removed = structure.subscription_blocks.remove(url).is_some();
        
        if removed {
            // 重新组装文件内容
            let new_content = structure.reconstruct();
            self.temp_content = Some(new_content);
        }
        
        Ok(removed)
    }
    
    /// 提交事务
    fn commit(self) -> Result<()> {
        if let Some(content) = self.temp_content {
            self.hosts_manager.write_hosts_file_atomic(&content)?;
        }
        Ok(())
    }
    
    /// 回滚事务
    fn rollback(self) -> Result<()> {
        if let Some(backup_path) = self.backup_path {
            self.hosts_manager.restore_from_backup(Some(backup_path.to_str().unwrap()))?;
        }
        Ok(())
    }
}

impl HostsFileStructure {
    fn new() -> Self {
        Self {
            other_content: Vec::new(),
            subscription_blocks: std::collections::HashMap::new(),
        }
    }
    
    /// 重新组装文件内容
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

/// 创建订阅块内容
fn create_subscription_block(url: &str, hosts_content: &str) -> String {
    let mut block = Vec::new();
    
    // 开始标记
    block.push(format!("# === xdev hosts subscription: {url} ==="));
    
    // 使用标准库格式化时间
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
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

/// 显示下载内容的预览
fn print_content_preview(content: &str) {
    println!("{}", t!("command.hosts.content_preview.title"));
    
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut valid_entries = 0;
    let mut preview_count = 0;
    const MAX_PREVIEW_LINES: usize = 8;
    
    for line in &lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        valid_entries += 1;
        
        if preview_count < MAX_PREVIEW_LINES {
            // 截取过长的行
            let display_line = if line.len() > 60 {
                format!("{}...", &line[..57])
            } else {
                line.to_string()
            };
            println!("  📍 {display_line}");
            preview_count += 1;
        }
    }
    
    if valid_entries > MAX_PREVIEW_LINES {
        println!("{}", t!("command.hosts.content_preview.more_entries", count = valid_entries - MAX_PREVIEW_LINES));
    }
    
    println!("{}", t!("command.hosts.content_preview.statistics", total = total_lines, valid = valid_entries));
}

/// 更新单个订阅
fn update_single_subscription(url: &str, hosts_manager: &HostsFileManager) -> Result<()> {
    // URL 验证
    validate_url(url)?;
    
    // 下载内容
    let content = download_and_validate_hosts(url)?;
    
    // 更新 hosts 文件
    hosts_manager.add_or_update_subscription(url, &content)?;
    
    Ok(())
}

/// 显示更新结果摘要
fn display_update_summary(success_count: usize, failed_urls: &[String], total_count: usize) {
    println!("{}", t!("command.hosts.update.summary_title"));
    println!("{}", t!("command.hosts.update.summary_total", count = total_count));
    println!("{}", t!("command.hosts.update.summary_success", count = success_count));
    println!("{}", t!("command.hosts.update.summary_failed", count = failed_urls.len()));
    
    if !failed_urls.is_empty() {
        println!();
        println!("{}", t!("command.hosts.update.failed_list"));
        for (i, url) in failed_urls.iter().enumerate() {
            println!("   {}. {}", i + 1, url);
        }
        println!();
        println!("{}", t!("command.hosts.update.suggestions_title"));
        println!("{}", t!("command.hosts.update.suggestion_network"));
        println!("{}", t!("command.hosts.update.suggestion_urls"));
        println!("{}", t!("command.hosts.update.suggestion_resubscribe"));
    }
    
    if success_count == total_count {
        println!();
        println!("{}", t!("command.hosts.update.all_success"));
    } else if success_count > 0 {
        println!();
        println!("{}", t!("command.hosts.update.partial_success"));
    } else {
        println!();
        println!("{}", t!("command.hosts.update.all_failed"));
    }
}

/// 显示可用的备份文件
fn display_available_backups(hosts_manager: &HostsFileManager) -> Result<()> {
    println!("{}", t!("command.hosts.backup_list.title"));
    
    if !hosts_manager.backup_dir.exists() {
        println!("{}", t!("command.hosts.backup_list.empty"));
        return Ok(());
    }
    
    let entries = std::fs::read_dir(&hosts_manager.backup_dir)?;
    let mut backups: Vec<std::fs::DirEntry> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(name) = entry.file_name().to_str() {
                name.starts_with("hosts_backup_") && name.ends_with(".txt")
            } else {
                false
            }
        })
        .collect();
    
    if backups.is_empty() {
        println!("{}", t!("command.hosts.backup_list.empty"));
        return Ok(());
    }
    
    // 按时间戳排序
    backups.sort_by_key(|entry| {
        entry.file_name().to_str()
            .and_then(|name| {
                name.strip_prefix("hosts_backup_")
                    .and_then(|s| s.strip_suffix(".txt"))
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0)
    });
    
    let total_backups = backups.len();
    
    // 显示最近的10个备份（倒序）
    backups.reverse();
    let show_count = std::cmp::min(10, backups.len());
    
    for (i, entry) in backups.iter().enumerate().take(show_count) {
        let file_path = entry.path();
        
        if let Some(file_name) = entry.file_name().to_str() {
            // 提取时间戳
            let timestamp = file_name
                .strip_prefix("hosts_backup_")
                .and_then(|s| s.strip_suffix(".txt"))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            
            // 获取文件大小
            let size = std::fs::metadata(&file_path)
                .map(|m| m.len())
                .unwrap_or(0);
            
            let marker = if i == 0 { "🔸" } else { "📄" };
            println!("   {} {} {}", 
                     marker, 
                     file_path.display(), 
                     t!("command.hosts.backup_list.file_info", timestamp = timestamp, size = size));
            
            if i == 0 {
                println!("{}", t!("command.hosts.backup_list.latest_marker"));
            }
        }
    }
    
    if show_count < total_backups {
        println!("{}", t!("command.hosts.backup_list.more_backups", count = total_backups - show_count));
    }
    
    Ok(())
}

/// 尝试回滚 hosts 文件到最新备份
fn attempt_hosts_rollback(hosts_manager: &HostsFileManager) -> Result<()> {
    // 查找最新的备份文件
    match hosts_manager.find_latest_backup() {
        Ok(latest_backup) => {
            // 尝试恢复最新备份
            hosts_manager.restore_from_backup(Some(latest_backup.to_str().unwrap()))?;
            Ok(())
        }
        Err(_) => {
            anyhow::bail!("未找到可用的备份文件进行回滚");
        }
    }
} 
