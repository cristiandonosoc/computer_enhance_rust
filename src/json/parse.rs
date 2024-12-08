use super::*;

#[derive(Debug, Clone)]
pub(super) struct ParseState {
    pub(super) index: usize,
    pub(super) line: usize,
    pub(super) char: usize,
}

impl ParseState {
    #[allow(dead_code)]
    fn new(index: usize) -> Self {
        ParseState {
            index,
            line: 0,
            char: 0,
        }
    }
}

impl std::fmt::Display for ParseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, char: {}", self.line + 1, self.char + 1)
    }
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

pub(super) struct ParseOutput<'a> {
    pub(super) value: JsonValue<'a>,
    // Sadly we have to copy the parse state around.
    pub(super) state: ParseState,
}

#[derive(Debug)]
pub(super) struct Error {
    pub(super) state: ParseState,
    pub(super) e: std::io::Error,
}

impl Error {
    fn new<E>(state: ParseState, e: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            state,
            e: std::io::Error::new(InvalidData, e),
        }
    }
}

pub(super) fn parse_value(bytes: &[u8], state: ParseState) -> Result<ParseOutput, Error> {
    let (token, state) = parse_token(bytes, state)?;
    parse_value_internal(bytes, token, state)
}

// This can be called as well from parse_array with the token.
fn parse_value_internal<'a>(
    bytes: &'a [u8],
    first_token: Token<'a>,
    state: ParseState,
) -> Result<ParseOutput<'a>, Error> {
    match first_token {
        Token::LeftBracket => return parse_array(bytes, state),
        Token::LeftBrace => return parse_object(bytes, state),
        Token::String(s) => {
            return Ok(ParseOutput {
                value: JsonValue::String(s),
                state,
            });
        }
        Token::Number(n) => {
            return Ok(ParseOutput {
                value: JsonValue::Number(n),
                state,
            })
        }
        Token::Bool(b) => {
            return Ok(ParseOutput {
                value: JsonValue::Bool(b),
                state,
            })
        }
        Token::Null => {
            return Ok(ParseOutput {
                value: JsonValue::Null,
                state,
            })
        }
        _ => (),
    }

    Err(Error::new(state, format!("Unexpected token {:?}", first_token)))
}

fn parse_array(bytes: &[u8], mut state: ParseState) -> Result<ParseOutput, Error> {
    // debug!("--> Parsing array at {}", state);
    let mut values = vec![];

    let prev_token = Token::Invalid;
    let mut token;
    loop {
        (token, state) = parse_token(bytes, state)?;
        match token {
            Token::RightBracket => break,
            Token::Comma => {
                if matches!(prev_token, Token::Comma) {
                    return Err(Error::new(state, "Double comma"));
                }
            }
            _ => {
                let output = parse_value_internal(bytes, token, state)?;
                values.push(output.value);
                state = output.state;
            }
        }
    }

    if matches!(prev_token, Token::Comma) {
        return Err(Error::new(state, "Comma at end of array"));
    }

    Ok(ParseOutput {
        value: JsonValue::Array(Array { values }),
        state,
    })
}

fn parse_object(bytes: &[u8], mut state: ParseState) -> Result<ParseOutput, Error> {
    // debug!("--> Parsing object at {}", state);
    let mut pairs = vec![];

    let prev_token = Token::Invalid;
    let mut token: Token;
    loop {
        (token, state) = parse_token(bytes, state)?;

        let key: &str;
        match token {
            Token::Comma => {
                if matches!(prev_token, Token::Comma) {
                    return Err(Error::new(state, "Double Comma"));
                }
                continue;
            }
            Token::RightBrace => break,
            Token::String(k) => key = k,
            _ => return Err(Error::new(state, "expected string as object key")),
        }

        if key.is_empty() {
            return Err(Error::new(state, "empty key"));
        }

        // TODO: Error check that key doesn't exist already.

        (token, state) = parse_token(bytes, state)?;
        let Token::Colon = token else {
            return Err(Error::new(state, "expected colon after object key"))?;
        };

        let output = parse_value(bytes, state)?;

        // debug!("--> Parsed pair ({:?}, {:?}) at {}", key, output.value, output.state);

        pairs.push((key, output.value));
        state = output.state;
    }

    Ok(ParseOutput {
        value: JsonValue::Object(Object { pairs }),
        state,
    })
}

fn parse_token(bytes: &[u8], state: ParseState) -> Result<(Token, ParseState), Error> {
    let (token, state) = parse_token_internal(bytes, state)?;

    // debug!("Parsed token {:?} at {}", token, state);

    Ok((token, state))
}

