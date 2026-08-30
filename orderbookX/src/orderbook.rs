//! `OrderBook` for the Matching Engine
//!
//! The `OrderBook` is the central data structure of the matching engine.
//! All matching engine logic is residing in the `OrderBook`.
//!
//! The `OrderBook` is responsible for inserting, removing, matching orders and executing trades.
//!
//! Normally, the orderbook should never be responsible for providing read-only information that needs transformation i.e.,
//! additional cost to compute a value from an existing one, if it can be done on the client side.

mod identifiable_order;
mod orders;
use core::fmt;
use std::collections::VecDeque;

pub use identifiable_order::IdentifiableOrder;
pub use orders::{Order, PriceLevel, QuantityOverflow};
use price::Price;
use tracing::{debug, trace};

use crate::{
    orderbook::orders::{OrderList, OrderQueue},
    traits::matching_engine::Orders,
};

/// `OrderBook` struct representing the orderbook
///
/// The `OrderBook` contains two `OrderList`, one for the buy side and one for the sell side.
#[derive(Default, Debug)]
pub struct OrderBook {
    bids: OrderList,
    asks: OrderList,
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buy:\n{}\nSell:\n{}", self.bids, self.asks)
    }
}

impl OrderBook {
    /// Create a new `OrderBook`
    #[must_use]
    pub fn new(bids: OrderList, asks: OrderList) -> Self {
        Self { bids, asks }
    }

    /// Create an `OrderBook` from serialized state
    #[must_use]
    pub fn new_from_state() -> Self {
        unimplemented!("Create OrderBook from serialized state")
    }

    /// Serialize `OrderBook` state and shut down
    pub fn exit_to_state(&self) {
        unimplemented!("Serialize OrderBook state")
    }
    /// Creates a small deterministic order book for examples and tests.
    #[must_use]
    pub fn create_test_orderbook() -> Self {
        let mut order_book = OrderBook::default();

        order_book.insert_buy_order(Order::new(
            Price::from_ticks(100),
            IdentifiableOrder::new(1, 50),
        ));
        order_book.insert_buy_order(Order::new(
            Price::from_ticks(200),
            IdentifiableOrder::new(2, 50),
        ));
        order_book.insert_sell_order(Order::new(
            Price::from_ticks(300),
            IdentifiableOrder::new(3, 50),
        ));
        order_book
    }
}

/// Non mutating functions used for polling
/// Should be used mainly for test & debugging purposes
impl OrderBook {
    /// Get the amount of open limit orders in the orderbook.
    /// Sum of open orders on buy and sell side.
    #[must_use]
    pub fn sum_open_orders(&self) -> usize {
        self.bids.amount_open_orders() + self.asks.amount_open_orders()
    }

    /// Returns current market price.
    /// Note: Should be used for test & debugging purposes
    ///
    /// Current market price is defined as the highest bid price currently in the orderbook.
    /// If no bids exist, then the price is the lowest ask price in the orderbook.
    ///
    /// It is **not** defined as the last matched trade price, users should receive this
    /// based on the last matched trade through events.
    #[must_use]
    pub fn market_price(&self) -> Option<&Price> {
        if let Some(price) = self
            .bids
            .order_list
            .last()
            .map(|highest_bids| highest_bids.0)
        {
            Some(price)
        } else {
            self.asks
                .order_list
                .first()
                .map(|lowest_asks| lowest_asks.0)
        }
    }

    /// Get the highest bid price from the orderbook
    /// Note: Should be used for test & debugging purposes
    /// TODO: Remove Option<&Price> and return Price directly
    #[must_use]
    pub fn highest_bid_price(&self) -> Option<&Price> {
        // get highest bid from buy side
        if let Some((price, _)) = self.bids.order_list.last() {
            Some(price)
        } else {
            None
        }
    }

    /// Get the lowest ask price from the orderbook
    /// Note: Should be used for test & debugging purposes
    /// TODO: Remove Option<&Price> and return Price directly
    #[must_use]
    pub fn lowest_ask_price(&self) -> Option<&Price> {
        // get lowest ask price from sell side
        if let Some((price, _)) = self.asks.order_list.first() {
            Some(price)
        } else {
            None
        }
    }

