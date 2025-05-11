use std::fs;
use std::path::{Path, PathBuf};
use clap::{Args, Parser, Subcommand};
use inquire::Text;
use inquire::validator::{ErrorMessage, Validation};
use regex::Regex;
use rust_embed::{Embed, RustEmbed};

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