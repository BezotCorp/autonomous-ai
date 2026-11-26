use std::collections::{BTreeMap, BTreeSet};

/// Parsed command line: a command, positional arguments and options. Each
/// command checks which options it accepts (see `Execution`).
pub(crate) struct CommandLine {
    command: String,
    positionals: Vec<String>,
    values: BTreeMap<String, Vec<String>>,
    flags: BTreeSet<String>,
}

impl CommandLine {
    /// Options that take no value.
    const FLAGS: [&'static str; 11] = [
        "--offline",
        "--neutral",
        "--sandbox",
        "--without-knowledge",
        "--without-routing",
        "--without-cache",
        "--without-influences",
        "--summary",
        "--detailed",
        "--deterministic",
        "--csv",
    ];

    pub(crate) const USAGE: &'static str = concat!(
        "Usage (individuals live in <data>/individuals/<name>, <data> defaulting to ./data):\n",
        "  autonomous-ai birth <name> [--seed <n>] [--dimensions <d>]\n",
        "  autonomous-ai live <name> [--for <seconds>] [--clock real|virtual]\n",
        "                    [--schedule <stimuli.ron>] [--checkpoint <seconds>]\n",
        "                    [--without <activity>]... [--without-influences]\n",
        "                    [--dormancy <seconds>] [--offline] [--deterministic]\n",
        "  autonomous-ai stimulate <name>   < event.ron\n",
        "  autonomous-ai stop <name>\n",
        "  autonomous-ai episode <name> [--neutral] [--sandbox] [--seed <n>] [--offline]\n",
        "                    [--without-knowledge] [--without-routing] [--without-cache]\n",
        "                    [--without-influences] [--max-invocations <n>]\n",
        "                    [--cost-weight <per-second>] [--scarcity-weight <w>]\n",
        "                    [--minimum-utility <u>] [--resource-budget <resource>=<units>]...\n",
        "                    [--summary | --detailed]   < event.ron\n",
        "  autonomous-ai duplicate <name> <new-name> [--reseed <n>]\n",
        "  autonomous-ai intervene <name> [--latent <dimension>=<value>]...\n",
        "                    [--bias <channel>=<value>]... [--scale <scale>=<factor>]...\n",
        "                    [--reason <text>]\n",
        "  autonomous-ai inspect <name> [--section <section>] [--last <n>] [--kind <kind>]\n",
        "     sections: summary identity constitution dispositions latent channels trajectory\n",
        "               timeline activities concerns stimuli reactions sequences episodes\n",
        "               statistics knowledge hypotheses scopes routing resources experiences\n",
        "               dreams corrections influences\n",
        "  autonomous-ai revert <name> --correction <id> [--reason <text>]\n",
        "  autonomous-ai relearn <name>\n",
        "  autonomous-ai individuals\n",
        "  autonomous-ai modules\n",
        "Common options: --data <dir>, --resources <dir> (default <data>/resources)",
    );

    pub(crate) fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments;
        let command = arguments.next().ok_or(Self::USAGE)?;
        let mut positionals = Vec::new();
        let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut flags = BTreeSet::new();

        while let Some(argument) = arguments.next() {
            if !argument.starts_with("--") {
                positionals.push(argument);
            } else if Self::FLAGS.contains(&argument.as_str()) {
                flags.insert(argument);
            } else {
                let value = arguments
                    .next()
                    .ok_or(format!("Missing value for {argument}"))?;

                values.entry(argument).or_default().push(value);
            }
        }

        Ok(Self {
            command,
            positionals,
            values,
            flags,
        })
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn positional(&self, index: usize, what: &str) -> Result<&str, String> {
        self.positionals
            .get(index)
            .map(String::as_str)
            .ok_or(format!("{what} is required\n{}", Self::USAGE))
    }

    pub(crate) fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }

    pub(crate) fn value(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .and_then(|values| values.last())
            .map(String::as_str)
    }

    pub(crate) fn all(&self, name: &str) -> &[String] {
        self.values.get(name).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn number<T: std::str::FromStr>(&self, name: &str) -> Result<Option<T>, String> {
        self.value(name)
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| format!("Invalid value {value} for {name}"))
            })
            .transpose()
    }

    /// `key=value` pairs given to a repeatable option.
    pub(crate) fn pairs<T: std::str::FromStr>(
        &self,
        name: &str,
    ) -> Result<Vec<(String, T)>, String> {
        self.all(name)
            .iter()
            .map(|pair| {
                let (key, value) = pair
                    .split_once('=')
                    .ok_or(format!("Invalid {name} {pair}: expected <key>=<value>"))?;
                let value = value
                    .parse()
                    .map_err(|_| format!("Invalid value in {name} {pair}"))?;

                Ok((key.to_owned(), value))
            })
            .collect()
    }

    /// Rejects options the command does not know, so that a typing error
    /// never silently changes an experiment.
    pub(crate) fn accept(&self, values: &[&str], flags: &[&str]) -> Result<(), String> {
        for name in self.values.keys() {
            if !values.contains(&name.as_str()) && name != "--data" && name != "--resources" {
                return Err(format!(
                    "Unknown option {name} for {}\n{}",
                    self.command,
                    Self::USAGE
                ));
            }
        }

        for name in &self.flags {
            if !flags.contains(&name.as_str()) {
                return Err(format!(
                    "Unknown option {name} for {}\n{}",
                    self.command,
                    Self::USAGE
                ));
            }
        }

        Ok(())
    }
}
