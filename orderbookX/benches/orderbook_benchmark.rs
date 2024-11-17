use criterion::{criterion_group, criterion_main, Criterion};
use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    traits::matching_engine::MatchingEngine,
};
use price::Price;
use rand::Rng;

fn fill_and_match() {
    // Prepare prerequisites for order
    let mut rng = rand::thread_rng();
    let mut orderbook = OrderBook::default();

    // fill orderbook
    for i in 1..10000 {
        let price = Price::from_ticks(i * 100);
        let qty = 500;
        let identifiable_order = IdentifiableOrder::new(std::u64::MAX, qty);
        let order = Order::new(price, identifiable_order);
        orderbook.insert_limit_buy(order);
    }

    // fill orderbook with more buys
    for i in 1..10000 {
        let price = Price::from_ticks(i * 100 + rng.gen_range(1..=90));
        let qty = 500;
        let identifiable_order = IdentifiableOrder::new(std::u64::MAX, qty);
        let order = Order::new(price, identifiable_order);
        orderbook.insert_limit_buy(order);
    }

    for i in 1..10000 {
        let price = Price::from_ticks((i + 10_000) * 100);
        let qty = 500;
        let identifiable_order = IdentifiableOrder::new(std::u64::MAX, qty);
        let order = Order::new(price, identifiable_order);
        orderbook.insert_limit_sell(order);
    }

    // Insert market sell order
    for _ in 1..10000 {
        let qty = 500;
        let identifiable_order = IdentifiableOrder::new(std::u64::MAX, qty);
        let order = Order::new(Price::ZERO, identifiable_order);

        orderbook.market_sell(order);
    }

    // Insert market buy order
    for _ in 1..10000 {
        let qty = 500;
        let identifiable_order = IdentifiableOrder::new(std::u64::MAX, qty);
        let order = Order::new(Price::ZERO, identifiable_order);
        orderbook.market_buy(order);
    }
}

fn orderbook_bench(c: &mut Criterion) {
    c.bench_function("fill_and_match", |b| b.iter(|| fill_and_match()));
}

fn simple_buy_insert(c: &mut Criterion) {
    let order = Order::new(Price::from_ticks(10_000), IdentifiableOrder::default());
    let mut orderbook = OrderBook::default();

    c.bench_function("insert_buy_order", |b| {
        b.iter(|| orderbook.insert_limit_buy(order.clone()))
    });
}

fn simple_sell_insert(c: &mut Criterion) {
    let order = Order::new(Price::from_ticks(10_000), IdentifiableOrder::default());
    let mut orderbook = OrderBook::default();

    c.bench_function("insert_sell_order", |b| {
        b.iter(|| orderbook.insert_limit_sell(order.clone()))
    });
}

criterion_group!(
    benches,
    orderbook_bench,
    simple_sell_insert,
    simple_buy_insert
);

criterion_main!(benches);
