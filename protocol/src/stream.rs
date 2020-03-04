//! # Gantry streaming protocol
//!
//! This module contains data types and traits for use with Gantry's module streaming
//! functionality. Gantry separates the streaming of a module's raw bytes from the management
//! of a module's metadata. The following operations are available for streaming:
//! * `stream_put` - Send the raw bytes for a module to Gantry, corresponding to a specific public key+revision pair
//! * `stream_get` - Retrieve the raw bytes for a module to Gantry, corresponding to a specific public key+revision pair

// Requests to initiate transfers
pub static SUBJECT_STREAM_DOWNLOAD: &str = "gantry.stream.get";
pub static SUBJECT_STREAM_UPLOAD: &str = "gantry.stream.put";

// Topics on which actual transfers occur
pub static SUBJECT_STREAM_DOWNLOAD_PREFIX: &str = "gantry.stream.download.";
pub static SUBJECT_STREAM_UPLOAD_PREFIX: &str = "gantry.stream.upload.";

/// A request to download a file from Gantry

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DownloadRequest {
    pub actor: String,
}

/// A request to upload a file to Gantry
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UploadRequest {
    pub actor: String,
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub total_chunks: u64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransferAck {
    pub success: bool,
    pub actor: String,
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub total_chunks: u64,
}

/// Acknowledgement of a single chunk
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ChunkAck {
    pub success: bool,
    pub sequence_no: u64,
    pub bytes_sent: u64,
}

/// A single chunk of a file
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FileChunk {
    pub sequence_no: u64,
    pub actor: String,
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub total_chunks: u64,
    pub chunk_bytes: Vec<u8>,
}
