use core::fmt;

use serde::{Deserialize, Serialize};

/// `IdentifiableOrder` is a struct that represents an order with an id and quantity
///
/// TODO: Multithread Read/Write lock this
#[derive(Default, Eq, PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiableOrder {
    id: u64,
    qty: u64,
}

impl IdentifiableOrder {
    /// Create a new `IdentifiableOrder`
    ///
    /// # Arguments
    ///
    /// * `id` - A u64 that represents the order id
    /// * `qty` - A u64 that represents the order quantity
    #[must_use]
    pub fn new(id: u64, qty: u64) -> Self {
        Self { id, qty }
    }
}

impl IdentifiableOrder {
    /// Get the id of the order
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the quantity of the order
    #[must_use]
    pub fn qty(&self) -> u64 {
        self.qty
    }

    /// Set the quantity of the order
    pub fn set_qty(&mut self, qty: u64) {
        self.qty = qty;
    }
}

impl fmt::Display for IdentifiableOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order id: {} \n Order Quantity: {}", self.id, self.qty)
    }
}
