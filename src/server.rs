// server
// - connect
// - loop that creates new connections

// connection
// - send messages

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use crate::connection::Connection;
use crate::utils::Data;

pub struct Server {
    address: SocketAddr,
    pub connections: Arc<Mutex<HashMap<usize, Connection>>>,
    pub table: Arc<Mutex<HashMap<String, Data>>>,
    debug: bool,
}

impl Server {
    pub fn new() -> Self {
        let address = SocketAddr::from(([127, 0, 0, 1], 8080));
        let connections = Arc::new(Mutex::new(HashMap::new()));
        let table = Arc::new(Mutex::new(HashMap::new()));
        Self {
            address,
            connections,
            table,
            debug: false,
        }
    }
    
    pub fn with_address(ip: &str, port: u16) -> Self {
        let address = SocketAddr::new(IpAddr::from_str(ip).unwrap(), port);
        let connections = Arc::new(Mutex::new(HashMap::new()));
        let table = Arc::new(Mutex::new(HashMap::new()));
        Self {
            address,
            connections,
            table,
            debug: false,
        }
    }
    
    pub fn debug_on(&mut self) {
        self.debug = true;
    }

    pub fn start(&self) {
        println!("Server started on {}", self.address);
        let listener = TcpListener::bind(self.address).unwrap();
        let connections = self.connections.clone();
        let table = self.table.clone();
        
        // spawn a new thread to accept new connections
        std::thread::spawn(move || {
            for (id, stream) in listener.incoming().enumerate() {
                let mut connection = Connection::new(id);

                match stream {
                    Ok(stream) => {
                        connection.watch(&mut stream.try_clone().unwrap(), table.clone(), connections.clone());
                    }
                    Err(e) => {
                        println!("Failed to accept connection: {}", e);
                    }
                }

                connections.lock().unwrap().insert(id, connection);
            }
        });
    }
    
    pub fn send(&self, id: usize, value: Value) {
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get_mut(&id) {
            connection.send(&value);
        }
    }
    
    pub fn broadcast(&self, value: Value) {
        let mut connections = self.connections.lock().unwrap();
        for connection in connections.values_mut() {
            connection.send(&value);
        }
    }
    
    pub fn display(&self, clear: bool) {
        let connections = self.connections.lock().unwrap();
        let table = self.table.lock().unwrap();
        
        if clear {
            crate::utils::clear_lines(table.len() as u16 + 100);
        }
        println!("----- INFO ------");
        println!("Address: {}", self.address);
        
        // display the connections
        println!("Connections: {}", connections.len());
        for (_, connection) in &*connections {
            connection.display();
        }
        
        // display the table
        println!("----- TABLE -----");
        for (key, value) in &*table {
            println!("{}        {}", key, value);
        }
        println!("-----------------");
    }
}