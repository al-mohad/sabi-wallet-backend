use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents monetary amounts in Nigerian Kobo (1 Naira = 100 Kobo).
/// Stored as a 64-bit integer to avoid floating-point inaccuracies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Kobo(pub i64);

impl Kobo {
    pub const ZERO: Kobo = Kobo(0);

    pub fn from_naira(naira: f64) -> Self {
        Kobo((naira * 100.0) as i64)
    }

    pub fn to_naira(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl fmt::Display for Kobo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Kobo", self.0)
    }
}

/// Represents Bitcoin amounts in Satoshis (1 BTC = 100,000,000 Satoshis).
/// Stored as a 64-bit integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sats(pub i64);

impl Sats {
    pub const ZERO: Sats = Sats(0);

    pub fn from_btc(btc: f64) -> Self {
        Sats((btc * 100_000_000.0) as i64)
    }

    pub fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

impl fmt::Display for Sats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Sats", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kobo_conversion() {
        let kobo = Kobo::from_naira(10.50);
        assert_eq!(kobo.0, 1050);
        assert_eq!(kobo.to_naira(), 10.50);

        let kobo_zero = Kobo::from_naira(0.0);
        assert_eq!(kobo_zero.0, 0);
        assert_eq!(kobo_zero.to_naira(), 0.0);
    }

    #[test]
    fn test_sats_conversion() {
        let sats = Sats::from_btc(0.00012345);
        assert_eq!(sats.0, 12345);
        assert_eq!(sats.to_btc(), 0.00012345);

        let sats_zero = Sats::from_btc(0.0);
        assert_eq!(sats_zero.0, 0);
        assert_eq!(sats_zero.to_btc(), 0.0);
    }
}
