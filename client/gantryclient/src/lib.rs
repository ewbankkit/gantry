use gantry_protocol as protocol;
pub use broker::CHUNK_SIZE;
pub use chunks::Chunks;
pub use protocol::catalog::{CatalogQuery, CatalogQueryResults, Token};
pub use protocol::stream::{DownloadRequest, FileChunk, TransferAck, UploadRequest};

pub mod broker;
pub mod chunks;

/// An instance of a Gantry client connection
#[derive(Clone)]
pub struct Client {
    natsclient: natsclient::Client,
}

impl Client {
    pub fn new(nats_urls: Vec<String>, jwt: &str, seed: &str) -> Client {
        Client {
            natsclient: broker::get_client(nats_urls, Some(jwt), Some(seed)).unwrap(),
        }
    }

    pub fn default() -> Client {
        Client {
            natsclient: broker::get_client(vec!["nats://localhost:4222".into()], None, None)
                .unwrap(),
        }
    }

    pub fn put_token(&self, token: &Token) -> Result<(), Box<dyn ::std::error::Error>> {
        broker::put(&self.natsclient, token)
    }

    pub fn query_catalog(
        &self,
        query: &CatalogQuery,
    ) -> Result<CatalogQueryResults, Box<dyn ::std::error::Error>> {
        broker::query(&self.natsclient, query)
    }

    pub fn remove_token(&self, _token: &Token) -> Result<(), Box<dyn ::std::error::Error>> {
        unimplemented!()
    }

    pub fn start_upload(
        &self,
        req: &UploadRequest,
    ) -> Result<TransferAck, Box<dyn ::std::error::Error>> {
        broker::start_upload(&self.natsclient, req)
    }

    pub fn upload_chunk(
        &self,
        sequence_no: u64,
        actor: &str,
        chunk_size: u64,
        total_bytes: u64,
        total_chunks: u64,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn ::std::error::Error>> {
        broker::upload_chunk(
            &self.natsclient,
            sequence_no,
            actor,
            chunk_size,
            total_bytes,
            total_chunks,
            bytes,
        )
    }

    pub fn download_actor<F>(
        &self,
        actor: &str,
        chunk_handler: F,
    ) -> Result<TransferAck, Box<dyn ::std::error::Error>>
    where
        F: Fn(FileChunk) -> Result<(), Box<dyn ::std::error::Error>> + Sync + Send,
        F: 'static,
    {
        let req = DownloadRequest {
            actor: actor.to_string(),
        };
        broker::request_download(&self.natsclient, req, chunk_handler)
    }
}
