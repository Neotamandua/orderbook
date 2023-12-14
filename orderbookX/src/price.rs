use core::fmt;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Price {
    main_unit: usize,
    // clamp this between 0 and 99
    sub_unit: u8,
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.main_unit, self.sub_unit)
    }
}

impl Price {
    /// Create a new Price, does not allow 0.00. Expects correct handling.
    ///
    /// Will default into 0.01 if Zero is provided.
    /// Will clamp the sub unit value between the allowed amount.
    pub fn new(main_unit: usize, sub_unit: u8) -> Self {
        let sub_unit = sub_unit.clamp(0, 99);
        // Price has to be at least 0.01
        if main_unit > 0 || sub_unit > 0 {
            Self {
                main_unit,
                sub_unit,
            }
        } else {
            Self {
                main_unit: 0,
                sub_unit: 1,
            }
        }
    }

    pub fn get_price_as_f64() -> f64 {
        todo!()
    }
}

/// Rounds the sub_unit to two decimal places
impl From<f64> for Price {
    /// Will default into 0.01 if Zero is provided.
    fn from(value: f64) -> Self {
        let value = if value == 0.00 { 0.01 } else { value };

        let main_unit = value as usize;
        let sub_unit = ((value - main_unit as f64) * 100.0).round() as u8;
        Price {
            main_unit,
            sub_unit,
        }
    }
}

impl From<Price> for f32 {
    fn from(value: Price) -> Self {
        value.main_unit as f32 + (value.sub_unit as f32 / 100.00)
    }
}

/// Rounds the sub_unit to two decimal places
impl From<f32> for Price {
    /// Will default into 0.01 if Zero is provided.
    fn from(value: f32) -> Self {
        let value = if value == 0.00 { 0.01 } else { value };
        let main_unit = value as usize;
        let sub_unit = ((value - main_unit as f32) * 100.0).round() as u8;
        Price {
            main_unit,
            sub_unit,
        }
    }
}

impl Hash for Price {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.main_unit.hash(state);
        self.sub_unit.hash(state);
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.main_unit.cmp(&other.main_unit) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.sub_unit.cmp(&other.sub_unit),
        }
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_positive_units() {
        let price = Price::new(10, 50);
        assert_eq!(price.main_unit, 10);
        assert_eq!(price.sub_unit, 50);
    }

    #[test]
    fn test_new_with_zero_units() {
        let price = Price::new(0, 0);
        assert_eq!(price.main_unit, 0);
        assert_eq!(price.sub_unit, 1); // sub_unit should be at least 1
    }

    #[test]
    fn test_new_with_high_sub_units() {
        let price = Price::new(100, 150);
        assert_eq!(price.main_unit, 100);
        assert_eq!(price.sub_unit, 99); // sub_unit should be clamped to 99 at max
    }

    #[test]
    fn test_f64_into_price() {
        let price: Price = 10.99.into();
        assert_eq!(price.main_unit, 10);
        assert_eq!(price.sub_unit, 99);
    }

    #[test]
    fn test_f32_into_price() {
        let price: Price = 580.37.into();
        assert_eq!(price.main_unit, 580);
        assert_eq!(price.sub_unit, 37);
    }

    #[test]
    fn test_f64_into_price_with_zero_units() {
        let price: Price = 0.00.into();
        assert_eq!(price.main_unit, 0);
        assert_eq!(price.sub_unit, 1); // sub_unit should be at least 1
    }

    #[test]
    fn test_f32_into_price_with_zero_units() {
        let price: Price = 0.00.into();
        assert_eq!(price.main_unit, 0);
        assert_eq!(price.sub_unit, 1); // sub_unit should be at least 1
    }

    #[test]
    fn test_f64_sub_unit_into_price() {
        let price: Price = 66.123.into();
        assert_eq!(price.main_unit, 66);
        assert_eq!(price.sub_unit, 12);
    }

    #[test]
    fn test_f32_sub_unit_into_price() {
        let price: Price = 580.123.into();
        assert_eq!(price.main_unit, 580);
        assert_eq!(price.sub_unit, 12);
    }

    #[test]
    fn test_price_ord() {
        let price1 = Price {
            main_unit: 10,
            sub_unit: 50,
        };
        let price2 = Price {
            main_unit: 10,
            sub_unit: 75,
        };
        let price3 = Price {
            main_unit: 12,
            sub_unit: 0,
        };

        assert!(price1 < price2);
        assert!(price2 > price1);
        assert!(price1 <= price2);
        assert!(price2 >= price1);
        assert!(price1 != price2);

        assert!(price2 < price3);
        assert!(price3 > price2);
        assert!(price2 <= price3);
        assert!(price3 >= price2);
        assert!(price2 != price3);
    }

    #[test]
    fn test_price_partial_ord() {
        let price1 = Price {
            main_unit: 10,
            sub_unit: 50,
        };
        let price2 = Price {
            main_unit: 10,
            sub_unit: 75,
        };
        let price3 = Price {
            main_unit: 12,
            sub_unit: 0,
        };

        assert_eq!(price1.partial_cmp(&price2), Some(Ordering::Less));
        assert_eq!(price2.partial_cmp(&price1), Some(Ordering::Greater));
        assert_eq!(price1.partial_cmp(&price2), Some(Ordering::Less));
        assert_eq!(price2.partial_cmp(&price1), Some(Ordering::Greater));
        assert_eq!(price1.partial_cmp(&price2), Some(Ordering::Less));

        assert_eq!(price2.partial_cmp(&price3), Some(Ordering::Less));
        assert_eq!(price3.partial_cmp(&price2), Some(Ordering::Greater));
        assert_eq!(price2.partial_cmp(&price3), Some(Ordering::Less));
        assert_eq!(price3.partial_cmp(&price2), Some(Ordering::Greater));
        assert_eq!(price2.partial_cmp(&price3), Some(Ordering::Less));
    }
}
