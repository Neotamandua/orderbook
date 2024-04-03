mod identifiable_order;
mod orders;
use core::fmt;
use std::collections::VecDeque;

pub use identifiable_order::IdentifiableOrder;
use indexmap::IndexMap;
pub use orders::Order;
use tracing::{debug, trace};

use self::orders::OrderList;
use crate::{
    price::Price,
    traits::matching_engine::{MatchingEngine, OrderType},
};

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
    pub fn new(bids: OrderList, asks: OrderList) -> Self {
        Self { bids, asks }
    }

    pub fn new_from_state() -> Self {
        // Create OrderBook from serialized state
        unimplemented!("Create OrderBook from serialized state")
    }

    pub fn exit_to_state(&self) {
        // Serialize OrderBook state and shut down
        unimplemented!("Serialize OrderBook state")
    }

    pub fn create_test_orderbook() -> Self {
        let mut order_book = OrderBook::default();

        order_book.insert_buy_order(Order::new(Price::new(1, 0), IdentifiableOrder::new(1, 50)));
        order_book.insert_buy_order(Order::new(Price::new(2, 0), IdentifiableOrder::new(2, 50)));
        order_book.insert_sell_order(Order::new(Price::new(3, 0), IdentifiableOrder::new(3, 50)));
        order_book
    }
}

// Non mutating functions
impl OrderBook {
    /// Get the amount of open limit orders in the orderbook
    pub fn get_amount_open_orders(&self) -> usize {
        self.bids.get_amount_open_orders() + self.asks.get_amount_open_orders()
    }

    /// Returns current market price
    /// Current market price is defined as the highest bid price currently in the orderbook.
    ///
    /// If not bids exist, then the price is the lowest ask price in the orderbook.
    pub fn get_price(&self) -> Option<&Price> {
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

    // Get the highest bid price from the orderbook
    pub fn highest_bid(&self) -> Option<(&Price, &IdentifiableOrder)> {
        // get highest bid from buy side
        if let Some((price, orders)) = self.bids.order_list.last() {
            // we can unwrap here as the first last call is already ensuring to have an element here?
            Some((price, orders.front().unwrap()))
        } else {
            None
        }
    }

    // Get the lowest ask price from the orderbook
    pub fn lowest_ask(&self) -> Option<(&Price, &IdentifiableOrder)> {
        // get lowest ask price from sell side
        if let Some((price, orders)) = self.asks.order_list.first() {
            Some((price, orders.front().unwrap()))
        } else {
            None
        }
    }

    pub fn highest_bids(&self) -> Option<(&Price, &VecDeque<IdentifiableOrder>)> {
        // get highest bidders from buy side
        self.bids.order_list.last()
    }

    pub fn lowest_asks(&self) -> Option<(&Price, &VecDeque<IdentifiableOrder>)> {
        // get lowest asks from sell side
        self.asks.order_list.first()
    }
}

// Mutating functions
impl OrderBook {
    fn highest_bids_mut(&mut self) -> Option<(&Price, &mut VecDeque<IdentifiableOrder>)> {
        // get highest bidders from buy side
        self.bids.order_list.last_mut()
    }

    fn lowest_asks_mut(&mut self) -> Option<(&Price, &mut VecDeque<IdentifiableOrder>)> {
        // get lowest asks from sell side
        self.asks.order_list.first_mut()
    }

    /// Insert Limit Buy Order **without** matching
    pub(crate) fn insert_buy_order(&mut self, insert_order: Order) {
        let order_list = &mut self.bids;
        // Insert Limit Order
        order_list.insert_order(insert_order)
    }

    /// Insert Limit Sell Order **without** matching
    pub(crate) fn insert_sell_order(&mut self, insert_order: Order) {
        let order_list = &mut self.asks;
        // Insert Limit Order
        order_list.insert_order(insert_order)
    }

    pub fn remove_buy_order(&mut self, remove_order: Order) {
        let order_book = &mut self.bids.order_list;
        Self::remove_order(remove_order, order_book)
    }

    pub fn remove_sell_order(&mut self, remove_order: Order) {
        let order_book = &mut self.asks.order_list;
        Self::remove_order(remove_order, order_book)
    }

    // ToDo: Rename this function, its hella confusing
    fn remove_ask_price_level(&mut self, key: &Price) -> Option<VecDeque<IdentifiableOrder>> {
        // ToDo: With some indexing magic in the matching functions this might be able to use remove
        self.asks.order_list.shift_remove(key) // O(n)
    }

    // ToDo: Rename this function, its hella confusing
    fn remove_bid_price_level(&mut self, key: &Price) -> Option<VecDeque<IdentifiableOrder>> {
        // ToDo: With some indexing magic in the matching functions this might be able to use remove
        self.bids.order_list.shift_remove(key) // O(n)
    }

