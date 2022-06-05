#[macro_use]
extern crate tracing;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod db;

use std::time::Duration;
use rumqttc::Event;

use pluto_network::coordinator::Coordinator;
use pluto_network::prelude::*;
use crate::db::Database;

const MOSQUITTO_USERNAME: &'static str = "coordinator";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok().unwrap();
    log_init();

    let database_url = std::env::var("DATABASE_URL").expect("No database url provided");

    let db = Database::new(database_url);

    let mut retries = 0;
    while db.check_connection().await.is_err() {
        if retries == 3 {
            panic!("Unable to connect to database.");
        }
        info!("Trying to connect to database...");
        tokio::time::sleep(Duration::from_millis(5000)).await;
        retries += 1;
    }
    info!("Connected to the database.");

    db.run_migrations().await.unwrap();

    let mosquitto_host = std::env::var("MOSQUITTO_HOST").expect("No Mosquitto host provided");
    let mosquitto_port: u16 = std::env::var("MOSQUITTO_PORT").expect("No Mosquitto port provided").parse().expect("Mosquitto port invalid");
    let mosquitto_password = std::env::var("MOSQUITTO_PASSWORD").expect("No Mosquitto password provided");

    let (coordinator, mut event_loop) = Coordinator::new(
        mosquitto_host,
        mosquitto_port,
        MOSQUITTO_USERNAME,
        mosquitto_password
    ).await.expect("Error creating coordinator");

    let mut retries = 0;
    // Poll connection acknowledgement.
    while event_loop.poll().await.is_err() {
        if retries == 3 {
            panic!("Unable to connect to broker.");
        }
        info!("Waiting for MQTT broker to start...");
        tokio::time::sleep(Duration::from_millis(5000)).await;
        retries += 1;
    }

    info!("Connected to MQTT broker.");

    tokio::spawn(async move {
        info!("Listening for MQTT events...");
        loop {
            let event = match event_loop.poll().await {
                Ok(e) => e,
                Err(e) => {
                    error!("{e:?}");
                    break;
                }
            };

            trace!("{:?}", event);
        }
    });

    loop {}
}

fn log_init() {
    use tracing_subscriber::filter::{ targets::Targets, LevelFilter };
    use tracing_subscriber::layer::{ SubscriberExt, Layer as _ };
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;

    let filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto_coordinator", LevelFilter::TRACE),
            ("pluto_network", LevelFilter::TRACE),
        ]);

    tracing_subscriber::registry()
        .with(Layer::new()
            .pretty()
            .with_ansi(true)
            .with_filter(filter)
        )
        .init();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{:?}", panic_info);

        hook(panic_info);
    }));
}
