// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2021 Adrian <adrian.eddy at gmail>

use std::time::Instant;
use argh::FromArgs;
use std::sync::{ Arc, atomic::AtomicBool };

use telemetry_parser::*;
use telemetry_parser::tags_impl::*;
use telemetry_parser::util; 

use serde_json::json;

/** gyro2bb v0.2.8-Author: Adrian <adrian.eddy@gmail.com> Extract gyro and metadata from camera files **/

#[derive(FromArgs)]
struct Opts {
    /// input file
    #[argh(positional)]
    input: String,

    /// IMU orientation (XYZ, ZXY etc, lowercase is negative, eg. xZy)
    #[argh(option)]
    imuo: Option<String>,
}


fn main() {
    let opts: Opts = argh::from_env();
    let _time = Instant::now();

    let mut stream = std::fs::File::open(&opts.input).unwrap();
    let filesize = stream.metadata().unwrap().len() as usize;

    let input = Input::from_stream(
        &mut stream, 
        filesize, 
        &opts.input, 
        |_|(), 
        Arc::new(AtomicBool::new(false))
    ).unwrap();

    let camera_type = input.camera_type().to_string();
    let camera_model = input.camera_model().unwrap_or(&"".into()).to_string();

    let samples = input.samples.as_ref().unwrap();

    // new "json-like" mode
    let imu_data = util::normalized_imu(&input, opts.imuo).unwrap();

    // collect Extra metadata
    let mut extra_metadata = None;
    if let Some(first) = samples.get(0) {
        if let Some(map) = &first.tag_map {
            if let Some(extra) = map.get(&GroupId::Default)
                                    .and_then(|g| g.get(&TagId::Metadata)) {
                extra_metadata = Some(extra.value.to_string());
            }
        }
    }

    let camera = json!({
        "camera": {
            "type": camera_type,
            "model": camera_model
        }   ,
        "extra_metadata": extra_metadata
    });

    println!("CAMERASTART");
    println!("{}", serde_json::to_string(&camera).unwrap());
    println!("CAMERAEND");
    println!("IMUSTART");

    // convert IMU into JSON array
    let mut i = 0;
    for v in imu_data {
        if v.gyro.is_some() || v.accl.is_some() {
            let gyro = v.gyro.unwrap_or_default();
            let accl = v.accl.unwrap_or_default();
            let imu = json!({
                "t": v.timestamp_ms,
                "g": { "x": -gyro[2], "y": gyro[1], "z": gyro[0] },
                "a": { "x": -accl[2] * 2048.0, "y": accl[1] * 2048.0, "z": accl[0] * 2048.0 }
            });
            println!("{}", serde_json::to_string(&imu).unwrap());
            i += 1;
        }
    }

    

    println!("IMUEND");
    
}
