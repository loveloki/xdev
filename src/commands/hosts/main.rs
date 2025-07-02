use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use crate::commands::hosts::{
    handle_backup, handle_list, handle_restore, handle_subscribe, handle_unsubscribe, handle_update,
};
use crate::core::i18n::t;

/// 注册 hosts 命令及其所有子命令
pub fn register_command(app: &mut Command) {
    *app = app.clone().subcommand(
        Command::new("hosts")
            .about(t!("command.hosts.description").to_string())
            .subcommand(
                Command::new("subscribe")
                    .about(t!("command.hosts.subscribe.description").to_string())
                    .arg(
                        Arg::new("url")
                            .help(t!("help.hosts_url").to_string())
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("unsubscribe")
                    .about(t!("command.hosts.unsubscribe.description").to_string())
                    .arg(
                        Arg::new("url")
                            .help(t!("help.hosts_url").to_string())
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("list").about(t!("command.hosts.list.description").to_string()),
            )
            .subcommand(
                Command::new("update").about(t!("command.hosts.update.description").to_string()),
            )
            .subcommand(
                Command::new("backup").about(t!("command.hosts.backup.description").to_string()),
            )
            .subcommand(
                Command::new("restore")
                    .about(t!("command.hosts.restore.description").to_string())
                    .arg(
                        Arg::new("backup_file")
                            .help(t!("help.backup_file").to_string())
                            .required(false)
                            .index(1),
                    ),
            ),
    );
}

/// 执行 hosts 命令
pub fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("subscribe", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").unwrap();
            handle_subscribe(url)
        }
        Some(("unsubscribe", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").unwrap();
            handle_unsubscribe(url)
        }
        Some(("list", _)) => handle_list(),
        Some(("update", _)) => handle_update(),
        Some(("backup", _)) => handle_backup(),
        Some(("restore", sub_matches)) => {
            let backup_file = sub_matches.get_one::<String>("backup_file");
            handle_restore(backup_file.map(|s| s.as_str()))
        }
        _ => {
            println!("{}", t!("command.hosts.help_message"));
            Ok(())
        }
    }
}
