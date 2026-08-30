use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    traits::matching_engine::Orders,
};
use price::Price;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

#[test]
fn simple_integration() {
    // Set up tracing subscriber
    let subscriber = FmtSubscriber::builder()
        // Capture all events with level higher or equal to TRACE
        .with_max_level(Level::TRACE)
        .finish();

    // Set the subscriber as the default to use in the program
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    trace!("Short Showcase of Orderbook:");
    // Create an empty orderbook
    let mut order_book = OrderBook::default();

    trace!("Empty Orderbook:\n{}", order_book);
    trace!("Amount of open orders: {}", order_book.sum_open_orders());

    // Insert limit buy orders
    order_book.insert_limit_buy(Order::new(
        Price::from_ticks(121),
        IdentifiableOrder::new(1, 50),
    ));
    order_book.insert_limit_buy(Order::new(
        Price::from_ticks(100),
        IdentifiableOrder::new(2, 50),
    ));
    order_book.insert_limit_buy(Order::new(
        Price::from_ticks(200),
        IdentifiableOrder::new(3, 50),
    ));

    trace!("Orderbook After Buy Order Insertion:\n{}", order_book);
    trace!("Amount of open orders: {}", order_book.sum_open_orders());

    // Insert limit sell orders
    order_book.insert_limit_sell(Order::new(
        Price::from_ticks(200),
        IdentifiableOrder::new(4, 60),
    ));

    order_book.insert_limit_sell(Order::new(
        Price::from_ticks(600),
        IdentifiableOrder::new(4, 60),
    ));
    trace!("Orderbook After Sell Order Insertion:\n{}", order_book);
    trace!("Amount of open orders: {}", order_book.sum_open_orders());

    // Insert single limit buy order again
    order_book.insert_limit_buy(Order::new(
        Price::from_ticks(500),
        IdentifiableOrder::new(4, 60),
    ));

    trace!("Orderbook After Buy Order Insertion:\n{}", order_book);
    trace!("Amount of open orders: {}", order_book.sum_open_orders());
}
