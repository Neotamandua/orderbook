//! gRPC server for the orderbook service.

use std::sync::{Arc, RwLock};

use orderbook_x::{
    orderbook::{IdentifiableOrder, Order, OrderBook, PriceLevel, QuantityOverflow},
    traits::matching_engine::Orders,
};
use price::Price;
use tonic::{transport::Server, Request, Response, Status};

mod api {
    tonic::include_proto!("stream");
    tonic::include_proto!("query");
    tonic::include_proto!("command");
}

use self::api::{
    command_api_server::{CommandApi, CommandApiServer},
    query_api_server::{QueryApi, QueryApiServer},
    BuySideRequest, ClosestOrderRequest, InsertLimitBuyOrderRequest, InsertLimitSellOrderRequest,
    InsertMarketBuyOrderRequest, InsertMarketSellOrderRequest, InsertOrderReply, OrderReply,
    OrderbookReply, PriceReply, RemoveOrderReply, RemoveOrderRequest, SellSideRequest,
};

#[derive(Debug, Default)]
pub struct OrderBookApi {
    orderbook: OrderBook,
}

fn price_level_reply(level: PriceLevel) -> OrderReply {
    OrderReply {
        price_ticks: level.price().ticks(),
        qty: level.qty(),
    }
}

fn quantity_overflow_status(error: QuantityOverflow) -> Status {
    Status::out_of_range(error.to_string())
}

#[tonic::async_trait]
impl CommandApi for Arc<RwLock<OrderBookApi>> {
    async fn insert_limit_buy_order(
        &self,
        request: Request<InsertLimitBuyOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::from_ticks(request.price_ticks),
            IdentifiableOrder::new(request.identifier, request.qty),
        );

        let success = self.write().unwrap().orderbook.insert_limit_buy(order);

        Ok(Response::new(InsertOrderReply { success }))
    }

    async fn insert_market_buy_order(
        &self,
        request: Request<InsertMarketBuyOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::MAX,
            IdentifiableOrder::new(request.identifier, request.qty),
        );

        let _ = self.write().unwrap().orderbook.market_buy_until(order);

        Ok(Response::new(InsertOrderReply { success: true }))
    }

    async fn insert_limit_sell_order(
        &self,
        request: Request<InsertLimitSellOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::from_ticks(request.price_ticks),
            IdentifiableOrder::new(request.identifier, request.qty),
        );

        let success = self.write().unwrap().orderbook.insert_limit_sell(order);

        Ok(Response::new(InsertOrderReply { success }))
    }

    async fn insert_market_sell_order(
        &self,
        request: Request<InsertMarketSellOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::ZERO,
            IdentifiableOrder::new(request.identifier, request.qty),
        );

        let _ = self.write().unwrap().orderbook.market_sell_until(order);

        Ok(Response::new(InsertOrderReply { success: true }))
    }

    async fn remove_buy_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::from_ticks(request.price_ticks),
            IdentifiableOrder::new(request.identifier, 0),
        );
        let success = self.write().unwrap().orderbook.remove_buy_order(&order);

        Ok(Response::new(RemoveOrderReply { success }))
    }

    async fn remove_sell_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        let request = request.into_inner();
        let order = Order::new(
            Price::from_ticks(request.price_ticks),
            IdentifiableOrder::new(request.identifier, 0),
        );
        let success = self.write().unwrap().orderbook.remove_sell_order(&order);

        Ok(Response::new(RemoveOrderReply { success }))
    }
}

#[tonic::async_trait]
impl QueryApi for Arc<RwLock<OrderBookApi>> {
    async fn get_lowest_ask(
        &self,
        _request: Request<ClosestOrderRequest>,
    ) -> Result<Response<PriceReply>, Status> {
        self.read()
            .unwrap()
            .orderbook
            .lowest_ask_price()
            .map(|price| {
                Response::new(PriceReply {
                    price_ticks: price.ticks(),
                })
            })
            .ok_or_else(|| Status::not_found("No lowest ask found"))
    }

    async fn get_highest_bid(
        &self,
        _request: Request<ClosestOrderRequest>,
    ) -> Result<Response<PriceReply>, Status> {
        self.read()
            .unwrap()
            .orderbook
            .highest_bid_price()
            .map(|price| {
                Response::new(PriceReply {
                    price_ticks: price.ticks(),
                })
            })
            .ok_or_else(|| Status::not_found("No highest bid found"))
    }

