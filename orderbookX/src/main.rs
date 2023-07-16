use orderbookX::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    price::Price,
    traits::matching_engine::{MatchingEngine, OrderType},
};

fn main() {
    println!("Short Showcase of Orderbook:");
    let mut order_book = OrderBook::default();
    println!("Empty Orderbook:\n{}", order_book);
    order_book.insert_buy_order(Order::new(
        Price::new(1, 211),
        IdentifiableOrder::new(1, 50),
    ));
    order_book.insert_buy_order(Order::new(Price::new(1, 0), IdentifiableOrder::new(2, 50)));
    order_book.insert_buy_order(Order::new(Price::new(2, 0), IdentifiableOrder::new(3, 50)));
    println!("Orderbook After Buy Order Insertion:\n{}", order_book);
    order_book.match_and_insert(
        Order::new(Price::new(2, 0), IdentifiableOrder::new(4, 60)),
        OrderType::Sell,
    );
    order_book.match_and_insert(
        Order::new(Price::new(6, 0), IdentifiableOrder::new(4, 60)),
        OrderType::Sell,
    );
    println!("Orderbook After Sell Order Insertion:\n{}", order_book);
    order_book.match_and_insert(
        Order::new(Price::new(5, 0), IdentifiableOrder::new(4, 60)),
        OrderType::Buy,
    );
    println!("Orderbook After Buy Order Insertion:\n{}", order_book);
}
