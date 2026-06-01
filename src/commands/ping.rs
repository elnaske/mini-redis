use super::as_simple_string;

#[derive(Debug, PartialEq, Default)]
pub struct Ping {}
impl Ping {
    pub fn new() -> Self {
        Ping {}
    }

    pub fn to_resp(self) -> String {
        String::from("*1\r\n$4\r\nPING\r\n")
    }

    pub fn execute(self) -> String {
        as_simple_string("PONG")
    }
}
