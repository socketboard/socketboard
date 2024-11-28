use std::collections::HashMap;
use std::fmt::Display;
use serde_json::Value;

pub enum Data {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Data>),
    Object(HashMap<String, Data>),
    None,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Data::String(s) => write!(f, "{}", s),
            Data::Number(n) => write!(f, "{}", n),
            Data::Boolean(b) => write!(f, "{}", b),
            Data::Array(a) => {
                write!(f, "[")?;
                for (i, data) in a.iter().enumerate() {
                    write!(f, "{}", data)?;
                    if i < a.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Data::Object(o) => {
                write!(f, "{{")?;
                for (i, (key, value)) in o.iter().enumerate() {
                    write!(f, "\"{}\": {}", key, value)?;
                    if i < o.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Data::None => write!(f, "None"),
        }
    }
}

pub trait JSON {
    fn to_json(&self) -> serde_json::Value;
    fn from_json(value: &serde_json::Value) -> Self;
}

impl JSON for Data {
    fn to_json(&self) -> Value {
        match self {
            Data::String(s) => Value::String(s.clone()),
            Data::Number(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap()),
            Data::Boolean(b) => Value::Bool(*b),
            Data::Array(a) => {
                let mut json_array = Vec::new();
                for data in a {
                    json_array.push(data.to_json());
                }
                Value::Array(json_array)
            }
            Data::Object(o) => {
                let mut json_object = serde_json::Map::new();
                for (key, value) in o {
                    json_object.insert(key.clone(), value.to_json());
                }
                Value::Object(json_object)
            }
            Data::None => Value::Null,
        }
    }
    
    fn from_json(value: &Value) -> Self {
        match value {
            Value::String(s) => Data::String(s.clone()),
            Value::Number(n) => Data::Number(n.as_f64().unwrap()),
            Value::Bool(b) => Data::Boolean(*b),
            Value::Array(a) => {
                let mut data_array = Vec::new();
                for json_value in a {
                    data_array.push(Data::from_json(json_value));
                }
                Data::Array(data_array)
            }
            Value::Object(o) => {
                let mut data_object = HashMap::new();
                for (key, json_value) in o {
                    data_object.insert(key.clone(), Data::from_json(json_value));
                }
                Data::Object(data_object)
            }
            Value::Null => Data::None,
        }
    }
}

pub fn clear_lines(n: u16) {
    println!("\x1b[{}A\x1b[J", n);
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}