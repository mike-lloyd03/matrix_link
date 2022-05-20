use clap::{App, Arg};
use log::error;
use reqwest::blocking::Client;
use std::process::exit;

pub mod utils;
use matrix_link::*;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

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

    let config_paths = vec!["/etc/matrix_link/config.yaml", "config.yaml"];
    let config = match load_config(config_paths) {
        Ok(config) => config,
        Err(err) => {
            error!("Unable to load configuration file");
            error!("{}", err);
            exit(1);
        }
    };

    let client = Client::new();

    let access_token = utils::login(&client, &config).expect("unable to login");
    let room_id = utils::join_room(
        &client,
        &config.server_url,
        &config.room_name,
        &access_token,
    )
    .expect("unable to get room id");
    utils::send_message(
        &client,
        &config.server_url,
        &room_id,
        &access_token,
        message,
    );
    utils::logout(&client, &config, &access_token);
}
