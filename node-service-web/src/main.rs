mod api;

#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use pluto_network::key::Keys;

use pluto_network::prelude::*;
use pluto_network::rumqttc::{ Event, Incoming, QoS };
use pluto_node::auth;
use pluto_node::db::Database;
use pluto_node::node::Node;

type KeysShared = Arc<RwLock<Option<Keys>>>;

#[tokio::main]
async fn main() {
    pluto_node::utils::setup_dirs();
    log_init();

    let db = Database::new();
    db.run_migrations().unwrap();

    let handler = Arc::new(IncomingHandler::new(HashMap::new()));
    let client_id = auth::get_mqtt_client_id();

    let (node, mut event_loop) = Node::new(pluto_node::config::COORDINATOR_HOST,
                                           pluto_node::config::COORDINATOR_PORT,
                                           client_id, handler.clone()).await.expect("Error creating node");

    let client = node.client().clone();
    let client_cloned = client.clone();
    let mut prev_conn_state = true;

    let keys: KeysShared = Arc::new(RwLock::new(None));
    let keys_cloned = keys.clone();

    tokio::spawn(async move {
        let client = client_cloned;
        let keys = keys_cloned;
        loop {
            match event_loop.poll().await {
                Ok(e) => {
                    if !prev_conn_state {
                        debug!("Resubscribing after reconnection");
                        debug!("Clearing event loop: {:?}", event_loop.state.clean());
                        pluto_node::utils::
                            subscribe_to_topics(client.clone(), &keys.read().await.as_ref().unwrap()).await.unwrap();
                    }
                    prev_conn_state = true;
                    client.set_connection_alive(true);
                    if let Event::Incoming(Incoming::Publish(event)) = e {
                        trace!("{:?}", event);
                        if let Err(e) = handler.handle(event, node.client().clone()).await {
                            error!("{e:?}")
                        }
                    }
                },
                Err(e) => {
                    debug!("MQTT disconnected, clearing event loop: {:?}", event_loop.state.clean());
                    prev_conn_state = false;
                    client.set_connection_alive(false);
                    error!("{e:?}");
                    tokio::time::sleep(Duration::from_secs(15)).await;
                }
            };
        }
    });


    if db.get_initial_setup_done().unwrap() {
        debug!("Node already set up.");
        *keys.write().await = Some(auth::get_saved_keys().unwrap());
        pluto_node::utils::subscribe_to_topics(client.clone(), &keys.read().await.as_ref().unwrap()).await.unwrap();
    }

    // info!("Node is ready.");
    // let passphrase = keys.seed().to_mnemonic().to_passphrase();
    // info!("Passphrase: {passphrase}");

    api::run(([127, 0, 0, 1], 8080), &client, &keys).await;
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
        .with_default(LevelFilter::DEBUG)
        .with_targets([
            ("pluto_network", LevelFilter::DEBUG),
            ("hyper", LevelFilter::INFO)
        ]);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_targets([
            ("pluto_network", LevelFilter::DEBUG),
            ("rumqtt", LevelFilter::INFO),
            ("hyper", LevelFilter::INFO)
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
        std::process::exit(99);
    }));
}
