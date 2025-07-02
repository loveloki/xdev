use crate::commands::config::file::get_config_path;
use crate::core::i18n::{set_language, t, validate_language};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub draft_path: String,
    pub lang: String,
    pub hosts_subscriptions: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            draft_path: "/tmp/zdocs".to_string(),
            lang: "zh-Hans".to_string(),
            hosts_subscriptions: Some(Vec::new()),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            // 如果配置文件不存在，创建默认配置
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).with_context(|| {
            t!("error.config_read_failed", path = config_path.display()).to_string()
        })?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| t!("error.config_parse_failed").to_string())?;

        // 兼容性处理：如果 hosts_subscriptions 字段不存在，初始化为空列表
        if config.hosts_subscriptions.is_none() {
            config.hosts_subscriptions = Some(Vec::new());
        }

        // 设置语言
        if validate_language(&config.lang) {
            let _ = set_language(&config.lang);
        } else {
            // 如果配置中的语言无效，重置为默认语言
            config.lang = "zh-Hans".to_string();
            let _ = set_language(&config.lang);
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                t!("error.config_dir_failed", path = parent.display()).to_string()
            })?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| t!("error.config_serialize_failed").to_string())?;

        fs::write(&config_path, content).with_context(|| {
            t!("error.config_write_failed", path = config_path.display()).to_string()
        })?;

        Ok(())
    }

    pub fn set_field(&mut self, field: &str, value: &str) -> Result<()> {
        match field {
            "draft_path" => self.draft_path = value.to_string(),
            "lang" => {
                if !validate_language(value) {
                    anyhow::bail!(t!("error.unsupported_language", lang = value).to_string());
                }
                self.lang = value.to_string();
                // 立即应用语言设置
                set_language(value)?;
            }
            _ => anyhow::bail!(t!("error.unknown_field", field = field).to_string()),
        }
        Ok(())
    }

    // Hosts 订阅管理方法
    pub fn add_hosts_subscription(&mut self, url: &str) -> Result<bool> {
        if self.hosts_subscriptions.is_none() {
            self.hosts_subscriptions = Some(Vec::new());
        }

        let subscriptions = self.hosts_subscriptions.as_mut().unwrap();
        if subscriptions.contains(&url.to_string()) {
            return Ok(false); // 已存在
        }

        subscriptions.push(url.to_string());
        Ok(true) // 添加成功
    }

    pub fn remove_hosts_subscription(&mut self, url: &str) -> Result<bool> {
        if let Some(subscriptions) = &mut self.hosts_subscriptions
            && let Some(pos) = subscriptions.iter().position(|x| x == url)
        {
            subscriptions.remove(pos);
            return Ok(true); // 删除成功
        }
        Ok(false) // 未找到
    }

    pub fn get_hosts_subscriptions(&self) -> Vec<String> {
        self.hosts_subscriptions
            .as_ref()
            .unwrap_or(&Vec::new())
            .clone()
    }

    pub fn get_field(&self, field: &str) -> Result<String> {
        match field {
            "draft_path" => Ok(self.draft_path.clone()),
            "lang" => Ok(self.lang.clone()),
            "hosts_subscriptions" => Ok(format!(
                "{:?}",
                self.hosts_subscriptions.as_ref().unwrap_or(&Vec::new())
            )),
            _ => anyhow::bail!(t!("error.unknown_field", field = field).to_string()),
        }
    }
}
