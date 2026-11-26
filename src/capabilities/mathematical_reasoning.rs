use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    mathematics::{expression::Expression, rational::Rational},
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        content::Content, content_kind::ContentKind, statement::Statement, term::Term,
    },
};

/// Exact mathematics over numbers, versions and expressions.
///
/// It answers needs about the fundamental relations of order and equality,
/// the elementary operations and the components of versions, producing either the statement or
/// its explicit negation. It also evaluates the expressions appearing in
/// statements (`value(expression, number)`).
pub(crate) struct MathematicalReasoning {
    descriptor: ModuleDescriptor,
}

impl MathematicalReasoning {
    const COMPARISONS: [&'static str; 6] = [
        "less_than",
        "less_or_equal",
        "greater_than",
        "greater_or_equal",
        "equal",
        "different",
    ];

    const OPERATIONS: [&'static str; 4] = ["sum", "difference", "product", "quotient"];

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "mathematical_reasoning",
                "Compares quantities and versions, computes operations and evaluates expressions exactly.",
            )
            .consuming(&[ContentKind::Need, ContentKind::Statement])
            .supplying(&[
                "less_than/2",
                "less_or_equal/2",
                "greater_than/2",
                "greater_or_equal/2",
                "equal/2",
                "different/2",
                "sum/3",
                "difference/3",
                "product/3",
                "quotient/3",
                "value/2",
                "version_major/2",
                "version_minor/2",
                "version_patch/2",
            ])
            .producing(&[EpistemicKind::Computation]),
        }
    }

    fn compare(left: &Term, right: &Term) -> Option<Ordering> {
        match (Self::number(left), Self::number(right)) {
            (Some(left), Some(right)) => return Some(left.cmp(&right)),
            (Some(_), None) | (None, Some(_)) => return None,
            (None, None) => {}
        }

        match (left, right) {
            (Term::Version(left), Term::Version(right)) => Some(left.cmp_precedence(right)),
            (Term::Symbol(left), Term::Symbol(right)) | (Term::Text(left), Term::Text(right)) => {
                (left == right).then_some(Ordering::Equal)
            }
            _ => None,
        }
    }

    fn number(term: &Term) -> Option<Rational> {
        match term {
            Term::Number(value) => Some(*value),
            Term::Expression(expression) => expression.evaluate(&BTreeMap::new()).ok(),
            _ => None,
        }
    }

    fn decide(predicate: &str, ordering: Ordering) -> bool {
        match predicate {
            "less_than" => ordering == Ordering::Less,
            "less_or_equal" => ordering != Ordering::Greater,
            "greater_than" => ordering == Ordering::Greater,
            "greater_or_equal" => ordering != Ordering::Less,
            "equal" => ordering == Ordering::Equal,
            _ => ordering != Ordering::Equal,
        }
    }

    fn operate(predicate: &str, left: Rational, right: Rational) -> Result<Rational, String> {
        match predicate {
            "sum" => left.add(&right),
            "difference" => left.subtract(&right),
            "product" => left.multiply(&right),
            _ => left.divide(&right),
        }
    }

    /// Answers a need, or explains why it cannot.
    fn answer(pattern: &Statement) -> Result<Statement, String> {
        let predicate = pattern.predicate();
        let arguments = pattern.arguments();

        if Self::COMPARISONS.contains(&predicate) {
            let [left, right] = arguments else {
                return Err(format!("{pattern}: two arguments expected"));
            };

            let ordering = Self::compare(left, right)
                .ok_or_else(|| format!("{pattern}: terms are not comparable"))?;

            let positive = Statement::new(predicate, arguments.to_vec());

            return Ok(if Self::decide(predicate, ordering) {
                positive
            } else {
                positive.complement()
            });
        }

        if Self::OPERATIONS.contains(&predicate) {
            let [left, right, result] = arguments else {
                return Err(format!("{pattern}: three arguments expected"));
            };

            let (Some(left), Some(right)) = (Self::number(left), Self::number(right)) else {
                return Err(format!("{pattern}: operands must be known numbers"));
            };

            let value = Term::Number(Self::operate(predicate, left, right)?);
            let statement = Statement::new(
                predicate,
                vec![arguments[0].clone(), arguments[1].clone(), value.clone()],
            );

            return Ok(match result {
                Term::Variable(_) => statement,
                known if Self::compare(known, &value) == Some(Ordering::Equal) => {
                    Statement::new(predicate, arguments.to_vec())
                }
                _ => Statement::negative(predicate, arguments.to_vec()),
            });
        }

        if let Some(component) = predicate.strip_prefix("version_") {
            let [Term::Version(version), _] = arguments else {
                return Err(format!("{pattern}: a known version is expected"));
            };

            let value = match component {
                "major" => version.major,
                "minor" => version.minor,
                _ => version.patch,
            };

            let value = Term::Number(Rational::integer(i128::from(value)));

            return Ok(match &arguments[1] {
                Term::Variable(_) => Statement::new(predicate, vec![arguments[0].clone(), value]),
                known if known == &value => Statement::new(predicate, arguments.to_vec()),
                _ => Statement::negative(predicate, arguments.to_vec()),
            });
        }

        let [expression, _] = arguments else {
            return Err(format!("{pattern}: two arguments expected"));
        };

        Self::value(expression).ok_or_else(|| format!("{pattern}: expression cannot be evaluated"))
    }

    fn value(term: &Term) -> Option<Statement> {
        let Term::Expression(expression) = term else {
            return None;
        };

        let value = match expression.simplify().ok()? {
            Expression::Number(number) => Term::Number(number),
            simplified => Term::Expression(simplified),
        };

        Some(Statement::new("value", vec![term.clone(), value]))
    }
}

