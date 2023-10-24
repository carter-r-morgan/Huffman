use std::fs::File;

#[derive(Debug)]
pub enum Mode {
    Encode,
    Decode
}

#[derive(Debug)]
pub struct Config {
    pub mode: Mode,
    pub input_path: String,
    pub output_path: String
}

impl Config {
    pub fn build(mut args: impl Iterator<Item=String>) -> Result<Config, &'static str> {
        let (mode, input_path) = match (args.nth(1), args.next()) {
            (Some(m), Some(ip)) => (m, ip),
            _ => return Err("Not enough arguments"),
        };

        let mode = if mode.eq_ignore_ascii_case("encode") {
            Mode::Encode
        }
        else if mode.eq_ignore_ascii_case("decode") {
            Mode::Decode
        }
        else {
            return Err("Unknown mode");
        };

        let output_path = match (args.next(), args.next()) {
            (Some(flag), Some(value)) if flag == "-o" => value,
            _ => String::from("out.txt"),
        };

        Ok( Config {mode, input_path, output_path} )
    }
}

pub fn run(config: Config) {
    let mut input = File::open(config.input_path).unwrap();
    let mut output = File::create(config.output_path).unwrap();

    match config.mode {
        Mode::Encode => crate::file::encode(&mut input, &mut output),
        Mode::Decode => crate::file::decode(&mut input, &mut output),
    }
}