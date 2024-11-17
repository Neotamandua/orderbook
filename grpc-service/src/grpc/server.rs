//! gRPC Server for the orderbook service
//!
//! Normally this would run in a container with state management/recover or something else
//! The container could be deployed for every trading pair
//! In order to keep the in-memory state of the orderbook service one could use state management through serialization/deserialization with e.g., memcached or redis or a database
//! This is important in case of a crash or restart of the service
//!
//! Additionally, one could split the api into multiple services, e.g., a command service and a query service so that the query service could also expose a constant data stream
//! through e.g., websockets

use std::sync::{Arc, RwLock};

use api::{
    command_api_server::{CommandApi, CommandApiServer},
    query_api_server::{QueryApi, QueryApiServer},
    ClosestOrderRequest, InsertOrderReply, OrderReply, OrderbookReply, RemoveOrderReply,
    RemoveOrderRequest,
};
use tonic::{transport::Server, Request, Response, Status};
pub mod api {
    tonic::include_proto!("api");
}

use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    traits::matching_engine::MatchingEngine,
};
use price::Price;

use self::api::{
    BuySideRequest, InsertLimitBuyOrderRequest, InsertLimitSellOrderRequest,
    InsertMarketBuyOrderRequest, InsertMarketSellOrderRequest, SellSideRequest,
};

#[derive(Debug, Default)]
pub struct OrderBookApi {
    orderbook: OrderBook,
}

#[tonic::async_trait]
impl CommandApi for Arc<RwLock<OrderBookApi>> {
    async fn insert_limit_buy_order(
        &self,
        request: Request<InsertLimitBuyOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let limit_buy_order_request = request.into_inner();

        let price = Price::from_ticks(limit_buy_order_request.price_ticks);
        let identifier = limit_buy_order_request.identifier;
        let qty = limit_buy_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();

            // ToDo: provide function to insert limit sell order without matching if services before can make sure no matching happens
            orderbook_api.orderbook.insert_limit_buy(order);
        }

        let reply = InsertOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn insert_market_buy_order(
        &self,
        request: Request<InsertMarketBuyOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let market_buy_order_request = request.into_inner();

        let price = Price::MAX;
        let identifier = market_buy_order_request.identifier;
        let qty = market_buy_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();

            // Market Buy
            let _ = orderbook_api.orderbook.market_buy_until(order);
        }
        let reply = InsertOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn insert_limit_sell_order(
        &self,
        request: Request<InsertLimitSellOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let limit_sell_order_request = request.into_inner();

        let price = Price::from_ticks(limit_sell_order_request.price_ticks);
        let identifier = limit_sell_order_request.identifier;
        let qty = limit_sell_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();
            // ToDo: provide function to insert limit sell order without matching if services before can make sure no matching happens
            orderbook_api.orderbook.insert_limit_sell(order);
        }

        let reply = InsertOrderReply { success: true };

        // ToDo: Remove result
        Ok(tonic::Response::new(reply))
    }

    async fn insert_market_sell_order(
        &self,
        request: Request<InsertMarketSellOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let market_sell_order_request = request.into_inner();

        // ToDo: Make this settable in the future in config
        // Zero ticks for a market sell because it matches the highest bid and subsequent bids.
        let price = Price::ZERO;

        let identifier = market_sell_order_request.identifier;
        let qty = market_sell_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();
            orderbook_api.orderbook.market_sell_until(order);
        }

        let reply = InsertOrderReply { success: true };

        // ToDo: Remove result
        Ok(tonic::Response::new(reply))
    }

    async fn remove_buy_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        // Access request message using `request.into_inner()`
        let remove_order_request = request.into_inner();
        let price = Price::from_ticks(remove_order_request.price_ticks);
        let identifier = remove_order_request.identifier;

        let identifiable_order = IdentifiableOrder::new(identifier, 0);
        let order = Order::new(price, identifiable_order);

        {
            let mut orderbook_api = self.write().unwrap();

            // ToDo: this operation needs a return
            orderbook_api.orderbook.remove_buy_order(order);
        }

        let reply = RemoveOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn remove_sell_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        // Access request message using `request.into_inner()`
        let remove_order_request = request.into_inner();
        let price = Price::from_ticks(remove_order_request.price_ticks);
        let identifier = remove_order_request.identifier;

        let identifiable_order = IdentifiableOrder::new(identifier, 0);
        let order = Order::new(price, identifiable_order);

        {
            let mut orderbook_api = self.write().unwrap();

            // ToDo: this operation needs a return
            orderbook_api.orderbook.remove_sell_order(order);
        }

        let reply = RemoveOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }
}

