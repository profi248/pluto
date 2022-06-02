#[macro_use]
extern crate tracing;

use rumqttc::Event;

use pluto_network::coordinator::Coordinator;
use pluto_network::prelude::*;

#[tokio::main]
async fn main() {
    log_init();

    let (coordinator, mut event_loop) = Coordinator::new("", "").await
        .expect("Error creating coordinator");

    tokio::spawn(async move {
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
