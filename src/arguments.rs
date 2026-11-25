use std::path::PathBuf;

pub(crate) enum Arguments {
    Receive(PathBuf),
    Inspect(PathBuf),
}

impl Arguments {
    pub(crate) fn parse() -> Result<Self, String> {
        let mut arguments = std::env::args_os().skip(1);

        match (arguments.next(), arguments.next(), arguments.next()) {
            (Some(option), Some(path), None) if option == "--state" => {
                Ok(Self::Receive(PathBuf::from(path)))
            }
            (Some(option), Some(path), None) if option == "--inspect" => {
                Ok(Self::Inspect(PathBuf::from(path)))
            }
            _ => Err(
                "Usage: autonomous-ai --state <memory-path> | --inspect <memory-path>".to_owned(),
            ),
        }
    }
}
