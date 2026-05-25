use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RESPError {
    OutOfBounds(usize),
    Expected { expected: u8, found: u8 },
    InvalidPrefix(u8),
    InvalidCommand(String),
    InvalidSize(i32),
    ParseSize(String),
    CommandError,
}
type RESPResult<T> = Result<T, RESPError>;

#[derive(Debug, PartialEq)]
pub enum RESPType {
    BulkString(String),
    NullString,
    SimpleString(String),
    Array(Vec<RESPType>),
}
impl RESPType {
    fn parse<'a>(buffer: &'a [u8], idx: &mut usize) -> RESPResult<Self> {
        match buffer.get(*idx) {
            Some(b'*') => {
                *idx += 1;
                let size = {
                    let chars = extract_line(buffer, idx)?;
                    String::from_utf8_lossy(chars)
                };

                match size.parse::<usize>() {
                    Ok(size) => {
                        let mut arr = Vec::<RESPType>::new();

                        for _ in 0..size {
                            arr.push(RESPType::parse(buffer, idx)?);
                        }

                        Ok(RESPType::Array(arr))
                    }
                    Err(_) => Err(RESPError::ParseSize(size.to_string())),
                }
            }
            Some(b'$') => {
                *idx += 1;
                let size = {
                    let chars = extract_line(buffer, idx)?;
                    String::from_utf8_lossy(chars)
                };

                match size.parse::<i32>() {
                    Ok(size) => {
                        if size >= 0 {
                            let size = size as usize;
                            let chars = extract_bytes(buffer, idx, size)?;

                            *idx += 2; // skip \r\n

                            Ok(RESPType::BulkString(
                                String::from_utf8(chars.to_vec()).unwrap(),
                            ))
                        } else if size == -1 {
                            Ok(RESPType::NullString)
                        } else {
                            Err(RESPError::InvalidSize(size))
                        }
                    }
                    Err(_) => Err(RESPError::ParseSize(size.to_string())),
                }
            }
            Some(b'+') => {
                *idx += 1;
                let s = extract_line(buffer, idx)?;
                Ok(RESPType::SimpleString(
                    String::from_utf8(s.to_vec()).unwrap(),
                ))
            }
            Some(other) => return Err(RESPError::InvalidPrefix(*other)),
            None => return Err(RESPError::OutOfBounds(*idx)),
        }
    }
}
impl fmt::Display for RESPType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BulkString(s) => write!(f, "${}\r\n{}\r\n", s.len(), s),
            Self::NullString => write!(f, "$-1\r\n"),
            Self::SimpleString(s) => write!(f, "+{}\r\n", s),
            Self::Array(arr) => {
                write!(
                    f,
                    "*{}\r\n{}",
                    arr.len(),
                    arr.iter()
                        .map(|ty| format!("{ty}"))
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Echo(String),
}
impl Command {
    pub fn parse(request: Vec<RESPType>) -> RESPResult<Self> {
        // println!("{request:?}");
        let mut idx = 0;
        match request.get(idx) {
            Some(RESPType::BulkString(s)) => match &s.to_uppercase()[..] {
                "PING" => Ok(Command::Ping),
                "ECHO" => {
                    idx += 1;
                    match request.get(idx) {
                        Some(RESPType::BulkString(s)) => Ok(Command::Echo(s.to_string())),
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::OutOfBounds(idx)),
                    }
                }
                _ => Err(RESPError::InvalidCommand(s.to_string())),
            },
            Some(_) => unimplemented!(),
            None => return Err(RESPError::OutOfBounds(idx)),
        }
    }
}

pub fn parse_input(buffer: &[u8]) -> RESPResult<Command> {
    let mut idx = 0;

    let request = match RESPType::parse(buffer, &mut idx) {
        Ok(RESPType::Array(arr)) => arr,
        Ok(_) => return Err(RESPError::CommandError),
        Err(e) => return Err(e),
    };

    let cmd = Command::parse(request)?;
    Ok(cmd)
}

fn extract_line<'a>(buffer: &'a [u8], idx: &mut usize) -> RESPResult<&'a [u8]> {
    let mut end_idx = *idx;

    while end_idx < buffer.len() {
        let c = buffer[end_idx];
        if end_idx > *idx && c == b'\n' && buffer[end_idx - 1] == b'\r' {
            end_idx -= 1;
            break;
        }
        end_idx += 1;
    }

    if end_idx >= buffer.len() {
        *idx = end_idx;
        Err(RESPError::OutOfBounds(end_idx))
    } else {
        let line = &buffer[*idx..end_idx];
        *idx = end_idx + 2;
        Ok(line)
    }
}

fn extract_bytes<'a>(buffer: &'a [u8], idx: &mut usize, size: usize) -> RESPResult<&'a [u8]> {
    match buffer.get(*idx..*idx + size) {
        Some(arr) => {
            *idx += size;
            Ok(arr)
        }
        None => Err(RESPError::OutOfBounds(*idx + size)),
    }
}