impl CognitiveModule for MathematicalReasoning {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();

        for node in graph.valid_nodes() {
            if node.revision() <= context.seen_revision() {
                continue;
            }

            let result = match node.content() {
                Content::Need(pattern) if self.descriptor.supplies(&pattern.signature()) => {
                    Some(Self::answer(pattern))
                }
                Content::Statement(statement) if statement.predicate() != "value" => {
                    for term in statement.arguments() {
                        if let Some(value) = Self::value(term) {
                            output.propose(Proposal::new(
                                Content::Statement(value),
                                EpistemicKind::Computation,
                                Vec::new(),
                            ));
                        }
                    }

                    None
                }
                _ => None,
            };

            match result {
                Some(Ok(statement)) => output.propose(Proposal::new(
                    Content::Statement(statement),
                    EpistemicKind::Computation,
                    Vec::new(),
                )),
                Some(Err(reason)) => output.note(reason),
                None => {}
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::MathematicalReasoning;

    #[test]
    fn answers_with_proofs_of_absence() {
        let answer = |text: &str| {
            MathematicalReasoning::answer(&text.parse().unwrap())
                .unwrap()
                .to_string()
        };

        assert_eq!(
            answer("less_than(v1.2.0, v1.10.0)"),
            "less_than(v1.2.0, v1.10.0)"
        );
        assert_eq!(
            answer("greater_than(1/3, 0.5)"),
            "not greater_than(1/3, 1/2)"
        );
        assert_eq!(answer("sum(2, 3/2, ?x)"), "sum(2, 3/2, 7/2)");
        assert_eq!(answer("product(2, 3, 7)"), "not product(2, 3, 7)");
        assert_eq!(
            answer("version_minor(v0.14.5, ?m)"),
            "version_minor(v0.14.5, 14)"
        );
        assert_eq!(
            answer("version_major(v1.0.0, 0)"),
            "not version_major(v1.0.0, 0)"
        );
        assert!(MathematicalReasoning::answer(&"less_than(v1.0.0, 3)".parse().unwrap()).is_err());
    }
}
