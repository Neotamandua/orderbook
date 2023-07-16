use api::{
    api_server::{Api, ApiServer},
    greeter_server::{Greeter, GreeterServer},
    HelloReply,
    HelloRequest,
    InsertOrderReply,
    InsertOrderRequest,
    RemoveOrderReply,
    RemoveOrderRequest,
};
use tonic::{transport::Server, Request, Response, Status};
pub mod api {
    tonic::include_proto!("api");
}

use orderbookX::orderbook::OrderBook;

#[derive(Debug, Default)]
pub struct OrderBookApi {}

#[tonic::async_trait]
impl Api for OrderBookApi {
    async fn remove_order(
        &self,
        request: Request<RemoveOrderRequest>,
    ) -> Result<Response<RemoveOrderReply>, Status> {
        // ToDo: Insert logic for RemoveOrder

        // Access request message using `request.into_inner()`
        let remove_order_request = request.into_inner();
        let order_price = remove_order_request.order_price;
        let identifier = remove_order_request.identifier;

        // ToDo: Perform operations and generate the RemoveOrderReply
        let reply = RemoveOrderReply { success: true };
        Ok(tonic::Response::new(reply))
    }

    async fn insert_order(
        &self,
        request: Request<InsertOrderRequest>,
    ) -> Result<Response<InsertOrderReply>, Status> {
        let insert_order_request = request.into_inner();
        let order_type = insert_order_request.order_type;
        let order_price = insert_order_request.order_price;
        let identifier = insert_order_request.identifier;
        let qty = insert_order_request.qty;

        let reply = InsertOrderReply { success: true };
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
        println!("Got a request: {:?}", request);

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

    // ToDo: Change Port
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
