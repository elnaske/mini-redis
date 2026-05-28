use super::as_simple_string;

#[derive(Debug, PartialEq, Default)]
pub struct Ping {}
impl Ping {
    pub fn new() -> Self {
        Ping {}
    }

    pub fn execute(self) -> String {
        as_simple_string("PONG")
    }
}
