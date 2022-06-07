#[macro_use] extern crate rouille;

use image::{RgbImage, Rgb, ImageOutputFormat};
use log::{info, warn, error};
use std::env::args;

const CACHE_PATH : &str = "cache";
const TILE_SIZE : u32 = 256;
const ITER_MAX : u32 = 512;
const ZOOM : f64 = 4.0;

fn main() {
    let host_port = args().nth(1).unwrap_or("0.0.0.0:8080".to_string());

    simple_logger::init().unwrap();

    rouille::start_server(host_port, move |request| {
        router!(request,
                (GET) (/) => {
                    rouille::Response::redirect_302("/pub/index.html")
                },
                (GET) (/api/{z: u32}/{x: u32}/{y: u32}) => {
                    send_tile(z, x, y)
                },
                _ => {
                    let response = rouille::match_assets(&request, ".");
                    if !response.is_success() {
                        error!("Error 404: Not found: {:?}", request.url());
                    }
                    response
                })
    });
}

fn send_tile(z: u32, x: u32, y: u32) -> rouille::Response {
    let path_name = format!("{}/{}/{}/{}.png", CACHE_PATH, z, x, y);
    let path = std::path::Path::new(&path_name);
    
    match std::fs::File::open(&path) {
        Ok(file) => {
            info!("-- get {:?}", &path);
            rouille::Response::from_file("image/png", file)
        },
        Err(_) => {
            warn!("== GEN {:?}", &path);
            let img = gen_tile(z, x, y);
            let _ = std::fs::create_dir_all(path.parent().unwrap());
            let _ = img.save(path);
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes),
                         ImageOutputFormat::Png).unwrap();            
            rouille::Response::from_data("image/png", bytes)
        }
    }
}

fn gen_tile(z: u32, x: u32, y: u32) -> RgbImage {
    let n = (2.0_f64).powf(z as f64);
    let x0 = -ZOOM/2.0 + ZOOM * (x as f64) / n;
    let y0 = -ZOOM/2.0 + ZOOM * (y as f64) / n;
    let step = ZOOM/n;
    
    RgbImage::from_fn(TILE_SIZE, TILE_SIZE, | x, y | {
        let x00 =  x0 + (step * x as f64 / TILE_SIZE as f64);
        let y00 =  y0 + (step * y as f64 / TILE_SIZE as f64);
        let val = mandel(x00, y00);
        Rgb([(val/2) as u8, val as u8, (val*32) as u8])
    })
}

fn mandel(x0: f64, y0: f64) -> u32 {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let out = 4.0;

    for i in 0..ITER_MAX {
        let x_x = x * x;
        let y_y = y * y;
        if x_x + y_y > out {
            return i;
        }
        y = (2.0 * x * y) + y0;
        x = (x_x - y_y) + x0;
    }
    0
}
