use crate::commands::{Command, Echo, Get, Ping, Set};

pub fn parse_command<S>(mut args: impl Iterator<Item = S>) -> Result<Command, String>
where
    S: AsRef<str>,
{
    match args.next() {
        Some(cmd) => match &cmd.as_ref().to_lowercase()[..] {
            "ping" => Ok(Command::Ping(Ping::new())),
            "echo" => {
                let Some(msg) = args.next() else {
                    return Err(String::from("Usage: client echo <message>"));
                };
                Ok(Command::Echo(Echo::new(msg.as_ref().to_owned())))
            }
            "set" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };
                let Some(value) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };

                Ok(Command::Set(Set::new(
                    key.as_ref().to_owned(),
                    value.as_ref().to_owned(),
                )))
            }
            "get" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client get <key>"));
                };

                Ok(Command::Get(Get::new(key.as_ref().to_owned())))
            }
            other => Err(format!("Command `{}` not implemented", other)),
        },
        None => Err(String::from("Usage: client <command> <args>")),
    }
}
