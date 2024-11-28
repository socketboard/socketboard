use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
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
                        Some(json_values) => {
                            for json in json_values {
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
                        }
                        None => {}
                    }
                    // connection aborted
                    Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionAborted => {
                        stream.shutdown(Shutdown::Both).unwrap();
                        println!("Connection aborted: ({}) {}", name.lock().unwrap().to_string(), id);
                        break;
                    }
                    Err(e) => {
                        // send last messages
                        let mut to_client = to_client.lock().unwrap();
                        let _ = Connection::write(&mut stream, &mut to_client);
                        
                        stream.shutdown(Shutdown::Both).unwrap();
                        
                        println!("Failed to read: {}", e.to_string());
                        break;
                    }
                }
                
                for connection in connections.lock().unwrap().values() {
                    let mut messages = connection.to_server.lock().unwrap();
                    Connection::write(&mut stream, &mut messages).unwrap();
                }
                to_server.lock().unwrap().clear();
                
                match Connection::write(&mut stream, &mut to_client.lock().unwrap()) {
                    Err(ref e) if e.kind() == ErrorKind::ConnectionAborted => {
                        println!("Connection aborted: ({}) {}", name.lock().unwrap().to_string(), id);
                        break;
                    }
                    _ => {}
                }
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
            println!("{} (id: {})", self.name.lock().unwrap(), self.id);
        }
    }

    pub fn send(&mut self, json_value: &Value) {
        let mut buffer = self.to_client.lock().unwrap();
        buffer.push(json_value.clone());
    }
    
    pub fn terminate(&mut self) {
        let mut buffer = self.to_client.lock().unwrap();
        buffer.push(json!({
            "terminate": true
        }));
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
                            "message": "Invalid client name",
                            "terminate": true
                        });
                        
                        send(&response, to_client);
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
    ) -> Result<Option<Vec<Value>>, Error>{
        let mut buffer = [0; 2048];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                let json_string = String::from_utf8_lossy(&buffer[..bytes_read]);
                // sometimes the client sends too many messages at once and they're read like:
                // { ... }{ ... }{ ... }
                // split this into separate messages and return a vector of JSON objects
                let json_strings: Vec<&str> = json_string.split("}{").collect();
                
                let mut json_values = Vec::new();
                
                for string in json_strings {
                    let mut json_string = string.to_string();
                    // if the string is empty, skip it
                    if json_string.is_empty() {
                        continue;
                    }
                    
                    // if the string doesn't start with a {, add one
                    if !json_string.starts_with("{") {
                        json_string = format!("{}{}", "{".to_string(), json_string);
                    }
                    // if the string doesn't end with a }, add one
                    if !json_string.ends_with("}") {
                        json_string = format!("{}{}", json_string, "}".to_string());
                    }
                    
                    match serde_json::from_str(&json_string) {
                        Ok(json) => {
                            json_values.push(json);
                        }
                        Err(e) => {
                            println!("Failed to parse JSON: {}", e);
                            println!("JSON: {}", json_string);
                        }
                    }
                }
                
                Ok(Some(json_values))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(_) => {
                Err(Error::new(ErrorKind::ConnectionAborted, "Failed to read from stream"))
            }
        }
    }

    fn write(
        stream: &mut TcpStream,
        message_buffer: &mut Vec<Value>,
    ) -> Result<(), Error> {
        while !message_buffer.is_empty() {
            let json_value = message_buffer.remove(0);
            let json_string = &json_value.to_string();
            
            let bytes = json_string.as_bytes();
            
            match stream.write_all(bytes) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
            
            // if there is a terminate: true, terminate the stream
            if json_value.get("terminate") == Some(&Value::Bool(true)) {
                println!("Terminating stream");
                stream.shutdown(Shutdown::Both)?;
                return Err(Error::new(ErrorKind::ConnectionAborted, "Terminating stream"));
            }
        }
        
        Ok(())
    }
}