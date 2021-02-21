use std::{env, sync::Arc};


use futures::{StreamExt, lock::Mutex};
use irc::client::{prelude::*};

// quick way to get token:
// https://twitchapps.com/tmi/

pub async fn incoming_message_loop(client: Arc<Mutex<Client>>) -> Result<(), failure::Error> {

    let mut stream = client.lock().await.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        println!("< {}", message.to_string());
    }
    
    Ok(())
}

pub async fn ping_loop(client: Arc<Mutex<Client>>) -> Result<(), failure::Error> {

    println!("Sending ping");

    let client = client.lock().await;
    client.send(Command::PING("".to_owned(), None))?;

    Ok(())
}


pub async fn start_instance() -> Result<(), failure::Error> {

    println!("starting instance");

    let token = env::var("TWITCH_TOKEN")?;

    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("battlechicken-chat-bot".to_owned()),
        server: Some("irc.chat.twitch.tv".to_owned()),
        password: Some(format!("oauth:{}", token)),
        // channels: vec!["#test".to_owned()],
        ..Config::default()
    };

    let client = Client::from_config(config).await?;

    client.identify()?;
    
    let client = Mutex::<_>::from(client);
    let client = Arc::<_>::from(client);
    
    
    
    let incoming_message_loop_task = incoming_message_loop(client.clone());
    let ping_task = ping_loop(client.clone());

    //let _ = incoming_message_loop_task.await;

    let _ = futures::join!(incoming_message_loop_task, ping_task);
    
    

    // while let Some(message) = stream.next().await.transpose()? {
    //     print!("{}", message);
    // }

    Ok(())
}
