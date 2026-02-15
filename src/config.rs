use serde::Deserialize;
use serde_json;

use std::fs::File;
use std::path::Path;

fn default_templates() -> String {
    String::from("templates")
}

#[derive(Deserialize)]
pub struct Config {
    pub device: Option<String>,
    pub host: String,
    #[serde(default = "default_templates")]
    pub templates: String,
}
impl std::default::Default for Config {
    fn default() -> Self {
        Config{
            device: None,
            host: String::from("127.0.0.1:5000"),
            templates: String::from("templates"),
        }
    }
}

pub fn read_config(path: &Path) -> Result<Config, String> {
    if !path.exists() {
        println!("Configuration file does not exist, using defaults");
        return Ok(Config::default());
    }

    let fh = match File::open(path) {
        Ok(fh) => fh,
        Err(e) => return Err(format!("Cannot open configuration file: {}", e)),
    };

    match serde_json::from_reader(fh) {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(format!("Cannot parse configuration file: {}", e)),
    }
}
