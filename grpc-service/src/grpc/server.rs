use std::sync::{Arc, RwLock};

use api::{
    api_server::{Api, ApiServer},
    greeter_server::{Greeter, GreeterServer},
    ClosestOrderRequest, HelloReply, HelloRequest, InsertOrderReply, InsertOrderRequest,
    OrderReply, OrderbookReply, RemoveOrderReply, RemoveOrderRequest, SideRequest,
};
use tonic::{transport::Server, Request, Response, Status};
pub mod api {
    tonic::include_proto!("api");
}

use orderbookX::traits::matching_engine::MatchingEngine;
use orderbookX::{
    orderbook::{IdentifiableOrder, Order, OrderBook},
    traits::matching_engine::OrderType,
};

#[derive(Debug, Default)]
pub struct OrderBookApi {
    orderbook: OrderBook,
}

#[tonic::async_trait]
impl Api for Arc<RwLock<OrderBookApi>> {
    async fn get_lowest_ask(
        &self,
        request: Request<ClosestOrderRequest>,
    ) -> Result<Response<OrderReply>, Status> {
        {
            let orderbook_api = self.read().unwrap();

            if let Some((price, orders)) = orderbook_api.orderbook.lowest_asks() {
                let mut qty = 0;
                for order in orders {
                    qty += order.get_qty();
                }

                return Ok(tonic::Response::new(OrderReply {
                    order_price: f32::from(*price),
                    qty,
                }));
            } else {
                return Err(Status::not_found("No lowest ask found"));
            }
        }
    }

    async fn get_highest_bid(
        &self,
        request: Request<ClosestOrderRequest>,
    ) -> Result<Response<OrderReply>, Status> {
        {
            let orderbook_api = self.read().unwrap();

            if let Some((price, orders)) = orderbook_api.orderbook.highest_bids() {
                let mut qty = 0;
                for order in orders {
                    qty += order.get_qty();
                }

                return Ok(tonic::Response::new(OrderReply {
                    order_price: f32::from(*price),
                    qty,
                }));
            } else {
                return Err(Status::not_found("No highest bid found"));
            }
        }
    }

    async fn get_bids(
        &self,
        request: Request<SideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        todo!()
    }

    async fn get_asks(
        &self,
        request: Request<SideRequest>,
    ) -> Result<Response<OrderbookReply>, Status> {
        todo!()
    }

    async fn insert_buy_order(
        &self,
        request: Request<InsertOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let insert_order_request = request.into_inner();

        let order_type = insert_order_request.order_type;
        let order_price = insert_order_request.order_price.into();
        let identifier = insert_order_request.identifier;
        let qty = insert_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(order_price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();

            match order_type {
                // Market Buy
                0 => {
                    let result = orderbook_api.orderbook.market_buy(order);
                }
                // GTC
                1 => {
                    // ToDo: create match_and_insert_buy & match_and_insert_sell to remove branching
                    orderbook_api
                        .orderbook
                        .match_and_insert(order, OrderType::Buy);
                }
                // FOK
                2 => {
                    // Unimplemented
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
                // IOC
                3 => {
                    // Unimplemented
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
                _ => {
                    // return false
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
            }
        }
        let reply = InsertOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn insert_sell_order(
        &self,
        request: Request<InsertOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        // Get underlying request data
        let insert_order_request = request.into_inner();

        let order_type = insert_order_request.order_type;
        let order_price = insert_order_request.order_price.into();
        let identifier = insert_order_request.identifier;
        let qty = insert_order_request.qty;

        let identifiable_order = IdentifiableOrder::new(identifier, qty);
        let order = Order::new(order_price, identifiable_order);
        {
            let mut orderbook_api = self.write().unwrap();

            match order_type {
                // Market Buy
                0 => {
                    let result = orderbook_api.orderbook.market_sell(order);
                }
                // GTC
                1 => {
                    // ToDo: create match_and_insert_buy & match_and_insert_sell to remove branching
                    orderbook_api
                        .orderbook
                        .match_and_insert(order, OrderType::Sell);
                }
                // FOK
                2 => {
                    // Unimplemented
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
                // IOC
                3 => {
                    // Unimplemented
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
                _ => {
                    // return false
                    return Ok(tonic::Response::new(InsertOrderReply { success: false }));
                }
            }
        }
        let reply = InsertOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn remove_buy_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        // Access request message using `request.into_inner()`
        let remove_order_request = request.into_inner();
        let order_price = remove_order_request.order_price.into();
        let identifier = remove_order_request.identifier;

        let identifiable_order = IdentifiableOrder::new(identifier, 0);
        let order = Order::new(order_price, identifiable_order);

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
        let order_price = remove_order_request.order_price.into();
        let identifier = remove_order_request.identifier;

        let identifiable_order = IdentifiableOrder::new(identifier, 0);
        let order = Order::new(order_price, identifiable_order);

        {
            let mut orderbook_api = self.write().unwrap();

            // ToDo: this operation needs a return
            orderbook_api.orderbook.remove_sell_order(order);
        }

        let reply = RemoveOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        //println!("Got a request: {:?}", request);

        let reply = api::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Orderbook
    let mut orderbook = OrderBook::default();

    let mut orderbook_api = Arc::new(RwLock::new(OrderBookApi { orderbook }));

    // ToDo: Change Port
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(ApiServer::new(orderbook_api))
        .serve(addr)
        .await?;

    Ok(())
}
