use std::path::Path;

pub const CONFIG_PATH_OPTION: &str = "config";
pub const CONFIG_DEFAULT_PATH: &str = "./conf.yml";


pub struct Config {
    /// where should the note output csv be stored
    note_output_path: String
}


pub fn read_from_path(path: &Path) -> Config {
    let conf = Config { note_output_path: path.to_str().unwrap().to_string() };
    return conf;
}
