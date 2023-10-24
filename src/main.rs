use std::env;
use huffman::cli::{self, Config};


// hzip [encode|decode] input-name [-o output-name]
fn main() {
    let args = env::args();
    let config = match Config::build(args) {
        Ok(cfg) => cfg,
        Err(msg) => { println!("{msg}"); return; },
    };

    cli::run(config);
}
