use crate::core::i18n::t;
use std::time::{SystemTime, UNIX_EPOCH};

/// 检查是否为有效的 IP 地址（IPv4 和 IPv6）
pub fn is_valid_ip(ip: &str) -> bool {
    // IPv4 验证
    if ip.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }

    // IPv6 验证
    if ip.parse::<std::net::Ipv6Addr>().is_ok() {
        return true;
    }

    false
}

/// 生成备份文件名
pub fn generate_backup_filename() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("hosts_backup_{timestamp}.txt")
}

/// 显示下载内容的预览
pub fn print_content_preview(content: &str) {
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

/// 验证 hosts 文件内容格式
pub fn validate_hosts_content(content: &str) -> anyhow::Result<()> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        anyhow::bail!("{}", t!("error.hosts_content_empty"));
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
            if line_num < 10 {
                // 只打印前10个错误
                println!(
                    "{}",
                    t!(
                        "command.hosts.download.validation_warning_line",
                        line_num = line_num + 1,
                        line = line
                    )
                );
            }
        }
    }

    if total_non_empty_lines == 0 {
        anyhow::bail!("{}", t!("error.hosts_content_invalid"));
    }

    // 如果有效行数少于总行数的 50%，可能不是 hosts 文件
    if valid_lines < total_non_empty_lines / 2 {
        println!("{}", t!("command.hosts.download.validation_warning_format"));
        println!(
            "{}",
            t!(
                "command.hosts.download.validation_line_stats",
                valid = valid_lines,
                total = total_non_empty_lines
            )
        );
    } else {
        println!(
            "{}",
            t!(
                "command.hosts.download.validation_success",
                valid = valid_lines,
                total = total_non_empty_lines
            )
        );
    }

    Ok(())
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