    /// Get the highest bid price and all orders at that price level
    #[must_use]
    pub fn highest_bids(&self) -> Option<(&Price, &VecDeque<IdentifiableOrder>)> {
        // get highest bidders from buy side
        self.bids
            .order_list
            .last()
            .map(|(price, orders)| (price, orders.orders()))
    }

    /// Get the lowest ask price and all orders at that price level
    #[must_use]
    pub fn lowest_asks(&self) -> Option<(&Price, &VecDeque<IdentifiableOrder>)> {
        // get lowest asks from sell side
        self.asks
            .order_list
            .first()
            .map(|(price, orders)| (price, orders.orders()))
    }

    /// Note: Should be used for test & debugging purposes
    /// Getting bids and asks should be handled through some event system/websocket
    /// that sends out information on changes, matches etc., and not through polling
    ///
    /// # Errors
    ///
    /// Returns [`QuantityOverflow`] if the aggregate quantity at a selected
    /// price level exceeds `u64`.
    pub fn asks(&self, depth: usize) -> Result<Vec<PriceLevel>, QuantityOverflow> {
        self.asks
            .order_list
            .iter()
            .take(depth)
            .map(|(price, orders)| PriceLevel::from_orders(*price, orders.orders()))
            .collect()
    }

    /// Note: Should be used for test & debugging purposes
    /// Getting bids and asks should be handled through some event system/websocket
    /// that sends out information on changes, matches etc., and not through polling
    ///
    /// Returns up to `depth` price levels closest to the market while preserving
    /// the order book's ascending internal price order.
    ///
    /// # Errors
    ///
    /// Returns [`QuantityOverflow`] if the aggregate quantity at a selected
    /// price level exceeds `u64`.
    pub fn bids(&self, depth: usize) -> Result<Vec<PriceLevel>, QuantityOverflow> {
        self.bids
            .order_list
            .iter()
            .rev()
            .take(depth)
            .rev()
            .map(|(price, orders)| PriceLevel::from_orders(*price, orders.orders()))
            .collect()
    }
}

/// Mutating functions
impl OrderBook {
    fn highest_bids_mut(&mut self) -> Option<(&Price, &mut OrderQueue)> {
        // get highest bidders from buy side
        self.bids.order_list.last_mut()
    }

    fn lowest_asks_mut(&mut self) -> Option<(&Price, &mut OrderQueue)> {
        // get lowest asks from sell side
        self.asks.order_list.first_mut()
    }

    /// Insert Limit Buy Order **without** matching
    fn insert_buy_order(&mut self, insert_order: Order) -> bool {
        let order_list = &mut self.bids;
        // Insert Limit Order
        order_list.insert_order(insert_order)
    }

    /// Insert Limit Sell Order **without** matching
    fn insert_sell_order(&mut self, insert_order: Order) -> bool {
        let order_list = &mut self.asks;
        // Insert Limit Order
        order_list.insert_order(insert_order)
    }

    /// Remove a buy order by price and identifier.
    ///
    /// Returns `true` when an order was removed.
    pub fn remove_buy_order(&mut self, remove_order: &Order) -> bool {
        let order_book = &mut self.bids;
        Self::remove_order(remove_order, order_book)
    }

    /// Remove a sell order by price and identifier.
    ///
    /// Returns `true` when an order was removed.
    pub fn remove_sell_order(&mut self, remove_order: &Order) -> bool {
        let order_book = &mut self.asks;
        Self::remove_order(remove_order, order_book)
    }

    /// Removes the given price level from the ask-side (sell-side) of the orderbook
    fn delete_sellside_price_level(&mut self, key: Price) -> Option<OrderQueue> {
        // ToDo: With some indexing magic in the matching functions this might be able to use .remove
        self.asks.order_list.shift_remove(&key) // O(n)
    }

    /// Removes the given price level from the bid-side (buy-side) of the orderbook
    fn delete_buyside_price_level(&mut self, key: Price) -> Option<OrderQueue> {
        // ToDo: With some indexing magic in the matching functions this might be able to use .remove
        self.bids.order_list.shift_remove(&key) // O(n)
    }

