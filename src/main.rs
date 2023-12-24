extern crate chrono;

use chrono::{DateTime, Utc};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::env;

fn days_elapsed_from_timestamp(timestamp: i64) -> i64 {
    let timestamp_datetime: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0).unwrap();
    let current_datetime: DateTime<Utc> = Utc::now();
    let duration = current_datetime.signed_duration_since(timestamp_datetime);
    let days_elapsed = duration.num_days();

    days_elapsed
}

async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    if let Ok(raw_timestamp) = env::var("TIMESTAMP") {
        if let Ok(timestamp) = raw_timestamp.parse::<i64>() {
            let days_elapsed = days_elapsed_from_timestamp(timestamp);
            let message = format!("Flora is {} days old today.", days_elapsed);

            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(message.into())
                .map_err(Box::new)?;

            Ok(resp)
        } else {
            Err("Error: Failed to parse TIMESTAMP value as an integer.".into())
        }
    } else {
        Err("Error: TIMESTAMP is not set.".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