fn parse_token_internal(bytes: &[u8], mut state: ParseState) -> Result<(Token, ParseState), Error> {
    if state.index >= bytes.len() {
        state.index = bytes.len();
        return Ok((Token::EOF, state));
    }

    loop {
        let curr = bytes[state.index];
        state.index += 1;
        state.char += 1;
        match curr {
            // Skip whitespace.
            b' ' | b'\r' | b'\t' => continue,
            b'\n' => {
                state.line += 1;
                state.char = 0;
                continue;
            }
            b'[' => return Ok((Token::LeftBracket, state)),
            b']' => return Ok((Token::RightBracket, state)),
            b'{' => return Ok((Token::LeftBrace, state)),
            b'}' => return Ok((Token::RightBrace, state)),
            b',' => return Ok((Token::Comma, state)),
            b'"' => {
                return parse_string(bytes, state);
            }
            b't' => {
                if let Some(state) = seek(bytes, state.clone(), b"rue") {
                    return Ok((Token::Bool(true), state));
                }
                return Err(Error::new(state, "Expected \"true\""));
            }
            b'f' => {
                if let Some(state) = seek(bytes, state.clone(), b"alse") {
                    return Ok((Token::Bool(false), state));
                }
                return Err(Error::new(state, "Expected \"false\""));
            }
            b'n' => {
                if let Some(state) = seek(bytes, state.clone(), b"ull") {
                    return Ok((Token::Null, state));
                }
                return Err(Error::new(state, "Expected \"null\""));
            }
            b'-' | b'0'..=b'9' => {
                // We reparse the first number.
                state.index -= 1;
                state.char -= 1;
                return parse_number(bytes, state);
            }
            b':' => return Ok((Token::Colon, state)),
            _ => return Err(Error::new(state, "Unexpected character")),
        }
    }
}

fn seek(bytes: &[u8], mut state: ParseState, pattern: &[u8]) -> Option<ParseState> {
    if state.index + pattern.len() >= bytes.len() {
        return None;
    }

    // Go one by one in the characters searching for mismatches.
    for b in pattern {
        if bytes[state.index] != *b {
            return None;
        }

        // We assume no new lines.
        state.index += 1;
        state.char += 1;
    }

    return Some(state);
}

// IMPORTANT: The index already considers parsing the initial double quote (one past it).
fn parse_string(bytes: &[u8], mut state: ParseState) -> Result<(Token, ParseState), Error> {
    let start_index: usize = state.index;
    let mut curr: u8 = bytes[state.index];

    // TODO: Support escaping.
    let mut closed = false;
    loop {
        match curr {
            b'"' => {
                closed = true;
                break;
            }
            b'\n' => {
                state.index += 1;
                state.line += 1;
                state.char = 0;
            }
            _ => {
                state.index += 1;
                state.char += 1;
            }
        }

        if state.index >= bytes.len() {
            break;
        }

        curr = bytes[state.index];
    }

    if !closed {
        return Err(Error::new(state, "Unclosed string"));
    }

    let string: &str;
    unsafe {
        string = std::str::from_utf8_unchecked(&bytes[start_index..state.index]);
    }

    // We return the index *after* the last double quote.
    state.index += 1;
    Ok((Token::String(string), state))
}

