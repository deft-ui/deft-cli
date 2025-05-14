use std::{fs, io};
use std::fs::{File};
use std::path::{Path, PathBuf};
use clap::{Args, Parser, Subcommand};
use inquire::Text;
use inquire::validator::{ErrorMessage, Validation};
use regex::Regex;
use rust_embed::{Embed, RustEmbed};
use serde_json::{Map, Value};

const DEFT_CONFIG_FILE: &str = "deft.config.json";

#[derive(Embed)]
#[folder = "template-android"]
struct AndroidAsset;

#[derive(Embed)]
#[folder = "template-ohos"]
struct OhosAsset;

#[derive(Parser)]
struct CliOptions {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitCommand),
}

#[derive(Args)]
pub struct InitCommand {
    #[command(subcommand)]
    platform: InitPlatformCommands,
}

#[derive(Subcommand)]
enum InitPlatformCommands {
    Ohos,
    Android,
}

fn main() {
    let cli_options = CliOptions::parse();
    match cli_options.command {
        Commands::Init(init_command) => {
            match init_command.platform {
                InitPlatformCommands::Ohos => init_ohos(),
                InitPlatformCommands::Android => init_android(),
            }
        }
    }
}

fn is_valid_appid(id: &str) -> bool {
    let reg = Regex::new("^([a-zA-Z_]\\w*)+([.][a-zA-Z_]\\w*)+$").unwrap();
    reg.is_match(id)
}

fn inquire_app_id() -> String {
    Text::new("Input your app id:")
        .with_validator(|str: &str| {
            let v = if is_valid_appid(str) {
                Validation::Valid
            } else {
                Validation::Invalid(ErrorMessage::Default)
            };
            Ok(v)
        })
        .prompt()
        .unwrap()
}

fn replace(path: &PathBuf, search: &str, replacement: &str) {
    let config_content = fs::read_to_string(&path).unwrap().replace(search, replacement);
    fs::write(&path, config_content).unwrap();
}

fn init_ohos() {
    let dst = Path::new("ohos").to_path_buf();
    if dst.exists() {
        println!("{} already exists", dst.display());
        return;
    }
    let app_id = inquire_app_id();
    dist::<OhosAsset>(&dst);

    replace(&dst.join("AppScope/app.json5"), "fun.kason.deftapp", &app_id);
    if let Err(error) =  write_app_id("ohos", &app_id) {
        eprintln!("{}", error);
    }
}


fn write_app_id(platform: &str, app_id: &str) -> Result<(), String> {
    let mut config = load_deft_config()?;
    let root = config.as_object_mut().ok_or("Invalid config found")?;
    let ohos = root.entry(platform)
        .or_insert(Value::Object(Map::new()))
        .as_object_mut().ok_or("Invalid config found")?;
    ohos.insert("appId".to_string(), Value::String(app_id.to_string()));
    save_deft_config(config)
}

fn save_deft_config(value: Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| e.to_string())?;
    fs::write(&DEFT_CONFIG_FILE, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_deft_config() -> Result<Value, String> {
    let mut config_content = "{}".to_string();
    let exists = fs::exists(&DEFT_CONFIG_FILE).map_err(|e| e.to_string())?;
    if exists {
        config_content = fs::read_to_string(DEFT_CONFIG_FILE)
            .map_err(|e| format!("{:?}", e))?;
    }
    let v: serde_json::Result<Value> = serde_json::from_str(&config_content);
    v.map_err(|e| format!("{:?}", e))
}

fn init_android() {
    let dst = Path::new("android").to_path_buf();
    if dst.exists() {
        println!("{} already exists", dst.display());
        return;
    }
    let app_id = inquire_app_id();
    dist::<AndroidAsset>(&dst);
    replace(&dst.join("app/build.gradle"), "fun.kason.deft_demo", &app_id);

    if let Err(error) =  write_app_id("android", &app_id) {
        eprintln!("{}", error);
    }
    #[cfg(unix)]
    if let Err(error) = fix_exec_permission("android/gradlew") {
        eprintln!("{}", error);
    }
}

#[cfg(unix)]
fn fix_exec_permission(file: &str) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let file = File::open(file)?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o744);
    file.set_permissions(permissions)?;
    Ok(())
}

fn dist<E: RustEmbed>(out_dir: &Path) {
    for e in E::iter() {
        let data = E::get(&e).unwrap();
        let out_path = out_dir.join(e.to_string());
        let out_dir = out_path.parent().unwrap();
        if !out_dir.exists() {
            fs::create_dir_all(&out_dir).unwrap();
        }
        fs::write(&out_path, data.data).unwrap();
    }
}