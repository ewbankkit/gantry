// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate wascc_actor as actor;
use gantry_protocol as protocol;
use actor::prelude::*;
use protocol::stream::{
    DownloadRequest, TransferAck, UploadRequest, SUBJECT_STREAM_DOWNLOAD_PREFIX,
    SUBJECT_STREAM_UPLOAD_PREFIX,
};

const CHUNK_SIZE: u64 = 256 * 1024; // 256KB chunks

actor_receive!(receive);

pub fn receive(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> ReceiveResult {
    match operation {
        messaging::OP_DELIVER_MESSAGE => handle_message(ctx, msg),
        blobstore::OP_RECEIVE_CHUNK => handle_blob_chunk(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("Unknown operation".into()),
    }
}

fn handle_blob_chunk(
    ctx: &CapabilitiesContext,
    msg: impl Into<blobstore::FileChunk>,
) -> ReceiveResult {
    let chunk = msg.into();
    ctx.log("Received chunk from blob store");
    let newchunk = convert_chunk(&chunk);
    let mut buf = Vec::new();
    newchunk.encode(&mut buf)?;
    ctx.log(&format!(
        "Converted chunk {} ({} bytes)",
        newchunk.sequence_no,
        buf.len()
    ));
    ctx.msg().publish(
        &format!(
            "{}{}",
            SUBJECT_STREAM_DOWNLOAD_PREFIX,
            chunk.id[0..chunk.id.len() - 5].to_string()
        ),
        None,
        &buf,
    )?;
    Ok(vec![])
}

fn convert_chunk(chunk: &blobstore::FileChunk) -> protocol::stream::FileChunk {
    protocol::stream::FileChunk {
        sequence_no: chunk.sequence_no,
        actor: chunk.id[0..chunk.id.len() - 5].to_string(),
        chunk_size: chunk.chunk_size,
        total_bytes: chunk.total_bytes,
        total_chunks: chunk.total_bytes / chunk.chunk_size,
        chunk_bytes: chunk.chunk_bytes.clone(),
    }
}

fn handle_message(
    ctx: &CapabilitiesContext,
    payload: impl Into<messaging::DeliverMessage>,
) -> ReceiveResult {
    let msg = payload.into();
    let subject = msg.message.as_ref().unwrap().subject.clone();

    if subject == protocol::stream::SUBJECT_STREAM_DOWNLOAD {
        let req = DownloadRequest::decode(msg.message.as_ref().unwrap().body.as_ref())?;
        handle_download(ctx, req, &msg.message.as_ref().unwrap().reply_to)
    } else if subject == protocol::stream::SUBJECT_STREAM_UPLOAD {
        let req = UploadRequest::decode(msg.message.as_ref().unwrap().body.as_ref())?;
        handle_upload(ctx, req, &msg.message.as_ref().unwrap().reply_to)
    } else if subject.starts_with(SUBJECT_STREAM_UPLOAD_PREFIX) {
        let chunk =
            protocol::stream::FileChunk::decode(msg.message.as_ref().unwrap().body.as_ref())?;
        handle_upload_chunk(ctx, chunk, &msg.message.as_ref().unwrap().reply_to)
    } else {
        Err("Unknown stream request".into())
    }
}

fn handle_upload_chunk(
    ctx: &CapabilitiesContext,
    chunk: protocol::stream::FileChunk,
    reply_to: &str, 
) -> ReceiveResult {
    ctx.log("Received file chunk");
    let xfer = blobstore::Transfer {
        total_size: chunk.chunk_bytes.len() as u64,
        blob_id: format!("{}.wasm", chunk.actor),
        container: "gantry".to_string(),
        chunk_size: chunk.chunk_size,
        total_chunks: chunk.total_chunks,
    };
    ctx.objectstore()
        .upload_chunk(&xfer, chunk.sequence_no, chunk.chunk_bytes.as_ref())?;
    let ack = protocol::stream::ChunkAck {
        bytes_sent: chunk.chunk_bytes.len() as u64,
        sequence_no: chunk.sequence_no,
        success: true
    };
    let mut buf = Vec::new();
    ack.encode(&mut buf)?;
    ctx.msg().publish(reply_to, None, &buf)?;
    Ok(vec![])
}

fn handle_upload(ctx: &CapabilitiesContext, req: UploadRequest, reply_to: &str) -> ReceiveResult {
    let filename = format!("{}.wasm", req.actor);

    let actors = catalog_get_actors(ctx)?;
    if !actors.contains(&req.actor) {
        return Err("Module is not registered in catalog".into());
    }
    let blob = blobstore::Blob {
        id: filename.to_string(),
        container: "gantry".to_string(),
        byte_size: req.total_bytes,
    };
    let ack = TransferAck {
        success: true,
        actor: req.actor,
        total_bytes: blob.byte_size,
        chunk_size: CHUNK_SIZE,
        total_chunks: blob.byte_size / CHUNK_SIZE,
    };    

    let mut buf = Vec::new();
    ack.encode(&mut buf)?;
    ctx.msg().publish(reply_to, None, &buf)?;
    ctx.objectstore()
        .start_upload(&blob, req.chunk_size, req.total_bytes)?;
    Ok(vec![])
}

fn handle_download(
    ctx: &CapabilitiesContext,
    req: DownloadRequest,
    reply_to: &str,
) -> ReceiveResult {
    let blob_id = format!("{}.wasm", req.actor);
    let actors = catalog_get_actors(ctx)?;
    if !actors.contains(&req.actor) {
        return Err("Module is not registered in catalog".into());
    }
    let blobinfo = ctx.objectstore().get_blob_info("gantry", &blob_id)?;
    ctx.log(&format!("Retrieve blob info: {:?}", blobinfo));
    if let Some(blobinfo) = blobinfo {
        let ack = TransferAck {
            success: true,
            actor: req.actor.to_string(),
            total_bytes: blobinfo.byte_size,
            chunk_size: CHUNK_SIZE,
            total_chunks: blobinfo.byte_size / CHUNK_SIZE,
        };

        let mut buf = Vec::new();
        ack.encode(&mut buf)?;
        ctx.msg().publish(reply_to, None, &buf)?;
        ctx.objectstore().start_download(&blobinfo, CHUNK_SIZE)?;
        Ok(vec![])
    } else {
        Err("There was no file found for this actor. Has it been uploaded?".into())
    }
}

const CATALOG_ACTOR: &str = "MCIXJVXAXKDX7UFYDFW2737SHVIRNZILS3ULODGEQOVCTWQ7HSGOHUY7";

fn catalog_get_actors(
    ctx: &CapabilitiesContext,
) -> ::std::result::Result<Vec<String>, Box<dyn ::std::error::Error>> {
    let results = ctx.raw().call(
        CATALOG_ACTOR,
        messaging::OP_DELIVER_MESSAGE,
        &gen_actor_query(),
    )?;
    let query_res = protocol::catalog::CatalogQueryResults::decode(results.as_ref())?;
    Ok(query_res
        .results
        .iter()
        .map(|r| r.subject.clone())
        .collect())
}

fn gen_actor_query() -> Vec<u8> {
    let mut buf = Vec::new();
    let q = protocol::catalog::CatalogQuery {
        issuer: "".to_string(),
        query_type: protocol::catalog::QueryType::Actor as i32,
    };
    q.encode(&mut buf).unwrap();
    let msg = messaging::DeliverMessage {
        message: Some(messaging::BrokerMessage {
            reply_to: "".to_string(),
            subject: protocol::catalog::SUBJECT_CATALOG_QUERY.to_string(),
            body: buf,
        }),
    };
    let mut outbuf = Vec::new();
    msg.encode(&mut outbuf).unwrap();
    outbuf
}
