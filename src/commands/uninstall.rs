use crate::core::i18n::t;
use anyhow::{Context, Result};
use clap::Command;
use std::fs;
use std::path::PathBuf;

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("uninstall").about(t!("command.uninstall.description").to_string()),
    );
}

pub fn execute() -> Result<()> {
    let install_dir = get_install_dir()?;
    let target_path = install_dir.join("xdev");

    if !target_path.exists() {
        println!(
            "{}",
            t!(
                "command.uninstall.not_installed",
                path = target_path.display()
            )
        );
        return Ok(());
    }

    println!(
        "{}",
        t!("command.uninstall.removing", path = target_path.display())
    );

    fs::remove_file(&target_path)
        .with_context(|| t!("error.remove_failed", path = target_path.display()).to_string())?;

    println!("{}", t!("command.uninstall.success"));

    Ok(())
}

fn get_install_dir() -> Result<PathBuf> {
    // 只使用 ~/.local/bin
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.home_dir_not_found").to_string()))?;

    let local_bin = home_dir.join(".local").join("bin");
    Ok(local_bin)
}