fn parse_number(bytes: &[u8], mut state: ParseState) -> Result<(Token, ParseState), Error> {
    // TODO: Maybe implement https://arxiv.org/abs/2101.11408 if we want to be mega fast?

    let start_index: usize = state.index;

    let mut curr: u8 = bytes[state.index];

    loop {
        match curr {
            b'-' | b'+' | b'e' | b'E' | b'.' | b'0'..=b'9' => {
                state.index += 1;
                state.char += 1;
            }
            _ => break,
        }

        if state.index >= bytes.len() {
            break;
        }

        curr = bytes[state.index];
    }

    let string: &str;
    unsafe {
        string = std::str::from_utf8_unchecked(&bytes[start_index..state.index]);
    }
    let num: f64 = string
        .parse()
        .map_err(|e| Error::new(state.clone(), format!("Parsing float \"{}\": {}", string, e)))?;

    Ok((Token::Number(num), state))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_number {
        use super::*;

        #[test]
        fn test_parse_basic_integers() {
            let bytes = b"123";
            let (token, state) = parse_number(bytes, ParseState::new(0)).unwrap();
            assert_eq!(token, Token::Number(123.0));
            assert_eq!(state.index, 3);
        }

        #[test]
        fn test_parse_decimal() {
            let bytes = b"123.456";
            let (token, state) = parse_number(bytes, ParseState::new(0)).unwrap();
            assert_eq!(token, Token::Number(123.456));
            assert_eq!(state.index, 7);
        }

        #[test]
        fn test_parse_negative() {
            let bytes = b"-123.456";
            let (token, state) = parse_number(bytes, ParseState::new(0)).unwrap();
            assert_eq!(token, Token::Number(-123.456));
            assert_eq!(state.index, 8);
        }

        #[test]
        fn test_parse_scientific() {
            let bytes = b"1.23e2";
            let (token, state) = parse_number(bytes, ParseState::new(0)).unwrap();
            assert_eq!(token, Token::Number(123.0));
            assert_eq!(state.index, 6);
        }

        #[test]
        fn test_parse_error_invalid() {
            let bytes = b"12.34.56";
            assert!(parse_number(bytes, ParseState::new(0)).is_err());
        }

        #[test]
        fn test_parse_at_offset() {
            let bytes = b"abc123.456   ";
            let (token, state) = parse_number(bytes, ParseState::new(3)).unwrap();
            assert_eq!(token, Token::Number(123.456));
            assert_eq!(state.index, 10);
        }
    }

    mod parse_string {
        use super::*;

        mod string_parsing {
            use super::*;

            #[test]
            fn test_basic_string() {
                let bytes = b"\"hello\"";
                let (token, state) = parse_string(bytes, ParseState::new(1)).unwrap();
                assert_eq!(token, Token::String("hello"));
                assert_eq!(state.index, 7);
            }

            #[test]
            fn test_empty_string() {
                let bytes = b"\"\"";
                let (token, state) = parse_string(bytes, ParseState::new(1)).unwrap();
                assert_eq!(token, Token::String(""));
                assert_eq!(state.index, 2);
            }

            #[test]
            fn test_string_with_spaces() {
                let bytes = b"\"hello world\"";
                let (token, state) = parse_string(bytes, ParseState::new(1)).unwrap();
                assert_eq!(token, Token::String("hello world"));
                assert_eq!(state.index, 13);
            }

            #[test]
            fn test_string_at_offset() {
                let bytes = b"abc\"hello\"";
                let (token, state) = parse_string(bytes, ParseState::new(4)).unwrap();
                assert_eq!(token, Token::String("hello"));
                assert_eq!(state.index, 10);
            }

            #[test]
            fn test_unclosed_string() {
                let bytes = b"\"hello";
                assert!(matches!(
                    parse_string(bytes, ParseState::new(1)),
                    Err(e) if e.e.kind() == InvalidData
                ));
            }

            #[test]
            fn test_empty_input() {
                // This case is special since our code assumes we already processed the first ".
                let bytes = b"\"";
                let (token, state) = parse_string(bytes, ParseState::new(0)).unwrap();
                assert_eq!(token, Token::String(""));
                assert_eq!(state.index, 1);
            }
        }
    }

    mod cases {
        use super::*;
        use crate::get_cargo_root;

        #[test]
        fn test_simple_case() {
            let bytes = b"{
\"pairs\":[
{
\"x0\":59.80934677533466,
\"y0\":-66.7913443851547,
\"x1\":80.60590027144059,
\"y1\":-37.56716479818089
},
]
}";

            let filter = "debug";
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter))
                .init();

            let value = parse(bytes).unwrap();
            let JsonValue::Object(object) = value else {
                panic!("Expected object");
            };

            assert_eq!(object.keys(), vec!["pairs"]);

            let JsonValue::Array(pairs) = object.get("pairs").unwrap() else {
                panic!("Expected array");
            };
            assert_eq!(pairs.values.len(), 1);

            let JsonValue::Object(coord) = &pairs.values[0] else {
                panic!("Expected object");
            };

            assert_eq!(coord.keys(), vec!["x0", "y0", "x1", "y1"]);

            // Validate each numeric value
            match coord.get("x0").unwrap() {
                JsonValue::Number(n) => assert!((n - 59.80934677533466).abs() < f64::EPSILON),
                _ => panic!("x0 should be a number"),
            }

            match coord.get("y0").unwrap() {
                JsonValue::Number(n) => assert!((n - (-66.7913443851547)).abs() < f64::EPSILON),
                _ => panic!("y0 should be a number"),
            }

            match coord.get("x1").unwrap() {
                JsonValue::Number(n) => assert!((n - 80.60590027144059).abs() < f64::EPSILON),
                _ => panic!("x1 should be a number"),
            }

            match coord.get("y1").unwrap() {
                JsonValue::Number(n) => assert!((n - (-37.56716479818089)).abs() < f64::EPSILON),
                _ => panic!("y1 should be a number"),
            }
        }

        #[test]
        fn read_many_points() {
            let cargo_root = get_cargo_root().unwrap();
            let path = cargo_root.join("extras/json/coords_100.json");

            let bytes = std::fs::read(path).unwrap();
            let value = parse(&bytes).unwrap();

            let JsonValue::Array(points) = value else {
                panic!("Expected array");
            };

            assert_eq!(points.values.len(), 100);
        }
    }
}
