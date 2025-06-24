use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// 导入 i18n 功能
use crate::core::i18n::{
    get_language_display_name, get_supported_languages, set_language, t, validate_language,
};

pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("config")
            .about(t!("command.config.description").to_string())
            .subcommand(
                Command::new("show").about(t!("command.config.show.description").to_string()),
            )
            .subcommand(
                Command::new("set")
                    .about(t!("command.config.set.description").to_string())
                    .arg(
                        Arg::new("field")
                            .help(t!("help.config_field").to_string())
                            .required(false)
                            .index(1),
                    )
                    .arg(
                        Arg::new("value")
                            .help(t!("help.config_value").to_string())
                            .required(false)
                            .index(2),
                    ),
            ),
    );
}

pub fn handle_command(matches: &ArgMatches) -> Result<()> {
    if let Some(config_matches) = matches.subcommand_matches("config") {
        match config_matches.subcommand() {
            Some(("show", _)) => show(),
            Some(("set", set_matches)) => {
                let field = set_matches.get_one::<String>("field").map(|s| s.as_str());
                let value = set_matches.get_one::<String>("value").map(|s| s.as_str());
                set_item(field, value)
            }
            _ => {
                show() // 默认显示配置
            }
        }
    } else {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub draft_path: String,
    pub lang: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            draft_path: "/tmp/zdocs".to_string(),
            lang: "zh-Hans".to_string(),
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

    pub fn get_field(&self, field: &str) -> Result<String> {
        match field {
            "draft_path" => Ok(self.draft_path.clone()),
            "lang" => Ok(self.lang.clone()),
            _ => anyhow::bail!(t!("error.unknown_field", field = field).to_string()),
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.config_dir_not_found").to_string()))?;

    Ok(config_dir.join("xdev").join("config"))
}

pub fn show() -> Result<()> {
    let config = Config::load()?;
    let config_path = get_config_path()?;

    println!(
        "{}",
        t!("command.config.show.title", path = config_path.display())
    );
    println!("┌─────────────────┬─────────────────────────────────┐");
    println!(
        "│ {:<15} │ {:<31} │",
        t!("command.config.show.table_header_setting"),
        t!("command.config.show.table_header_value")
    );
    println!("├─────────────────┼─────────────────────────────────┤");
    println!(
        "│ {:<15} │ {:<31} │",
        t!("fields.draft_path"),
        config.draft_path
    );
    println!(
        "│ {:<15} │ {:<31} │",
        t!("fields.lang"),
        format!(
            "{} ({})",
            config.lang,
            get_language_display_name(&config.lang)
        )
    );
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
            println!(
                "{}",
                t!("command.config.set.success", field = field, value = value)
            );
        }
        (Some(field), None) => {
            // 交互式设置单个字段
            let current_value = config.get_field(field)?;
            let new_value = prompt_for_field(field, &current_value)?;

            if new_value != current_value {
                config.set_field(field, &new_value)?;
                config.save()?;
                println!(
                    "{}",
                    t!(
                        "command.config.set.updated",
                        field = field,
                        old = current_value,
                        new = new_value
                    )
                );
            } else {
                println!("{}", t!("command.config.set.no_changes", field = field));
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
    println!("{}", t!("command.config.set.interactive_title"));
    println!("{}", t!("command.config.set.interactive_prompt"));

    loop {
        let items = vec![
            format!(
                "{} - {}",
                t!("fields.draft_path"),
                t!("general.draft_path_description")
            ),
            format!("{} - {}", t!("fields.lang"), t!("command.lang.description")),
            t!("command.config.set.show_current").to_string(),
            t!("command.config.set.exit").to_string(),
        ];

        let selection = Select::new()
            .with_prompt(t!("general.choose_option").as_ref())
            .items(&items)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                let new_value = prompt_for_field("draft_path", &config.draft_path)?;
                if new_value != config.draft_path {
                    config.draft_path = new_value.clone();
                    println!(
                        "{}",
                        t!(
                            "command.config.set.updated",
                            field = "draft_path",
                            old = &config.draft_path,
                            new = &new_value
                        )
                    );
                }
            }
            1 => {
                let new_value = prompt_for_language(&config.lang)?;
                if new_value != config.lang {
                    config.set_field("lang", &new_value)?;
                    println!(
                        "{}",
                        t!(
                            "command.config.set.updated",
                            field = "lang",
                            old = &config.lang,
                            new = &new_value
                        )
                    );
                }
            }
            2 => {
                show()?;
                continue;
            }
            3 => break,
            _ => continue,
        }

        // 保存配置
        config.save()?;

        // 询问是否继续
        if !Confirm::new()
            .with_prompt(t!("command.config.set.continue_prompt").as_ref())
            .default(true)
            .interact()?
        {
            break;
        }
    }

    println!("{}", t!("command.config.set.completed"));
    Ok(())
}

fn prompt_for_field(field: &str, current_value: &str) -> Result<String> {
    if field == "lang" {
        prompt_for_language(current_value)
    } else {
        let field_name = match field {
            "draft_path" => t!("fields.draft_path").to_string(),
            _ => field.to_string(),
        };
        let prompt = t!(
            "command.lang.current_prompt",
            field = field_name,
            current = current_value
        );
        let input: String = Input::new()
            .with_prompt(prompt.as_ref())
            .default(current_value.to_string())
            .interact_text()?;

        Ok(input)
    }
}

fn prompt_for_language(current_lang: &str) -> Result<String> {
    let languages = get_supported_languages();
    let language_items: Vec<String> = languages
        .iter()
        .map(|lang| format!("{} ({})", get_language_display_name(lang), lang))
        .collect();

    let current_index = languages
        .iter()
        .position(|&lang| lang == current_lang)
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt(t!("fields.lang").as_ref())
        .items(&language_items)
        .default(current_index)
        .interact()?;

    Ok(languages[selection].to_string())
}
