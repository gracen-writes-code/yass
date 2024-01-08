use semver::{Version, VersionReq};
use std::{fs, io::Read, path::PathBuf};
use yaml_rust::{Yaml, YamlLoader};

struct ModuleReq {
    name: String,
    version_req: VersionReq,
}

struct Module {
    name: String,
    version: Version,
    dependencies: Vec<ModuleReq>,
}

pub struct Profile {
    name: String,
    modules: Vec<ModuleReq>,
}

impl Profile {
    pub fn load(path: PathBuf) -> Self {
        let mut yaml_file = fs::File::open(path.join("profile.yml")).unwrap();
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
        }; // TODO parse module reqs

        Self { name, modules }
    }
}

pub struct Game {
    pub profile: Profile,

    module_pool: Vec<Module>,
}

impl Game {
    pub fn new(profile: Profile, modules_dir: PathBuf) -> Self {
        let module_paths: Vec<Module> = fs::read_dir(modules_dir)
            .unwrap()
            .map()
            .;

        Self {
            profile,
            module_pool,
        }
    }

    fn match_modules(&self, module_req: ModuleReq) -> Vec<Module> {
        todo!() // find all modules that match the module req
    }
}
