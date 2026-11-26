use serde::{Deserialize, Serialize};

/// Dense square matrix of reals, stored row by row.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SquareMatrix {
    size: usize,
    values: Vec<f64>,
}

impl SquareMatrix {
    pub(crate) fn zero(size: usize) -> Self {
        Self {
            size,
            values: vec![0.0; size * size],
        }
    }

    pub(crate) fn from_fn(size: usize, mut value: impl FnMut(usize, usize) -> f64) -> Self {
        let mut matrix = Self::zero(size);

        for row in 0..size {
            for column in 0..size {
                matrix.values[row * size + column] = value(row, column);
            }
        }

        matrix
    }

    pub(crate) fn add(&mut self, row: usize, column: usize, delta: f64) {
        self.values[row * self.size + column] += delta;
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = &[f64]> {
        self.values.chunks(self.size.max(1))
    }

    /// Product with a vector of the same size.
    pub(crate) fn apply(&self, vector: &[f64]) -> Vec<f64> {
        self.rows()
            .take(self.size)
            .map(|row| row.iter().zip(vector).map(|(a, b)| a * b).sum())
            .collect()
    }

    /// Element-wise sum with another matrix of the same size.
    pub(crate) fn plus(&self, other: &Self) -> Self {
        Self {
            size: self.size,
            values: self
                .values
                .iter()
                .zip(&other.values)
                .map(|(a, b)| a + b)
                .collect(),
        }
    }

    pub(crate) fn frobenius(&self) -> f64 {
        self.values
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt()
    }

    /// Multiplies every coefficient (used to keep a norm within a bound).
    pub(crate) fn scale(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub(crate) fn is_finite(&self) -> bool {
        self.values.iter().all(|value| value.is_finite())
    }
}