#[cfg(test)]
mod test {
    use core::panic;
    use std::marker::PhantomPinned;

    use super::*;

    #[test]
    fn parse_input_ping() {
        let buffer = "*1\r\n$4\r\nPING\r\n".as_bytes();

        let cmd = parse_input(buffer).unwrap();

        assert_eq!(cmd, Command::Ping);
    }

    #[test]
    fn parse_input_echo() {
        let buffer = "*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n".as_bytes();

        let cmd = parse_input(buffer).unwrap();

        assert_eq!(cmd, Command::Echo("hello".to_string()));
    }

    #[test]
    fn parse_input_not_array() {
        let buffer = "$4\r\nPING\r\n".as_bytes();

        match parse_input(buffer) {
            Err(e) => assert_eq!(e, RESPError::CommandError),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut idx = 0;

        let ty = RESPType::parse(buffer, &mut idx).unwrap();

        assert_eq!(ty, RESPType::SimpleString("OK".to_string()));
    }

    #[test]
    fn parse_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut idx = 0;

        let ty = RESPType::parse(buffer, &mut idx).unwrap();

        assert_eq!(ty, RESPType::BulkString("OK".to_string()));
    }

    #[test]
    fn parse_null_string() {
        let buffer = "$-1\r\n".as_bytes();
        let mut idx = 0;

        let ty = RESPType::parse(buffer, &mut idx).unwrap();

        assert_eq!(ty, RESPType::NullString);
    }

    #[test]
    fn parse_bulk_string_invalid_size() {
        let buffer = "$-2\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::InvalidSize(size)) => {
                assert_eq!(size, -2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_bulk_string_size_unparsable() {
        let buffer = "$foo\r\nOK\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::ParseSize(size)) => {
                assert_eq!(size, "foo");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_bulk_string_too_short() {
        let buffer = "$5\r\nOK\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(i)) => {
                assert_eq!(i, 9);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_arr() {
        let buffer = "*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n".as_bytes();
        let mut idx = 0;

        let ty = RESPType::parse(buffer, &mut idx).unwrap();

        assert_eq!(
            ty,
            RESPType::Array(vec![
                RESPType::BulkString("ECHO".to_string()),
                RESPType::BulkString("hello".to_string())
            ])
        );
    }

    #[test]
    fn parse_arr_size_unparsable_neg() {
        let buffer = "*-1\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::ParseSize(size)) => {
                assert_eq!(size, "-1");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_arr_size_unparsable_not_int() {
        let buffer = "*foo\r\n$1\r\nOK\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::ParseSize(size)) => {
                assert_eq!(size, "foo");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_invalid_prefix() {
        let buffer = "f1\r\n$1\r\nOK\r\n".as_bytes();
        let mut idx = 0;

        match RESPType::parse(buffer, &mut idx) {
            Err(RESPError::InvalidPrefix(c)) => {
                assert_eq!(c, b'f');
            }
            _ => panic!(),
        }
    }

    #[test]
    fn command_ping() {
        let cmd = Command::parse(vec![RESPType::BulkString("PING".to_string())]).unwrap();
        assert_eq!(cmd, Command::Ping);
    }

    #[test]
    fn command_echo() {
        let cmd = Command::parse(vec![
            RESPType::BulkString("ECHO".to_string()),
            RESPType::BulkString("hello".to_string()),
        ])
        .unwrap();
        assert_eq!(cmd, Command::Echo("hello".to_string()));
    }

    #[test]
    fn command_invalid() {
        match Command::parse(vec![RESPType::BulkString("foo".to_string())]) {
            Err(RESPError::InvalidCommand(cmd)) => assert_eq!(cmd, "foo"),
            _ => panic!(),
        }
    }

    #[test]
    fn extract_line_std() {
        let buffer = "OK\r\n".as_bytes();
        let mut idx: usize = 0;

        let out = extract_line(buffer, &mut idx).unwrap();

        assert_eq!(out, "OK".as_bytes());
        assert_eq!(idx, 4);
    }

    #[test]
    fn extract_line_imm_return() {
        let buffer = "\r\n".as_bytes();
        let mut idx: usize = 0;

        let out = extract_line(buffer, &mut idx).unwrap();

        assert_eq!(out, "".as_bytes());
        assert_eq!(idx, 2);
    }

    #[test]
    fn extract_line_empty() {
        let buffer = "".as_bytes();
        let mut idx: usize = 0;

        match extract_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn extract_line_no_separator() {
        let buffer = "OK".as_bytes();
        let mut idx: usize = 0;

        match extract_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn extract_line_idx_oob() {
        let buffer = "OK".as_bytes();
        let mut idx: usize = 3;

        match extract_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn extract_line_half_separator() {
        let buffer = "OK\r".as_bytes();
        let mut idx: usize = 0;

        match extract_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn extract_line_wrong_separator() {
        let buffer = "OK\n".as_bytes();
        let mut idx: usize = 0;

        match extract_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }
}
