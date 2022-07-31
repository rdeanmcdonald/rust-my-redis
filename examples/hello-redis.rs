use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a conn to the redis addr
    let mut client = client::connect("127.0.0.1:6379").await?;

    let a = client.set("hello", "world".into()).await;

    match a {
        Ok(_) => println!("SUCCESSFULL SET"),
        Err(e) => println!("ERROR ON SET: {:?}", e),
    }

    let result = client.get("hello").await?;

    println!("got value from redis: {:?}", result);

    Ok(())
}
