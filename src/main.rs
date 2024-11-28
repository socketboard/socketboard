mod server;
mod utils;
mod connection;
mod command_manager;
mod commands;

use server::Server;
use command_manager::CommandManager;

fn main() {
    let server = Server::with_address("127.0.0.1", 8080);
    let mut commands = CommandManager::new(&server.table, &server.connections, server.address);
    
    commands.add(commands::help());
    commands.add(commands::exit());
    commands.add(commands::clear());
    commands.add(commands::connections());
    commands.add(commands::status());
    commands.add(commands::table());
    commands.add(commands::terminate());
    
    server.start();
    
    loop {
        // wait for input from the user
        let input = CommandManager::read_line();
        commands.run(input);
        
        // wait 100 ms
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
