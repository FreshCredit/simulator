//! Economic utilities for compensated summation

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// kahan accumulator.
pub struct KahanAccumulator {
    sum: f32,
    compensation: f32,
}

impl KahanAccumulator {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self {
            sum: 0.0,
            compensation: 0.0,
        }
    }
    /// add.
    pub fn add(&mut self, value: f32) {
        let y = value - self.compensation;
        let t = self.sum + y;
        self.compensation = (t - self.sum) - y;
        self.sum = t;
    }
    #[must_use]
    /// sum.
    pub const fn sum(&self) -> f32 {
        self.sum
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[must_use]
/// kahan sum.
pub fn kahan_sum(values: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    let mut compensation = 0.0f32;
    for &value in values {
        let y = value - compensation;
        let t = sum + y;
        compensation = (t - sum) - y;
        sum = t;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kahan_accumulator_new() {
        let acc = KahanAccumulator::new();
        assert_eq!(acc.sum(), 0.0);
    }

    #[test]
    fn test_kahan_accumulator_add() {
        let mut acc = KahanAccumulator::new();
        acc.add(1.0);
        acc.add(2.0);
        acc.add(3.0);
        assert!((acc.sum() - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_kahan_sum() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert!((kahan_sum(&values) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_kahan_sum_empty() {
        let values: Vec<f32> = vec![];
        assert_eq!(kahan_sum(&values), 0.0);
    }
}
