use crate::commands::config::model::Config;
use crate::core::globals::APP_NAME;
use crate::core::i18n::t;
use crate::core::table::{add_table_row, create_config_table, print_table, set_table_header};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!(t!("error.config_dir_not_found").to_string()))?;

    Ok(config_dir.join(APP_NAME).join("config.toml"))
}

pub fn show() -> Result<()> {
    let config = Config::load()?;
    let config_path = get_config_path()?;

    println!(
        "{}",
        t!("command.config.show.title", path = config_path.display())
    );

    let mut table = create_config_table();
    set_table_header(
        &mut table,
        vec![
            t!("command.config.show.table_header_setting").into_owned(),
            t!("command.config.show.table_header_value").into_owned(),
        ],
    );
    add_table_row(
        &mut table,
        vec![
            "draft_path".to_string(),
            config.draft_path.clone(),
        ],
    );
    add_table_row(
        &mut table,
        vec![
            "lang".to_string(),
            config.lang.clone(),
        ],
    );
    print_table(&table);

    Ok(())
}
