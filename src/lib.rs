use std::{env, ffi::{self, CStr, CString}, os::raw::c_char, sync::Arc};


use futures::{StreamExt, executor::block_on, lock::Mutex};
use irc::client::{prelude::*};

// quick way to get token:
// https://twitchapps.com/tmi/


async fn process_input(_input: &str, _client: Arc<Mutex<Client>>) -> Result<(), failure::Error> {
    Ok(())
}

#[allow(dead_code)]
async fn command_line_loop(client: Arc<Mutex<Client>>) -> Result<(), failure::Error>{
	
	println!("Starting command line interface");

	let stdin = async_std::io::stdin();
	
	loop { 
		let mut line = String::new();
		let result = stdin.read_line(&mut line).await;

		if result.is_ok() {
			match line.as_str().trim() {
				"exit" => { 
					client.lock().await.send_quit("".to_owned())?;
					break;
				},
				line => { 
					process_input(line, client.clone()).await?;
				},
			}
		}
	}
	

    Ok(())
}


async fn incoming_message_loop(client: Arc<Mutex<Client>>, on_message: impl Fn(&str)) -> Result<(), failure::Error> {

    let mut stream = client.lock().await.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        on_message(&message.to_string())
        //println!("> {}", message.to_string());
    }
    
    Ok(())
}

async fn ping_loop(client: Arc<Mutex<Client>>) -> Result<(), failure::Error> {

    loop {
        println!("Sending ping");

        let client = client.lock().await;
        client.send(Command::PING("".to_owned(), None))?;

        async_std::task::sleep(std::time::Duration::from_secs(120)).await;
    }
    
    
    #[allow(unreachable_code)]
    Ok(())
}


async fn init_client() -> Result<Arc<Mutex<Client>>, failure::Error> {

    let token = env::var("TWITCH_TOKEN")?;

    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("battlechicken-chat-bot".to_owned()),
        server: Some("irc.chat.twitch.tv".to_owned()),
        password: Some(format!("oauth:{}", token)),
        //channels: vec!["#battlechicken".to_owned()],
        ..Config::default()
    };

    let client = Client::from_config(config).await?;
    let client = Mutex::<_>::from(client);
    let client = Arc::<_>::from(client);
    
    return Ok(client);
}

async fn listen_channel(channel_name: &str, client: &Arc<Mutex<Client>>, on_message: impl Fn(&str)) -> Result<(), failure::Error> {

    println!("starting instance");

    //let client = client.clone();

    {
        let client = client.lock().await;

        client.identify()?;
        client.send(Command::JOIN(channel_name.to_owned(), None, None))?;
    
        drop(client);
    }
    
    
    let incoming_message_loop_task = incoming_message_loop(client.clone(), on_message);
    let ping_task = ping_loop(client.clone());
    

    let _ = futures::join!(incoming_message_loop_task, ping_task);
    

    Ok(())
}

pub async fn start_instance(channel_name: &str, on_message: impl Fn(&str)) -> Result<(), failure::Error> {
    let client = init_client().await?;

    listen_channel(channel_name, &client, on_message).await?;

    Ok(())
}



#[derive(Clone, Copy)]
#[repr(C)]
pub struct OnMessage {
    callback: fn (*mut ffi::c_void, message: *const c_char),
    user_data: *mut ffi::c_void,
}

pub struct AppState {
    client: Arc<Mutex<Client>>,
    on_message: Option<OnMessage>,
}

impl AppState {
    fn from_client(client: Arc<Mutex<Client>>) -> AppState {
        return AppState {
            client,
            on_message: None,
        };
    }
}


#[no_mangle]
pub extern "C" fn init_app() -> *mut AppState {
    unsafe {
        let client = match block_on(init_client()) {
            Ok(client) => client,
            _ => { return std::ptr::null_mut(); },
        };

        let app_state = Box::new(AppState::from_client(client));

        return std::mem::transmute(app_state);
    }
}

#[no_mangle]
pub extern "C" fn release_app(app_state: *mut AppState) {
    unsafe {
        //let _: Box<AppState> = std::mem::transmute(app_state);
        std::mem::transmute::<_, Box<AppState>>(app_state);
    }
}

#[no_mangle]
pub extern "C" fn set_message_handler(app_state: &mut AppState, on_message: &OnMessage) {
    app_state.on_message = Some(on_message.clone());
}

#[no_mangle]
pub extern "C" fn run(app_state: &mut AppState, channel_name: *const c_char) -> bool {

    
    let channel_name = unsafe { CStr::from_ptr(channel_name) };
    
    let channel_name = match channel_name.to_str() {
        Ok(channel_name) => channel_name,
        _ => { return false; }
    };
    
    let on_message = match app_state.on_message {
        Some(on_message) => on_message,
        None => { return false; }
    };

    let _ = block_on(listen_channel(channel_name, &app_state.client, |str| {
        let c_str = CString::new(str).unwrap_or(CString::default());
        (on_message.callback)(on_message.user_data, c_str.as_ptr());
    }));

    return true;
}
