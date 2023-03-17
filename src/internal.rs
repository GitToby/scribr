use std::collections::HashMap;
use std::fs::File as Fs;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use dirs::home_dir;

use crate::model::{File, Settings, SCRIBR_CONFIG_FILE_NAME};

pub fn get_default_init_files(gist_id: Option<&str>) -> HashMap<String, File> {
    let settings = match gist_id {
        None => Settings::default(),
        Some(gist_id) => Settings::new_with_gist_id(gist_id),
    };
    HashMap::from([(
        SCRIBR_CONFIG_FILE_NAME.to_string(),
        File {
            content: serde_yaml::to_string(&settings).expect("Default settings are incorrect."),
        },
    )])
}

pub fn get_scribr_home_dir() -> PathBuf {
    home_dir().unwrap().join(".scribr")
}

pub fn get_scribr_config_file() -> PathBuf {
    get_scribr_home_dir().join(SCRIBR_CONFIG_FILE_NAME)
}

pub fn read_file(file_path: &PathBuf) -> Option<String> {
    if file_path.exists() {
        let mut file = Fs::open(file_path).expect("bad open of settings file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        Some(contents)
    } else {
        None
    }
}

pub fn get_settings_from_disk(settings_file: Option<PathBuf>) -> Settings {
    let settings = match settings_file {
        None => Settings::default(),
        Some(settings_file) => {
            if settings_file.exists() {
                let file = Fs::open(settings_file).expect("bad open of settings file");
                let reader = BufReader::new(file);
                serde_yaml::from_reader(reader).expect("Bad format of settings file")
            } else {
                println!(
                    "Cannot locate settings file at {} have you deleted it?",
                    settings_file.display()
                );
                Settings::default()
            }
        }
    };
    if let Some(remote) = &settings.remote {
        if remote.gist_id == None {
            println!("Warning! remote settings are missing data! backups may fail")
        }
    }

    settings
}

pub fn scriber_files_setup() -> bool {
    // make this a little more structural
    get_scribr_home_dir().exists()
}

#[cfg(test)]
mod tests {
    use crate::model::RemoteSettings;

    use super::*;

    #[test]
    fn test_get_default_settings_none_asked() {
        let s = get_settings_from_disk(None);
        assert_eq!(s, Settings::default())
    }
    #[test]
    fn test_get_default_settings_path_not_exists() {
        let s = get_settings_from_disk(Some(PathBuf::from("this/path/probably/does/not/exists")));
        assert_eq!(s, Settings::default())
    }

    #[test]
    fn test_get_default_settings_path_empty_remote() {
        let resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("tests")
            .join("settings_empty_remote.yaml");
        let actual = get_settings_from_disk(Some(resources_dir));
        let expected = Settings {
            default_notebook: "my_notes.txt".to_string(),
            verbosity: 0,
            no_magic_commands: true,
            remote: Some(RemoteSettings { gist_id: None }),
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_get_default_settings_path_exists_remote() {
        let resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("tests")
            .join("settings_with_remote.yaml");
        let actual = get_settings_from_disk(Some(resources_dir));
        let expected = Settings {
            default_notebook: "my_notes.txt".to_string(),
            verbosity: 0,
            no_magic_commands: true,
            remote: Some(RemoteSettings {
                gist_id: Some("tests-gist-id".to_string()),
            }),
        };
        assert_eq!(actual, expected)
    }
    #[test]
    fn test_get_default_settings_path_exists_with_missing() {
        let resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("tests")
            .join("settings_with_missing.yaml");
        let actual = get_settings_from_disk(Some(resources_dir));
        let expected = Settings {
            default_notebook: "my_notes.txt".to_string(),
            verbosity: 0,
            no_magic_commands: false,
            remote: None,
        };
        assert_eq!(actual, expected)
    }
}