#[tonic::async_trait]
impl QueryApi for Arc<RwLock<OrderBookApi>> {
    async fn get_lowest_ask(
        &self,
        _request: Request<ClosestOrderRequest>,
    ) -> Result<Response<OrderReply>, Status> {
        {
            let orderbook_api = self.read().unwrap();

            if let Some((price, orders)) = orderbook_api.orderbook.lowest_asks() {
                let mut qty = 0;
                for order in orders {
                    qty += order.get_qty();
                }

                return Ok(tonic::Response::new(OrderReply {
                    price_ticks: price.ticks(),
                    qty,
                }));
            } else {
                return Err(Status::not_found("No lowest ask found"));
            }
        }
    }

    async fn get_highest_bid(
        &self,
        _request: Request<ClosestOrderRequest>,
    ) -> Result<Response<OrderReply>, Status> {
        {
            let orderbook_api = self.read().unwrap();

            if let Some((price, orders)) = orderbook_api.orderbook.highest_bids() {
                let mut qty = 0;
                for order in orders {
                    qty += order.get_qty();
                }

                return Ok(tonic::Response::new(OrderReply {
                    price_ticks: price.ticks(),
                    qty,
                }));
            } else {
                return Err(Status::not_found("No highest bid found"));
            }
        }
    }

    async fn get_bids(
        &self,
        _request: Request<BuySideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        unimplemented!("Not implemented")
    }

    async fn get_asks(
        &self,
        _request: Request<SellSideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        unimplemented!("Not implemented")
    }
}

#[cfg(test)]
mod price_tests {
    use super::*;

    fn service() -> Arc<RwLock<OrderBookApi>> {
        Arc::new(RwLock::new(OrderBookApi::default()))
    }

    async fn insert_limit_buy(service: &Arc<RwLock<OrderBookApi>>, price_ticks: u64) {
        service
            .insert_limit_buy_order(Request::new(InsertLimitBuyOrderRequest {
                price_ticks,
                identifier: price_ticks,
                qty: 1,
            }))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn command_processing_uses_submitted_ticks_directly() {
        let service = service();

        insert_limit_buy(&service, 199).await;
        assert_eq!(
            service.read().unwrap().orderbook.highest_bid().unwrap().0,
            &Price::from_ticks(199)
        );

        insert_limit_buy(&service, 200).await;
        assert_eq!(
            service.read().unwrap().orderbook.highest_bid().unwrap().0,
            &Price::from_ticks(200)
        );
    }

    #[tokio::test]
    async fn query_returns_the_exact_submitted_tick_value() {
        let service = service();
        insert_limit_buy(&service, 12_345).await;

        let reply = service
            .get_highest_bid(Request::new(ClosestOrderRequest {}))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(reply.price_ticks, 12_345);
        assert_eq!(Price::from_ticks(reply.price_ticks).to_string(), "123.45");
    }

    #[tokio::test]
    async fn large_price_round_trip_preserves_every_tick() {
        const LARGE_PRICE_TICKS: u64 = 5_000_000;

        let service = service();
        insert_limit_buy(&service, LARGE_PRICE_TICKS).await;

        let internal_ticks = service
            .read()
            .unwrap()
            .orderbook
            .highest_bid()
            .unwrap()
            .0
            .ticks();
        let returned_ticks = service
            .get_highest_bid(Request::new(ClosestOrderRequest {}))
            .await
            .unwrap()
            .into_inner()
            .price_ticks;

        assert_eq!(internal_ticks, LARGE_PRICE_TICKS);
        assert_eq!(returned_ticks, LARGE_PRICE_TICKS);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Orderbook
    let orderbook = OrderBook::default();

    let orderbook_api = Arc::new(RwLock::new(OrderBookApi { orderbook }));

    // ToDo: Change Port
    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(CommandApiServer::new(orderbook_api.clone()))
        .add_service(QueryApiServer::new(orderbook_api))
        .serve(addr)
        .await?;

    Ok(())
}
