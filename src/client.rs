use crate::commands::{Command, Echo, Get, Ping, Set};

pub fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    args.next();
    match args.next() {
        Some(cmd) => match &cmd.to_lowercase()[..] {
            "ping" => Ok(Command::Ping(Ping::new())),
            "echo" => {
                let Some(msg) = args.next() else {
                    return Err(String::from("Usage: client echo <message>"));
                };
                Ok(Command::Echo(Echo::new(msg.to_owned())))
            }
            "set" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };
                let Some(value) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };

                Ok(Command::Set(Set::new(key, value)))
            }
            "get" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client get <key>"));
                };

                Ok(Command::Get(Get::new(key)))
            }
            other => Err(format!("Command `{}` not implemented", other)),
        },
        None => Err(String::from("Usage: client <command> <args>")),
    }
}
