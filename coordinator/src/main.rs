#[macro_use]
extern crate tracing;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub mod db;
pub mod handlers;
mod logic;
mod coordinator;

use rumqttc::{ Event, Packet };
use x25519_dalek::{ StaticSecret, PublicKey };

use std::time::Duration;
use std::sync::Arc;

use once_cell::sync::OnceCell;

use crate::coordinator::Coordinator;
use pluto_network::prelude::*;

use crate::db::Database;

const MOSQUITTO_USERNAME: &'static str = "coordinator";

lazy_static::lazy_static! {
    pub static ref DATABASE: OnceCell<Database> = OnceCell::new();
}

lazy_static::lazy_static! {
    pub static ref COORDINATOR_PUBKEY: PublicKey = {
        let array: [u8; 32] = base64::decode(
            &std::env::var("COORDINATOR_PUBKEY").expect("Coordiator pubkey not provided")
        ).expect("Pubkey not valid base64").try_into().expect("Invalid pubkey length");

        PublicKey::from(array)
    };

    pub static ref COORDINATOR_PRIVKEY: StaticSecret = {
        let array: [u8; 32] = base64::decode(
            &std::env::var("COORDINATOR_PRIVKEY").expect("Coordiator privkey not provided")
        ).expect("Pubkey not valid base64").try_into().expect("Invalid privkey length");

        StaticSecret::from(array)
    };
}

#[tokio::main]
async fn main() {
    if std::env::args().len() == 2 {
        if std::env::args().nth(1).unwrap() == "keygen" {
            logic::keygen::generate_print_initial_keys();
            return;
        }
    }

    dotenv::dotenv().ok().unwrap();
    log_init();

    let database_url = std::env::var("DATABASE_URL").expect("No database url provided");

    let db = Database::new(database_url);
    DATABASE.set(db.clone()).ok().unwrap();

    let mut retries = 0;
    while let Err(e) = db.check_connection().await {
        if retries == 3 {
            panic!("Unable to connect to database: {e:?}");
        }
        info!("Waiting for database to start...");
        tokio::time::sleep(Duration::from_millis(5000)).await;
        retries += 1;
    }
    info!("Connected to the database.");

    db.run_migrations().await.unwrap();

    let mosquitto_host = std::env::var("MOSQUITTO_HOST").expect("No Mosquitto host provided");
    let mosquitto_port: u16 = std::env::var("MOSQUITTO_PORT").expect("No Mosquitto port provided").parse().expect("Mosquitto port invalid");
    let mosquitto_password = std::env::var("MOSQUITTO_PASSWORD").expect("No Mosquitto password provided");

    let handler = Arc::new(IncomingHandler::new(handlers::HANDLERS.clone()));

    let (coordinator, mut event_loop) = Coordinator::new(
        mosquitto_host,
        mosquitto_port,
        MOSQUITTO_USERNAME,
        mosquitto_password,
        handler.clone()
    ).await.expect("Error creating coordinator");

    let mut retries = 0;
    // Poll connection acknowledgement.
    while let Err(e) = event_loop.poll().await {
        if retries == 3 {
            panic!("Unable to connect to broker: {e:?}");
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

            if let Event::Incoming(Packet::Publish(packet)) = event {
                if let Err(e) = handler.handle(packet, coordinator.client().clone()).await {
                    error!("{e:?}");
                }
            }
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
