use std::{collections::BTreeSet, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::representation::{notation_parser::NotationParser, statement::Statement};

/// Implication between statement patterns: when every premise holds, the
/// conclusion holds. Two rules in opposite directions express an equivalence.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(remote = "Self")]
pub(crate) struct Rule {
    premises: Vec<Statement>,
    conclusion: Statement,
}

impl Rule {
    pub(crate) fn new(premises: Vec<Statement>, conclusion: Statement) -> Result<Self, String> {
        if premises.is_empty() {
            return Err(format!("Rule without premise: {conclusion}"));
        }

        let bound: BTreeSet<_> = premises.iter().flat_map(Statement::variables).collect();

        if let Some(unbound) = conclusion.variables().find(|name| !bound.contains(name)) {
            return Err(format!(
                "Variable ?{unbound} of the conclusion is not bound by a premise"
            ));
        }

        Ok(Self {
            premises,
            conclusion,
        })
    }

    pub(crate) fn premises(&self) -> &[Statement] {
        &self.premises
    }

    pub(crate) fn conclusion(&self) -> &Statement {
        &self.conclusion
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} :- ", self.conclusion)?;

        for (index, premise) in self.premises.iter().enumerate() {
            if index > 0 {
                write!(formatter, ", ")?;
            }

            write!(formatter, "{premise}")?;
        }

        Ok(())
    }
}

impl FromStr for Rule {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        NotationParser::new(value).rule_only()
    }
}

impl Serialize for Rule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            Self::serialize(self, serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)
        } else {
            let rule = Self::deserialize(deserializer)?;

            Self::new(rule.premises, rule.conclusion).map_err(serde::de::Error::custom)
        }
    }
}
