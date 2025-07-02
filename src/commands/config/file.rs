use crate::commands::config::model::Config;
use crate::core::i18n::{get_language_display_name, t};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.config_dir_not_found").to_string()))?;

    Ok(config_dir.join("xdev").join("config.toml"))
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
