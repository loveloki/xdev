use anyhow::{Context, Result};
use dialoguer::{Select, Input, Confirm};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub draft_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            draft_path: "/tmp/zdocs".to_string(),
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
        
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
        Ok(())
    }
    
    pub fn set_field(&mut self, field: &str, value: &str) -> Result<()> {
        match field {
            "draft_path" => self.draft_path = value.to_string(),
            _ => anyhow::bail!("Unknown config field: {}", field),
        }
        Ok(())
    }
    
    pub fn get_field(&self, field: &str) -> Result<String> {
        match field {
            "draft_path" => Ok(self.draft_path.clone()),
            _ => anyhow::bail!("Unknown config field: {}", field),
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    
    Ok(config_dir.join("xdev").join("config"))
}

pub fn show() -> Result<()> {
    let config = Config::load()?;
    let config_path = get_config_path()?;
    
    println!("📋 Configuration ({})", config_path.display());
    println!("┌─────────────────┬─────────────────────────────────┐");
    println!("│ Setting         │ Value                           │");
    println!("├─────────────────┼─────────────────────────────────┤");
    println!("│ draft_path      │ {:<31} │", config.draft_path);
    println!("└─────────────────┴─────────────────────────────────┘");
    
    Ok(())
}

pub fn set_item(field: Option<&str>, value: Option<&str>) -> Result<()> {
    let mut config = Config::load()?;
    
    match (field, value) {
        (Some(field), Some(value)) => {
            // 直接设置指定字段
            config.set_field(field, value)?;
            config.save()?;
            println!("✅ Set {} = {}", field, value);
        }
        (Some(field), None) => {
            // 交互式设置单个字段
            let current_value = config.get_field(field)?;
            let new_value = prompt_for_field(field, &current_value)?;
            
            if new_value != current_value {
                config.set_field(field, &new_value)?;
                config.save()?;
                println!("✅ Updated {} from '{}' to '{}'", field, current_value, new_value);
            } else {
                println!("ℹ️  No changes made to {}", field);
            }
        }
        (None, None) => {
            // 进入交互式设置模式
            interactive_config_setup(&mut config)?;
        }
        _ => {
            anyhow::bail!("Invalid arguments combination");
        }
    }
    
    Ok(())
}

fn interactive_config_setup(config: &mut Config) -> Result<()> {
    println!("🔧 Interactive Configuration Setup");
    println!("Select a configuration item to modify:");
    
    loop {
        let items = vec![
            "draft_path - Path for draft documents",
            "Show current config",
            "Exit"
        ];
        
        let selection = Select::new()
            .with_prompt("Choose an option")
            .items(&items)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                let new_value = prompt_for_field("draft_path", &config.draft_path)?;
                if new_value != config.draft_path {
                    config.draft_path = new_value.clone();
                    println!("✅ Updated draft_path to '{}'", new_value);
                }
            }
            1 => {
                show()?;
                continue;
            }
            2 => break,
            _ => continue,
        }
        
        // 保存配置
        config.save()?;
        
        // 询问是否继续
        if !Confirm::new()
            .with_prompt("Continue configuring?")
            .default(true)
            .interact()? {
            break;
        }
    }
    
    println!("🎉 Configuration completed!");
    Ok(())
}

fn prompt_for_field(field: &str, current_value: &str) -> Result<String> {
    let prompt = format!("{} (current: {})", field, current_value);
    let input: String = Input::new()
        .with_prompt(&prompt)
        .default(current_value.to_string())
        .interact_text()?;
    
    Ok(input)
} 
