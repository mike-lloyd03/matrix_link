use clap::{App, Arg};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::error::Error as StdError;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Deserialize)]
struct Config {
    username: String,
    password: String,
    host: String,
    room_name: String,
}

fn main() {
    let args = App::new("matrix_link")
        .about("Sends messages to a Matrix server")
        .arg(
            Arg::new("message")
                .required(true)
                .index(1)
                .help("The message to be sent"),
        )
        .get_matches();

    let message = args
        .value_of("message")
        .expect("message argument not provided")
        .to_string();
    let config = load_config().expect("failed to load config file");
    let client = Client::new();

    let access_token = login(&client, &config).expect("unable to login");
    let room_id = join_room(&client, &config.host, &config.room_name, &access_token)
        .expect("unable to get room id");
    send_message(&client, &config.host, &room_id, &access_token, message);
    logout(&client, &config, &access_token);
}

fn login(client: &Client, creds: &Config) -> Option<String> {
    #[derive(Serialize, Debug)]
    struct Login {
        r#type: String,
        user: String,
        password: String,
    }
    #[derive(Deserialize, Debug)]
    struct LoginResponse {
        device_id: String,
        user_id: String,
        access_token: String,
        home_server: String,
    }

    let login = Login {
        r#type: "m.login.password".to_string(),
        user: creds.username.to_owned(),
        password: creds.password.to_owned(),
    };

    let url = format!("http://{}/_matrix/client/r0/login", creds.host);
    let resp = client
        .post(url)
        .json(&login)
        .send()
        .expect("failed to send login request");
    if resp.status().is_success() {
        println!("successfully logged in");
        let resp_data = resp
            .json::<LoginResponse>()
            .expect("could not deserialize response");
        Some(resp_data.access_token)
    } else if resp.status().is_client_error() {
        println!("failed to authenticate");
        None
    } else if resp.status().is_server_error() {
        println!("server error");
        None
    } else {
        println!("something went wrong");
        None
    }
}

fn logout(client: &Client, creds: &Config, access_token: &String) -> Option<String> {
    let url = format!(
        "http://{}/_matrix/client/r0/logout?access_token={}",
        creds.host, access_token
    );
    let resp = client
        .post(url)
        .send()
        .expect("failed to send logout request");
    if resp.status().is_success() {
        println!("successfully logged out");
        Some("".to_string())
    } else if resp.status().is_client_error() {
        println!("failed to logout: {}", resp.status().as_str());
        None
    } else if resp.status().is_server_error() {
        println!("server error");
        None
    } else {
        println!("something went wrong");
        None
    }
}

fn join_room(
    client: &Client,
    host: &String,
    room_name: &String,
    access_token: &String,
) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct JoinResponse {
        room_id: String,
    }

    let url = format!(
        "http://{}/_matrix/client/r0/join/{}?access_token={}",
        host, room_name, access_token
    );

    let resp = client.post(url).send().expect("failed to join room");

    if resp.status().is_success() {
        // println!("{:?}", resp.json::<HashMap<String, String>>());
        Some(
            resp.json::<JoinResponse>()
                .expect("failed to parse response")
                .room_id,
        )
    } else if resp.status().is_client_error() {
        println!("failed to logout: {}", resp.status().as_str());
        None
    } else if resp.status().is_server_error() {
        println!("server error");
        None
    } else {
        println!("something went wrong");
        None
    }
}

fn send_message(
    client: &Client,
    host: &String,
    room_id: &String,
    access_token: &String,
    message: String,
) {
    #[derive(Serialize, Debug)]
    struct MessageBody {
        msgtype: String,
        body: String,
    }

    let msg_body = MessageBody {
        msgtype: "m.text".to_string(),
        body: message,
    };

    let url = format!(
        "http://{}/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        host, room_id, access_token
    );
    let resp = client
        .post(url)
        .json(&msg_body)
        .send()
        .expect("failed to join room");

    if resp.status().is_success() {
        println!("message sent successfully");
    } else if resp.status().is_client_error() {
        println!("failed to logout: {}", resp.status().as_str());
    } else if resp.status().is_server_error() {
        println!("server error");
    } else {
        println!("something went wrong");
    }
}

fn load_config() -> Result<Config, Box<dyn StdError>> {
    let matrix_cfg_path = "matrix_link/config.yaml";
    let sys_config_path = Path::new("/etc").join(matrix_cfg_path);
    let user_config_path = dirs::config_dir()
        .expect("cannot get user's config dir")
        .join(matrix_cfg_path);

    let f: File;
    if sys_config_path.exists() {
        f = File::open(sys_config_path)?;
    } else if user_config_path.exists() {
        f = File::open(user_config_path)?;
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No configuration file could be found",
        ));
    }

    Ok(serde_yaml::from_reader(f)?)
}
