use std::sync::{Arc, Mutex};

use hickory_resolver::{system_conf::read_system_conf, TokioAsyncResolver};
use tokio::task;
use tracing::{
    debug, error, info, info_span,
    instrument::{self, WithSubscriber},
    Instrument, Level,
};
use tracing_subscriber::{
    filter,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, Registry,
};

fn filtered_layer<W>(writer: W, is_terminal: bool) -> impl Layer<Registry>
where
    W: for<'writer> MakeWriter<'writer> + 'static,
{
    let filter = filter::Targets::new()
        .with_default(Level::WARN) //filter out external crates
        .with_target(env!("CARGO_PKG_NAME"), Level::INFO);
    tracing_subscriber::fmt::layer::<Registry>()
        .with_writer(writer)
        .with_ansi(is_terminal)
        .with_filter(filter)
}

#[tokio::main]
async fn main() {
    // Create the main tracing subscriber that logs to stdout
    let stdout_subscriber =
        tracing_subscriber::registry().with(filtered_layer(std::io::stdout, true));

    // Set this subscriber as the default for the rest of the application
    stdout_subscriber.init();

    info!("started!");
    debug!("filtered main");

    let resolver = Arc::new({
        let (resolver_config, mut options) =
            read_system_conf().expect("failed retrieving system DNS config");
        options.preserve_intermediates = true;
        TokioAsyncResolver::tokio(resolver_config, options)
    });

    let mut set = task::JoinSet::new();
    for i in 0..10000 {
        set.spawn(some_task(format!("task {i}"), resolver.clone()));
    }

    let _ = set.join_all();

    info!("finished!");
}

async fn some_task(task_name: String, resolver: Arc<TokioAsyncResolver>) {
    let span = info_span!("boxing {}", task_name);

    Box::pin(async move {
        let buffer = Vec::new();
        let stderr_subscriber =
            tracing_subscriber::registry().with(filtered_layer(Mutex::new(buffer), false));

        debug!("filtered task");

        let _res = deeper(task_name, resolver)
            .with_subscriber(stderr_subscriber)
            .await;
    })
    .instrument(span)
    .await;

    info!("work_finished!");
}

async fn deeper(name: String, resolver: Arc<TokioAsyncResolver>) {
    info!("This log goes to stderr from {}", name);
    let _ = tokio::spawn(
        do_some_work(name.to_owned(), resolver).instrument(tracing::info_span!("{}", name)),
    )
    .await;
    error!("An error occurred in {}", name);
}

async fn do_some_work(task_name: String, resolver: Arc<TokioAsyncResolver>) {
    info!("Doing some work in {}", task_name);
    match resolver.lookup_ip("www.example.com.").await {
        Ok(result) => info!("Good: {:?}", result),
        Err(e) => error!("Bad: {:?}", e),
    }
}
