#[macro_use]
extern crate tracing;

use pluto_network::node::{
    key::{ Keys, Mnemonic, Seed },
    Node
};
use pluto_network::prelude::*;

pub const PLUTO_DIR: &'static str = ".pluto";
pub const LOG_FILE: &'static str = "log.txt";

#[tokio::main]
async fn main() {
    setup_dirs();
    let _guard = log_init();

    let node = Node::new("tcp://localhost:1883").await.expect("Error creating node");


    // vector_to_seed("breeze once city absorb inspire field staff ensure six verify cliff float board picnic true acoustic evidence exit crash flee denial screen kitchen liquid".to_string().split_whitespace().map(ToOwned::to_owned)
    //                    .collect()).unwrap();


    //seed_to_mnemonic([27, 147, 80, 165, 128, 103, 84, 171, 245, 2, 90, 201, 222, 84, 171, 172, 145, 139, 72, 122, 80, 16, 77, 201, 252, 201, 172, 99, 167, 131, 30, 196].to_vec()).unwrap();
    // println!("{:?}", pack_bits([1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,1,1,1,1,1,1].to_vec())
    //     .map(|a| a.into_iter().map(|b| format!("{b:b}")).collect::<Vec<String>>())
    // );
    // [27, 147, 80, 165, 128, 103, 84, 171, 245, 2, 90, 201, 222, 84, 171, 172, 145, 139, 72, 122, 80, 16, 77, 201, 252, 201, 172, 99, 167, 131, 30, 196]
    // [27, 147, 80, 165, 128, 103, 84, 171, 245, 2, 90, 201, 222, 84, 171, 172, 145, 139, 72, 122, 80, 16, 77, 201, 252, 201, 172, 99, 167, 131, 30, 196]

    // node.register_to_network(&keys).await.expect("Error registering to network");
}

fn setup_dirs() {
    let mut path = dirs::home_dir().unwrap();
    path.push(PLUTO_DIR);

    if !path.exists() { std::fs::create_dir(&path).unwrap(); }

    if !path.is_dir() { panic!("{} is a file.", PLUTO_DIR); }
}

fn log_init() -> tracing::dispatcher::DefaultGuard {
    use tracing_subscriber::filter::{ targets::Targets, LevelFilter };
    use tracing_subscriber::layer::{ SubscriberExt, Layer as _ };
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;

    let mut path = dirs::home_dir().unwrap();
    path.push(PLUTO_DIR);
    path.push(LOG_FILE);

    let log_file = std::fs::File::options()
        .create(true)
        .append(false)
        .write(true)
        .open(path)
        .expect("Could not open/create file.");

    let file_filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto-cli", LevelFilter::DEBUG),
            ("pluto-network", LevelFilter::DEBUG),
        ]);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto-cli", LevelFilter::INFO),
            ("pluto-network", LevelFilter::INFO),
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
        .set_default()
}
