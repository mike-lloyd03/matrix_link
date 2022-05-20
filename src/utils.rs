use log::{debug, error, info, warn};
use matrix_link::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub fn login(client: &Client, creds: &Config) -> Option<String> {
    #[derive(Serialize, Debug)]
    struct Login {
        r#type: String,
        user: String,
        password: String,
    }
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
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

    let url = format!("{}/_matrix/client/r0/login", creds.server_url);
    let resp = client
        .post(url)
        .json(&login)
        .send()
        .expect("Failed to send login request");
    if resp.status().is_success() {
        debug!("Successfully logged in");
        let resp_data = resp
            .json::<LoginResponse>()
            .expect("Could not deserialize response");
        Some(resp_data.access_token)
    } else if resp.status().is_client_error() {
        error!("Failed to authenticate");
        None
    } else if resp.status().is_server_error() {
        error!("Server error");
        None
    } else {
        error!("Something went wrong");
        None
    }
}

pub fn logout(client: &Client, creds: &Config, access_token: &str) -> Option<String> {
    let url = format!(
        "{}/_matrix/client/r0/logout?access_token={}",
        creds.server_url, access_token
    );
    let resp = client
        .post(url)
        .send()
        .expect("Failed to send logout request");
    if resp.status().is_success() {
        debug!("Successfully logged out");
        Some("".to_string())
    } else if resp.status().is_client_error() {
        warn!("Failed to logout: {}", resp.status().as_str());
        None
    } else if resp.status().is_server_error() {
        error!("Server error");
        None
    } else {
        error!("Something went wrong");
        None
    }
}

pub fn join_room(
    client: &Client,
    host: &str,
    room_name: &str,
    access_token: &str,
) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct JoinResponse {
        room_id: String,
    }

    let url = format!(
        "{}/_matrix/client/r0/join/{}?access_token={}",
        host, room_name, access_token
    );

    let resp = client.post(url).send().expect("failed to join room");

    if resp.status().is_success() {
        // debug!("{:?}", resp.json::<HashMap<String, String>>());
        Some(
            resp.json::<JoinResponse>()
                .expect("failed to parse response")
                .room_id,
        )
    } else if resp.status().is_client_error() {
        warn!("Failed to logout: {}", resp.status().as_str());
        None
    } else if resp.status().is_server_error() {
        error!("Server error");
        None
    } else {
        error!("Something went wrong");
        None
    }
}

pub fn send_message(
    client: &Client,
    host: &str,
    room_id: &str,
    access_token: &str,
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
        "{}/_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
        host, room_id, access_token
    );
    let resp = client
        .post(url)
        .json(&msg_body)
        .send()
        .expect("failed to join room");

    if resp.status().is_success() {
        info!("Message sent successfully");
    } else if resp.status().is_client_error() {
        warn!("Failed to logout: {}", resp.status().as_str());
    } else if resp.status().is_server_error() {
        error!("Server error");
    } else {
        error!("Something went wrong");
    }
}
