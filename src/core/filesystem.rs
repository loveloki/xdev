//! 文件系统管理模块
//!
//! 提供通用的文件操作、备份管理和原子性写入功能，
//! 可被多个命令模块复用。

use crate::core::i18n::t;
use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// 文件管理器
pub struct FileManager {
    /// 目标文件路径
    pub file_path: PathBuf,
    /// 备份目录
    pub backup_dir: PathBuf,
}

impl FileManager {
    /// 创建新的文件管理器
    pub fn new(file_path: PathBuf, backup_dir: PathBuf) -> Result<Self> {
        // 确保备份目录存在
        fs::create_dir_all(&backup_dir)?;

        Ok(Self {
            file_path,
            backup_dir,
        })
    }

    /// 创建带有分类备份目录的文件管理器
    pub fn with_typed_backup(file_path: PathBuf, backup_type: &str) -> Result<Self> {
        // 创建带有分类的备份目录
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("{}", t!("error.config_dir_failed", path = "config")))?;
        let backup_dir = config_dir.join("xdev").join("backups").join(backup_type);

        Self::new(file_path, backup_dir)
    }

    /// 读取文件内容
    pub fn read_file(&self) -> Result<String> {
        fs::read_to_string(&self.file_path)
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_file_read_failed", error = e)))
    }

    /// 备份文件到指定文件名
    pub fn backup_file(&self, backup_filename: &str) -> Result<PathBuf> {
        let backup_path = self.backup_dir.join(backup_filename);

        let content = self.read_file()?;
        fs::write(&backup_path, content)?;

        Ok(backup_path)
    }

    /// 原子性写入文件
    pub fn write_file_atomic(&self, content: &str) -> Result<()> {
        // 使用临时文件进行原子性写入
        let temp_path = self.file_path.with_extension("tmp");

        // 写入临时文件
        {
            let mut temp_file = fs::File::create(&temp_path)?;
            temp_file.write_all(content.as_bytes())?;
            temp_file.sync_all()?; // 确保数据写入磁盘
        }

        // 原子性重命名
        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| anyhow::anyhow!("{}", t!("error.hosts_file_write_failed", error = e)))?;

        Ok(())
    }

    /// 恢复文件从备份
    pub fn restore_from_backup(&self, backup_filename: &str) -> Result<()> {
        let backup_file_path = self.backup_dir.join(backup_filename);

        if !backup_file_path.exists() {
            anyhow::bail!(
                "{}",
                t!(
                    "error.hosts_backup_file_not_exist",
                    path = backup_file_path.display()
                )
            );
        }

        // 读取备份文件内容
        let backup_content = fs::read_to_string(&backup_file_path).map_err(|_e| {
            anyhow::anyhow!(
                "{}",
                t!(
                    "error.hosts_backup_file_not_exist",
                    path = backup_file_path.display()
                )
            )
        })?;

        // 原子性写入到目标文件
        self.write_file_atomic(&backup_content)?;

        Ok(())
    }

    /// 列出备份目录中的所有文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.backup_dir)?;
        let backups: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect();

        Ok(backups)
    }
}

/// 通用的文件结构管理器
pub struct StructuredFileManager {
    file_manager: FileManager,
}

impl StructuredFileManager {
    /// 创建新的结构化文件管理器
    pub fn new(file_manager: FileManager) -> Self {
        Self { file_manager }
    }

    /// 解析文件内容为结构化数据
    pub fn parse_file<T: FileStructure>(&self) -> Result<T> {
        let content = self.file_manager.read_file()?;
        Ok(T::parse(&content))
    }

    /// 更新文件结构并备份
    pub fn update_structure_with_backup<T: FileStructure>(
        &self,
        structure: &T,
        backup_filename: &str,
    ) -> Result<()> {
        // 先备份
        self.file_manager.backup_file(backup_filename)?;

        // 重构内容并写入
        let content = structure.reconstruct();
        self.file_manager.write_file_atomic(&content)?;

        Ok(())
    }

    /// 获取文件管理器的引用
    pub fn file_manager(&self) -> &FileManager {
        &self.file_manager
    }
}

/// 文件结构 trait
pub trait FileStructure {
    fn parse(content: &str) -> Self;
    fn reconstruct(&self) -> String;
}
