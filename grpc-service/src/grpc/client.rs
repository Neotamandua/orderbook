use api::{
    api_client::ApiClient, greeter_client::GreeterClient, ClosestOrderRequest, HelloRequest,
    InsertOrderRequest,
};

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let mut orderbook_client = ApiClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);
    for i in 0..100 {
        // Buy Order
        let request = tonic::Request::new(InsertOrderRequest {
            order_type: 1,
            order_price: i as f32,
            identifier: 1,
            qty: 500,
        });

        orderbook_client.insert_buy_order(request).await?;
    }

    // check highest bid
    let request = tonic::Request::new(ClosestOrderRequest {});
    let response = orderbook_client.get_highest_bid(request).await;
    println!("RESPONSE={:?}", response);

    for i in (50..100).rev() {
        // Sell Order
        let request = tonic::Request::new(InsertOrderRequest {
            order_type: 1,
            order_price: i as f32,
            identifier: 1,
            qty: 500,
        });

        orderbook_client.insert_sell_order(request).await?;
    }

    // check highest bid
    let request = tonic::Request::new(ClosestOrderRequest {});
    let response = orderbook_client.get_highest_bid(request).await;
    println!("RESPONSE={:?}", response);

    // check lowest ask
    let request = tonic::Request::new(ClosestOrderRequest {});
    let response = orderbook_client.get_lowest_ask(request).await;
    println!("RESPONSE={:?}", response);

    Ok(())
}
