use parse::*;
use std::io::ErrorKind::InvalidData;

pub mod args;
mod parse;

#[allow(unused_imports)]
use log::debug;

#[derive(Debug)]
pub enum JsonValue<'a> {
    Array(Array<'a>),
    Object(Object<'a>),
    String(&'a str),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub struct Array<'a> {
    pub values: Vec<JsonValue<'a>>,
}

#[derive(Debug)]
pub struct Object<'a> {
    pub pairs: Vec<(&'a str, JsonValue<'a>)>,
}

impl<'a> Object<'a> {
    pub fn get(&'a self, want: &str) -> Option<&'a JsonValue<'a>> {
        // Go over the keys.
        for (key, value) in &self.pairs {
            if *key == want {
                return Some(value);
            }
        }

        None
    }

    pub fn keys(&self) -> Vec<&str> {
        let mut keys = Vec::with_capacity(self.pairs.len());
        for (key, _) in &self.pairs {
            keys.push(*key);
        }

        keys
    }
}

pub fn parse(bytes: &[u8]) -> Result<JsonValue, std::io::Error> {
    if bytes.is_empty() {
        return Err(std::io::Error::new(InvalidData, "Empty data"));
    }

    let state = ParseState {
        index: 0,
        line: 0,
        char: 0,
    };

    let result = parse_value(bytes, state);

    let output: ParseOutput;
    match result {
        Ok(o) => output = o,
        Err(e) => {
            return Err(std::io::Error::new(
                InvalidData,
                format!("line: {}, char: {} -> {}", e.state.line + 1, e.state.char + 1, e.e,),
            ));
        }
    }

    Ok(output.value)
}
