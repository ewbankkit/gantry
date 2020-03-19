#[macro_use]
extern crate log;

extern crate wascc_codec as codec;
mod middleware;

use middleware::JWTDecoder;
use std::{collections::HashMap, path::PathBuf};
use structopt::clap::AppSettings;
use structopt::StructOpt;
use wascap::jwt::{Claims, Operator};
use wascc_host::{host, Actor, NativeCapability};

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
    name = "gantry-server", 
    about = "Gantry is a secure registry for waSCC WebAssembly modules")]
struct Cli {
    #[structopt(flatten)]
    command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
struct CliCommand {
    /// Path to the signed WebAssembly module responsible for catalog management
    #[structopt(short = "c", long = "catalog", parse(from_os_str))]
    catalog_path: PathBuf,

    /// Path to the signed WebAssembly module responsible for streams management
    #[structopt(short = "s", long = "stream", parse(from_os_str))]
    streamer_path: PathBuf,

    /// Path to the capability providers used by Gantry
    #[structopt(short = "p", long = "provider", parse(from_os_str))]
    provider_paths: Vec<PathBuf>,

    /// The Gantry operator JWT. Used for provenance verification of all WebAssembly
    /// modules stored in the registry.
    #[structopt(short = "o", long = "operator")]
    operator_jwt: String,
}

fn handle_command(cmd: CliCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    let operator: Claims<Operator> = Claims::<Operator>::decode(&cmd.operator_jwt)?;
    info!("Gantry operator is : {}", operator.subject);
    host::add_actor(Actor::from_file(cmd.catalog_path)?)?;
    host::add_actor(Actor::from_file(cmd.streamer_path)?)?;
    host::add_middleware(JWTDecoder::new());
    cmd.provider_paths.iter().for_each(|p| {
        host::add_native_capability(NativeCapability::from_file(p).unwrap()).unwrap();
    });

    host::configure(
        "MCIXJVXAXKDX7UFYDFW2737SHVIRNZILS3ULODGEQOVCTWQ7HSGOHUY7",
        "wascc:keyvalue",
        redis_config(),
    )?;

    host::configure(
        "MCIXJVXAXKDX7UFYDFW2737SHVIRNZILS3ULODGEQOVCTWQ7HSGOHUY7",
        "wascc:messaging",
        generate_config("gantry.catalog.tokens.*"),
    )?;

    host::configure(
        "MATR36QS6IWITSNUS2I7V72R2I3ALJCIS2Y4FJQJZ33KQN5MRXDNJMJ2",
        "wascc:messaging",
        generate_config("gantry.stream.get,gantry.stream.put,gantry.stream.upload.*"),
    )?;

    host::configure(
        "MCIXJVXAXKDX7UFYDFW2737SHVIRNZILS3ULODGEQOVCTWQ7HSGOHUY7",
        "MCIXJVXAXKDX7UFYDFW2737SHVIRNZILS3ULODGEQOVCTWQ7HSGOHUY7",
        operator_config(
            &operator.subject,
            operator.metadata.unwrap().valid_signers.as_ref().unwrap(),
        ),
    )?;

    host::configure(
        "MATR36QS6IWITSNUS2I7V72R2I3ALJCIS2Y4FJQJZ33KQN5MRXDNJMJ2",
        "wascc:blobstore",
        generate_fs_config(),
    )?;

    std::thread::park();
    Ok(())
}

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let args = Cli::from_args();
    let cmd = args.command;
    env_logger::init();

    match handle_command(cmd) {
        Ok(_) => {}
        Err(e) => {
            println!("Command line failure: {}", e);
        }
    }
    Ok(())
}

fn generate_config(sub: &str) -> HashMap<String, String> {
    let mut hm = HashMap::new();
    hm.insert("SUBSCRIPTION".to_string(), sub.to_string());
    hm.insert("URL".to_string(), "nats://localhost:4222".to_string());

    hm
}

fn redis_config() -> HashMap<String, String> {
    let mut hm = HashMap::new();
    hm.insert("URL".to_string(), "redis://127.0.0.1:6379".to_string());

    hm
}

fn operator_config(op: &str, valid_signers: &[String]) -> HashMap<String, String> {
    let mut hm = HashMap::new();
    hm.insert("operator".to_string(), op.to_string());
    hm.insert("signers".to_string(), valid_signers.join(","));

    hm
}

fn generate_fs_config() -> HashMap<String, String> {
    let mut hm = HashMap::new();
    //hm.insert("ROOT".to_string(), "/tmp".to_string());
    hm.insert("ENDPOINT".to_string(), "http://localhost:9000".to_string());
    hm.insert("REGION".to_string(), "us-east-1".to_string());
    hm.insert("AWS_ACCESS_KEY".to_string(), "minioadmin".to_string());
    hm.insert(
        "AWS_SECRET_ACCESS_KEY".to_string(),
        "minioadmin".to_string(),
    );



    hm
}