    async fn get_bids(
        &self,
        request: Request<BuySideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        let depth = request
            .into_inner()
            .depth
            .try_into()
            .map_err(|_| Status::invalid_argument("depth exceeds the supported range"))?;
        let bids = self
            .read()
            .unwrap()
            .orderbook
            .bids(depth)
            .map_err(quantity_overflow_status)?;
        let orders = bids.into_iter().rev().map(price_level_reply).collect();

        Ok(Response::new(OrderbookReply { orders }))
    }

    async fn get_asks(
        &self,
        request: Request<SellSideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        let depth = request
            .into_inner()
            .depth
            .try_into()
            .map_err(|_| Status::invalid_argument("depth exceeds the supported range"))?;
        let asks = self
            .read()
            .unwrap()
            .orderbook
            .asks(depth)
            .map_err(quantity_overflow_status)?;
        let orders = asks.into_iter().map(price_level_reply).collect();

        Ok(Response::new(OrderbookReply { orders }))
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
            service.read().unwrap().orderbook.highest_bid_price(),
            Some(&Price::from_ticks(199))
        );

        insert_limit_buy(&service, 200).await;
        assert_eq!(
            service.read().unwrap().orderbook.highest_bid_price(),
            Some(&Price::from_ticks(200))
        );
    }

    #[tokio::test]
    async fn query_returns_formattable_canonical_ticks() {
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
            .highest_bid_price()
            .unwrap()
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

    #[tokio::test]
    async fn bid_depth_returns_best_prices_as_canonical_ticks() {
        let service = service();
        for ticks in [199, 5_000_000, 200] {
            insert_limit_buy(&service, ticks).await;
        }
        assert!(
            service
                .insert_limit_buy_order(Request::new(InsertLimitBuyOrderRequest {
                    price_ticks: 5_000_000,
                    identifier: 5_000_001,
                    qty: 4,
                }))
                .await
                .unwrap()
                .into_inner()
                .success
        );

        let reply = service
            .get_bids(Request::new(BuySideRequest { depth: 2 }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(
            reply
                .orders
                .iter()
                .map(|order| (order.price_ticks, order.qty))
                .collect::<Vec<_>>(),
            [(5_000_000, 5), (200, 1)]
        );
    }

    #[tokio::test]
    async fn depth_query_reports_aggregate_quantity_overflow() {
        let service = service();
        for (identifier, qty) in [(1, u64::MAX), (2, 1)] {
            assert!(
                service
                    .insert_limit_buy_order(Request::new(InsertLimitBuyOrderRequest {
                        price_ticks: 100,
                        identifier,
                        qty,
                    }))
                    .await
                    .unwrap()
                    .into_inner()
                    .success
            );
        }

        let status = service
            .get_bids(Request::new(BuySideRequest { depth: 1 }))
            .await
            .unwrap_err();
        assert_eq!(status.code(), tonic::Code::OutOfRange);
    }

    #[tokio::test]
    async fn cancellation_reply_reports_whether_an_order_was_removed() {
        let service = service();
        insert_limit_buy(&service, 12_345).await;
        let request = || {
            Request::new(RemoveOrderRequest {
                price_ticks: 12_345,
                identifier: 12_345,
            })
        };

        assert!(
            service
                .remove_buy_order(request())
                .await
                .unwrap()
                .into_inner()
                .success
        );
        assert!(
            !service
                .remove_buy_order(request())
                .await
                .unwrap()
                .into_inner()
                .success
        );
    }

    #[tokio::test]
    async fn duplicate_identifier_at_one_price_is_rejected() {
        let service = service();
        let request = |qty| {
            Request::new(InsertLimitBuyOrderRequest {
                price_ticks: 12_345,
                identifier: 7,
                qty,
            })
        };

        assert!(
            service
                .insert_limit_buy_order(request(10))
                .await
                .unwrap()
                .into_inner()
                .success
        );
        assert!(
            !service
                .insert_limit_buy_order(request(99))
                .await
                .unwrap()
                .into_inner()
                .success
        );

        let orderbook = service.read().unwrap();
        let (_, orders) = orderbook.orderbook.highest_bids().unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders.front().unwrap().qty(), 10);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orderbook_api = Arc::new(RwLock::new(OrderBookApi::default()));
    let address = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(CommandApiServer::new(orderbook_api.clone()))
        .add_service(QueryApiServer::new(orderbook_api))
        .serve(address)
        .await?;

    Ok(())
}
