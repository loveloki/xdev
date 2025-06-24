mod commands;

use std::process;

fn main() {
    let matches = commands::register_command().get_matches();
    
    if let Err(e) = commands::handle_command(&matches) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
