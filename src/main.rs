extern crate chrono;

use chrono::{FixedOffset, Utc};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use std::env;
use std::io::Cursor;
use serde_json::json;
use then_backend::{days_elapsed_between_timestamps, project};
use image::{DynamicImage, GenericImage, Rgba};
use base64::encode;
use regex::Regex;

async fn days_counter(_request: Request) -> Result<Response<Body>, Error> {
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

fn wikipedia(n: usize) -> [f64; 3] {
    match n % 16 {
        0 => [66.0, 30.0, 15.0],
        1 => [25.0, 7.0, 26.0],
        2 => [9.0, 1.0, 47.0],
        3 => [4.0, 4.0, 73.0],
        4 => [0.0, 7.0, 100.0],
        5 => [12.0, 44.0, 138.0],
        6 => [24.0, 82.0, 177.0],
        7 => [57.0, 125.0, 209.0],
        8 => [134.0, 181.0, 229.0],
        9 => [211.0, 236.0, 248.0],
        10 => [241.0, 233.0, 191.0],
        11 => [248.0, 201.0, 95.0],
        12 => [255.0, 170.0, 0.0],
        13 => [204.0, 128.0, 0.0],
        14 => [153.0, 87.0, 0.0],
        15 => [106.0, 52.0, 3.0],
        _ => unreachable!(),
    }
}

fn coarse_color_map(n: usize, _t_re: f64, _t_im: f64) -> [u8; 4] {
    let color = wikipedia(n);

    let r = color[0] as u8;
    let g = color[1] as u8;
    let b = color[2] as u8;

    [r, g, b, 255]
}

const RADIUS: f64 = 4.0;

fn mandelbrot_escape_time_factory<F>(color_map: F) -> impl Fn(f64, f64, usize) -> [u8; 4]
    where F: Fn(usize, f64, f64) -> [u8; 4]
{
    move |re_0: f64, im_0: f64, max_iter: usize| -> [u8; 4] {
        let mut z = (re_0, im_0);
        let mut t_re: f64;
        let mut t_im: f64;

        for n in 0..max_iter {
            t_re = z.0 * z.0;
            t_im = z.1 * z.1;
            if t_re + t_im > RADIUS {
                return color_map(n, t_re, t_im);
            }
            z.1 = 2.0 * z.0 * z.1 + im_0;
            z.0 = t_re - t_im + re_0;
        }

        [0, 0, 0, 0]
    }
}

fn julia_escape_time_factory<F>(color_map: F, c: (f64, f64)) -> impl Fn(f64, f64, usize) -> [u8; 4]
    where F: Fn(usize, f64, f64) -> [u8; 4]
{
    move |re_0: f64, im_0: f64, max_iter: usize| -> [u8; 4] {
        let mut z = (re_0, im_0);

        for i in 0..max_iter {
            if z.0 * z.0 + z.1 * z.1 > RADIUS {
                return color_map(i, z.0, z.1);
            }
            z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
        }

        [0, 0, 0, 0]
    }
}

async fn fractal<F>(request: Request, x: i32, y: i32, z: i32, escape_time: F) -> Result<Response<Body>, Error>
    where F: Fn(f64, f64, usize) -> [u8; 4] {
    let (xmin, ymin) = project(x, y, z);
    let (xmax, ymax) = project(x + 1, y + 1, z);
    let size = request
        .query_string_parameters_ref()
        .and_then(|params| params.first("size"))
        .map(|s| s.parse::<usize>().unwrap_or(256))
        .unwrap();


    let width: usize = size;
    let height: usize = size;

    println!("{} {} {}", size, width, width);

    let re_range: Vec<f64> = (0..width).map(|i| xmin + (xmax - &xmin) * i as f64 / (width - 1) as f64).collect();
    let im_range: Vec<f64> = (0..height).map(|j| ymin + (ymax - &ymin) * j as f64 / (height - 1) as f64).collect();


    let mut imbuf = DynamicImage::new_rgba8(width as u32, height as u32);

    for i in 0..width {
        for j in 0..height {
            imbuf.put_pixel(i as u32, j as u32, Rgba(escape_time(re_range[i], im_range[j], 256)));
        }
    }

    let mut bytes = Vec::new();

    imbuf.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .expect("Failed to encode image");

    let base64_encoded = encode(bytes);
    Ok(Response::builder()
        .status(200)
        .header("content-type", "image/png")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET,OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token,X-Amz-User-Agent")
        .body(
            json!({
            "imageBase64": base64_encoded,
          }).to_string().into())
        .map_err(Box::new)?)
}

async fn not_found(_request: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(404)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET,OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token,X-Amz-User-Agent")
        .body(
            json!({
            "message": "Not found.",
          }).to_string().into())
        .map_err(Box::new)?)
}

async fn router(request: Request) -> Result<Response<Body>, Error> {
    tracing::info!("raw_http_path: {}", request.raw_http_path());
    tracing::info!("query_string_parameters: {:#?}", request.query_string_parameters());
    tracing::info!("path_parameters: {:#?}", request.path_parameters());
    tracing::info!("stage_variables: {:#?}", request.stage_variables());

    let resp = if request.raw_http_path() == "/flora" {
        days_counter(request).await.unwrap()
    } else if let Some(captures) = Regex::new(r"^/fractal/(\w+)/(\d+)/(\d+)/(\d+).png$").unwrap().captures(request.raw_http_path()) {
        // Extract values from captures
        let fractal_name = &captures[1];
        let x: i32 = captures[2].parse().unwrap();
        let y: i32 = captures[3].parse().unwrap();
        let z: i32 = captures[4].parse().unwrap();

        match fractal_name {
            "julia" => {
                fractal(request, x, y, z, julia_escape_time_factory(coarse_color_map, (-0.4, 0.6))).await.unwrap()
            }
            "mandelbrot" => {
                fractal(request, x, y, z, mandelbrot_escape_time_factory(coarse_color_map)).await.unwrap()
            }
            _ => {
                not_found(request).await.unwrap()
            }
        }
    } else {
        not_found(request).await.unwrap()
    };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(router)).await
}
