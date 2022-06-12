#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use rumqttc::Event;

use std::sync::Arc;

use pluto_network::node::{
    key::{ Keys, Mnemonic, Seed },
    Node,
};
use pluto_network::{
    topics::*, protos::auth::*,
};
use pluto_network::prelude::*;
use rumqttc::{ QoS };

#[tokio::main]
async fn main() {
    pluto_node::utils::setup_dirs();
    log_init();

    let handler = Arc::new(IncomingHandler::new(HashMap::new()));
    let (node, mut event_loop) = Node::new("localhost", 1883, handler).await.expect("Error creating node");

    tokio::spawn(async move {
        loop {
            let event = match event_loop.poll().await {
                Ok(e) => e,
                Err(e) => {
                    error!("{e:?}");
                    break;
                }
            };

            if let Event::Incoming(event) = event {
                trace!("{:?}", event);
            }
        }
    });

    // let mut a = AuthNodeInit::default();
    // a.pubkey = vec![0x1a; 5];
    // debug!("{:?}", a);
    // node.client().send_and_listen(
    //     topic!(Coordinator::Auth).topic(),
    //     a,
    //     QoS::AtMostOnce,
    //     false,
    //     std::time::Duration::from_secs(10)
    // ).await.expect("error");

    let keys = Keys::generate();

    pluto_node::db::Database::run_migrations().unwrap();
    pluto_node::auth::register_node(node.client(), &keys).await.unwrap();

    loop {}
}

fn log_init() {
    use tracing_subscriber::filter::{ targets::Targets, LevelFilter };
    use tracing_subscriber::layer::{ SubscriberExt, Layer as _ };
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;

    let mut path = pluto_node::utils::get_log_file_path();

    let log_file = std::fs::File::options()
        .create(true)
        .append(false)
        .write(true)
        .open(path)
        .expect("Could not open/create file.");

    let file_filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto_cli", LevelFilter::DEBUG),
            ("pluto_network", LevelFilter::DEBUG),
        ]);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto_cli", LevelFilter::INFO),
            ("pluto_network", LevelFilter::INFO),
        ]);

    tracing_subscriber::registry()
        .with(Layer::new()
            .compact()
            .with_ansi(false)
            .with_writer(log_file)
            .with_filter(file_filter)
        )
        .with(Layer::new()
            .pretty()
            .with_ansi(true)
            .with_filter(stdout_filter)
        )
        .init();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{:?}", panic_info);

        hook(panic_info);
    }));
}
