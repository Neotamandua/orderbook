use core::fmt;
use std::collections::{HashSet, VecDeque};

use indexmap::IndexMap;
use price::Price;
use serde::{Deserialize, Serialize};

use super::identifiable_order::IdentifiableOrder;

/// An immutable aggregate snapshot of one order-book price level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceLevel {
    price: Price,
    qty: u64,
}

impl PriceLevel {
    pub(super) fn from_orders(
        price: Price,
        orders: &VecDeque<IdentifiableOrder>,
    ) -> Result<Self, QuantityOverflow> {
        let qty = orders.iter().try_fold(0_u64, |total, order| {
            total
                .checked_add(order.qty())
                .ok_or(QuantityOverflow { price })
        })?;

        Ok(Self { price, qty })
    }

    /// Returns the canonical price for this level.
    #[must_use]
    pub const fn price(self) -> Price {
        self.price
    }

    /// Returns the aggregate open quantity at this level.
    #[must_use]
    pub const fn qty(self) -> u64 {
        self.qty
    }
}

/// The aggregate quantity at a price level exceeded the supported `u64` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantityOverflow {
    price: Price,
}

impl QuantityOverflow {
    /// Returns the price whose aggregate quantity overflowed.
    #[must_use]
    pub const fn price(self) -> Price {
        self.price
    }
}

impl fmt::Display for QuantityOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "aggregate order quantity at price {} exceeds the supported range",
            self.price
        )
    }
}

impl std::error::Error for QuantityOverflow {}

/// Order represents a limit order with a price and an `IdentifiableOrder`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Order {
    price: Price,
    price_level_index: Option<usize>,
    details: IdentifiableOrder,
}

impl Order {
    /// Create a new `Order`
    ///
    /// # Arguments
    ///
    /// * `price` - The price of the order
    /// * `identifiable_order` - The order with an id and quantity
    #[must_use]
    pub fn new(price: Price, identifiable_order: IdentifiableOrder) -> Self {
        Self {
            price,
            price_level_index: None,
            details: identifiable_order,
        }
    }

    /// Get the price of the order
    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// Get the index of the price level in the order list
    #[must_use]
    pub fn price_level_index(&self) -> Option<usize> {
        self.price_level_index
    }

    /// Get the order as a mutable reference
    pub fn order_mut(&mut self) -> &mut IdentifiableOrder {
        &mut self.details
    }

    /// Get the order as a reference
    #[must_use]
    pub fn order(&self) -> &IdentifiableOrder {
        &self.details
    }
}

/// FIFO orders and their identifiers at one price level.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(super) struct OrderQueue {
    orders: VecDeque<IdentifiableOrder>,
    identifiers: HashSet<u64>,
}

impl OrderQueue {
    fn with_order(order: IdentifiableOrder) -> Self {
        let mut queue = Self::default();
        let inserted = queue.push_back(order);
        debug_assert!(inserted);
        queue
    }

    pub(super) fn orders(&self) -> &VecDeque<IdentifiableOrder> {
        &self.orders
    }

    pub(super) fn contains_identifier(&self, identifier: u64) -> bool {
        self.identifiers.contains(&identifier)
    }

    pub(super) fn push_back(&mut self, order: IdentifiableOrder) -> bool {
        if !self.identifiers.insert(order.id()) {
            return false;
        }

        self.orders.push_back(order);
        true
    }

    pub(super) fn front(&self) -> Option<&IdentifiableOrder> {
        self.orders.front()
    }

    pub(super) fn front_mut(&mut self) -> Option<&mut IdentifiableOrder> {
        self.orders.front_mut()
    }

    pub(super) fn pop_front(&mut self) -> Option<IdentifiableOrder> {
        let order = self.orders.pop_front()?;
        let removed = self.identifiers.remove(&order.id());
        debug_assert!(removed);
        Some(order)
    }

    pub(super) fn remove(&mut self, identifier: u64) -> bool {
        if !self.identifiers.contains(&identifier) {
            return false;
        }

        let index = self
            .orders
            .iter()
            .position(|order| order.id() == identifier)
            .expect("identifier index must match the order queue");
        self.orders.remove(index);
        let removed = self.identifiers.remove(&identifier);
        debug_assert!(removed);
        true
    }

    pub(super) fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

/// `IndexMap` to keep track of all orders
type Orders = IndexMap<Price, OrderQueue>;

/// `OrderList` represents sell-side or buy-side for a specific financial instrument.
/// It uses an `IndexMap` data structure [Orders] where the keys are canonical tick prices
/// and the values are queues of orders (`IdentifiableOrder`) at that price.
/// The vector is a time priority list for orders at the given price, where the first element is the first order to be matched.
/// Together with the price as a key in the `IndexMap`, two `OrderList` result in a price/time priority orderbook
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OrderList {
    pub(super) order_list: Orders,
}

// Non mutating functions
impl OrderList {
    pub(crate) fn amount_open_orders(&self) -> usize {
        self.order_list
            .values()
            .map(|orders| orders.orders.len())
            .sum()
    }

    pub(crate) fn contains_identifier(&self, price: Price, identifier: u64) -> bool {
        self.order_list
            .get(&price)
            .is_some_and(|orders| orders.contains_identifier(identifier))
    }
}

// Mutating functions
impl OrderList {
    /// Inserts a limit order at the right price and fifo queue position
    pub(crate) fn insert_order(&mut self, order: Order) -> bool {
        // Check if Price level exists
        if let Some(orders_on_price_level) = self.order_list.get_mut(&order.price) {
            // Add order to existing price level FIFO Queue
            return orders_on_price_level.push_back(order.details); // O(1)
        }

        // Create new price level
        let new_fifo_queue = OrderQueue::with_order(order.details);
        self.order_list.insert(order.price, new_fifo_queue); // O(1)

        /*
        Sort the Indexmap, so that the new price level is at the correct position
        Keys will never exist twice, so unstable sort is possible
        Uses Rayon parallelization
        */

        self.order_list.par_sort_unstable_keys(); // O(n log n + c)

        true
    }
}

impl fmt::Display for OrderList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vector: Vec<String> = Vec::with_capacity(self.order_list.len());
        for (price, orders) in &self.order_list {
            let level = PriceLevel::from_orders(*price, orders.orders()).map_err(|_| fmt::Error)?;
            vector.push(format!("{}, {}", level.price(), level.qty()));
        }
        write!(f, "{vector:#?}")
    }
}
