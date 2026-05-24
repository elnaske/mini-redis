#[derive(Debug)]
pub enum RESPError {
    OutOfBounds(usize),
}
type RESPResult<T> = Result<T, RESPError>;

pub fn parse_input(buffer: &[u8]) -> RESPResult<()> {
    unimplemented!()
}

fn parse_line<'a>(buffer: &'a [u8], idx: &mut usize) -> RESPResult<&'a [u8]> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_line_std() {
        let buffer = "OK\r\n".as_bytes();
        let mut idx: usize = 0;

        let out = parse_line(buffer, &mut idx).unwrap();

        assert_eq!(out, "OK".as_bytes());
        assert_eq!(idx, 4);
    }

    #[test]
    fn parse_line_imm_return() {
        let buffer = "\r\n".as_bytes();
        let mut idx: usize = 0;

        let out = parse_line(buffer, &mut idx).unwrap();

        assert_eq!(out, "".as_bytes());
        assert_eq!(idx, 2);
    }

    #[test]
    fn parse_line_empty() {
        let buffer = "".as_bytes();
        let mut idx: usize = 0;

        match parse_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_line_no_separator() {
        let buffer = "OK".as_bytes();
        let mut idx: usize = 0;

        match parse_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_line_idx_oob() {
        let buffer = "OK".as_bytes();
        let mut idx: usize = 3;

        match parse_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_line_half_separator() {
        let buffer = "OK\r".as_bytes();
        let mut idx: usize = 0;

        match parse_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_line_wrong_separator() {
        let buffer = "OK\n".as_bytes();
        let mut idx: usize = 0;

        match parse_line(buffer, &mut idx) {
            Err(RESPError::OutOfBounds(idx)) => {
                assert_eq!(idx, buffer.len());
            }
            _ => panic!(),
        }
    }
}