    /// Order Modification: Remove/Cancel an Order
    fn remove_order(
        remove_order: Order,
        order_book: &mut IndexMap<Price, VecDeque<IdentifiableOrder>>,
    ) {
        if let Some(orders_on_price_level) = order_book.get_mut(remove_order.get_price()) {
            // If the first statement is wrong, the second never gets executed.
            // If the first statement is correct, the second statement never panics.
            if orders_on_price_level.len() == 1
                && orders_on_price_level.front().unwrap() == remove_order.get_order()
            {
                // If there is only one entry, we can delete the whole indexmap entry
                order_book.remove_entry(remove_order.get_price());
            } else {
                // Decide if we want to add price level index to orders or not
                //orders_on_price_level.remove(remove_order.get_price_level_index().unwrap());
                // If not we may do something like:
                for (i, order) in orders_on_price_level.iter().enumerate() {
                    if order == remove_order.get_order() {
                        // Multiple orders on that level, only delete the relevant entry
                        orders_on_price_level.remove(i);
                        break;
                    }
                }
                // ToDo: think about memory reallocation and if the order can be yanked somehow first before removing it
                // ToDo: maybe let matching remove it or some worker in the background during low load times
                //todo!()
            }
        }
    }
}

impl MatchingEngine for OrderBook {
    fn market_buy_until(&mut self, mut buy_order: Order) -> (bool, u64, u64, Order) {
        // Market buy order quantity
        let market_buy_qty = buy_order.get_order().get_qty();

        // Get first sell orders to match
        let Some(mut price_level) = self.lowest_asks_mut() else {
            // Orderbook is empty, nothing to match, order stays the same
            return (true, market_buy_qty, 0, buy_order);
        };
        // Market price level right now
        let mut market_price = *price_level.0;

        // Check until
        if buy_order.get_price() < &market_price {
            // buy order price is lower than any asks, nothing to match, order stays the same
            return (true, market_buy_qty, 0, buy_order);
        }

        // Orders at price level
        let mut orders = price_level.1;

        debug!("Market buy order quantity: {}", market_buy_qty);

        // Accumulates the qty until it reaches the orders amount or reaches buy_order price
        let mut accumulator: u64 = 0;
        while accumulator < market_buy_qty && &market_price <= buy_order.get_price() {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.get_qty();
                // Settle execution
                if accumulator <= market_buy_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_buy_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.remove_ask_price_level(&market_price).unwrap();
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
                let _ = self.remove_ask_price_level(&market_price).unwrap();
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
            };
        }
        // Reduce order by filled amount
        buy_order
            .get_order_mut()
            .set_qty(market_buy_qty - accumulator);
        // Usual Outcome:
        // All orders are removed including indexmap price levels if they are completely filled
        // The last remaining order in the FIFO queue of the given price level was either exactly equal and was completely filled or only partially filled
        (true, market_buy_qty, accumulator, buy_order)
    }

    fn market_sell_until(&mut self, mut sell_order: Order) -> (bool, u64, u64, Order) {
        let market_sell_qty = sell_order.get_order().get_qty();

        // Get first buy orders to match
        let Some(mut price_level) = self.highest_bids_mut() else {
            // Orderbook is empty, nothing to match, order stays the same
            return (true, market_sell_qty, 0, sell_order);
        };
        // Market price level right now
        let mut market_price = *price_level.0;

        // Check until
        if sell_order.get_price() > &market_price {
            // sell order price is higher than any bids, nothing to match, order stays the same
            return (true, market_sell_qty, 0, sell_order);
        }

        let mut orders = price_level.1;

        debug!("Market buy order quantity: {}", market_sell_qty);

        // Accumulates the qty until it reaches the orders amount or reaches sell_order price
        let mut accumulator: u64 = 0;
        while accumulator < market_sell_qty && sell_order.get_price() <= &market_price {
            if let Some(matching_candidate) = orders.front() {
                accumulator += matching_candidate.get_qty();
                // Settle execution
                if accumulator <= market_sell_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_sell_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.remove_bid_price_level(&market_price).unwrap();
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
                let _ = self.remove_bid_price_level(&market_price).unwrap();
                debug!("Removed Price Level: {}", market_price);

                if let Some(next_matches) = self.highest_bids_mut() {
                    price_level = next_matches;

                    market_price = *price_level.0;

                    orders = price_level.1;
                } else {
                    break;
                }
            };
        }

        sell_order
            .get_order_mut()
            .set_qty(market_sell_qty - accumulator);

        (true, market_sell_qty, accumulator, sell_order)
    }

    /// Execute Market Buy Order.
    ///
    /// Behaves like an IOC Market Order, cancels any unfilled amount if orderbook lacks liquidity.
    /// Removes Liquidity/Orders from the Orderbook.
    fn market_buy(&mut self, buy_order: Order) -> (bool, u64, u64) {
        // Market buy order quantity
        let market_buy_qty = buy_order.get_order().get_qty();

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
                accumulator += matching_candidate.get_qty();
                // Settle execution
                if accumulator <= market_buy_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_buy_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.remove_ask_price_level(&market_price).unwrap();
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
                let _ = self.remove_ask_price_level(&market_price).unwrap();
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
            };
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
        let market_sell_qty = sell_order.get_order().get_qty();

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
                accumulator += matching_candidate.get_qty();
                // Settle execution
                if accumulator <= market_sell_qty {
                    // We can remove matched order from the orderbook
                    let order_to_remove = orders.pop_front().unwrap();
                    // This is a rare case, this should be handled differently instead of branching unnecessarily in 99% of the cases
                    if accumulator == market_sell_qty && orders.is_empty() {
                        // No orders left at the given price
                        // Remove price level from orderbook
                        let _ = self.remove_bid_price_level(&market_price).unwrap();
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
                let _ = self.remove_bid_price_level(&market_price).unwrap();
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
            };
        }

