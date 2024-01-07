use std::{fs::File, io::Read, path::PathBuf};
use yaml_rust::{Yaml, YamlLoader};

struct ModuleRegistry {}

pub struct Profile {
    name: String,
    modules: Vec<String>,
}

impl Profile {
    pub fn load(path: PathBuf) -> Self {
        let mut yaml_file = File::open(path.join("profile.yml")).unwrap();
        let mut yaml_str = String::new();
        yaml_file.read_to_string(&mut yaml_str).unwrap();

        let yaml = YamlLoader::load_from_str(&yaml_str).unwrap()[0];

        let name = match yaml["name"] {
            Yaml::String(ref s) => s.clone(),
            _ => panic!("invalid name"),
        };
        let modules = match yaml["modules"] {
            Yaml::Array(arr) => arr
                .into_iter()
                .map(|module| match module {
                    Yaml::String(ref s) => s.clone(),
                    _ => panic!("invalid module"),
                })
                .collect::<Vec<String>>(),
            _ => panic!("invalid module list"),
        };

        Self { name, modules }
    }
}

pub struct Game {
    profile: Profile,

    module_registry: ModuleRegistry,
}

impl Game {
    pub fn new(profile: Profile, modules_dir: PathBuf) -> Self {
        Self {
            profile,
            module_registry
        }
    }
}
