use crate::core::globals::APP_NAME;
use crate::core::i18n::t;
use anyhow::{Context, Result};
use clap::Command;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(Command::new("install").about(t!("command.install.description").to_string()));
}

pub fn execute() -> Result<()> {
    let install_dir = get_install_dir()?;
    let binary_path = get_binary_path()?;
    let target_path = install_dir.join(APP_NAME);

    println!(
        "{}",
        t!("command.install.installing", path = target_path.display())
    );

    // 复制二进制文件
    fs::copy(&binary_path, &target_path).with_context(|| {
        t!(
            "error.copy_failed",
            from = binary_path.display(),
            to = target_path.display()
        )
        .to_string()
    })?;

    println!(
        "{}",
        t!("command.install.success", path = target_path.display())
    );

    // 检查是否在 PATH 中
    check_path_warning(&install_dir);

    Ok(())
}

fn check_path_warning(install_dir: &Path) {
    let path_var = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => return, // 如果无法获取 PATH，静默返回
    };

    let install_dir_str = install_dir.to_string_lossy();
    let path_dirs: Vec<&str> = path_var.split(':').collect();

    if !path_dirs.contains(&install_dir_str.as_ref()) {
        println!(
            "{}",
            t!("command.install.path_warning", path = install_dir.display())
        );
        println!("{}", t!("command.install.path_instruction"));
        println!(
            "{}",
            t!("command.install.path_export", path = install_dir.display())
        );
    }
}

fn get_install_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.home_dir_not_found").to_string()))?;

    let local_bin = home_dir.join(".local").join("bin");

    // 创建目录（如果不存在）
    fs::create_dir_all(&local_bin).with_context(|| {
        t!(
            "error.create_install_dir_failed",
            path = local_bin.display()
        )
        .to_string()
    })?;

    Ok(local_bin)
}

fn get_binary_path() -> Result<PathBuf> {
    // 直接使用当前运行的二进制文件
    let current_exe = env::current_exe().context(t!("error.current_exe_failed").to_string())?;
    Ok(current_exe)
}