        // Usual Outcome:
        // All orders are removed including indexmap price levels if they are completely filled
        // The last remaining order in the FIFO queue of the given price level was either exactly equal and was completely filled or only partially filled
        (true, market_sell_qty, accumulator)
    }

    /// Insert Limit Buy Order by matching and inserting the remaining order into the orderbook
    fn insert_limit_buy(&mut self, order: Order) {
        let (result, qty, filled, order) = self.market_buy_until(order);
        if order.get_order().get_qty() > 0 {
            self.insert_buy_order(order);
        }
    }

    /// Insert Limit Sell Order by matching and inserting the remaining order into the orderbook
    fn insert_limit_sell(&mut self, order: Order) {
        let (result, qty, filled, order) = self.market_sell_until(order);
        if order.get_order().get_qty() > 0 {
            self.insert_sell_order(order);
        }
    }

    fn limit_or_cancel_insert(&mut self, order: Order, order_type: OrderType) {
        match order_type {
            OrderType::Buy => {}
            OrderType::Sell => {}
        }
    }

    fn immediate_or_cancel_insert(&mut self, order: Order, order_type: OrderType) {
        match order_type {
            OrderType::Buy => {}
            OrderType::Sell => {}
        }
    }

    fn fill_or_kill_insert(&mut self, order: Order, order_type: OrderType) {
        match order_type {
            OrderType::Buy => {}
            OrderType::Sell => {}
        }
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
       fn test_order_book(qty: u32, main_unit: u32, sub_unit: u8) {
            let order_book = fill_bids_pseudorandom();
       }
    }

    // Pseudorandom Fill
    fn fill_bids_pseudorandom() -> (OrderList, Vec<Order>) {
        let mut bid_list = OrderList::default();
        let mut remove_list = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let price = rng.gen_range(1..=100) as f64;
            let mut hasher = DefaultHasher::new();
            hasher.write_i64(price as i64);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
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

        for _ in 0..1000 {
            let price = rng.gen_range(1..=100) as f64;
            let mut hasher = DefaultHasher::new();
            hasher.write_i64(price as i64);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            remove_list.push(order.clone());
            ask_list.insert_order(order);
        }

        (ask_list, remove_list)
    }

    // Pseudorandom Orders
    fn generate_orders_pseudorandom(amount: u64) -> Vec<Order> {
        let mut orders = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..amount {
            let price = rng.gen_range(1..=100) as f64;
            let mut hasher = DefaultHasher::new();
            hasher.write_i64(price as i64);
            let qty = hasher.finish() % 250_000;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
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
            order_book.remove_buy_order(remove_order)
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
            order_book.remove_sell_order(remove_order)
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
        let result = order_book.market_buy(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Normal Market Buy Test.
    #[test]
    fn test_market_buy_normal() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=10 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Market Buy Full Fill Test
    #[test]
    fn test_market_buy_full_fill() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 500);
        let result = order_book.market_buy(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 500, 500));
    }

    /// Market Buy more than Orderbook has test
    #[test]
    fn test_market_buy_exceeding_orderbook() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_sell_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 500));
    }

    /// Orderbook Empty Market Buy Test
    #[test]
    fn test_market_buy_empty_book() {
        let mut order_book = OrderBook::default();
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_buy(Order::new(Price::new(1, 0), identifiable_order));
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
        let result = order_book.market_sell(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Normal Market Sell Test.
    #[test]
    fn test_market_sell_normal() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=10 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_buy_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 512));
    }

    /// Market Sell Full Fill Test
    #[test]
    fn test_market_sell_full_fill() {
        //initialize_tracing();
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_buy_order(order);
        }
        debug!("Created Orderbook: {:?}", order_book);
        let identifiable_order = IdentifiableOrder::new(5, 500);
        let result = order_book.market_sell(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 500, 500));
    }

    /// Market Buy more than Orderbook has test
    #[test]
    fn test_market_sell_exceeding_orderbook() {
        let mut order_book = OrderBook::default();
        // Fill orderbook
        for i in 1..=5 {
            let price = Price::new(i, 0);
            let qty = 100;
            let identifiable_order = IdentifiableOrder::new(1, qty);
            let order = Order::new(price.into(), identifiable_order);
            order_book.insert_buy_order(order);
        }

        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (true, 512, 500));
    }

    /// Orderbook Empty Market Buy Test
    #[test]
    fn test_market_sell_empty_book() {
        let mut order_book = OrderBook::default();
        let identifiable_order = IdentifiableOrder::new(5, 512);
        let result = order_book.market_sell(Order::new(Price::new(1, 0), identifiable_order));
        assert_eq!(result, (false, 512, 0));
    }
}
