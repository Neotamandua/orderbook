//! Canonical tick-based price representation.

use core::fmt;

/// The number of supported price ticks in one whole unit.
pub const TICKS_PER_UNIT: u64 = 100;

/// A price represented as an integer number of `0.01` ticks.
///
/// Keeping a single integer as the representation makes equality, hashing, and
/// ordering agree by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceError {
    Negative,
    NonFinite,
    NotOnTick,
    OutOfRange,
    InvalidFraction,
}

impl fmt::Display for PriceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Negative => "price cannot be negative",
            Self::NonFinite => "price must be finite",
            Self::NotOnTick => "price is not aligned to the 0.01 tick size",
            Self::OutOfRange => "price is outside the supported range",
            Self::InvalidFraction => "fractional units must be between 0 and 99",
        };
        f.write_str(message)
    }
}

impl std::error::Error for PriceError {}

impl Price {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

    /// Creates a price from a number of minimum price ticks.
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Creates a price from a number of minimum price ticks.
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the canonical number of minimum price ticks.
    pub const fn ticks(self) -> u64 {
        self.0
    }

    /// Creates a price from whole units and hundredths without normalization.
    pub fn try_from_units(whole_units: u64, fractional_units: u8) -> Result<Self, PriceError> {
        if fractional_units >= TICKS_PER_UNIT as u8 {
            return Err(PriceError::InvalidFraction);
        }

        whole_units
            .checked_mul(TICKS_PER_UNIT)
            .and_then(|ticks| ticks.checked_add(u64::from(fractional_units)))
            .map(Self)
            .ok_or(PriceError::OutOfRange)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:02}",
            self.0 / TICKS_PER_UNIT,
            self.0 % TICKS_PER_UNIT
        )
    }
}

impl TryFrom<f64> for Price {
    type Error = PriceError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(PriceError::NonFinite);
        }
        if value < 0.0 {
            return Err(PriceError::Negative);
        }

        let scaled = value * TICKS_PER_UNIT as f64;
        if !scaled.is_finite() || scaled >= u64::MAX as f64 {
            return Err(PriceError::OutOfRange);
        }

        let rounded = scaled.round();
        let tolerance = 2.0 * f64::EPSILON * scaled.abs().max(1.0);
        if tolerance >= 0.5 {
            return Err(PriceError::OutOfRange);
        }
        if (scaled - rounded).abs() > tolerance {
            return Err(PriceError::NotOnTick);
        }

        Ok(Self(rounded as u64))
    }
}

impl TryFrom<f32> for Price {
    type Error = PriceError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(PriceError::NonFinite);
        }
        if value < 0.0 {
            return Err(PriceError::Negative);
        }

        // Scale as f64 so conversion does not introduce another f32 rounding
        // step. The tolerance accounts for the original f32 representation.
        let scaled = f64::from(value) * TICKS_PER_UNIT as f64;
        if scaled >= u64::MAX as f64 {
            return Err(PriceError::OutOfRange);
        }

        let rounded = scaled.round();
        let tolerance = f64::from(f32::EPSILON) * scaled.abs().max(1.0);
        if tolerance >= 0.5 {
            return Err(PriceError::OutOfRange);
        }
        if (scaled - rounded).abs() > tolerance {
            return Err(PriceError::NotOnTick);
        }

        Ok(Self(rounded as u64))
    }
}

impl From<Price> for f64 {
    fn from(value: Price) -> Self {
        value.0 as f64 / TICKS_PER_UNIT as f64
    }
}

impl From<Price> for f32 {
    fn from(value: Price) -> Self {
        value.0 as f32 / TICKS_PER_UNIT as f32
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{hash_map::DefaultHasher, HashSet};
    use std::hash::{Hash, Hasher};

    use super::*;

    #[test]
    fn constructs_common_prices_as_ticks() -> Result<(), PriceError> {
        assert_eq!(Price::try_from(1.00)?.ticks(), 100);
        assert_eq!(Price::try_from(1.01)?.ticks(), 101);
        assert_eq!(Price::try_from(10.99)?.ticks(), 1_099);
        assert_eq!(Price::try_from(2.00)?.ticks(), 200);
        assert_eq!(Price::try_from(580.37_f32)?.ticks(), 58_037);
        Ok(())
    }

    #[test]
    fn zero_is_not_silently_changed() -> Result<(), PriceError> {
        let price = Price::try_from(0.0)?;
        assert_eq!(price, Price::ZERO);
        assert_eq!(price.ticks(), 0);
        assert_eq!(price.to_string(), "0.00");
        Ok(())
    }

    #[test]
    fn rejects_invalid_float_input() {
        assert_eq!(Price::try_from(-0.01), Err(PriceError::Negative));
        assert_eq!(Price::try_from(f64::NAN), Err(PriceError::NonFinite));
        assert_eq!(Price::try_from(f64::INFINITY), Err(PriceError::NonFinite));
        assert_eq!(Price::try_from(1.999), Err(PriceError::NotOnTick));
        assert_eq!(Price::try_from(66.123_f32), Err(PriceError::NotOnTick));
        assert_eq!(Price::try_from(f64::MAX), Err(PriceError::OutOfRange));
    }

    #[test]
    fn rejects_invalid_split_units_instead_of_clamping() {
        assert_eq!(
            Price::try_from_units(1, 100),
            Err(PriceError::InvalidFraction)
        );
        assert_eq!(
            Price::try_from_units(u64::MAX, 0),
            Err(PriceError::OutOfRange)
        );
    }

    #[test]
    fn carries_a_float_boundary_into_the_canonical_tick() -> Result<(), PriceError> {
        let value_just_below_two_due_to_float_error = 1.999_999_999_999_999_8;
        let price = Price::try_from(value_just_below_two_due_to_float_error)?;
        assert_eq!(price, Price::from_ticks(200));
        assert_eq!(price.to_string(), "2.00");
        Ok(())
    }

    #[test]
    fn orders_across_decimal_boundaries() -> Result<(), PriceError> {
        assert!(Price::try_from(1.99)? < Price::try_from(2.00)?);
        assert!(Price::try_from(2.00)? < Price::try_from(2.01)?);
        assert!(Price::try_from(10.99)? < Price::try_from(11.00)?);
        Ok(())
    }

    #[test]
    fn equality_and_hash_use_the_same_canonical_ticks() -> Result<(), PriceError> {
        let from_float = Price::try_from(12.34)?;
        let from_ticks = Price::from_ticks(1_234);
        let from_units = Price::try_from_units(12, 34)?;

        assert_eq!(from_float, from_ticks);
        assert_eq!(from_ticks, from_units);

        let mut hashes = HashSet::new();
        hashes.insert(from_float);
        hashes.insert(from_ticks);
        hashes.insert(from_units);
        assert_eq!(hashes.len(), 1);

        let hash = |price: Price| {
            let mut hasher = DefaultHasher::new();
            price.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(from_float), hash(from_units));
        Ok(())
    }

    #[test]
    fn supports_the_full_tick_range_and_formats_large_prices() {
        assert_eq!(Price::MAX.ticks(), u64::MAX);
        assert_eq!(Price::MAX.to_string(), "184467440737095516.15");
    }

    #[test]
    fn ordering_always_matches_the_canonical_numerical_value() {
        let boundary_ticks = [0, 1, 99, 100, 101, 199, 200, 201, u64::MAX - 1, u64::MAX];

        for &left in &boundary_ticks {
            for &right in &boundary_ticks {
                assert_eq!(
                    Price::from_ticks(left).cmp(&Price::from_ticks(right)),
                    left.cmp(&right)
                );
            }
        }
    }
}
