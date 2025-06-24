pub mod version;
pub mod install;
pub mod config;

use anyhow::Result;

pub fn handle_version() -> Result<()> {
    version::execute()
}

pub fn handle_install() -> Result<()> {
    install::execute()
}

pub fn handle_uninstall() -> Result<()> {
    install::uninstall()
}

pub fn handle_config_show() -> Result<()> {
    config::show()
}

pub fn handle_config_set(field: Option<&str>, value: Option<&str>) -> Result<()> {
    config::set_item(field, value)
} 
