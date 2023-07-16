use core::fmt;

// ToDo: Multithread Read/Write lock this
#[derive(Default, Eq, PartialEq, PartialOrd, Debug, Clone)]
pub struct IdentifiableOrder {
    // This shouldn't be an i64 if it's used for production
    id: u64,
    qty: u64,
}

impl IdentifiableOrder {
    pub fn new(id: u64, qty: u64) -> Self {
        Self { id, qty }
    }
}

impl IdentifiableOrder {
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_qty(&self) -> u64 {
        self.qty
    }

    pub fn set_qty(&mut self, qty: u64) {
        self.qty = qty;
    }
}

impl fmt::Display for IdentifiableOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order id: {} \n Order Quantity: {}", self.id, self.qty)
    }
}
