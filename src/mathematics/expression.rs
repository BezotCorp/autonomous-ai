use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::mathematics::rational::Rational;

/// Structured arithmetic expression. Expressions are values that the
/// reasoning layer can carry, compare, simplify and evaluate exactly.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Expression {
    Number(Rational),
    Variable(String),
    Negate(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Evaluates the expression when every variable has a known value.
    pub(crate) fn evaluate(&self, values: &BTreeMap<String, Rational>) -> Result<Rational, String> {
        match self {
            Self::Number(value) => Ok(*value),
            Self::Variable(name) => values
                .get(name)
                .copied()
                .ok_or_else(|| format!("Unknown value for variable {name}")),
            Self::Negate(value) => value.evaluate(values)?.negate(),
            Self::Add(left, right) => left.evaluate(values)?.add(&right.evaluate(values)?),
            Self::Subtract(left, right) => {
                left.evaluate(values)?.subtract(&right.evaluate(values)?)
            }
            Self::Multiply(left, right) => {
                left.evaluate(values)?.multiply(&right.evaluate(values)?)
            }
            Self::Divide(left, right) => left.evaluate(values)?.divide(&right.evaluate(values)?),
        }
    }

    /// Folds every constant sub-expression while keeping unknown variables.
    pub(crate) fn simplify(&self) -> Result<Self, String> {
        let simplified = match self {
            Self::Number(_) | Self::Variable(_) => return Ok(self.clone()),
            Self::Negate(value) => Self::Negate(Box::new(value.simplify()?)),
            Self::Add(left, right) => {
                Self::Add(Box::new(left.simplify()?), Box::new(right.simplify()?))
            }
            Self::Subtract(left, right) => {
                Self::Subtract(Box::new(left.simplify()?), Box::new(right.simplify()?))
            }
            Self::Multiply(left, right) => {
                Self::Multiply(Box::new(left.simplify()?), Box::new(right.simplify()?))
            }
            Self::Divide(left, right) => {
                Self::Divide(Box::new(left.simplify()?), Box::new(right.simplify()?))
            }
        };

        if simplified.variables().is_empty() {
            return simplified.evaluate(&BTreeMap::new()).map(Self::Number);
        }

        Ok(simplified)
    }

    pub(crate) fn variables(&self) -> Vec<&str> {
        let mut variables = Vec::new();
        self.collect_variables(&mut variables);
        variables.sort_unstable();
        variables.dedup();
        variables
    }

    fn collect_variables<'a>(&'a self, variables: &mut Vec<&'a str>) {
        match self {
            Self::Number(_) => {}
            Self::Variable(name) => variables.push(name),
            Self::Negate(value) => value.collect_variables(variables),
            Self::Add(left, right)
            | Self::Subtract(left, right)
            | Self::Multiply(left, right)
            | Self::Divide(left, right) => {
                left.collect_variables(variables);
                right.collect_variables(variables);
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => write!(formatter, "{value}"),
            Self::Variable(name) => write!(formatter, "{name}"),
            Self::Negate(value) => write!(formatter, "-({value})"),
            Self::Add(left, right) => write!(formatter, "({left} + {right})"),
            Self::Subtract(left, right) => write!(formatter, "({left} - {right})"),
            Self::Multiply(left, right) => write!(formatter, "({left} * {right})"),
            Self::Divide(left, right) => write!(formatter, "({left} / {right})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Expression;
    use crate::mathematics::rational::Rational;

    #[test]
    fn simplifies_constant_parts_only() {
        let expression = Expression::Add(
            Box::new(Expression::Multiply(
                Box::new(Expression::Number(Rational::integer(2))),
                Box::new(Expression::Number(Rational::integer(3))),
            )),
            Box::new(Expression::Variable("x".to_owned())),
        );

        assert_eq!(
            expression.simplify().unwrap(),
            Expression::Add(
                Box::new(Expression::Number(Rational::integer(6))),
                Box::new(Expression::Variable("x".to_owned())),
            )
        );
    }
}
