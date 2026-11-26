use std::{collections::BTreeMap, path::PathBuf};

use crate::interface::output::Output;

/// Parsed command line.
pub(crate) enum Arguments {
    /// Reasons about an event read from standard input.
    Run {
        memory: PathBuf,
        resources: PathBuf,
        seed: Option<u64>,
        acquired_knowledge: bool,
        learned_routing: bool,
        offline: bool,
        maximum_invocations: Option<usize>,
        cost_weight: Option<f64>,
        scarcity_weight: Option<f64>,
        minimum_utility: Option<f64>,
        budgets: BTreeMap<String, f64>,
        output: Output,
    },
    /// Prints a section of the persistent memory.
    Inspect { memory: PathBuf, section: String },
    /// Recomputes the routing memory from the experience register.
    Relearn { memory: PathBuf },
    /// Lists the registered cognitive modules.
    Modules,
}

impl Arguments {
    pub(crate) const USAGE: &'static str = concat!(
        "Usage:\n",
        "  autonomous-ai run --memory <memory.bin> --resources <dir> [--seed <n>]\n",
        "                    [--without-knowledge] [--without-routing] [--offline]\n",
        "                    [--max-invocations <n>] [--cost-weight <per-second>]\n",
        "                    [--scarcity-weight <w>] [--minimum-utility <u>]\n",
        "                    [--resource-budget <resource>=<units>]...\n",
        "                    [--summary | --detailed]   < event.ron\n",
        "  autonomous-ai inspect --memory <memory.bin>\n",
        "                    [--section summary|statistics|knowledge|hypotheses|scopes|routing|resources|experiences]\n",
        "  autonomous-ai relearn --memory <memory.bin>\n",
        "  autonomous-ai modules",
    );

    pub(crate) fn parse() -> Result<Self, String> {
        let mut arguments = std::env::args().skip(1);
        let command = arguments.next().ok_or(Self::USAGE)?;

        let mut memory = None;
        let mut resources = None;
        let mut seed = None;
        let mut section = "summary".to_owned();
        let mut acquired_knowledge = true;
        let mut learned_routing = true;
        let mut offline = false;
        let mut maximum_invocations = None;
        let mut cost_weight = None;
        let mut scarcity_weight = None;
        let mut minimum_utility = None;
        let mut budgets = BTreeMap::new();
        let mut output = Output::Report;

        while let Some(option) = arguments.next() {
            let mut value = || {
                arguments
                    .next()
                    .ok_or(format!("Missing value for {option}"))
            };

            match option.as_str() {
                "--memory" => memory = Some(PathBuf::from(value()?)),
                "--resources" => resources = Some(PathBuf::from(value()?)),
                "--section" => section = value()?,
                "--seed" => seed = Some(value()?.parse().map_err(|_| "Invalid seed")?),
                "--max-invocations" => {
                    maximum_invocations =
                        Some(value()?.parse().map_err(|_| "Invalid invocation budget")?)
                }
                "--cost-weight" => {
                    cost_weight = Some(value()?.parse().map_err(|_| "Invalid cost weight")?)
                }
                "--scarcity-weight" => {
                    scarcity_weight = Some(value()?.parse().map_err(|_| "Invalid scarcity weight")?)
                }
                "--resource-budget" => {
                    let budget = value()?;
                    let (resource, units) = budget.split_once('=').ok_or(format!(
                        "Invalid resource budget {budget}: expected <resource>=<units>"
                    ))?;
                    let units: f64 = units
                        .parse()
                        .map_err(|_| format!("Invalid number of units in {budget}"))?;

                    budgets.insert(resource.to_owned(), units);
                }
                "--minimum-utility" => {
                    minimum_utility = Some(value()?.parse().map_err(|_| "Invalid minimum utility")?)
                }
                "--without-knowledge" => acquired_knowledge = false,
                "--without-routing" => learned_routing = false,
                "--offline" => offline = true,
                "--detailed" => output = Output::Detailed,
                "--summary" => output = Output::Summary,
                other => return Err(format!("Unknown option {other}\n{}", Self::USAGE)),
            }
        }

        let memory = || {
            memory
                .clone()
                .ok_or(format!("--memory is required\n{}", Self::USAGE))
        };

        match command.as_str() {
            "run" => Ok(Self::Run {
                memory: memory()?,
                resources: resources.ok_or(format!("--resources is required\n{}", Self::USAGE))?,
                seed,
                acquired_knowledge,
                learned_routing,
                offline,
                maximum_invocations,
                cost_weight,
                scarcity_weight,
                minimum_utility,
                budgets,
                output,
            }),
            "inspect" => Ok(Self::Inspect {
                memory: memory()?,
                section,
            }),
            "relearn" => Ok(Self::Relearn { memory: memory()? }),
            "modules" => Ok(Self::Modules),
            _ => Err(Self::USAGE.to_owned()),
        }
    }
}
