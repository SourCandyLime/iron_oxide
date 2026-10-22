use crate::{
    to_model,
    Model
};
use super::raw::RawPcad2;

pub fn parse_from_file(dir: &str) -> Model{
    let source =
        std::fs::read_to_string(dir)
            .expect("Failed to read Model");

    let parse =
        raw_parse(&source)
            .expect("Failed to parse Model");
    
    to_model(&parse)
}
pub fn raw_parse(source: &str) -> Result<RawPcad2, serde_json::Error> {
    serde_json::from_str(source)
}