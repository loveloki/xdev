use clap::Command;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use anyhow::{Context, Result};

fn main() {
    let matches = Command::new("xdev")
        .version("0.1.0")
        .author("xdev CLI Tool")
        .about("A development CLI tool")
        .disable_version_flag(true) // 禁用自动生成的--version参数
        .subcommand(
            Command::new("version")
                .about("Show version information")
        )
        .subcommand(
            Command::new("install")
                .about("Install xdev binary to system")
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall xdev binary from system")
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("version", _)) => {
            println!("xdev {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(("install", _)) => install_binary(),
        Some(("uninstall", _)) => uninstall_binary(),
        _ => {
            println!("xdev CLI tool - use --help for more information");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn get_install_dir() -> Result<PathBuf> {
    // 优先使用 ~/.local/bin，如果不存在则使用 /usr/local/bin
    if let Some(home_dir) = dirs::home_dir() {
        let local_bin = home_dir.join(".local").join("bin");
        if local_bin.exists() || fs::create_dir_all(&local_bin).is_ok() {
            return Ok(local_bin);
        }
    }
    
    // 如果 ~/.local/bin 不可用，使用 /usr/local/bin (需要sudo权限)
    Ok(PathBuf::from("/usr/local/bin"))
}

fn get_binary_path() -> Result<PathBuf> {
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;
    
    // 如果我们在开发环境中，使用 target/release/xdev
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;
    
    let release_binary = current_dir.join("target").join("release").join("xdev");
    if release_binary.exists() {
        Ok(release_binary)
    } else {
        Ok(current_exe)
    }
}

fn install_binary() -> Result<()> {
    println!("🔨 Building release binary...");
    
    // 构建 release 版本
    let output = process::Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .context("Failed to execute cargo build")?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to build binary:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    
    let install_dir = get_install_dir()?;
    let binary_path = get_binary_path()?;
    let target_path = install_dir.join("xdev");
    
    println!("📦 Installing xdev to {}...", target_path.display());
    
    // 复制二进制文件
    fs::copy(&binary_path, &target_path)
        .with_context(|| format!("Failed to copy binary from {} to {}", 
            binary_path.display(), target_path.display()))?;
    
    // 设置执行权限 (Unix系统)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }
    
    println!("✅ Successfully installed xdev to {}", target_path.display());
    
    // 检查是否在 PATH 中
    if let Ok(path_var) = env::var("PATH") {
        let path_dirs: Vec<&str> = path_var.split(':').collect();
        let install_dir_str = install_dir.to_string_lossy();
        
        if !path_dirs.contains(&install_dir_str.as_ref()) {
            println!("⚠️  Warning: {} is not in your PATH", install_dir.display());
            println!("   Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):");
            println!("   export PATH=\"{}:$PATH\"", install_dir.display());
        }
    }
    
    Ok(())
}

fn uninstall_binary() -> Result<()> {
    let install_dir = get_install_dir()?;
    let target_path = install_dir.join("xdev");
    
    if !target_path.exists() {
        println!("ℹ️  xdev is not installed at {}", target_path.display());
        return Ok(());
    }
    
    println!("🗑️  Removing xdev from {}...", target_path.display());
    
    fs::remove_file(&target_path)
        .with_context(|| format!("Failed to remove binary from {}", target_path.display()))?;
    
    println!("✅ Successfully uninstalled xdev");
    
    Ok(())
}
