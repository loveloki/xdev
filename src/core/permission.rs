//! 权限管理模块
//!
//! 提供通用的权限检查和验证功能，
//! 可被多个命令模块复用。

use anyhow::Result;
use nix::unistd::geteuid;
use std::process::Command;

use crate::core::i18n::t;

/// 确保当前用户具有 sudo 权限
pub fn ensure_sudo_privileges() -> Result<()> {
    // 首先检查当前进程是否已经是 root
    if geteuid().is_root() {
        return Ok(());
    }

    // 如果不是 root，检查当前用户是否有 sudo 权限
    let sudo_check = Command::new("sudo").arg("-n").arg("true").output();

    match sudo_check {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                anyhow::bail!("{}", t!("error.permission_denied"));
            }
        }
        Err(_) => {
            // sudo 命令不存在或无法执行
            anyhow::bail!("{}", t!("error.permission_denied"));
        }
    }
}
