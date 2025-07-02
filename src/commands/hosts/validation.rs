use crate::core::i18n::t;

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
