use std::{cmp::Ordering, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Exact rational number used for every quantity manipulated by the
/// reasoning layer. Values are always normalised: the denominator is
/// strictly positive and the fraction is irreducible.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(try_from = "RationalParts", into = "RationalParts")]
pub(crate) struct Rational {
    numerator: i128,
    denominator: i128,
}

#[derive(Deserialize, Serialize)]
struct RationalParts(i128, i128);

impl TryFrom<RationalParts> for Rational {
    type Error = String;

    fn try_from(parts: RationalParts) -> Result<Self, Self::Error> {
        Self::new(parts.0, parts.1)
    }
}

impl From<Rational> for RationalParts {
    fn from(value: Rational) -> Self {
        Self(value.numerator, value.denominator)
    }
}

impl Rational {
    pub(crate) fn new(numerator: i128, denominator: i128) -> Result<Self, String> {
        if denominator == 0 {
            return Err("Division by zero".to_owned());
        }

        let divisor = Self::gcd(numerator.unsigned_abs(), denominator.unsigned_abs());
        let divisor = i128::try_from(divisor).map_err(|_| "Rational overflow".to_owned())?;
        let sign = if denominator < 0 { -1 } else { 1 };

        Ok(Self {
            numerator: sign * numerator / divisor,
            denominator: sign * denominator / divisor,
        })
    }

    pub(crate) fn integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub(crate) fn is_integer(&self) -> bool {
        self.denominator == 1
    }

    pub(crate) fn add(&self, other: &Self) -> Result<Self, String> {
        let numerator = self
            .numerator
            .checked_mul(other.denominator)
            .zip(other.numerator.checked_mul(self.denominator))
            .and_then(|(left, right)| left.checked_add(right));

        let denominator = self.denominator.checked_mul(other.denominator);

        match (numerator, denominator) {
            (Some(numerator), Some(denominator)) => Self::new(numerator, denominator),
            _ => Err("Rational overflow".to_owned()),
        }
    }

    pub(crate) fn negate(&self) -> Result<Self, String> {
        self.numerator
            .checked_neg()
            .map(|numerator| Self {
                numerator,
                denominator: self.denominator,
            })
            .ok_or_else(|| "Rational overflow".to_owned())
    }

    pub(crate) fn subtract(&self, other: &Self) -> Result<Self, String> {
        self.add(&other.negate()?)
    }

    pub(crate) fn multiply(&self, other: &Self) -> Result<Self, String> {
        match (
            self.numerator.checked_mul(other.numerator),
            self.denominator.checked_mul(other.denominator),
        ) {
            (Some(numerator), Some(denominator)) => Self::new(numerator, denominator),
            _ => Err("Rational overflow".to_owned()),
        }
    }

    pub(crate) fn divide(&self, other: &Self) -> Result<Self, String> {
        if other.numerator == 0 {
            return Err("Division by zero".to_owned());
        }

        self.multiply(&Self::new(other.denominator, other.numerator)?)
    }

    pub(crate) fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    fn gcd(mut left: u128, mut right: u128) -> u128 {
        while right != 0 {
            (left, right) = (right, left % right);
        }

        left.max(1)
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.numerator.checked_mul(other.denominator),
            other.numerator.checked_mul(self.denominator),
        ) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => self.to_f64().total_cmp(&other.to_f64()),
        }
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_integer() {
            write!(formatter, "{}", self.numerator)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl FromStr for Rational {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if let Some((numerator, denominator)) = value.split_once('/') {
            let numerator = numerator.trim().parse().map_err(|_| invalid(value))?;
            let denominator = denominator.trim().parse().map_err(|_| invalid(value))?;

            return Self::new(numerator, denominator);
        }

        if let Some((whole, fraction)) = value.split_once('.') {
            let negative = whole.trim_start().starts_with('-');
            let digits = fraction.len() as u32;
            let scale = 10_i128.checked_pow(digits).ok_or_else(|| invalid(value))?;
            let whole: i128 = whole.parse().map_err(|_| invalid(value))?;
            let fraction: i128 = fraction.parse().map_err(|_| invalid(value))?;
            let fraction = if negative { -fraction } else { fraction };

            let numerator = whole
                .checked_mul(scale)
                .and_then(|whole| whole.checked_add(fraction))
                .ok_or_else(|| invalid(value))?;

            return Self::new(numerator, scale);
        }

        value.parse().map(Self::integer).map_err(|_| invalid(value))
    }
}

fn invalid(value: &str) -> String {
    format!("Invalid rational number: {value}")
}

#[cfg(test)]
mod tests {
    use super::Rational;

    #[test]
    fn normalises_and_compares_exactly() {
        let half: Rational = "2/4".parse().unwrap();
        let decimal: Rational = "0.5".parse().unwrap();

        assert_eq!(half, decimal);
        assert!(Rational::new(1, 3).unwrap() < half);
        assert_eq!(
            half.add(&Rational::new(1, 3).unwrap()).unwrap(),
            Rational::new(5, 6).unwrap()
        );
        assert!(half.divide(&Rational::integer(0)).is_err());
    }
}
