use crate::core::filesystem::{FileManager, StructuredFileManager};
use crate::core::globals::{
    BACKUP_FILE_PREFIX, BACKUP_FILE_SUFFIX, MAX_PREVIEW_LINE_LENGTH, MAX_PREVIEW_LINES,
    PREVIEW_LINE_DISPLAY_LENGTH,
};
use crate::core::i18n::t;
use anyhow::Result;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 创建 hosts 文件管理器（统一函数）
pub fn create_hosts_manager() -> Result<StructuredFileManager> {
    let file_manager = FileManager::with_typed_backup(PathBuf::from("/etc/hosts"), "hosts")?;
    Ok(StructuredFileManager::new(file_manager))
}

/// 获取当前时间戳
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 生成备份文件名
pub fn generate_backup_filename() -> String {
    let timestamp = get_current_timestamp();
    format!("{BACKUP_FILE_PREFIX}{timestamp}{BACKUP_FILE_SUFFIX}")
}

/// 显示下载内容的预览
pub fn print_content_preview(content: &str) {
    println!("{}", t!("command.hosts.content_preview.title"));

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut valid_entries = 0;
    let mut preview_count = 0;

    for line in &lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        valid_entries += 1;

        if preview_count < MAX_PREVIEW_LINES {
            // 截取过长的行
            let display_line = if line.len() > MAX_PREVIEW_LINE_LENGTH {
                format!("{}...", &line[..PREVIEW_LINE_DISPLAY_LENGTH])
            } else {
                line.to_string()
            };
            println!("  📍 {display_line}");
            preview_count += 1;
        }
    }

    if valid_entries > MAX_PREVIEW_LINES {
        println!(
            "{}",
            t!(
                "command.hosts.content_preview.more_entries",
                count = valid_entries - MAX_PREVIEW_LINES
            )
        );
    }

    println!(
        "{}",
        t!(
            "command.hosts.content_preview.statistics",
            total = total_lines,
            valid = valid_entries
        )
    );
}

/// 显示更新结果摘要
pub fn display_update_summary(success_count: usize, failed_urls: &[String], total_count: usize) {
    println!("{}", t!("command.hosts.update.summary_title"));
    println!(
        "{}",
        t!("command.hosts.update.summary_total", count = total_count)
    );
    println!(
        "{}",
        t!(
            "command.hosts.update.summary_success",
            count = success_count
        )
    );
    println!(
        "{}",
        t!(
            "command.hosts.update.summary_failed",
            count = failed_urls.len()
        )
    );

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
