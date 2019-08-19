#![feature(async_await)]

use std::{
    env,
    io::{self, BufReader, Read},
    path::Path,
};

use clap::{crate_version, App, Arg};
use log::{error, info};

use divvun_pipeline::{
    file::{load_pipeline_file, PIPELINE_EXTENSION},
    run::run,
};

#[runtime::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let pipeline = "pipeline";

    let matches = App::new("divvun-pipeline")
        .version(crate_version!())
        .author("projektir <oprojektir@gmail.com>")
        .about("Asynchronous parallel pipeline for text processing.")
        .arg(
            Arg::with_name(pipeline)
                .help(&format!(
                    "The .{} file with the requested pipeline flow and required resources",
                    PIPELINE_EXTENSION
                ))
                .index(1),
        )
        .get_matches();

    let mut vec_buffer = Vec::new();
    let reader = BufReader::new(io::stdin());
    reader.buffer().read_to_end(&mut vec_buffer).unwrap();

    if let Some(pipeline_file) = matches.value_of(pipeline) {
        match load_pipeline_file(Path::new(pipeline_file)) {
            Ok((pipeline, resources)) => {
                let output = run(pipeline, resources, vec_buffer).await;
                info!("Output: {}", &output);
            }
            Err(e) => {
                error!("Error loading pipeline file: {:?}", e);
                return;
            }
        }
    }
}
