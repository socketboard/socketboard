use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::connection::Connection;
use crate::utils::Data;

type ExecFn = Arc<dyn Fn(&ParsedCommand, &CommandManager)>;
type HelpFn = Arc<dyn Fn()>;

pub struct ParsedCommand {
    pub command: String,
    pub flags: HashMap<String, Option<String>>,
    pub args: Vec<String>,
}

pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub exec: ExecFn,
    pub help: HelpFn,
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            aliases: self.aliases.clone(),
            description: self.description.clone(),
            exec: self.exec.clone(),
            help: self.help.clone(),
        }
    }
}

impl Command {
    pub fn new(name: &str, description: &str, exec: ExecFn, help: HelpFn) -> Self {
        if name.is_empty() {
            panic!("Command must have at least one name");
        }
        
        Self {
            name: name.to_string(),
            aliases: vec![],
            description: description.to_string(),
            exec,
            help,
        }
    }
    
    pub fn with_aliases(name: &str, names: Vec<&str>, description: &str, exec: ExecFn, help: HelpFn) -> Self {
        if names.is_empty() {
            panic!("Command must have at least one name");
        }
        
        Self {
            name: name.to_string(),
            aliases: names.iter().map(|n| n.to_string()).collect(),
            description: description.to_string(),
            exec,
            help,
        }
    }
}

pub struct CommandManager {
    pub commands: HashMap<String, Command>,
    command_map: HashMap<String, Command>,
    pub table: Arc<Mutex<HashMap<String, Data>>>,
    pub connections: Arc<Mutex<HashMap<usize, Connection>>>,
    pub address: SocketAddr,
}

impl CommandManager {
    pub fn new(
        table: &Arc<Mutex<HashMap<String, Data>>>,
        connections: &Arc<Mutex<HashMap<usize, Connection>>>,
        socket_addr: SocketAddr,
    ) -> Self {
        Self {
            command_map: HashMap::new(),
            commands: HashMap::new(),
            table: table.clone(),
            connections: connections.clone(),
            address: socket_addr,
        }
    }

    pub fn add(&mut self, command: Command) {
        self.command_map.insert(command.name.clone(), command.clone());
        for alias in &command.aliases {
            self.command_map.insert(alias.clone(), command.clone());
        }
        let name = command.name.clone();
        self.commands.insert(name, command);
    }
    
    pub fn run(&self, string: String) {
        let command = match parse_command(&string) {
            Ok(c) => c,
            Err(e) => {
                println!("Error parsing command: {}", e);
                return;
            }
        };

        self.run_parsed(command);
    }

    fn run_parsed(&self, command: ParsedCommand) {
        match self.command_map.get(&command.command) {
            Some(cmd) => { 
                if command.flags.contains_key("help") {
                    (cmd.help)();
                } else {
                    (cmd.exec)(&command, self);
                }
            },
            None => println!("Unknown command. Type 'help' for a list of commands"),
        }
    }

    pub fn read_line() -> String {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        input
    }

    pub fn no_help() {
        println!("No help available for this command");
    }
}

fn parse_command(input: &str) -> Result<ParsedCommand, String> {
    // loop over the input string
    let tokenized = match tokenize(input) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // if there are no tokens, return an empty ParsedCommand
    if tokenized.is_empty() {
        return Err("No command".to_string());
    }

    // the first token is the command
    let command = tokenized[0].clone();

    // if there is only one token, return a ParsedCommand with the command and no flags or arguments
    if tokenized.len() == 1 {
        return Ok(ParsedCommand {
            command,
            flags: HashMap::new(),
            args: Vec::new(),
        });
    }

    // the rest of the tokens are flags and arguments
    let arguments = tokenized[1..].to_vec();
    let flags = get_flags(arguments.clone());

    Ok(ParsedCommand {
        command,
        flags,
        args: arguments,
    })
}

fn get_flags(arguments: Vec<String>) -> HashMap<String, Option<String>> {
    let mut flags = HashMap::new();
    let mut current_flag = String::new();

    for arg in arguments {
        if arg.starts_with("--") || arg.starts_with("-") {
            // if there was a current flag, add that and set the current flag to the new flag
            if !current_flag.is_empty() {
                flags.insert(current_flag.clone(), None);
            }
            // remove the -- or -
            let mut arg = arg.trim_start_matches("-").to_string();
            if arg.starts_with("-") {
                arg = arg.trim_start_matches("-").to_string();
            }
            current_flag = arg.clone();
        } else if !current_flag.is_empty() {
            // if there is a current flag, add the argument to the flag and clear the current flag
            flags.insert(current_flag.clone(), Some(arg.clone()));
            current_flag.clear();
        }
    }

    if !current_flag.is_empty() {
        flags.insert(current_flag, None);
    }

    flags
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else {
            match c {
                '\\' => {
                    escaped = true;
                }
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    // if the last character was a quote, return an error
    if in_quotes {
        return Err("Unterminated quote".to_string());
    }

    Ok(args)
}