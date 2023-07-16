use std::collections::VecDeque;

use crate::orderbook::Order;

/// MatchingEngine providing the given order types.
/// Iceberg orders or any form of hidden orders, stop loss orders/take profit orders, one cancels other (OCO) are not supported, as users can execute them independently using API access and bots.
pub trait MatchingEngine {
    fn market_buy_until(&mut self, buy_order: Order) -> (bool, u64, u64, Order);

    fn market_sell_until(&mut self, sell_order: Order) -> (bool, u64, u64, Order);

    /// Market Buy
    fn market_buy(&mut self, buy_order: Order) -> (bool, u64, u64);

    /// Market sell
    fn market_sell(&mut self, sell_order: Order) -> (bool, u64, u64);

    /// Limit Order (Good till Cancel)
    /// A Good till Cancel (GTC) order is a buy or sell order that remains active until it is either filled or manually canceled by the trader.
    /// Unlike immediate execution orders, GTC orders can stay in the market for an extended period until they are executed or revoked by the trader.
    fn match_and_insert(&mut self, order: Order, order_type: OrderType);

    /// Limit or Cancel
    fn limit_or_cancel_insert(&mut self, order: Order, order_type: OrderType);

    /// Limit Order (Immediate or Cancel)
    /// An Immediate-Or-Cancel (IOC) order is a buy or sell order that requires immediate execution.
    /// If an IOC order cannot be completed instantly, the remaining unexecuted amount will be automatically canceled.
    /// Allows partial execution
    fn immediate_or_cancel_insert(&mut self, order: Order, order_type: OrderType);

    /// Limit Order (Fill or Kill)
    /// A Fill or Kill (FOK) order is a buy or sell order that must be executed in its entirety immediately.
    /// If the order cannot be filled completely at once, it will be canceled instead of being partially executed.
    /// Does not allow partial execution
    fn fill_or_kill_insert(&mut self, order: Order, order_type: OrderType);
}

pub enum OrderType {
    Buy,
    Sell,
}