    /// Order Modification: Remove/Cancel an Order
    fn remove_order(remove_order: &Order, order_book: &mut OrderList) -> bool {
        let price = remove_order.price();
        let identifier = remove_order.order().id();
        let Some(orders_on_price_level) = order_book.order_list.get_mut(price) else {
            return false;
        };
        if !orders_on_price_level.remove(identifier) {
            return false;
        }

        if orders_on_price_level.is_empty() {
            // Preserve the sorted price-level order used by matching.
            order_book.order_list.shift_remove(price);
        }

        true
    }
}

impl Orders for OrderBook {
    fn market_buy_until(&mut self, mut buy_order: Order) -> (bool, u64, u64, Order) {
        // Market buy order quantity
        let market_buy_qty = buy_order.order().qty();

        // Get first sell orders to match
        let Some(mut price_level) = self.lowest_asks_mut() else {
            // Orderbook is empty, nothing to match, order stays the same
            return (true, market_buy_qty, 0, buy_order);
        };
        // Market price level right now
        let mut market_price = *price_level.0;

        // Check until
        if buy_order.price() < &market_price {
            // buy order price is lower than any asks, nothing to match, order stays the same
            return (true, market_buy_qty, 0, buy_order);
        }

        // Orders at price level
        let mut orders = price_level.1;

        debug!("Market buy order quantity: {}", market_buy_qty);

        // Accumulates the qty until it reaches the orders amount or reaches buy_order price
        let mut accumulator: u64 = 0;
        while accumulator < market_buy_qty && &market_price <= buy_order.price() {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.qty();
                // Settle execution
                if accumulator <= market_buy_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_buy_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.delete_sellside_price_level(market_price).unwrap();
                        // We are done here
                        break;
                    }
                    // Fire Event
                    trace!("Order to remove: {}", order_to_remove);
                } else {
                    // We need to reduce matched order in the orderbook (partial fill)
                    let order_to_reduce = orders.front_mut().unwrap();
                    // remaining unfilled amount on last maker limit order
                    let remainder = accumulator - market_buy_qty;
                    // Only reduce FIFO Queue Orders
                    debug!("Order to reduce: {}", order_to_reduce);
                    order_to_reduce.set_qty(remainder);
                    // Set accumulator to final filled amount
                    accumulator -= remainder;
                    // We can break since we are finished here
                    trace!("Unfilled amount of last limit sell order: {}", remainder);
                    break;
                }
                // Go to next element on same price level
            } else {
                // No orders left at the given price, go to a higher price level
                // 1. Remove price level from orderbook
                let _ = self.delete_sellside_price_level(market_price).unwrap();
                debug!("Removed Price Level: {}", market_price);
                // 2. Update to next price level
                if let Some(next_matches) = self.lowest_asks_mut() {
                    // Update Matches
                    price_level = next_matches;
                    // Update Market Price
                    market_price = *price_level.0;
                    // Update Orders
                    orders = price_level.1;
                } else {
                    // Orderbook is empty
                    break;
                    // Unusual Outcome:
                    // All orders are removed and the orderbook is empty
                    // Limit buy is only partially executed
                }
            }
        }
        // Reduce order by filled amount
        buy_order.order_mut().set_qty(market_buy_qty - accumulator);
        // Usual Outcome:
        // All orders are removed including indexmap price levels if they are completely filled
        // The last remaining order in the FIFO queue of the given price level was either exactly equal and was completely filled or only partially filled
        (true, market_buy_qty, accumulator, buy_order)
    }

    fn market_sell_until(&mut self, mut sell_order: Order) -> (bool, u64, u64, Order) {
        let market_sell_qty = sell_order.order().qty();

        // Get first buy orders to match
        let Some(mut price_level) = self.highest_bids_mut() else {
            // Orderbook is empty, nothing to match, order stays the same
            return (true, market_sell_qty, 0, sell_order);
        };
        // Market price level right now
        let mut market_price = *price_level.0;

        // Check until
        if sell_order.price() > &market_price {
            // sell order price is higher than any bids, nothing to match, order stays the same
            return (true, market_sell_qty, 0, sell_order);
        }

        let mut orders = price_level.1;

        debug!("Market buy order quantity: {}", market_sell_qty);

        // Accumulates the qty until it reaches the orders amount or reaches sell_order price
        let mut accumulator: u64 = 0;
        while accumulator < market_sell_qty && sell_order.price() <= &market_price {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.qty();
                // Settle execution
                if accumulator <= market_sell_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_sell_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.delete_buyside_price_level(market_price).unwrap();
                        break;
                    }
                    // Fire Event
                    trace!("Order to remove: {}", order_to_remove);
                } else {
                    // We need to reduce matched order in the orderbook (partial fill)
                    let order_to_reduce = orders.front_mut().unwrap();
                    // remaining unfilled amount on last maker limit order
                    let remainder = accumulator - market_sell_qty;
                    // Only reduce FIFO Queue Orders
                    debug!("Order to reduce: {}", order_to_reduce);
                    order_to_reduce.set_qty(remainder);
                    // Set accumulator to final filled amount
                    accumulator -= remainder;
                    trace!("Unfilled amount of last limit sell order: {}", remainder);
                    break;
                }
                // Go to next element on same price level
            } else {
                // No orders left at the given price, go to a lower price level
                let _ = self.delete_buyside_price_level(market_price).unwrap();
                debug!("Removed Price Level: {}", market_price);

                if let Some(next_matches) = self.highest_bids_mut() {
                    price_level = next_matches;

                    market_price = *price_level.0;

                    orders = price_level.1;
                } else {
                    break;
                }
            }
        }

        sell_order
            .order_mut()
            .set_qty(market_sell_qty - accumulator);

        (true, market_sell_qty, accumulator, sell_order)
    }

    /// Execute Market Buy Order.
    ///
    /// Behaves like an IOC Market Order, cancels any unfilled amount if orderbook lacks liquidity.
    /// Removes Liquidity/Orders from the Orderbook.
    fn market_buy(&mut self, buy_order: Order) -> (bool, u64, u64) {
        // Market buy order quantity
        let market_buy_qty = buy_order.order().qty();

        // Get first sell orders to match
        let Some(mut price_level) = self.lowest_asks_mut() else {
            // Orderbook is empty
            return (false, market_buy_qty, 0);
        };
        // Market price level right now
        let mut market_price = *price_level.0;
        // Orders at price level
        let mut orders = price_level.1;

        debug!("Market buy order quantity: {}", market_buy_qty);

        // Accumulates the qty until it reaches the orders amount
        let mut accumulator: u64 = 0;
        while accumulator < market_buy_qty {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.qty();
                // Settle execution
                if accumulator <= market_buy_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_buy_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.delete_sellside_price_level(market_price).unwrap();
                        // We are done here
                        break;
                    }
                    // Fire Event
                    trace!("Order to remove: {}", order_to_remove);
                } else {
                    // We need to reduce matched order in the orderbook (partial fill)
                    let order_to_reduce = orders.front_mut().unwrap();
                    // remaining unfilled amount on last maker limit order
                    let remainder = accumulator - market_buy_qty;
                    // Only reduce FIFO Queue Orders
                    debug!("Order to reduce: {}", order_to_reduce);
                    order_to_reduce.set_qty(remainder);
                    // Set accumulator to final filled amount
                    accumulator -= remainder;
                    // We can break since we are finished here
                    trace!("Unfilled amount of last limit sell order: {}", remainder);
                    break;
                }
                // Go to next element on same price level
            } else {
                // No orders left at the given price, go to a higher price level
                // 1. Remove price level from orderbook
                let _ = self.delete_sellside_price_level(market_price).unwrap();
                debug!("Removed Price Level: {}", market_price);
                // 2. Update to next price level
                if let Some(next_matches) = self.lowest_asks_mut() {
                    // Update Matches
                    price_level = next_matches;
                    // Update Market Price
                    market_price = *price_level.0;
                    // Update Orders
                    orders = price_level.1;
                } else {
                    // Orderbook is empty
                    break;
                    // Unusual Outcome:
                    // All orders are removed and the orderbook is empty
                    // Market buy is only partially executed
                }
            }
        }

        // Usual Outcome:
        // All orders are removed including indexmap price levels if they are completely filled
        // The last remaining order in the FIFO queue of the given price level was either exactly equal and was completely filled or only partially filled
        (true, market_buy_qty, accumulator)
    }

    /// Execute Market Sell Order.
    ///
    /// Behaves like an IOC Market Order, cancels any unfilled amount if orderbook lacks liquidity.
    /// Removes Liquidity/Orders from the Orderbook.
    fn market_sell(&mut self, sell_order: Order) -> (bool, u64, u64) {
        // Market sell order quantity
        let market_sell_qty = sell_order.order().qty();

        // Get first buy orders to match
        debug!("Highest Bids: {:?}", self.highest_bids_mut());
        let Some(mut price_level) = self.highest_bids_mut() else {
            // Orderbook is empty
            return (false, market_sell_qty, 0);
        };
        debug!("Price level: {:?}", price_level);
        // Market price level right now
        let mut market_price = *price_level.0;
        // Orders at price level
        let mut orders = price_level.1;

        debug!("Market sell order quantity: {}", market_sell_qty);

        // Accumulates the qty until it reaches the orders amount
        let mut accumulator: u64 = 0;
        while accumulator < market_sell_qty {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.qty();
                // Settle execution
                if accumulator <= market_sell_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_sell_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.delete_buyside_price_level(market_price).unwrap();
                        // We are done here
                        break;
                    }
                    // Fire Event
                    trace!("Order to remove: {}", order_to_remove);
                } else {
                    // We need to reduce matched order in the orderbook (partial fill)
                    let order_to_reduce = orders.front_mut().unwrap();
                    // remaining unfilled amount on last maker limit order
                    let remainder = accumulator - market_sell_qty;
                    // Only reduce FIFO Queue Orders
                    debug!("Order to reduce: {}", order_to_reduce);
                    order_to_reduce.set_qty(remainder);
                    // Set accumulator to final filled amount
                    accumulator -= remainder;
                    // We can break since we are finished here
                    trace!("Unfilled amount of last limit buy order: {}", remainder);
                    break;
                }
                // Go to next element on same price level
            } else {
                // No orders left at the given price, go to a lower price level
                debug!("Orders in the FIFO Queue: {:?}", orders);
                debug!("Current Price Level: {}", market_price);
                // 1. Remove price level from orderbook
                let _ = self.delete_buyside_price_level(market_price).unwrap();
                debug!("Removed Price Level: {}", market_price);
                // 2. Update to next price level
                if let Some(next_matches) = self.highest_bids_mut() {
                    // Update Matches
                    price_level = next_matches;
                    // Update Market Price
                    market_price = *price_level.0;
                    // Update Orders
                    orders = price_level.1;
                } else {
                    // Orderbook is empty
                    break;
                    // Unusual Outcome:
                    // All orders are removed and the orderbook is empty
                    // Market sell is only partially executed
                }
            }
        }

        // Usual Outcome:
        // All orders are removed including indexmap price levels if they are completely filled
        // The last remaining order in the FIFO queue of the given price level was either exactly equal and was completely filled or only partially filled
        (true, market_sell_qty, accumulator)
    }

    /// Insert Limit Buy Order by matching and inserting the remaining order into the orderbook
    fn insert_limit_buy(&mut self, order: Order) -> bool {
        if self
            .bids
            .contains_identifier(*order.price(), order.order().id())
        {
            return false;
        }

        // TODO: change this logic. The insert_limit_buy should not invoke matching every time
        // TODO: return result, qty, filled, order
        let (_result, _qty, _filled, order) = self.market_buy_until(order);
        if order.order().qty() > 0 {
            return self.insert_buy_order(order);
        }

        true
    }

    /// Insert Limit Sell Order by matching and inserting the remaining order into the orderbook
    fn insert_limit_sell(&mut self, order: Order) -> bool {
        if self
            .asks
            .contains_identifier(*order.price(), order.order().id())
        {
            return false;
        }

        // TODO: change this logic. The insert_limit_buy should not invoke matching every time
        // TODO: return result, qty, filled, order
        let (_result, _qty, _filled, order) = self.market_sell_until(order);
        if order.order().qty() > 0 {
            return self.insert_sell_order(order);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    use proptest::prelude::*;
    use rand::Rng;
    use tracing::{event, Level};
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    // Debug Tracing for tests
    fn initialize_tracing() {
        // Create a `LevelFilter` with the desired tracing level
        let filter = tracing::Level::DEBUG;

        // Create a `FmtSubscriber` with the filter and desired formatting options
        let subscriber = tracing_subscriber::fmt().with_max_level(filter).finish();

        // Set the subscriber as the global default
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    }

    proptest! {
       #[test]
       fn test_order_book(_qty: u32, _ticks: u64) {
            let _order_book = fill_bids_pseudorandom();
       }
    }

    // Pseudorandom Fill
    fn fill_bids_pseudorandom() -> (OrderList, Vec<Order>) {
        let mut bid_list = OrderList::default();
        let mut remove_list = vec![];
        let mut rng = rand::thread_rng();

        for identifier in 0..1000_u64 {
            let price = rng.gen_range(1..=100_u64);
            let mut hasher = DefaultHasher::new();
            hasher.write_u64(price);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(identifier, qty);
            let order = Order::new(Price::from_ticks(price * 100), identifiable_order);
            remove_list.push(order.clone());
            bid_list.insert_order(order);
        }

        (bid_list, remove_list)
    }

    // Pseudorandom Fill
    fn fill_asks_pseudorandom() -> (OrderList, Vec<Order>) {
        let mut ask_list = OrderList::default();
        let mut remove_list = vec![];
        let mut rng = rand::thread_rng();

        for identifier in 0..1000_u64 {
            let price = rng.gen_range(1..=100_u64);
            let mut hasher = DefaultHasher::new();
            hasher.write_u64(price);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(identifier, qty);
            let order = Order::new(Price::from_ticks(price * 100), identifiable_order);
            remove_list.push(order.clone());
            ask_list.insert_order(order);
        }

        (ask_list, remove_list)
    }

    // Pseudorandom Orders
    fn generate_orders_pseudorandom(amount: u64) -> Vec<Order> {
        let mut orders = vec![];
        let mut rng = rand::thread_rng();

        for identifier in 0..amount {
            let price = rng.gen_range(1..=100_u64);
            let mut hasher = DefaultHasher::new();
            hasher.write_u64(price);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(identifier, qty);
            let order = Order::new(Price::from_ticks(price * 100), identifiable_order);
            orders.push(order);
        }

        orders
    }

    use super::*;
    #[test]
    fn test_inserts() {
        let (buy_side, _) = fill_bids_pseudorandom();
        assert_eq!(buy_side.order_list.len(), 100);
        let (sell_side, _) = fill_asks_pseudorandom();
        assert_eq!(sell_side.order_list.len(), 100);
    }

    #[test]
    fn decimal_boundary_prices_sort_and_match_in_numerical_order() {
        let prices = [
            Price::try_from(1.99).unwrap(),
            Price::try_from(2.00).unwrap(),
            Price::try_from(2.01).unwrap(),
        ];
        let mut order_book = OrderBook::default();

        for price in prices.into_iter().rev() {
            order_book.insert_sell_order(Order::new(price, IdentifiableOrder::new(1, 1)));
        }

        assert_eq!(
            order_book
                .asks
                .order_list
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            prices
        );

        order_book.market_buy(Order::new(Price::MAX, IdentifiableOrder::new(2, 1)));
        assert_eq!(order_book.lowest_ask_price().copied(), Some(prices[1]));
    }

    #[test]
    fn depth_queries_select_price_levels_closest_to_the_market() {
        let mut order_book = OrderBook::default();

        for ticks in [300, 100, 200] {
            let order = Order::new(Price::from_ticks(ticks), IdentifiableOrder::new(ticks, 1));
            order_book.insert_buy_order(order.clone());
            order_book.insert_sell_order(order);
        }
        order_book.insert_buy_order(Order::new(
            Price::from_ticks(300),
            IdentifiableOrder::new(301, 4),
        ));

        assert_eq!(
            order_book
                .bids(2)
                .unwrap()
                .into_iter()
                .map(|level| (level.price(), level.qty()))
                .collect::<Vec<_>>(),
            [(Price::from_ticks(200), 1), (Price::from_ticks(300), 5)]
        );
        assert_eq!(
            order_book
                .asks(2)
                .unwrap()
                .into_iter()
                .map(|level| (level.price(), level.qty()))
                .collect::<Vec<_>>(),
            [(Price::from_ticks(100), 1), (Price::from_ticks(200), 1)]
        );
    }

    #[test]
    fn depth_aggregation_reports_quantity_overflow() {
        let mut order_book = OrderBook::default();
        let price = Price::from_ticks(100);

        order_book.insert_buy_order(Order::new(price, IdentifiableOrder::new(1, u64::MAX)));
        order_book.insert_buy_order(Order::new(price, IdentifiableOrder::new(2, 1)));

        assert_eq!(order_book.bids(1).unwrap_err().price(), price);
    }

    #[test]
    fn cancellation_uses_identifier_and_preserves_price_order() {
        let mut order_book = OrderBook::default();

        for (ticks, identifier, qty) in [(100, 1, 10), (200, 2, 20), (200, 3, 30), (300, 4, 40)] {
            order_book.insert_buy_order(Order::new(
                Price::from_ticks(ticks),
                IdentifiableOrder::new(identifier, qty),
            ));
        }

        let cancellation = Order::new(Price::from_ticks(200), IdentifiableOrder::new(3, 0));
        assert!(order_book.remove_buy_order(&cancellation));
        assert_eq!(
            order_book.bids.order_list[&Price::from_ticks(200)]
                .orders()
                .iter()
                .map(IdentifiableOrder::id)
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(!order_book.remove_buy_order(&cancellation));

        let remaining_order = Order::new(Price::from_ticks(200), IdentifiableOrder::new(2, 0));
        assert!(order_book.remove_buy_order(&remaining_order));
        assert_eq!(
            order_book
                .bids
                .order_list
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            [Price::from_ticks(100), Price::from_ticks(300)]
        );
        assert_eq!(
            order_book.highest_bid_price(),
            Some(&Price::from_ticks(300))
        );

        let unknown_price = Order::new(Price::from_ticks(999), IdentifiableOrder::new(1, 0));
        assert!(!order_book.remove_buy_order(&unknown_price));
    }

    #[test]
    fn duplicate_identifiers_at_one_price_are_rejected_before_matching() {
        let mut order_book = OrderBook::default();
        let price = Price::from_ticks(200);

        assert!(order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(7, 10),)));
        assert!(!order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(7, 99),)));

        let (_, orders) = order_book.highest_bids().unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders.front().unwrap().qty(), 10);

        // The price remains part of the cancellation key, so reusing an
        // identifier at another price is unambiguous and remains supported.
        assert!(order_book.insert_limit_buy(Order::new(
            Price::from_ticks(201),
            IdentifiableOrder::new(7, 20),
        )));
        let cancellation = Order::new(price, IdentifiableOrder::new(7, 0));
        assert!(order_book.remove_buy_order(&cancellation));
        assert_eq!(
            order_book.highest_bid_price(),
            Some(&Price::from_ticks(201))
        );
    }

    #[test]
    fn identifier_index_tracks_cancelled_and_filled_orders() {
        let mut order_book = OrderBook::default();
        let price = Price::from_ticks(200);

        assert!(order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(1, 10),)));
        assert!(order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(2, 10),)));

        let cancellation = Order::new(price, IdentifiableOrder::new(1, 0));
        assert!(order_book.remove_buy_order(&cancellation));
        assert!(order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(1, 10),)));

        order_book.market_sell(Order::new(Price::ZERO, IdentifiableOrder::new(99, 10)));
        assert!(order_book.insert_limit_buy(Order::new(price, IdentifiableOrder::new(2, 10),)));

        let (_, orders) = order_book.highest_bids().unwrap();
        assert_eq!(
            orders.iter().map(IdentifiableOrder::id).collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn test_inserts_remove() {
        let (buy_side, buy_remove_list) = fill_bids_pseudorandom();
        let (sell_side, sell_remove_list) = fill_bids_pseudorandom();
        let order_book = OrderBook {
            bids: buy_side,
            asks: sell_side,
        };
        remove(order_book, buy_remove_list, sell_remove_list);
    }

    fn remove(
        mut order_book: OrderBook,
        buy_remove_list: Vec<Order>,
        sell_remove_list: Vec<Order>,
    ) {
        debug!(
            "Before Remove Bid Orderbook length {}",
            order_book.bids.order_list.len()
        );
        for remove_order in buy_remove_list {
            assert!(order_book.remove_buy_order(&remove_order));
        }
        debug!(
            "After Remove Bid Orderbook length {}",
            order_book.bids.order_list.len()
        );
        debug!(
            "Before Remove Ask Orderbook length {}",
            order_book.asks.order_list.len()
        );
        for remove_order in sell_remove_list {
            assert!(order_book.remove_sell_order(&remove_order));
        }
        debug!(
            "After Remove Ask Orderbook length {}",
            order_book.asks.order_list.len()
        );
        assert_eq!(order_book.bids.order_list.len(), 0);
        assert_eq!(order_book.asks.order_list.len(), 0);
    }

    #[test]
    fn test_insert_match_remove() {
        let (bids, buy_remove_list) = fill_bids_pseudorandom();
        let (asks, sell_remove_list) = fill_bids_pseudorandom();
        let mut order_book = OrderBook { bids, asks };
        // Put in equivalent buy limit orders now as sell market orders
        // Empties the orderbook completely (qty of all sell market orders == qty of all buy limit orders)
        for order in buy_remove_list {
            order_book.market_sell(order);
        }

        // Put in equivalent sell limit orders now as buy market orders
        // Should empty the orderbook completely (qty of all buy market orders == qty of all sell limit orders)
        for order in sell_remove_list {
            order_book.market_buy(order);
        }
        assert_eq!(order_book.bids.order_list.len(), 0);
        assert_eq!(order_book.asks.order_list.len(), 0);
    }

    #[test]
    fn test_insert_random_limit_orders_remove() {
        let (bids, _) = fill_bids_pseudorandom();
        let orders = generate_orders_pseudorandom(1000);
        let mut order_book = OrderBook::new(bids, OrderList::default());

        for order in orders {
            order_book.insert_limit_sell(order);
        }
        debug!("{}", order_book);
    }

    /*
        Market Buy Tests
    */

    /// Market Buy Test with randomly filled orderbook
    #[test]
    fn test_market_buy_random() {
        let (sell_side, _) = fill_asks_pseudorandom();
        let mut order_book = OrderBook::new(OrderList::default(), sell_side);
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Normal Market Buy Test.
    #[test]
    fn test_market_buy_normal() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=10 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Market Buy Full Fill Test
    #[test]
    fn test_market_buy_full_fill() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 500);
        let result = order_book.market_buy(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 500, 500));
    }

    /// Market Buy more than Orderbook has test
    #[test]
    fn test_market_buy_exceeding_orderbook() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 500));
    }

    /// Orderbook Empty Market Buy Test
    #[test]
    fn test_market_buy_empty_book() {
        let mut order_book = OrderBook::default();
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (false, 512, 0));
    }

    /*
        Market Sell Tests
    */

    /// Market Sell Test with randomly filled orderbook
    #[test]
    fn test_market_sell_random() {
        let (buy_side, _) = fill_bids_pseudorandom();
        let mut order_book = OrderBook::new(buy_side, OrderList::default());
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Normal Market Sell Test.
    #[test]
    fn test_market_sell_normal() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=10 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_buy_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Market Sell Full Fill Test
    #[test]
    fn test_market_sell_full_fill() {
        //initialize_tracing();
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_buy_order(order);
        }
        debug!("Created Orderbook: {:?}", order_book);
        let identifiable_order = IdentifiableOrder::new(5, 500);
        let result = order_book.market_sell(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 500, 500));
    }

    /// Market Buy more than Orderbook has test
    #[test]
    fn test_market_sell_exceeding_orderbook() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::from_ticks(i * 100);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price, identifiable_order);
            order_book.insert_buy_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (true, 512, 500));
    }

    /// Orderbook Empty Market Buy Test
    #[test]
    fn test_market_sell_empty_book() {
        let mut order_book = OrderBook::default();
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::from_ticks(100), identifiable_order));
        assert_eq!(result, (false, 512, 0));
    }
}
