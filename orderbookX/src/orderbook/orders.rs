use core::fmt;
use std::collections::VecDeque;

use indexmap::IndexMap;
use rayon::vec;

use super::identifiable_order::IdentifiableOrder;
use crate::price::Price;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Order {
    price: Price,
    identifiable_order: IdentifiableOrder,
}

impl Order {
    pub fn new(price: Price, identifiable_order: IdentifiableOrder) -> Self {
        Self {
            price,
            identifiable_order,
        }
    }

    pub fn get_price(&self) -> &Price {
        &self.price
    }

    pub fn get_order_mut(&mut self) -> &mut IdentifiableOrder {
        &mut self.identifiable_order
    }

    pub fn get_order(&self) -> &IdentifiableOrder {
        &self.identifiable_order
    }
}

/// IndexMap to keep track of all orders
type Orders = IndexMap<Price, VecDeque<IdentifiableOrder>>;

/// OrderList represents sell-side or buy-side for a specific financial instrument.
/// It uses an IndexMap data structure [Orders] where the keys are prices (f64) for orders and the values are vectors (Vec) of orders (IdentifiableOrder) at that price.
/// The vector is a time priority list for orders at the given price, where the first element is the first order to be matched.
/// Together with the price as a key in the IndexMap, two OrderList result in a price/time priority orderbook
#[derive(Default, Debug)]
pub struct OrderList {
    pub order_list: Orders,
}

impl OrderList {
    /// Inserts a limit order at the right price and fifo queue position
    pub fn insert_order(&mut self, order: Order) {
        // Check if Price level exists
        if let Some(orders_on_price_level) = self.order_list.get_mut(&order.price) {
            // Add order to existing price level FIFO Queue
            orders_on_price_level.push_back(order.identifiable_order) // O(1)
        } else {
            // Create new price level
            let mut new_fifo_queue = VecDeque::with_capacity(2);
            new_fifo_queue.push_back(order.identifiable_order);
            self.order_list.insert(order.price, new_fifo_queue); // O(1)

            /*
            Sort the Indexmap, so that the new price level is at the correct position
            Keys will never exist twice, so unstable sort is possible
            Uses Rayon parallelization
            */

            self.order_list.par_sort_unstable_keys(); // O(n log n + c)
        }
    }
}

impl fmt::Display for OrderList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vector: Vec<String> = vec![];
        for price_level in self.order_list.clone().iter_mut() {
            let mut size_on_price_level: u64 = 0;
            for element in price_level.1.make_contiguous().iter() {
                size_on_price_level += element.get_qty();
            }
            vector.push(format!(
                "{}, {}",
                price_level.0.clone(),
                size_on_price_level
            ))
        }
        write!(f, "{:#?}", vector)
    }
}
