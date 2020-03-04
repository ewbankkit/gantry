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

#[macro_use]
extern crate lazy_static;

extern crate wascc_actor as actor;
use gantry_protocol as protocol;

use actor::prelude::*;
use std::sync::RwLock;
mod catalog;

lazy_static! {
    static ref OPERATOR_SIGNERS: RwLock<Vec<String>> = RwLock::new(Vec::new());
}

actor_handlers!{ messaging::OP_DELIVER_MESSAGE => handle_message,
                core::OP_CONFIGURE => handle_config,
                core::OP_HEALTH_REQUEST => health }

pub fn health(_ctx: &CapabilitiesContext, _req: core::HealthRequest) -> ReceiveResult {
    Ok(vec![])
}


fn handle_config(
    ctx: &CapabilitiesContext,
    config: core::CapabilityConfiguration,
) -> ReceiveResult {    
    let mut lock = OPERATOR_SIGNERS.write().unwrap();
    lock.push(config.values.get("operator").unwrap().to_string());
    for signer in config.values.get("signers").unwrap().split(',') {
        lock.push(signer.to_string());
    }
    ctx.log(&format!(
        "Catalog configured with the following valid operator signers: {}",
        lock.join(",")
    ));
    Ok(vec![])
}

fn handle_message(
    ctx: &CapabilitiesContext,
    msg:  messaging::DeliverMessage,
) -> ReceiveResult {    
    let subject = msg.message.subject.clone();

    if subject == protocol::catalog::SUBJECT_CATALOG_PUT_TOKEN {
        let token = deserialize::<protocol::catalog::Token>(msg.message.body.as_ref())?;
        publish_results(
            ctx,
            &msg.message.reply_to,
            serialize(catalog::put_token(ctx, &token)?)?,
        )
    } else if subject == protocol::catalog::SUBJECT_CATALOG_QUERY {
        let query =
            deserialize::<protocol::catalog::CatalogQuery>(msg.message.body.as_ref())?;
        publish_results(
            ctx,
            &msg.message.reply_to,
            serialize(catalog::query_catalog(ctx, &query)?)?,
        )
    } else {
        Err("Unknown catalog request subject".into())
    }
}

fn publish_results(
    ctx: &CapabilitiesContext,
    subject: &str,
    results: Vec<u8>,
) -> ReceiveResult {    
    if !subject.is_empty() {
        ctx.log(&format!("About to publish to {}", subject));
        ctx.msg().publish(subject, None, &results)?;
    }
    Ok(results)
}
