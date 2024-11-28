use std::cmp::max;
use std::sync::Arc;
use crate::command_manager::{Command, CommandManager, ParsedCommand};
use crate::utils;

pub fn help() -> Command {
    Command::new(
        "help",
        "Display help information",
        Arc::new(|command: &ParsedCommand, commands: &CommandManager| {
            if command.args.len() == 0 {
                println!("Call --help on a command to get specific help.\nCommands available:");
                for (_, cmd) in &commands.commands {
                    // pad the name out to 20 characters
                    let mut names = cmd.name.to_string() + ", " + &cmd.aliases.join(", ");
                    if cmd.aliases.is_empty() {
                        names = names.trim_end_matches(", ").to_string();
                    }
                    let padding = max(40 - names.len(), 3);
                    let padding = " ".repeat(padding);
                    println!("{}{}{}", names, padding, cmd.description);
                    
                }
            } else {
                let command_name = &command.args[0];
                if let Some(command) = commands.commands.get(command_name) {
                    (command.help)();
                } else {
                    println!("Command not found: {}", command_name);
                }
            }
        }),
        Arc::new(|| {
            println!("help [command?] - Display help information");
        }),
    )
}

fn display() -> Command {
    Command::new(
        "display",
        "Display the server information",
        Arc::new(|_: &ParsedCommand, server: &CommandManager| {
            // a
        }),
        Arc::new(|| {
            println!("display - Display the server information");
        }),
    )
}

pub fn clear() -> Command {
    Command::new(
        "clear",
        "Clear the console",
        Arc::new(|_: &ParsedCommand, _: &CommandManager| {
            utils::clear_screen();
        }),
        Arc::new(|| {
            println!("clear - Clear the screen");
        })
    )
}

pub fn connections() -> Command {
    Command::with_aliases(
        "connections",
        vec!["conn", "c"],
        "Display the connections",
        Arc::new(|_: &ParsedCommand, server: &CommandManager| {
            let connections = server.connections.lock().unwrap();
            if connections.len() == 0 {
                println!("No connections");
            } else {
                println!("Connections: ({})", connections.len());
            }
            for (_, connection) in &*connections {
                connection.display();
            }
        }),
        Arc::new(|| {
            println!("connections - Display the connections");
        }),
    )
}

pub fn terminate() -> Command {
    Command::new(
        "terminate",
        "Terminate a connection",
        Arc::new(|command: &ParsedCommand, server: &CommandManager| {
            // check if the id is provided
            let string_id = match command.args.get(0) {
                Some(id) => {
                    id
                }
                _ => {
                    println!("Missing id flag");
                    return;
                }
            };

            let id;
            match string_id.parse::<usize>() {
                Ok(i) => {
                    id = i;
                }
                Err(e) => {
                    println!("Invalid id: {}", e);
                    return;
                }
            }

            let connections = server.connections.lock().unwrap();
            let connection = connections.get(&id);

            if let Some(connection) = connection {
                connection.to_client.lock().unwrap().push(
                    serde_json::json!({
                        "terminate": true
                    })
                );
            } else {
                println!("Connection not found: {}", id);
            }
        }),
        Arc::new(|| {
            println!("terminate [id] - Terminate a connection");
        }),
    )
}

pub fn exit() -> Command {
    Command::new(
        "exit",
        "Exit the program",
        Arc::new(|_: &ParsedCommand, _: &CommandManager| {
            std::process::exit(0);
        }),
        Arc::new(|| {
            println!("exit - Exit the program");
        }),
    )
}