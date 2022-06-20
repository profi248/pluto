#[macro_use]
extern crate tracing;

use std::collections::HashMap;

use std::sync::Arc;

use pluto_network::key::Keys;
use pluto_network::{
    topics::*, protos::auth::*,
};
use pluto_network::prelude::*;
use rumqttc::{ Event, Incoming, QoS };
use pluto_node::db::Database;
use pluto_node::auth;

use pluto_node::node::Node;

#[tokio::main]
async fn main() {
    pluto_node::utils::setup_dirs();
    log_init();
    Database::run_migrations().unwrap();

    let handler = Arc::new(IncomingHandler::new(HashMap::new()));
    let client_id = auth::get_mqtt_client_id();

    let (node, mut event_loop) = Node::new(pluto_node::config::COORDINATOR_HOST,
                                     pluto_node::config::COORDINATOR_PORT,
                                     client_id, handler.clone()).await.expect("Error creating node");

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
                if let Err(e) = handler.handle(event, node.client().clone()).await {
                    error!("{e:?}")
                }
            }
        }
    });


    let keys;

    if Database::get_initial_setup_done().unwrap() {
        debug!("Node already set up.");
        keys = auth::get_saved_keys().unwrap();
        let node_topic_id = pluto_network::utils::
            get_node_topic_id(keys.public_key().as_bytes().to_vec());

        client.client().subscribe(format!("node/{node_topic_id}/#"), QoS::AtMostOnce).await.unwrap();
    } else {
        match inquire::Confirm::new("Do you want to restore a existing passphrase?").prompt() {
            Ok(true) => {
                match inquire::Text::new("Enter your passphrase: ").prompt() {
                    Ok(passphrase) => {
                        keys = auth::restore_keys_from_passphrase(passphrase).unwrap();

                        let node_topic_id = pluto_network::utils::
                            get_node_topic_id(keys.public_key().as_bytes().to_vec());

                        client.client().subscribe(format!("node/{node_topic_id}/#"), QoS::AtMostOnce).await.unwrap();

                        auth::save_credentials_to_storage(&keys).unwrap();
                        info!("Keys from passphrase restored.");
                    },
                    Err(_) => todo!()
                }
            },
            Ok(false) => {
                info!("Registering node to the network.");
                keys = Keys::generate();

                let node_topic_id = pluto_network::utils::
                    get_node_topic_id(keys.public_key().as_bytes().to_vec());

                client.client().subscribe(format!("node/{node_topic_id}/#"), QoS::AtMostOnce).await.unwrap();
                auth::register_node(&client, &keys).await.unwrap();
            },
            Err(_) => todo!(),
        }
    }


    info!("Node is ready.");
    let passphrase = keys.seed().to_mnemonic().to_passphrase();
    info!("Passphrase: {passphrase}");

    let remote_jobs = pluto_node::backup_job::get_remote_backup_jobs(&client, &keys).await.unwrap();
    debug!("backup jobs: {remote_jobs:?}");

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
        .with_default(LevelFilter::DEBUG)
        .with_targets([
            ("pluto_cli", LevelFilter::DEBUG),
            ("pluto_network", LevelFilter::DEBUG),
        ]);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::DEBUG)
        .with_targets([
            ("pluto_cli", LevelFilter::DEBUG),
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
