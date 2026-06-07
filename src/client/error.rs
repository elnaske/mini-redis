use std::fmt;

use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError;

use super::Message;
use crate::resp::error::RESPError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    Send(SendError<Message>),
    Recv(RecvError),
    Io(std::io::Error),
    Parse(RESPError),
    ConnectionClosed,
    MissingArgs,
    UnknownArg(String),
    UnknownCmd(String),
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Send(err) => write!(f, "{}", err),
            ClientError::Recv(err) => write!(f, "{}", err),
            ClientError::Io(err) => write!(f, "{}", err),
            ClientError::Parse(err) => write!(f, "{}", err),
            ClientError::ConnectionClosed => write!(f, "Connection closed"),
            ClientError::MissingArgs => {
                let err = "Usage: mini-redis-cli <command> <args>
                
                Available commands:
                    PING
                    ECHO <message>
                    SET <key> <value> [EX | PX <expiry time>]
                    GET <key>";

                write!(f, "{}", err)
            }
            ClientError::UnknownArg(arg) => write!(f, "Unknown argument `{}`", arg),
            ClientError::UnknownCmd(cmd) => write!(f, "Unimplemented command `{}`", cmd),
        }
    }
}
