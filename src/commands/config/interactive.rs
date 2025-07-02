use crate::commands::config::{file::show, model::Config};
use crate::core::i18n::{get_language_display_name, get_supported_languages, t};
use anyhow::Result;
use inquire::{Confirm, Select, Text};

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

        let selection = Select::new(t!("general.choose_option").as_ref(), items)
            .with_starting_cursor(0)
            .prompt()?;

        // 根据选中的字符串匹配操作
        if selection.starts_with(&t!("fields.draft_path").to_string()) {
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
        } else if selection.starts_with(&t!("fields.lang").to_string()) {
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
        } else if selection == t!("command.config.set.show_current") {
            show()?;
            continue;
        } else if selection == t!("command.config.set.exit") {
            break;
        }

        // 保存配置
        config.save()?;

        // 询问是否继续
        if !Confirm::new(t!("command.config.set.continue_prompt").as_ref())
            .with_default(true)
            .prompt()?
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
        let input: String = Text::new(prompt.as_ref())
            .with_default(current_value)
            .prompt()?;

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

    let selection = Select::new(t!("fields.lang").as_ref(), language_items)
        .with_starting_cursor(current_index)
        .prompt()?;

    // 从选中的显示文本中提取语言代码 (格式: "Display Name (code)")
    if let Some(start) = selection.rfind('(')
        && let Some(end) = selection.rfind(')')
        && start < end
    {
        return Ok(selection[start + 1..end].to_string());
    }

    // 如果解析失败，返回默认语言
    Ok("zh-Hans".to_string())
}
