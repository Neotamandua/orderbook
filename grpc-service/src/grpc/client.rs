//! gRPC example client for the orderbook service
//!
//! Normally this would be it's own application, but for simplicity it's included here as a test client for reference
//! This could be (multiple) backend services that would be used by a frontend application
//! The backend services may use some message queue to communicate with each other and ingest data e.g., kafka or rabbitmq
//!
//! This client is a simple CLI that allows the user to interact with the orderbook service

use api::{
    command_api_client::CommandApiClient, query_api_client::QueryApiClient, BuySideRequest,
    InsertLimitBuyOrderRequest, InsertLimitSellOrderRequest, InsertMarketBuyOrderRequest,
    InsertMarketSellOrderRequest, SellSideRequest,
};
use rand::{Rng, RngCore};
use tonic::transport::Channel;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut command_orderbook_client = CommandApiClient::connect("http://[::1]:50051").await?;
    let mut query_orderbook_client = QueryApiClient::connect("http://[::1]:50051").await?;

    // Prepare prerequisites for order
    let mut rng = rand::thread_rng();

    // Ask for user input as often as possible
    loop {
        let result = parse_input(
            &mut command_orderbook_client,
            &mut query_orderbook_client,
            &mut rng,
        )
        .await;

        println!("RESPONSE={:?}", result);
    }
}

async fn parse_input<R: RngCore>(
    command_orderbook_client: &mut CommandApiClient<Channel>,
    query_orderbook_client: &mut QueryApiClient<Channel>,
    rng: &mut R,
    // todo change result type to generic response
) -> Result<String> {
    // fetch user input from cli
    let mut input = String::new();
    println!("Enter a command (limit prices are integer ticks): ");
    std::io::stdin().read_line(&mut input).unwrap();

    // parse user input
    let mut input = input.split_whitespace();
    let command = input.next().unwrap_or_default();
    let price_ticks = input.next().unwrap_or_default();
    let identifier: u64 = rng.gen_range(0..u64::MAX);
    let qty = input.next().unwrap_or_default();

    match command {
        "buy" => {
            let request = tonic::Request::new(InsertMarketBuyOrderRequest {
                identifier,
                qty: qty.parse::<u64>().unwrap_or_default(),
            });

            let x = command_orderbook_client
                .insert_market_buy_order(request)
                .await;

            Ok(format!("{:?}", x))
        }
        "limit buy" => {
            let request = tonic::Request::new(InsertLimitBuyOrderRequest {
                price_ticks: price_ticks.parse::<u64>().unwrap_or_default(),
                identifier,
                qty: qty.parse::<u64>().unwrap_or_default(),
            });

            let x = command_orderbook_client
                .insert_limit_buy_order(request)
                .await;

            Ok(format!("{:?}", x))
        }
        "sell" => {
            let request = tonic::Request::new(InsertMarketSellOrderRequest {
                identifier,
                qty: qty.parse::<u64>().unwrap_or_default(),
            });

            let x = command_orderbook_client
                .insert_market_sell_order(request)
                .await;
            Ok(format!("{:?}", x))
        }
        "limit sell" => {
            let request = tonic::Request::new(InsertLimitSellOrderRequest {
                price_ticks: price_ticks.parse::<u64>().unwrap_or_default(),
                identifier,
                qty: qty.parse::<u64>().unwrap_or_default(),
            });

            let x = command_orderbook_client
                .insert_limit_sell_order(request)
                .await;
            Ok(format!("{:?}", x))
        }
        "order close" => {
            todo!("Close order unimplemented on client side");
        }
        "order ls" => {
            let asks = query_orderbook_client
                .get_asks(tonic::Request::new(SellSideRequest { depth: 10 }))
                .await;
            let bids = query_orderbook_client
                .get_bids(tonic::Request::new(BuySideRequest { depth: 10 }))
                .await;

            Ok(format!("{:?} \n {:?}", asks, bids))
        }
        _ => Err(anyhow::Error::msg("Invalid command")),
    }
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::api::{InsertLimitBuyOrderRequest, OrderReply};

    #[test]
    fn protobuf_round_trip_preserves_large_tick_values() {
        const LARGE_PRICE_TICKS: u64 = 5_000_000;

        let request = InsertLimitBuyOrderRequest {
            price_ticks: LARGE_PRICE_TICKS,
            identifier: 1,
            qty: 500,
        };
        let decoded_request =
            InsertLimitBuyOrderRequest::decode(request.encode_to_vec().as_slice()).unwrap();

        let reply = OrderReply {
            price_ticks: decoded_request.price_ticks,
            qty: decoded_request.qty,
        };
        let decoded_reply = OrderReply::decode(reply.encode_to_vec().as_slice()).unwrap();

        assert_eq!(decoded_request.price_ticks, LARGE_PRICE_TICKS);
        assert_eq!(decoded_reply.price_ticks, LARGE_PRICE_TICKS);
    }
}
