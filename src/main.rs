mod server;
mod utils;
mod connection;

use server::Server;

fn main() {
    let server = Server::with_address("127.0.0.1", 8080);
    server.start();
    
    server.display(false);
    loop {
        // read from stdin
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let args: Vec<&str> = input.trim().split_whitespace().collect();
        
        match args[0] {
            "disp" => server.display(true),
            "table" => server.display(false),
            "exit" => break,
            _ => println!("Unknown command"),
        }
        
        server.display(true);
        
        // wait 100 ms
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}