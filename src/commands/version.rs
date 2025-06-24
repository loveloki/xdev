use anyhow::Result;

pub fn execute() -> Result<()> {
    println!("xdev {}", env!("CARGO_PKG_VERSION"));
    Ok(())
} 
