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

#[derive(Debug, Clone, PartialEq)]
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

fn parse_token(bytes: &[u8], mut index: usize) -> Result<(Token, usize), Error> {
    if index >= bytes.len() {
        return Ok((Token::EOF, bytes.len()));
    }

    loop {
        let curr = bytes[index];
        index += 1;
        match curr {
            // Skip whitespace.
            b' ' | b'\r' | b'\t' | b'\n' => continue,
            b'[' => return Ok((Token::LeftBracket, index)),
            b']' => return Ok((Token::RightBracket, index)),
            b'{' => return Ok((Token::LeftBrace, index)),
            b'}' => return Ok((Token::RightBrace, index)),
            b',' => return Ok((Token::Comma, index)),
            b'"' => {
                return parse_string(bytes, index);
            }
            b't' => {
                if let Some(index) = seek(bytes, index, b"rue") {
                    return Ok((Token::Bool(true), index));
                }
                return Err(Error::new(InvalidData, "Expected \"true\""));
            }
            b'f' => {
                if let Some(index) = seek(bytes, index, b"alse") {
                    return Ok((Token::Bool(false), index));
                }
                return Err(Error::new(InvalidData, "Expected \"false\""));
            }
            b'n' => {
                if let Some(index) = seek(bytes, index, b"ull") {
                    return Ok((Token::Null, index));
                }
                return Err(Error::new(InvalidData, "Expected \"null\""));
            }
            b'-' | b'0'..=b'9' => {
                // We reparse the first number.
                return parse_number(bytes, index - 1);
            }
            b':' => return Ok((Token::Colon, index)),
            _ => return Err(Error::new(InvalidData, "Unexpected character")),
        }
    }
}

fn seek(bytes: &[u8], mut index: usize, pattern: &[u8]) -> Option<usize> {
    if index + pattern.len() >= bytes.len() {
        return None;
    }

    // Go one by one in the characters searching for mismatches.
    for b in pattern {
        if bytes[index] != *b {
            return None;
        }

        index += 1
    }

    return Some(index);
}

// IMPORTANT: The index already considers parsing the initial double quote (one past it).
fn parse_string(bytes: &[u8], mut index: usize) -> Result<(Token, usize), Error> {
    let start_index: usize = index;
    let mut curr: u8 = bytes[index];

    // TODO: Support escaping.
    let mut closed = false;
    loop {
        match curr {
            b'"' => {
                closed = true;
                break;
            }
            _ => index += 1,
        }

        if index >= bytes.len() {
            break;
        }

        curr = bytes[index];
    }

    if !closed {
        return Err(Error::new(InvalidData, "Unclosed string"));
    }

    let string: &str;
    unsafe {
        string = std::str::from_utf8_unchecked(&bytes[start_index..index]);
    }

    // We return the index *after* the last double quote.
    Ok((Token::String(string), index + 1))
}

fn parse_number(bytes: &[u8], mut index: usize) -> Result<(Token, usize), Error> {
    // TODO: Maybe implement https://arxiv.org/abs/2101.11408 if we want to be mega fast?

    let start_index: usize = index;

    let mut curr: u8 = bytes[index];

    loop {
        match curr {
            b'-' | b'+' | b'e' | b'E' | b'.' | b'0'..=b'9' => index += 1,
            _ => break,
        }

        if index >= bytes.len() {
            break;
        }

        curr = bytes[index];
    }

    let string: &str;
    unsafe {
        string = std::str::from_utf8_unchecked(&bytes[start_index..index]);
    }
    let num: f64 = string
        .parse()
        .map_err(|e| Error::new(InvalidData, format!("Parsing float \"{}\": {}", string, e)))?;

    Ok((Token::Number(num), index))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_number {
        use super::*;

        #[test]
        fn test_parse_basic_integers() {
            let bytes = b"123";
            let (token, index) = parse_number(bytes, 0).unwrap();
            assert_eq!(token, Token::Number(123.0));
            assert_eq!(index, 3);
        }

        #[test]
        fn test_parse_decimal() {
            let bytes = b"123.456";
            let (token, index) = parse_number(bytes, 0).unwrap();
            assert_eq!(token, Token::Number(123.456));
            assert_eq!(index, 7);
        }

        #[test]
        fn test_parse_negative() {
            let bytes = b"-123.456";
            let (token, index) = parse_number(bytes, 0).unwrap();
            assert_eq!(token, Token::Number(-123.456));
            assert_eq!(index, 8);
        }

        #[test]
        fn test_parse_scientific() {
            let bytes = b"1.23e2";
            let (token, index) = parse_number(bytes, 0).unwrap();
            assert_eq!(token, Token::Number(123.0));
            assert_eq!(index, 6);
        }

        #[test]
        fn test_parse_error_invalid() {
            let bytes = b"12.34.56";
            assert!(parse_number(bytes, 0).is_err());
        }

        #[test]
        fn test_parse_at_offset() {
            let bytes = b"abc123.456   ";
            let (token, index) = parse_number(bytes, 3).unwrap();
            assert_eq!(token, Token::Number(123.456));
            assert_eq!(index, 10);
        }
    }

    mod parse_string {
        use super::*;

        mod string_parsing {
            use super::*;

            #[test]
            fn test_basic_string() {
                let bytes = b"\"hello\"";
                let (token, index) = parse_string(bytes, 1).unwrap();
                assert_eq!(token, Token::String("hello"));
                assert_eq!(index, 7);
            }

            #[test]
            fn test_empty_string() {
                let bytes = b"\"\"";
                let (token, index) = parse_string(bytes, 1).unwrap();
                assert_eq!(token, Token::String(""));
                assert_eq!(index, 2);
            }

            #[test]
            fn test_string_with_spaces() {
                let bytes = b"\"hello world\"";
                let (token, index) = parse_string(bytes, 1).unwrap();
                assert_eq!(token, Token::String("hello world"));
                assert_eq!(index, 13);
            }

            #[test]
            fn test_string_at_offset() {
                let bytes = b"abc\"hello\"";
                let (token, index) = parse_string(bytes, 4).unwrap();
                assert_eq!(token, Token::String("hello"));
                assert_eq!(index, 10);
            }

            #[test]
            fn test_unclosed_string() {
                let bytes = b"\"hello";
                assert!(matches!(
                    parse_string(bytes, 1),
                    Err(e) if e.kind() == InvalidData
                ));
            }

            #[test]
            fn test_empty_input() {
                // This case is special since our code assumes we already processed the first ".
                let bytes = b"\"";
                let (token, index) = parse_string(bytes, 0).unwrap();
                assert_eq!(token, Token::String(""));
                assert_eq!(index, 1);
            }
        }
    }

    //mod cases {
    //    use super::*;

    //    #[test]
    //    fn test_simple_case() {
    //        let bytes = b"
    //        {
    //            \"pairs\":[
    //                {
    //                    \"X0\":59.80934677533466,
    //                    \"Y0\":-66.7913443851547,
    //                    \"X1\":80.60590027144059,
    //                    \"Y1\":-37.56716479818089
    //                },
    //            ]
    //        }";

    //        let value = parse(bytes).unwrap();
    //        let JsonValue::Object(object) = value else {
    //            panic!("Expected object");
    //        };

    //        assert_eq!(object.keys(), vec!["pairs"]);
    //    }
    //}
}
