use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    price::Price,
    traits::matching_engine::MatchingEngine,
};
use tracing::{trace, Level};
use tracing_subscriber::{self, FmtSubscriber};

#[test]
fn simple_integration() {
    // Set up tracing subscriber
    let subscriber = FmtSubscriber::builder()
        // capture all events with level higher or equal to TRACE
        .with_max_level(Level::TRACE)
        .finish();

    // set the subscriber as the default to use in the program
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    trace!("Short Showcase of Orderbook:");
    // Create an empty orderbook
    let mut order_book = OrderBook::default();

    trace!("Empty Orderbook:\n{}", order_book);
    trace!(
        "Amount of open orders: {}",
        order_book.get_amount_open_orders()
    );

    // Insert limit buy orders
    order_book.insert_limit_buy(Order::new(
        Price::new(1, 211),
        IdentifiableOrder::new(1, 50),
    ));
    order_book.insert_limit_buy(Order::new(Price::new(1, 0), IdentifiableOrder::new(2, 50)));
    order_book.insert_limit_buy(Order::new(Price::new(2, 0), IdentifiableOrder::new(3, 50)));

    trace!("Orderbook After Buy Order Insertion:\n{}", order_book);
    trace!(
        "Amount of open orders: {}",
        order_book.get_amount_open_orders()
    );

    // Insert limit sell orders
    order_book.insert_limit_sell(Order::new(Price::new(2, 0), IdentifiableOrder::new(4, 60)));

    order_book.insert_limit_sell(Order::new(Price::new(6, 0), IdentifiableOrder::new(4, 60)));
    trace!("Orderbook After Sell Order Insertion:\n{}", order_book);
    trace!(
        "Amount of open orders: {}",
        order_book.get_amount_open_orders()
    );

    // Insert single limit buy order again
    order_book.insert_limit_buy(Order::new(Price::new(5, 0), IdentifiableOrder::new(4, 60)));

    trace!("Orderbook After Buy Order Insertion:\n{}", order_book);
    trace!(
        "Amount of open orders: {}",
        order_book.get_amount_open_orders()
    );
}
