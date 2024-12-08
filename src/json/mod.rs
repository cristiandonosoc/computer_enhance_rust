use std::io::{Error, ErrorKind::InvalidData};

#[derive(Debug)]
pub enum JsonValue<'a> {
    Array { values: Vec<JsonValue<'a>> },
    Object(Object<'a>),
    String(&'a str),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub struct Object<'a> {
    pub pairs: Vec<(&'a str, JsonValue<'a>)>,
}

impl<'a> Object<'a> {
    pub fn get<'b>(&'a self, want: &'b str) -> Option<&'a JsonValue<'a>> {
        // Go over the keys.
        for (key, value) in &self.pairs {
            if *key == want {
                return Some(value);
            }
        }

        None
    }
}

pub fn parse(bytes: &[u8]) -> Result<JsonValue, Error> {
    if bytes.is_empty() {
        return Err(Error::new(InvalidData, "Empty data"));
    }

    let output = parse_value(bytes, 0)?;

    let (eof_token, index) = parse_token(bytes, output.index)?;
    if !matches!(eof_token, Token::EOF) {
        return Err(Error::new(InvalidData, "Could not parse until EOF"));
    }

    if index != bytes.len() {
        return Err(Error::new(InvalidData, "Did not parse all the data"));
    }

    Ok(output.value)
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Invalid,
    EOF,
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Colon,        // :
    Comma,        // ,
    String(&'a str),
    Integer,
    Number(f64),
    Bool(bool),
    Null,
}

struct ParseOutput<'a> {
    value: JsonValue<'a>,
    // The index of the current parsing after parsing the value.
    index: usize,
}

fn parse_value(bytes: &[u8], index: usize) -> Result<ParseOutput, Error> {
    let (token, index) = parse_token(bytes, index)?;
    match token {
        Token::LeftBrace => return parse_array(bytes, index),
        Token::LeftBracket => return parse_object(bytes, index),
        Token::String(s) => {
            return Ok(ParseOutput {
                value: JsonValue::String(s),
                index,
            });
        }
        Token::Number(n) => {
            return Ok(ParseOutput {
                value: JsonValue::Number(n),
                index,
            })
        }
        Token::Bool(b) => {
            return Ok(ParseOutput {
                value: JsonValue::Bool(b),
                index,
            })
        }
        Token::Null => {
            return Ok(ParseOutput {
                value: JsonValue::Null,
                index,
            })
        }
        _ => (),
    }

    Err(Error::new(InvalidData, "Unexpected token"))
}

fn parse_array(bytes: &[u8], mut index: usize) -> Result<ParseOutput, Error> {
    let mut values = vec![];

    let prev_token = Token::Invalid;
    let mut token;
    loop {
        (token, index) = parse_token(bytes, index)?;
        match token {
            Token::RightBracket => break,
            Token::Comma => {
                if matches!(prev_token, Token::Comma) {
                    return Err(Error::new(InvalidData, "Double comma"));
                }
            }
            _ => {
                let output = parse_value(bytes, index)?;
                values.push(output.value);
                index = output.index;
            }
        }
    }

    if matches!(prev_token, Token::Comma) {
        return Err(Error::new(InvalidData, "Comma at end of array"));
    }

    Ok(ParseOutput {
        value: JsonValue::Array { values },
        index,
    })
}

fn parse_object(bytes: &[u8], mut index: usize) -> Result<ParseOutput, Error> {
    let mut pairs = vec![];

    let prev_token = Token::Invalid;
    let mut token: Token;
    loop {
        (token, index) = parse_token(bytes, index)?;

        let key: &str;
        match token {
            Token::Comma => {
                if matches!(prev_token, Token::Comma) {
                    return Err(Error::new(InvalidData, "Double Comma"));
                }
                continue;
            }
            Token::RightBracket => break,
            Token::String(k) => key = k,
            _ => return Err(Error::new(InvalidData, "expected string as object key")),
        }

        if key.is_empty() {
            return Err(Error::new(InvalidData, "empty key"));
        }

        // TODO: Error check that key doesn't exist already.

        (token, index) = parse_token(bytes, index)?;
        let Token::Colon = token else {
            return Err(Error::new(InvalidData, "expected colon after object key"))?;
        };

        let output = parse_value(bytes, index)?;
        pairs.push((key, output.value));
        index = output.index;
    }

    Ok(ParseOutput {
        value: JsonValue::Object(Object { pairs }),
        index,
    })
}

fn parse_token(bytes: &[u8], index: usize) -> Result<(Token, usize), Error> {
    todo!("Parse stuff")
}

const LEFT_BRACKET: u8 = b'[';
const RIGHT_BRACKET: u8 = b']';
const LEFT_BRACE: u8 = b'{';
const RIGHT_BRACE: u8 = b'}';
const COMMA: u8 = b',';
