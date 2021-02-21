extern crate twitch_ts;

use tokio;

#[tokio::main]
async fn main() {
    let result = twitch_ts::start_instance().await;

    if let Err(error) = result {
        println!("Exited with error: {}", error.to_string());
    }
}

