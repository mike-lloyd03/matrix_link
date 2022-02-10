use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;

struct Creds {
    username: String,
    password: String,
    host: String,
}
fn main() {
    let username =
        env::var("matrix_username").expect("must set 'matrix_username' environment variable");
    let password =
        env::var("matrix_password").expect("must set 'matrix_password' environment variable");
    let host = env::var("matrix_host").expect("must set 'matrix_host' environment variable");

    let creds = Creds {
        username,
        password,
        host,
    };
    let client = Client::new();
    let access_token = login(&client, &creds).expect("unable to login");
    logout(&client, &creds, access_token);
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
        user: &creds.username,
        password: &creds.password,
    };

    let url = format!("http://{}/_matrix/client/r0/login", creds.host);
    let resp = client
        .post(url)
        .json(&login)
        .send()
        .expect("failed to send login request");
    if resp.status().is_success() {
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

fn logout(client: &Client, creds: &Creds, access_token: String) -> Option<String> {
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
