use gantryclient::{Chunks, Client, CHUNK_SIZE};
use gantry_protocol as protocol;
use protocol::catalog::*;
use std::io::Read;
use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
    name = "gantry", 
    about = "A command line utility for accessing a Gantry waSCC registry")]
struct Cli {
    #[structopt(flatten)]
    command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
enum CliCommand {
    /// Query the Gantry registry
    #[structopt(name = "get")]
    Get(GetCommand),
    /// Puts a token in the registry
    #[structopt(name = "put")]
    Put(PutCommand),
    /// Downloads an actor module from the registry
    #[structopt(name = "download")]
    Download(DownloadCommand),
    /// Uploads an actor module to the registry
    #[structopt(name = "upload")]
    Upload(UploadCommand),
}

#[derive(Debug, Clone, StructOpt)]
struct DownloadCommand {
    /// The public key of the actor to download
    #[structopt(short = "a", long = "actor")]
    actor: String,
}

#[derive(Debug, Clone, StructOpt)]
struct UploadCommand {
    /// Path to the actor to be uploaded
    #[structopt(short = "a", long = "actor", parse(from_os_str))]
    actor_path: PathBuf,
}

#[derive(Debug, Clone, StructOpt)]
struct PutCommand {
    /// The raw, encoded token to insert
    #[structopt(short = "t", long = "token")]
    token: String,
}

#[derive(Debug, Clone, StructOpt)]
enum GetCommand {
    /// Get the list of actors
    #[structopt(name = "actors")]
    Actor,
    /// Get the list of operators
    #[structopt(name = "operators")]
    Operator,
    /// Get the list of accounts
    #[structopt(name = "accounts")]
    Account,
}

fn to_catalog_query_type(cmd: &GetCommand) -> i32 {
    match cmd {
        GetCommand::Actor => QueryType::Actor as i32,
        GetCommand::Operator => QueryType::Operator as i32,
        GetCommand::Account => QueryType::Account as i32,
    }
}

fn handle_command(cmd: CliCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    match cmd {
        CliCommand::Get(get_cmd) => query(get_cmd),
        CliCommand::Put(put_cmd) => put(put_cmd),
        CliCommand::Download(download_cmd) => download(download_cmd),
        CliCommand::Upload(upload_cmd) => upload(upload_cmd),
    }
}

fn query(cmd: GetCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    let query = CatalogQuery {
        query_type: to_catalog_query_type(&cmd),
        issuer: "".to_string(),
    };
    let client = Client::default();
    let results = client.query_catalog(&query)?;
    if results.results.is_empty() {
        println!("No results.");
        return Ok(());
    }

    let mut table = term_table::Table::new();
    table.max_column_width = 60;

    table.style = term_table::TableStyle::extended();
    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new_with_alignment(
            "Gantry Query Results",
            2,
            term_table::table_cell::Alignment::Center,
        ),
    ]));
    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new_with_alignment(
            "Name",
            1,
            term_table::table_cell::Alignment::Center,
        ),
        term_table::table_cell::TableCell::new_with_alignment(
            "Subject / Issuer",
            1,
            term_table::table_cell::Alignment::Center,
        ),
    ]));

    for res in results.results {
        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new_with_alignment(
                res.name,
                1,
                term_table::table_cell::Alignment::Center,
            ),
            term_table::table_cell::TableCell::new_with_alignment(
                format!("{}\n{}", res.subject, res.issuer),
                1,
                term_table::table_cell::Alignment::Center,
            ),
        ]));
    }
    println!("{}", table.render());
    Ok(())
}

fn put(cmd: PutCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    let token = Token {
        raw_token: cmd.token.clone(),
        decoded_token_json: "".to_string(),
        validation_result: None,
    };
    let client = Client::default();
    client.put_token(&token)?;
    Ok(())
}

fn download(cmd: DownloadCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    let client = Client::default();
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));
    pb.set_message(&format!("{}.wasm", cmd.actor));

    let filename = format!("{}.wasm", cmd.actor);

    let _ack = client.download_actor(&cmd.actor, move |chunk| {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .unwrap();
        pb.set_length(chunk.total_bytes);
        file.write(&chunk.chunk_bytes).unwrap();
        let new = (chunk.sequence_no * chunk.chunk_size) + chunk.chunk_bytes.len() as u64;
        pb.set_position(new);
        if chunk.sequence_no == chunk.total_chunks {
            pb.finish_with_message("downloaded");
        }
        Ok(())
    })?;

    ::std::thread::sleep(std::time::Duration::from_millis(5000)); //TODO: this is a hack. stop it.
    Ok(())
}

fn upload(cmd: UploadCommand) -> Result<(), Box<dyn ::std::error::Error>> {
    use indicatif::{ProgressBar, ProgressStyle};

    let mut f = ::std::fs::File::open(&cmd.actor_path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let claims = wascap::wasm::extract_claims(&buf)?.unwrap();
    let fsize = f.metadata().unwrap().len();
    let actor = claims.claims.subject;

    println!("Uploading {}", actor);

    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    pb.set_length(f.metadata()?.len());

    let req = protocol::stream::UploadRequest {
        actor: actor.to_string(),
        chunk_size: CHUNK_SIZE,
        total_bytes: fsize,
        total_chunks: fsize / CHUNK_SIZE,
    };
    let client = Client::default();
    let _ack = client.start_upload(&req)?;

    let f = ::std::fs::File::open(&cmd.actor_path)?;
    let chunks = Chunks::new(f, CHUNK_SIZE as usize);
    chunks.enumerate().for_each(|(i, chunk)| {
        let chunk = chunk.unwrap();
        pb.set_position(i as u64 * CHUNK_SIZE + chunk.len() as u64);
        client
            .upload_chunk(
                i as u64,
                &actor,
                fsize,
                CHUNK_SIZE,
                fsize / CHUNK_SIZE,
                chunk,
            )
            .unwrap();
    });
    pb.finish_with_message("uploaded");
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
