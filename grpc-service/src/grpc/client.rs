//! gRPC example client for the orderbook service
//!
//! Normally this would be its own application, but for simplicity it is included here as an
//! example client. Limit prices cross the service boundary as canonical integer ticks.

use std::fmt::{Display, Formatter, Result as FmtResult};

use api::{
    command_api_client::CommandApiClient, query_api_client::QueryApiClient, BuySideRequest,
    InsertLimitBuyOrderRequest, InsertLimitSellOrderRequest, InsertMarketBuyOrderRequest,
    InsertMarketSellOrderRequest, SellSideRequest,
};
use inquire::Select;
use rand::{rngs::ThreadRng, Rng, RngCore};
use tonic::transport::Channel;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::build().await?;
    let select = Select::new("Desired Action", SelectOptions::VARIANTS.to_vec()).prompt()?;

    let answer = match select {
        SelectOptions::MarketOrder => client.market_order(client.order_direction()?).await?,
        SelectOptions::LimitOrder => client.limit_order(client.order_direction()?).await?,
        SelectOptions::OrderClose => client.close_order().await?,
        SelectOptions::ListOrders => {
            let orders = client.list_orders().await?;
            if orders.is_empty() {
                "Orderbook is empty".to_string()
            } else {
                orders
            }
        }
    };

    println!("{answer}");
    Ok(())
}

struct Client<R: RngCore> {
    command_orderbook_client: CommandApiClient<Channel>,
    query_orderbook_client: QueryApiClient<Channel>,
    rng: R,
}

impl Client<ThreadRng> {
    async fn build() -> Result<Self> {
        let command_orderbook_client = CommandApiClient::connect("http://[::1]:50051").await?;
        let query_orderbook_client = QueryApiClient::connect("http://[::1]:50051").await?;

        Ok(Self {
            command_orderbook_client,
            query_orderbook_client,
            rng: rand::thread_rng(),
        })
    }

    fn order_direction(&self) -> Result<Direction> {
        Ok(Select::new("Order Type", Direction::VARIANTS.to_vec()).prompt()?)
    }

    async fn market_order(&mut self, direction: Direction) -> Result<String> {
        let identifier = self.rng.gen_range(0..u64::MAX);
        let qty = inquire::prompt_u64("Quantity")?;

        let result = match direction {
            Direction::Buy => {
                let request = tonic::Request::new(InsertMarketBuyOrderRequest { identifier, qty });
                self.command_orderbook_client
                    .insert_market_buy_order(request)
                    .await
            }
            Direction::Sell => {
                let request = tonic::Request::new(InsertMarketSellOrderRequest { identifier, qty });
                self.command_orderbook_client
                    .insert_market_sell_order(request)
                    .await
            }
        };

        Ok(format!("{result:?}"))
    }

    async fn limit_order(&mut self, direction: Direction) -> Result<String> {
        let identifier = self.rng.gen_range(0..u64::MAX);
        let qty = inquire::prompt_u64("Quantity")?;
        let price_ticks = inquire::prompt_u64("Price ticks")?;

        let result = match direction {
            Direction::Buy => {
                let request = tonic::Request::new(InsertLimitBuyOrderRequest {
                    price_ticks,
                    identifier,
                    qty,
                });
                self.command_orderbook_client
                    .insert_limit_buy_order(request)
                    .await
            }
            Direction::Sell => {
                let request = tonic::Request::new(InsertLimitSellOrderRequest {
                    price_ticks,
                    identifier,
                    qty,
                });
                self.command_orderbook_client
                    .insert_limit_sell_order(request)
                    .await
            }
        };

        Ok(format!("{result:?}"))
    }

    async fn close_order(&self) -> Result<String> {
        let _order_id = inquire::prompt_u64("Order ID")?;
        Ok("Close order unimplemented on client side".to_string())
    }

    async fn list_orders(&mut self) -> Result<String> {
        let asks = self
            .query_orderbook_client
            .get_asks(tonic::Request::new(SellSideRequest { depth: 10 }))
            .await;
        let bids = self
            .query_orderbook_client
            .get_bids(tonic::Request::new(BuySideRequest { depth: 10 }))
            .await;

        Ok(format!("{asks:?}\n{bids:?}"))
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Buy,
    Sell,
}

impl Direction {
    const VARIANTS: &'static [Self] = &[Self::Buy, Self::Sell];
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy)]
enum SelectOptions {
    MarketOrder,
    LimitOrder,
    OrderClose,
    ListOrders,
}

impl SelectOptions {
    const VARIANTS: &'static [Self] = &[
        Self::MarketOrder,
        Self::LimitOrder,
        Self::OrderClose,
        Self::ListOrders,
    ];
}

impl Display for SelectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::MarketOrder => write!(f, "Market Order"),
            Self::LimitOrder => write!(f, "Limit Order"),
            Self::OrderClose => write!(f, "Close Order"),
            Self::ListOrders => write!(f, "List Orders"),
        }
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
