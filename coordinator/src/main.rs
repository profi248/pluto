#[macro_use]
extern crate tracing;

use pluto_network::coordinator::Coordinator;
use pluto_network::prelude::*;

#[tokio::main]
async fn main() {
    let _guard = log_init();

    let coordinator = Coordinator::new("username", "password").await.expect("Error creating coordinator");


}

fn log_init() -> tracing::dispatcher::DefaultGuard {
    use tracing_subscriber::filter::{ targets::Targets, LevelFilter };
    use tracing_subscriber::layer::{ SubscriberExt, Layer as _ };
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::prelude::*;

    let filter = Targets::new()
        .with_default(LevelFilter::INFO)
        .with_targets([
            ("pluto-coordinator", LevelFilter::TRACE),
            ("pluto-network", LevelFilter::TRACE),
        ]);

    tracing_subscriber::registry()
        .with(Layer::new()
            .pretty()
            .with_ansi(true)
            .with_filter(filter)
        )
        .set_default()
}
