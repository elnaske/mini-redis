use std::fmt;

#[derive(Debug)]
pub enum RESPError {
    OutOfBounds(usize),
    Expected { expected: u8, found: u8 },
    InvalidPrefix(u8),
    InvalidCommand(String),
    CommandError,
    IDKMan,
}
type RESPResult<T> = Result<T, RESPError>;

#[derive(Debug, PartialEq)]
pub enum RESPType {
    BulkString(String),
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
                    String::from_utf8_lossy(chars).parse::<usize>().unwrap() // TODO 
                };
                let mut arr = Vec::<RESPType>::new();

                for _ in 0..size {
                    arr.push(RESPType::parse(buffer, idx)?);
                }

                Ok(RESPType::Array(arr))
            }
            Some(b'$') => {
                *idx += 1;
                let size = {
                    let chars = extract_line(buffer, idx)?;
                    String::from_utf8_lossy(chars).parse::<usize>().unwrap() // TODO 
                };

                let chars = extract_n_chars(buffer, idx, size)?;

                Ok(RESPType::BulkString(String::from_utf8(chars.to_vec()).unwrap()))
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
        println!("{request:?}");
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

fn extract_n_chars<'a>(buffer: &'a [u8], idx: &mut usize, n: usize) -> RESPResult<&'a [u8]> {
    match buffer.get(*idx..*idx+n) {
        Some(arr) => {
            *idx += n + 2;
            Ok(arr)
        }
        None => Err(RESPError::OutOfBounds(n))
    }
}

#[cfg(test)]
mod test {
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

    // TODO: more tests (error handling)

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
