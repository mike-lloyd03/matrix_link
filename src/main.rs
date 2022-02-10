use clap::{App, Arg};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

struct Creds {
    username: String,
    password: String,
    host: String,
    room_name: String,
}
fn main() {
    let args = App::new("matrix_link")
        .arg(Arg::with_name("message").required(true).index(1))
        .get_matches();

    let message = args
        .value_of("message")
        .expect("message argument not provided")
        .to_string();

    let username =
        env::var("matrix_username").expect("must set 'matrix_username' environment variable");
    let password =
        env::var("matrix_password").expect("must set 'matrix_password' environment variable");
    let host = env::var("matrix_host").expect("must set 'matrix_host' environment variable");
    let room_name =
        env::var("matrix_room_name").expect("must set 'matrix_room_name' environment variable");

    let creds = Creds {
        username,
        password,
        host,
        room_name,
    };
    let client = Client::new();
    let access_token = login(&client, &creds).expect("unable to login");
    let room_id = join_room(&client, &creds.host, &creds.room_name, &access_token)
        .expect("unable to get room id");
    send_message(&client, &creds.host, &room_id, &access_token, message);
    logout(&client, &creds, &access_token);
}

fn login(client: &Client, creds: &Creds) -> Option<String> {
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
            .expect("could not deserialze response");
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

fn logout(client: &Client, creds: &Creds, access_token: &String) -> Option<String> {
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
        println!("{:?}", resp.json::<HashMap<String, String>>());
        // Some(
        //     resp.json::<JoinResponse>()
        //         .expect("failed to parse response")
        //         .room_id,
        // )
    } else if resp.status().is_client_error() {
        println!("failed to logout: {}", resp.status().as_str());
        // None
    } else if resp.status().is_server_error() {
        println!("server error");
        // None
    } else {
        println!("something went wrong");
        // None
    }
}
