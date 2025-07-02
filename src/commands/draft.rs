use crate::core::i18n::t;
use anyhow::Result;
use clap::Command;
use std::fs;
use std::path::{Path, PathBuf};

const ZDOCS_PATH: &str = "/tmp/zdocs";
const REQUIRED_FILES: &[&str] = &[
    "content.json",
    "meta.json",
    "numbering.json",
    "relations.json",
    "settings.json",
    "styles.json",
];

pub fn register_command(app: &mut Command) {
    *app = app
        .clone()
        .subcommand(Command::new("draft").about(t!("command.draft.description").to_string()));
}

pub fn execute() -> Result<()> {
    match find_latest_draft_directory() {
        Ok(path) => {
            println!("{}: {}", t!("command.draft.found"), path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}: {}", t!("command.draft.error"), e);
            Err(e)
        }
    }
}

/// 查找最新的 draft 目录
fn find_latest_draft_directory() -> Result<PathBuf> {
    // 检查 zdocs 目录是否存在
    let zdocs_path = Path::new(ZDOCS_PATH);
    if !zdocs_path.exists() {
        anyhow::bail!(t!("error.zdocs_not_found", path = ZDOCS_PATH));
    }

    // 获取最新的子目录
    let latest_subdir = get_latest_subdirectory(zdocs_path)?;

    // 在最新子目录中查找包含所有必需文件的目录
    find_draft_directory(&latest_subdir)
}

/// 获取指定目录下最新的子目录
fn get_latest_subdirectory(dir: &Path) -> Result<PathBuf> {
    let mut latest_dir: Option<PathBuf> = None;
    let mut latest_time: Option<std::time::SystemTime> = None;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let metadata = entry.metadata()?;
            let modified_time = metadata.modified()?;

            // 如果这是第一个目录，或者比当前记录的最新时间更新
            if latest_time.is_none() || modified_time > latest_time.unwrap() {
                latest_dir = Some(path);
                latest_time = Some(modified_time);
            }
        }
    }

    match latest_dir {
        Some(dir) => Ok(dir),
        None => anyhow::bail!(t!("error.no_subdirs_found", path = dir.display())),
    }
}

/// 在指定目录的子目录中查找包含所有必需文件的目录
fn find_draft_directory(dir: &Path) -> Result<PathBuf> {
    // 只检查直接子目录
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && has_all_required_files(&path) {
            return Ok(path);
        }
    }

    anyhow::bail!(t!("error.draft_not_found", path = dir.display()))
}

/// 检查目录是否包含所有必需的文件
fn has_all_required_files(dir: &Path) -> bool {
    REQUIRED_FILES.iter().all(|file| dir.join(file).exists())
}
