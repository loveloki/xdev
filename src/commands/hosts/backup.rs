use crate::commands::hosts::{create_hosts_manager, helpers::generate_backup_filename};
use crate::core::i18n::t;
use crate::core::permission::ensure_sudo_privileges;
use crate::core::table::{add_table_row, create_backup_table, print_table, set_table_header};
use anyhow::Result;
use std::path::PathBuf;

/// 处理备份命令
pub fn handle_backup() -> Result<()> {
    println!("{}", t!("command.hosts.backup.starting"));

    // 权限检查（读取文件需要权限）
    let hosts_manager = create_hosts_manager()?;

    // 执行备份
    let backup_filename = generate_backup_filename();
    match hosts_manager.file_manager().backup_file(&backup_filename) {
        Ok(backup_path) => {
            println!("{}", t!("command.hosts.backup.success"));
            println!(
                "{}",
                t!(
                    "command.hosts.backup.file_path",
                    path = backup_path.display()
                )
            );
            println!(
                "{}",
                t!(
                    "command.hosts.hosts_file_backup_saved",
                    path = backup_path.display()
                )
            );

            // 显示备份文件大小
            if let Ok(metadata) = std::fs::metadata(&backup_path) {
                println!(
                    "{}",
                    t!("command.hosts.backup.file_size", size = metadata.len())
                );
            }

            // 显示所有备份文件
            println!();
            display_available_backups()?;
        }
        Err(e) => {
            println!("{}", t!("command.hosts.backup.failed", error = e));
            return Err(e);
        }
    }

    Ok(())
}

/// 处理恢复命令
pub fn handle_restore(backup_file: Option<&str>) -> Result<()> {
    println!("{}", t!("command.hosts.restore.starting"));

    // 权限检查
    ensure_sudo_privileges()?;

    let hosts_manager = create_hosts_manager()?;

    match backup_file {
        Some(file_path) => {
            // 使用指定的备份文件
            println!(
                "{}",
                t!("command.hosts.restore.from_specified", path = file_path)
            );

            // 检查文件是否存在
            if !std::path::Path::new(file_path).exists() {
                anyhow::bail!(
                    "{}",
                    t!("error.hosts_backup_file_not_exist", path = file_path)
                );
            }

            // 显示文件信息
            if let Ok(metadata) = std::fs::metadata(file_path) {
                println!(
                    "{}",
                    t!("command.hosts.restore.file_size", size = metadata.len())
                );
                if let Ok(modified) = metadata.modified()
                    && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
                {
                    println!(
                        "{}",
                        t!(
                            "command.hosts.restore.backup_time",
                            timestamp = duration.as_secs()
                        )
                    );
                }
            }

            // 执行恢复
            hosts_manager
                .file_manager()
                .restore_from_backup(file_path)?;
            println!(
                "{}",
                t!("command.hosts.hosts_file_backup_restored", path = file_path)
            );
        }
        None => {
            // 显示可用备份
            display_available_backups()?;

            // 提示用户指定备份文件
            println!("{}", t!("command.hosts.restore.specify_backup"));
            anyhow::bail!("{}", t!("command.hosts.restore.specify_backup"));
        }
    }

    println!("{}", t!("command.hosts.restore.success"));
    println!("{}", t!("command.hosts.restore.check_suggestion"));

    Ok(())
}

/// 尝试回滚 hosts 文件到最新备份
pub fn attempt_hosts_rollback() -> Result<()> {
    let hosts_manager = create_hosts_manager()?;

    // 获取所有备份文件
    let backups = hosts_manager.file_manager().list_backups()?;

    // 过滤出 hosts 备份文件并按时间戳排序
    let mut hosts_backups: Vec<PathBuf> = backups
        .into_iter()
        .filter(|path| {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                name.starts_with("hosts_backup_") && name.ends_with(".txt")
            } else {
                false
            }
        })
        .collect();

    if hosts_backups.is_empty() {
        anyhow::bail!("{}", t!("error.hosts_backup_not_found"));
    }

    // 按时间戳排序，获取最新的
    hosts_backups.sort_by_key(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| {
                name.strip_prefix("hosts_backup_")
                    .and_then(|s| s.strip_suffix(".txt"))
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0)
    });

    // 恢复最新备份
    let latest_backup = hosts_backups
        .last()
        .ok_or_else(|| anyhow::anyhow!("{}", t!("error.no_backups_available")))?;
    let backup_filename = latest_backup
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("{}", t!("error.invalid_backup_filename")))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("{}", t!("error.invalid_backup_filename")))?;
    hosts_manager
        .file_manager()
        .restore_from_backup(backup_filename)?;

    Ok(())
}

/// 显示可用的备份文件
fn display_available_backups() -> Result<()> {
    println!("{}", t!("command.hosts.backup_list.title"));

    let hosts_manager = create_hosts_manager()?;
    let backup_dir = &hosts_manager.file_manager().backup_dir;

    if !backup_dir.exists() {
        println!("{}", t!("command.hosts.backup_list.empty"));
        return Ok(());
    }

    let entries = std::fs::read_dir(backup_dir)?;
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
        entry
            .file_name()
            .to_str()
            .and_then(|name| {
                name.strip_prefix("hosts_backup_")
                    .and_then(|s| s.strip_suffix(".txt"))
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0)
    });

    let total_backups = backups.len();
    backups.reverse();
    let show_count = std::cmp::min(10, backups.len());

    let mut table = create_backup_table();
    set_table_header(
        &mut table,
        vec![
            "标记".to_string(),
            "文件名".to_string(),
            "时间戳".to_string(),
            "大小 (字节)".to_string(),
        ],
    );

    for (i, entry) in backups.iter().enumerate().take(show_count) {
        let file_path = entry.path();
        if let Some(file_name) = entry.file_name().to_str() {
            let timestamp = file_name
                .strip_prefix("hosts_backup_")
                .and_then(|s| s.strip_suffix(".txt"))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
            let marker = if i == 0 { "🔸最新" } else { "📄" };
            add_table_row(
                &mut table,
                vec![
                    marker.to_string(),
                    file_name.to_string(),
                    timestamp.to_string(),
                    size.to_string(),
                ],
            );
        }
    }
    print_table(&table);

    if show_count < total_backups {
        println!(
            "{}",
            t!(
                "command.hosts.backup_list.more_backups",
                count = total_backups - show_count
            )
        );
    }

    Ok(())
}
