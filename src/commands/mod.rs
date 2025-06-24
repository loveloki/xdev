pub mod version;
pub mod install;

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
