use tokio::task;
use tracing::{error, info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Create the main tracing subscriber that logs to stdout
    let stdout_subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .finish();

    // Set this subscriber as the default for the rest of the application
    stdout_subscriber.init();

    // Create an error span
    let span = tracing::error_span!("tasks", "Starting tasks");

    // Spawn some tasks
    let task1 = task::spawn(some_task("Task 1").instrument(span));
    let task2 = task::spawn(some_task("Task 2").instrument(span));

    task1.await;
    task2.await;
}

async fn some_task(task_name: &str) {
    // Create a new subscriber for this task that logs to stderr
    let stderr_subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();

    // Execute a block with the new subscriber
    tracing::dispatcher::with_default(&stderr_subscriber, || {
        info!("This log goes to stderr from {}", task_name);
        // Simulate some work
        do_some_work(task_name);
        error!("An error occurred in {}", task_name);
    });
}

fn do_some_work(task_name: &str) {
    info!("Doing some work in {}", task_name);
}
