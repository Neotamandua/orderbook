//! gRPC Benchmark

use api::{
    command_api_client::CommandApiClient, InsertLimitBuyOrderRequest, InsertLimitSellOrderRequest,
    InsertMarketBuyOrderRequest, InsertMarketSellOrderRequest,
};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
type Result<T> = anyhow::Result<T, anyhow::Error>;

pub mod api {
    tonic::include_proto!("command");
}

async fn fill_and_match() -> Result<()> {
    let mut command_orderbook_client = CommandApiClient::connect("http://[::1]:50051")
        .await
        .unwrap();

    // Prepare prerequisites for order
    let mut rng = rand::thread_rng();

    // fill orderbook
    let identifier = std::u64::MAX;
    for i in 1..10000 {
        let request = tonic::Request::new(InsertLimitBuyOrderRequest {
            price_ticks: i * 100,
            identifier,
            qty: 500,
        });

        let _ = command_orderbook_client
            .insert_limit_buy_order(request)
            .await?;
    }

    // fill orderbook with more buys
    for i in 1..10000 {
        let request = tonic::Request::new(InsertLimitBuyOrderRequest {
            price_ticks: i * 100 + rng.gen_range(1..=90),
            identifier,
            qty: 500,
        });

        let _ = command_orderbook_client
            .insert_limit_buy_order(request)
            .await?;
    }

    for i in 1..10000 {
        let request = tonic::Request::new(InsertLimitSellOrderRequest {
            price_ticks: (i + 10_000) * 100,
            identifier,
            qty: 500,
        });

        let _ = command_orderbook_client
            .insert_limit_sell_order(request)
            .await?;
    }

    // Insert market sell order
    for _ in 1..10000 {
        let request = tonic::Request::new(InsertMarketSellOrderRequest {
            identifier,
            qty: 500,
        });

        let _ = command_orderbook_client
            .insert_market_sell_order(request)
            .await?;
    }

    // Insert market buy order
    for _ in 1..10000 {
        let request = tonic::Request::new(InsertMarketBuyOrderRequest {
            identifier,
            qty: 500,
        });

        let _ = command_orderbook_client
            .insert_market_buy_order(request)
            .await?;
    }

    Ok(())
}

fn grpc_orderbook_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill_and_match");
    group.sample_size(10);
    group.bench_function("fill_and_match", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| fill_and_match())
    });
}

criterion_group!(benches, grpc_orderbook_bench);
criterion_main!(benches);
