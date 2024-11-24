use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use crate::utils::{Data, JSON};

pub struct Connection {
    pub to_client: Arc<Mutex<Vec<Value>>>,
    pub to_server: Arc<Mutex<Vec<Value>>>,
    name: Arc<Mutex<String>>,
    id: usize
}

impl Connection {
    pub fn new(
        id: usize,
    ) -> Self {
        Self {
            to_client: Arc::new(Mutex::new(Vec::new())),
            to_server: Arc::new(Mutex::new(Vec::new())),
            name: Arc::new(Mutex::new(String::new())),
            id
        }
    }
    
    pub fn watch(
        &mut self,
        stream: &mut TcpStream,
        table: Arc<Mutex<HashMap<String, Data>>>,
        connections: Arc<Mutex<HashMap<usize, Connection>>>,
    ) {
        // new thread
        let table = table.clone();
        let connections = connections.clone();
        let to_client = self.to_client.clone();
        let to_server = self.to_server.clone();
        let name = self.name.clone();
        let id = self.id;
        let mut stream = stream.try_clone().unwrap();
        
        stream.set_nonblocking(true).unwrap();
        
        std::thread::spawn(move || {
            println!("New connection with id: {}", id);
            
            let mut handshake = false;

            fn send(json_value: &Value, message_buffer: &Arc<Mutex<Vec<Value>>>) {
                let mut buffer = message_buffer.lock().unwrap();
                buffer.push(json_value.clone());
            }
            
            loop {
                // read from stream
                match Connection::read(&mut stream) {
                    Ok(result) => match result {
                        Some(json) => {
                            match Connection::handle(
                                &mut handshake,
                                &json,
                                &table,
                                &to_client,
                                &to_server,
                                &name,
                                &id,
                                &mut send,
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Failed to handle: {}", e);
                                    break;
                                }
                            }
                        }
                        None => {}
                    }
                    Err(e) => {
                        println!("Failed to read: {}", e);
                        break;
                    }
                }
                
                for connection in connections.lock().unwrap().values() {
                    let mut messages = connection.to_server.lock().unwrap();
                    Connection::write(&mut stream, &mut messages);
                }
                to_server.lock().unwrap().clear();
                
                Connection::write(&mut stream, &mut to_client.lock().unwrap());
            }
            
            let mut connections = connections.lock().unwrap();
            connections.remove(&id);
        });
    }
    
    pub fn get_name(&self) -> String {
        let name = self.name.lock().unwrap();
        name.clone()
    }
    
    pub fn display(&self) {
        if self.name.lock().unwrap().is_empty() {
            println!("Connection ({})", self.id);
        } else {
            println!("{} ({})", self.id, self.name.lock().unwrap());
        }
    }

    pub fn send(&mut self, json_value: &Value) {
        let mut buffer = self.to_client.lock().unwrap();
        buffer.push(json_value.clone());
    }
    
    fn handle(
        handshake: &mut bool,
        json: &Value,
        server_table: &Arc<Mutex<HashMap<String, Data>>>,
        to_client: &Arc<Mutex<Vec<Value>>>,
        to_server: &Arc<Mutex<Vec<Value>>>,
        name: &Arc<Mutex<String>>,
        id: &usize,
        send: &mut dyn FnMut(&Value, &Arc<Mutex<Vec<Value>>>),
    ) -> Result<(), Error> {
        // println!("handle");
        // check if the JSON object has a type
        let response_type = match json.get("type") {
            Some(response_type) => response_type.as_str().unwrap(),
            None => "",
        };
        
        // if there's no response type, return an error
        if response_type.is_empty() {
            return Err(Error::new(ErrorKind::Other, "No response type"));
        }
        
        match response_type {
            "handshake" => {
                if !*handshake {
                    *handshake = true;
                    
                    let json_name = json.get("name").unwrap().as_str().unwrap();
                    
                    // if json_name includes any non-alphanumeric characters, return an error
                    if !json_name.chars().all(|c| c.is_alphanumeric()) {
                        // send a response
                        let response = json!({
                            "type": "handshake",
                            "status": "error",
                            "message": "Invalid client name"
                        });
                        
                        send(&response, to_client);
                        return Err(Error::new(ErrorKind::Other, "Invalid name"));
                    }
                    
                    let mut name = name.lock().unwrap();
                    *name = json_name.to_string();
                    
                    // send the server table
                    let table = Value::Object(server_table.lock().unwrap().iter().map(|(key, value)| {
                        (key.clone(), value.to_json())
                    }).collect());
                    
                    // send a response
                    let response = json!({
                        "type": "handshake",
                        "status": "ok",
                        "id": id,
                        "table": table
                    });
                    
                    send(&response, to_client);
                    
                    return Ok(());
                }
                
                Err(Error::new(ErrorKind::Other, "Handshake already completed"))
            }
            "update" => {
                // get the table from the JSON object
                match json.get("table") {
                    Some(table) => {
                        // iterate over the table
                        let table = table.as_object().unwrap();
                        for (key, value) in table.iter() {
                            // update the server table
                            server_table.lock().unwrap().insert(key.clone(), Data::from_json(value));
                        }
                        
                        send(&json!({
                            "type": "update",
                            "status": "ok",
                            "table": Value::Object(table.clone())
                        }), to_server);
                        
                        Ok(())
                    }
                    None => Err(Error::new(ErrorKind::Other, "No table in JSON object"))
                }
            }
            _ => {
                Err(Error::new(ErrorKind::Other, "Invalid response type"))
            }
        }
    }

    fn read(
        stream: &mut TcpStream,
    ) -> Result<Option<Value>, Error>{
        let mut buffer = [0; 2048];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                let json_string = String::from_utf8_lossy(&buffer[..bytes_read]);
                match serde_json::from_str(&json_string) {
                    Ok(json) => {
                        Ok(Some(json))
                    }
                    Err(e) => {
                        println!("Failed to parse JSON: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(_) => {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read from stream"))
            }
        }
    }

    fn write(
        stream: &mut TcpStream,
        message_buffer: &mut Vec<Value>,
    ) {
        while !message_buffer.is_empty() {
            let json_value = message_buffer.remove(0);
            let json_string = &json_value.to_string();
            let bytes = json_string.as_bytes();
            
            // println!("Sending: {}", json_string);

            match stream.write_all(bytes) {
                Ok(_) => {
                    // println!("Sent: {}", json_string);
                }
                Err(e) => {
                    println!("Failed to send: {}", e);
                    break;
                }
            }
        }
    }
}