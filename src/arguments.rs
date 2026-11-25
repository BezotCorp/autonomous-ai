use std::path::PathBuf;

pub(crate) enum Arguments {
    Receive(PathBuf),
    Inspect(PathBuf),
    GenerateKnowledge {
        source: PathBuf,
        destination: PathBuf,
    },
    Project {
        memory: PathBuf,
        dataset: PathBuf,
    },
}

impl Arguments {
    pub(crate) fn parse() -> Result<Self, String> {
        let arguments: Vec<_> = std::env::args_os().skip(1).collect();

        match arguments.as_slice() {
            [option, path] if option == "--state" => Ok(Self::Receive(PathBuf::from(path))),

            [option, path] if option == "--inspect" => Ok(Self::Inspect(PathBuf::from(path))),

            [option, source, destination] if option == "--generate-knowledge" => {
                Ok(Self::GenerateKnowledge {
                    source: PathBuf::from(source),
                    destination: PathBuf::from(destination),
                })
            }

            [option, memory, dataset] if option == "--project" => Ok(Self::Project {
                memory: PathBuf::from(memory),
                dataset: PathBuf::from(dataset),
            }),

            _ => Err(concat!(
                "Usage: autonomous-ai ",
                "--state <memory-path> | ",
                "--inspect <memory-path> | ",
                "--generate-knowledge <ron-path> <bin-path> | ",
                "--project <memory-path> <dataset-path>",
            )
            .to_owned()),
        }
    }
}
