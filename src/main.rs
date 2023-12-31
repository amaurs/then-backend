extern crate chrono;

use chrono::{FixedOffset, Utc};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::env;
use serde_json::json;
use then_backend::days_elapsed_between_timestamps;

async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    if let Ok(raw_timestamp) = env::var("TIMESTAMP") {
        if let Ok(timestamp) = raw_timestamp.parse::<i64>() {
            let (is_before, days_elapsed) = days_elapsed_between_timestamps(timestamp, Utc::now().timestamp(), FixedOffset::west_opt(8 * 3600).unwrap());

            let message = if is_before {
                format!("Flora Clementina will be {} days old today.", days_elapsed + 1)
            } else {
                format!("Flora Clementina is {} days old.", days_elapsed)
            };


            let resp = Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET,OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token,X-Amz-User-Agent")
                .body(
                    json!({
            "message": message,
          }).to_string().into())
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
