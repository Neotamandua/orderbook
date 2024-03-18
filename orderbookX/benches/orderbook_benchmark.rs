use criterion::{criterion_group, criterion_main, Criterion};
use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    traits::matching_engine::MatchingEngine,
};
use rand::Rng;

fn direct_orderbook_bench() {
    let mut rng = rand::thread_rng();
    let mut orderbook = OrderBook::default();

    for i in 1..10000 {
        let price = i as f64;
        let qty = rng.gen_range(1..=10000);
        let identifiable_order = IdentifiableOrder::new(1, qty);
        let order = Order::new(price.into(), identifiable_order);
        orderbook.insert_limit_buy(order);
    }

    let identifiable_order = IdentifiableOrder::new(1, 500);
    for i in 1..10000 {
        let price = i as f64 + (rng.gen_range(1..=90) as f64 / 100.0);
        let order = Order::new(price.into(), identifiable_order.clone());
        orderbook.insert_limit_buy(order);
    }

    for i in 1..10000 {
        let price = rng.gen_range(1..=10000) as f64;
        let qty = rng.gen_range(1..=10000);
        let identifiable_order = IdentifiableOrder::new(1, qty);
        let order = Order::new(price.into(), identifiable_order);
        orderbook.insert_limit_sell(order);
    }

    for i in 1..10000 {
        let price = i as f64;
        let qty = rng.gen_range(1..=10000);
        let identifiable_order = IdentifiableOrder::new(1, qty);
        let order = Order::new(price.into(), identifiable_order);
        orderbook.insert_limit_buy(order);
    }

    for i in 1..10000 {
        let price = i as f64;
        let qty = rng.gen_range(1..=10000);
        let identifiable_order = IdentifiableOrder::new(1, qty);
        let order = Order::new(price.into(), identifiable_order);
        orderbook.insert_limit_sell(order);
    }

    println!(
        "Amount of open orders: {}",
        orderbook.get_amount_open_orders()
    );
}

fn bench_insert_match(c: &mut Criterion) {
    c.bench_function("direct_orderbook_bench", |b| {
        b.iter(|| direct_orderbook_bench())
    });
}

fn simple_buy_insert(c: &mut Criterion) {
    let order = Order::new(100.0.into(), IdentifiableOrder::default());
    let mut orderbook = OrderBook::default();

    c.bench_function("insert_buy_order", |b| {
        b.iter(|| orderbook.insert_limit_buy(order.clone()))
    });
}

fn simple_sell_insert(c: &mut Criterion) {
    let order = Order::new(100.0.into(), IdentifiableOrder::default());
    let mut orderbook = OrderBook::default();

    c.bench_function("insert_sell_order", |b| {
        b.iter(|| orderbook.insert_limit_sell(order.clone()))
    });
}

criterion_group!(
    benches,
    bench_insert_match,
    simple_sell_insert,
    simple_buy_insert
);

criterion_main!(benches);
