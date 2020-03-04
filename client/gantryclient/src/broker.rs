use gantry_protocol as protocol;
use natsclient::{AuthenticationStyle, Client, ClientOptions};
use protocol::catalog::*;
use protocol::stream::*;
use std::time::Duration;
use protocol::{serialize, deserialize};

pub const CHUNK_SIZE: u64 = 256 * 1024; // 256KB

pub(crate) fn query(
    client: &Client,
    query: &CatalogQuery,
) -> Result<CatalogQueryResults, Box<dyn ::std::error::Error>> {
    let buf = serialize(&query)?;    
    let reply = client.request(
        "gantry.catalog.tokens.query",
        &buf,
        Duration::from_millis(700),
    )?;

    Ok(deserialize::<CatalogQueryResults>(reply.payload.as_ref())?)
}

pub(crate) fn put(client: &Client, token: &Token) -> Result<(), Box<dyn ::std::error::Error>> {
    let buf = serialize(token)?;    
    let reply = client.request(
        "gantry.catalog.tokens.put",
        &buf,
        Duration::from_millis(100),
    )?;

    let res = deserialize::<CatalogQueryResult>(reply.payload.as_ref())?;
    println!(
        "Token '{}' with issuer {}, subject {} registered.",
        res.name, res.issuer, res.subject
    );
    Ok(())
}

pub(crate) fn start_upload(
    client: &Client,
    req: &UploadRequest,
) -> Result<TransferAck, Box<dyn ::std::error::Error>> {
    let buf = serialize(req)?;
    
    let res = client.request(
        protocol::stream::SUBJECT_STREAM_UPLOAD,
        &buf,
        ::std::time::Duration::from_millis(100),
    )?;
    let tack = deserialize::<TransferAck>(res.payload.as_ref())?;
    Ok(tack)
}

pub(crate) fn request_download<F>(
    client: &Client,
    req: DownloadRequest,
    chunk_handler: F,
) -> Result<TransferAck, Box<dyn ::std::error::Error>>
where
    F: Fn(FileChunk) -> Result<(), Box<dyn ::std::error::Error>> + Sync + Send,
    F: 'static,
{
    let buf = serialize(&req)?;
    
    let dltopic = format!(
        "{}{}",
        protocol::stream::SUBJECT_STREAM_DOWNLOAD_PREFIX,
        req.actor
    );

    client.subscribe(&dltopic, move |msg| {
        let chunk = deserialize::<FileChunk>(msg.payload.as_ref()).unwrap();
        chunk_handler(chunk).unwrap(); // TODO: get rid of unwrap
        Ok(())
    })?;

    let res = client.request(
        protocol::stream::SUBJECT_STREAM_DOWNLOAD,
        &buf,
        std::time::Duration::from_millis(100),
    )?;
    let tack = deserialize::<TransferAck>(res.payload.as_ref())?;
    Ok(tack)
}

pub(crate) fn upload_chunk(
    c: &Client,
    sequence_no: u64,
    actor: &str,
    chunk_size: u64,
    total_bytes: u64,
    total_chunks: u64,
    bytes: Vec<u8>,
) -> Result<(), Box<dyn ::std::error::Error>> {
    let chunk = protocol::stream::FileChunk {
        actor: actor.to_string(),
        chunk_bytes: bytes,
        chunk_size,
        sequence_no,
        total_bytes,
        total_chunks,
    };
    let buf = serialize(&chunk)?;    
    let subject = format!(
        "{}{}",
        protocol::stream::SUBJECT_STREAM_UPLOAD_PREFIX,
        actor
    );
    let _res = c.request(&subject, &buf, std::time::Duration::from_millis(2000))?;
    Ok(())
}

pub(crate) fn get_client(
    nats_urls: Vec<String>,
    jwt: Option<&str>,
    seed: Option<&str>,
) -> Result<Client, Box<dyn ::std::error::Error>> {
    let mut auth_style = AuthenticationStyle::Anonymous;
    if jwt.is_some() && seed.is_some() {
        auth_style = AuthenticationStyle::UserCredentials(
            jwt.unwrap().to_string(),
            seed.unwrap().to_string(),
        );
    }

    let opts = ClientOptions::builder()
        .cluster_uris(nats_urls)
        .authentication(auth_style)
        .build()?;

    let client = Client::from_options(opts)?;
    client.connect()?;
    Ok(client)
}
