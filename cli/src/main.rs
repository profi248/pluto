#[macro_use]
extern crate tracing;

use std::collections::HashMap;

use std::sync::Arc;

use pluto_network::key::{ Keys, Mnemonic, Seed };
use pluto_network::{
    topics::*, protos::auth::*,
};
use pluto_network::prelude::*;
use rumqttc::{ Event, Incoming, QoS };

use pluto_node::node::Node;

#[tokio::main]
async fn main() {
    pluto_node::utils::setup_dirs();
    log_init();
    pluto_node::db::Database::run_migrations().unwrap();

    let keys = Keys::generate();

    let handler = Arc::new(IncomingHandler::new(HashMap::new()));
    let client_id = pluto_network::utils::get_node_topic_id(keys.public_key().as_bytes().to_vec());

    let (node, mut event_loop) = Node::new("localhost", 1883, client_id, handler.clone()).await.expect("Error creating node");

    let client = node.client().clone();

    tokio::spawn(async move {
        loop {
            let event = match event_loop.poll().await {
                Ok(e) => e,
                Err(e) => {
                    error!("{e:?}");
                    break;
                }
            };

            if let Event::Incoming(Incoming::Publish(event)) = event {
                trace!("{:?}", event);
                if let Err(e) = handler.handle(event, client.clone()).await {
                    error!("{e:?}")
                }
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

    pluto_node::auth::register_node(node.client(), &keys).await.unwrap();

    loop {}
}

fn log_init() {
    use tracing_subscriber::filter::{ targets::Targets, LevelFilter };
    use tracing_subscriber::layer::{ SubscriberExt, Layer as _ };
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;

    let path = pluto_node::utils::get_log_file_path();

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
