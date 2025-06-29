use crate::core::i18n::t;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::process;

// 编译时确定的 DNS 更新命令
#[cfg(target_os = "linux")]
const DNS_UPDATE_COMMAND: &str = r#"sed -i "/# GitHub520 Host Start/Q" /etc/hosts && curl https://raw.hellogithub.com/hosts >> /etc/hosts"#;

#[cfg(target_os = "macos")]
const DNS_UPDATE_COMMAND: &str = r#"sed -i "" "/# GitHub520 Host Start/,/# Github520 Host End/d" /etc/hosts && curl https://raw.hellogithub.com/hosts | sudo tee -a /etc/hosts"#;

// 不支持的平台编译时错误
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("DNS update is only supported on Linux and macOS platforms");

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("dns")
            .about(t!("command.dns.description").to_string())
            .arg(
                Arg::new("source")
                    .help(t!("help.dns_source").to_string())
                    .value_name("SOURCE")
                    .default_value("github")
                    .value_parser(["github"])
            )
    );
}

pub fn execute(matches: &clap::ArgMatches) -> Result<()> {
    let source = matches.get_one::<String>("source")
        .map(|s| s.as_str())
        .unwrap_or("github");
    
    match source {
        "github" => update_github_dns(),
        _ => {
            anyhow::bail!(t!("error.unsupported_dns_source", source = source).to_string());
        }
    }
}

fn update_github_dns() -> Result<()> {
    println!("{}", t!("command.dns.checking_permissions"));
    
    // 检查是否有 sudo 权限
    let sudo_check = process::Command::new("sudo")
        .args(["-n", "true"])
        .output()
        .context(t!("error.sudo_check_failed").to_string())?;
    
    if !sudo_check.status.success() {
        println!("{}", t!("command.dns.sudo_required"));
        println!("{}", t!("command.dns.sudo_instruction"));
    }
    
    println!("{}", t!("command.dns.updating_github_dns"));
    
    // 使用编译时确定的平台特定命令
    let output = process::Command::new("sudo")
        .args(["sh", "-c", DNS_UPDATE_COMMAND])
        .output()
        .context(t!("error.dns_update_failed").to_string())?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            t!("error.dns_update_command_failed", error = error_msg).to_string()
        );
    }
    
    println!("{}", t!("command.dns.success"));
    println!("{}", t!("command.dns.hosts_updated"));
    
    // 显示更新的主机条目数量（可选）
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", t!("command.dns.update_details"));
        }
    }
    
    Ok(())
} 
