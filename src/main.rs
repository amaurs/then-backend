extern crate chrono;

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::env;
use serde_json::json;

fn days_elapsed_from_timestamp(timestamp: i64) -> (bool, i64) {
    let hour = 3600;
    let offset = FixedOffset::east_opt(8 * hour).unwrap();
    let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let timestamp_datetime: DateTime<FixedOffset> = offset.from_utc_datetime(&naive);
    let current_datetime_naive: NaiveDateTime = Utc::now().naive_utc();
    let current_datetime: DateTime<FixedOffset> = offset.from_utc_datetime(&current_datetime_naive);
    let duration = current_datetime.signed_duration_since(timestamp_datetime);
    let days_elapsed = duration.num_days();

    (current_datetime.date_naive() == timestamp_datetime.date_naive() && current_datetime.time() == timestamp_datetime.time(), days_elapsed)
}



async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    if let Ok(raw_timestamp) = env::var("TIMESTAMP") {
        if let Ok(timestamp) = raw_timestamp.parse::<i64>() {
            let (is_before, days_elapsed) = days_elapsed_from_timestamp(timestamp);

            let message = if is_before {
                format!("Flora will be {} days old today.", days_elapsed)
            } else {
                format!("Flora is {} days old today.", days_elapsed)
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
