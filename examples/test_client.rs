extern crate tokio;


fn on_message(message: &str) {
    println!("> {}", message.to_string());
}

#[tokio::main]
async fn main() {
    let result = twitch_ts::start_instance("#battlechicken", on_message).await;

    if let Err(error) = result {
        println!("Exited with error: {}", error.to_string());
    }
}

