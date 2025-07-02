use crate::core::i18n::t;
use anyhow::{Context, Result};
use clap::Command;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(Command::new("install").about(t!("command.install.description").to_string()));
}

pub fn execute() -> Result<()> {
    println!("{}", t!("command.install.building"));

    // 构建 release 版本
    let output = process::Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .context(t!("error.cargo_build_failed").to_string())?;

    if !output.status.success() {
        anyhow::bail!(
            t!(
                "error.build_failed",
                error = String::from_utf8_lossy(&output.stderr)
            )
            .to_string()
        );
    }

    let install_dir = get_install_dir()?;
    let binary_path = get_binary_path()?;
    let target_path = install_dir.join("xdev");

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

    // 设置执行权限 (Unix系统)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }

    println!(
        "{}",
        t!("command.install.success", path = target_path.display())
    );

    // 检查是否在 PATH 中
    if let Ok(path_var) = env::var("PATH") {
        let path_dirs: Vec<&str> = path_var.split(':').collect();
        let install_dir_str = install_dir.to_string_lossy();

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

    Ok(())
}

fn get_install_dir() -> Result<PathBuf> {
    // 只使用 ~/.local/bin
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.home_dir_not_found").to_string()))?;

    let local_bin = home_dir.join(".local").join("bin");

    // 如果目录不存在，尝试创建
    if !local_bin.exists() {
        fs::create_dir_all(&local_bin).with_context(|| {
            t!(
                "error.create_install_dir_failed",
                path = local_bin.display()
            )
            .to_string()
        })?;
    }

    Ok(local_bin)
}

fn get_binary_path() -> Result<PathBuf> {
    let current_exe = env::current_exe().context(t!("error.current_exe_failed").to_string())?;

    // 如果我们在开发环境中，使用 target/release/xdev
    let current_dir = env::current_dir().context(t!("error.current_dir_failed").to_string())?;

    let release_binary = current_dir.join("target").join("release").join("xdev");
    if release_binary.exists() {
        Ok(release_binary)
    } else {
        Ok(current_exe)
    }
}
