use semver::{Version, VersionReq};
use std::{fs, io::Read, path::PathBuf};
use yaml_rust::{Yaml, YamlLoader};

struct ModuleReq {
    name: String,
    version_req: VersionReq,
}

impl ModuleReq {
    fn parse(string: String) -> Self {
        todo!()
    }
}

struct PreloadModule {
    name: String,
    version: Version,
    dependencies: Vec<ModuleReq>,

    dir: PathBuf,
}

impl PreloadModule {
    fn new(dir: PathBuf) -> Self {
        todo!()
    }
}

struct Module {}

impl Module {
    fn load(preload: PreloadModule) -> Self {
        todo!()
    }
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

        let yaml = &YamlLoader::load_from_str(&yaml_str).unwrap()[0];

        let name = match yaml["name"] {
            Yaml::String(ref s) => s.clone(),
            _ => panic!("invalid name"),
        };
        let modules = match &yaml["modules"] {
            Yaml::Array(arr) => arr
                .into_iter()
                .map(|module| match module {
                    Yaml::String(ref s) => s.clone(),
                    _ => panic!("invalid module"),
                })
                .collect::<Vec<String>>(),
            _ => panic!("invalid module list"),
        }
        .into_iter()
        .map(|s| ModuleReq::parse(s))
        .collect();

        Self { name, modules }
    }
}

pub struct Game {
    pub profile: Profile,

    module_pool: Vec<PreloadModule>,
}

impl Game {
    pub fn new(profile: Profile, modules_dir: PathBuf) -> Self {
        let module_pool = fs::read_dir(modules_dir)
            .unwrap()
            .filter_map(|res| res.ok())
            .filter(|entry| entry.file_type().unwrap().is_dir())
            .map(|entry| PreloadModule::new(entry.path()))
            .collect();

        Self {
            profile,
            module_pool,
        }
    }

    fn match_modules(&self, module_req: ModuleReq) -> Vec<Module> {
        todo!() // find all modules that match the module req
    }
}
