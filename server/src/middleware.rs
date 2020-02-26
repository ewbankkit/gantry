use codec::messaging;
use prost::Message;
use wascap::jwt;
use wascap::jwt::Account;
use wascap::jwt::Actor;
use wascap::jwt::Operator;
use wascc_host::host::{Invocation, InvocationResponse};
use wascc_host::Middleware;
use gantry_protocol as protocol;

pub(crate) struct JWTDecoder {}

impl JWTDecoder {
    pub fn new() -> Self {
        JWTDecoder {}
    }
}

impl Middleware for JWTDecoder {
    fn actor_pre_invoke(&self, inv: Invocation) -> wascc_host::Result<Invocation> {
        if inv.operation == messaging::OP_DELIVER_MESSAGE {
            let msg = decode_deliver_message(inv.msg.as_slice())?;
            if let Some(msg) = msg.message {
                if msg.subject == protocol::catalog::SUBJECT_CATALOG_PUT_TOKEN {
                    info!("Unpacking and Augmenting incoming JWT");
                    let newinv = augment_token_message(
                        msg.body.as_slice(),
                        msg.reply_to,
                        msg.subject,
                        &inv,
                    )?;
                    return Ok(newinv);
                }
            }
        }
        Ok(inv)
    }
    fn actor_post_invoke(
        &self,
        response: InvocationResponse,
    ) -> wascc_host::Result<InvocationResponse> {
        Ok(response)
    }
    fn capability_pre_invoke(&self, inv: Invocation) -> wascc_host::Result<Invocation> {
        Ok(inv)
    }
    fn capability_post_invoke(
        &self,
        response: InvocationResponse,
    ) -> wascc_host::Result<InvocationResponse> {
        Ok(response)
    }
}

fn decode_deliver_message(msg: &[u8]) -> wascc_host::Result<messaging::DeliverMessage> {
    messaging::DeliverMessage::decode(msg).map_err(|e| e.into())
}

fn augment_token_message(
    body: &[u8],
    reply_to: String,
    subject: String,
    inv: &Invocation,
) -> wascc_host::Result<Invocation> {
    let token = protocol::catalog::Token::decode(body)?;

    // Operators and Accounts can safely decode actor tokens (lossy), so this is
    // an "okay" way to get at the claim subject, then re-decode once we figure
    // out what kind of token it is.
    // Feels hacky. Need to improve.
    let claims: jwt::Claims<Operator> = jwt::Claims::decode(&token.raw_token).unwrap(); // TODO: kill the unwrap

    let (claims_string, vres) = {
        if claims.subject.starts_with('O') {
            (
                serde_json::to_string(&claims).unwrap(),
                jwt::validate_token::<Operator>(&token.raw_token).unwrap(),
            ) // TODO: kill the unwrap
        } else if claims.subject.starts_with('A') {
            let c = jwt::Claims::<Account>::decode(&token.raw_token).unwrap();
            (
                serde_json::to_string(&c).unwrap(),
                jwt::validate_token::<Account>(&token.raw_token).unwrap(),
            )
        } else {
            let c = jwt::Claims::<Actor>::decode(&token.raw_token).unwrap();
            (
                serde_json::to_string(&c).unwrap(),
                jwt::validate_token::<Actor>(&token.raw_token).unwrap(),
            )
        }
    };

    let new_token = protocol::catalog::Token {
        raw_token: token.raw_token,
        decoded_token_json: claims_string,
        validation_result: Some(protocol::catalog::TokenValidation {
            expires_human: vres.expires_human,
            expired: vres.expired,
            not_before_human: vres.not_before_human,
            cannot_use_yet: vres.cannot_use_yet,
            signature_valid: vres.signature_valid,
        }),
    };

    let mut buf = Vec::new();
    new_token.encode(&mut buf).unwrap();

    let delivermsg = messaging::DeliverMessage {
        message: Some(messaging::BrokerMessage {
            body: buf,
            reply_to,
            subject,
        }),
    };

    let mut buf_final = Vec::new();
    delivermsg.encode(&mut buf_final).unwrap();

    Ok(Invocation {
        origin: inv.origin.clone(),
        operation: inv.operation.clone(),
        msg: buf_final,
    })
}

#[cfg(test)]
mod test {
    use super::JWTDecoder;
    use codec::messaging;
    use nkeys::KeyPair;
    use prost::Message;
    use wascap::jwt;
    use wascc_host::host::Invocation;
    use wascc_host::Middleware;

    #[test]
    fn middleware_augments_valid_token() {
        // Test that when an actor is sent a message from a wascap:messaging capability containing a token for
        // insertion into the catalog, the middleware will crack open the raw token, perform validation, and
        // store the validation results and a raw JSON string version of the decoded token.
        let (claims, issuer) = gen_valid_token();
        let message = wrap_token(&claims, &issuer);
        let inv = make_invocation(message);

        let decoder = JWTDecoder {};

        let res = decoder.actor_pre_invoke(inv).unwrap();
        let new_token = extract_token(&res);

        assert!(new_token.validation_result.is_some());
        assert!(new_token.validation_result.unwrap().signature_valid);

        let claims: jwt::Claims<jwt::Actor> =
            serde_json::from_str(&new_token.decoded_token_json).unwrap();
        let actor_metadata = claims.metadata.unwrap();
        assert_eq!(actor_metadata.name.unwrap(), "test actor");
    }

    // The chain is pretty deep...
    // Invocation (contains)-> DeliverMessage (contains)-> BrokerMessage (contains)->Token

    fn make_invocation(message: messaging::DeliverMessage) -> Invocation {
        let mut buf = Vec::new();
        message.encode(&mut buf).unwrap();
        Invocation {
            operation: messaging::OP_DELIVER_MESSAGE.to_string(),
            origin: "wascc:messaging".to_string(),
            msg: buf,
        }
    }

    fn wrap_token(claims: &jwt::Claims<jwt::Actor>, issuer: &KeyPair) -> messaging::DeliverMessage {
        let encoded = claims.encode(issuer).unwrap();
        let token = protocol::catalog::Token {
            raw_token: encoded,
            decoded_token_json: "".to_string(),
            validation_result: None,
        };
        let mut buf = Vec::new();
        token.encode(&mut buf).unwrap();
        messaging::DeliverMessage {
            message: Some(messaging::BrokerMessage {
                reply_to: "reply".to_string(),
                subject: protocol::catalog::SUBJECT_CATALOG_PUT_TOKEN.to_string(),
                body: buf,
            }),
        }
    }

    fn gen_valid_token() -> (jwt::Claims<jwt::Actor>, KeyPair) {
        let issuer = KeyPair::new_account();
        let subject = KeyPair::new_module();
        (
            jwt::Claims::<jwt::Actor>::new(
                "test actor".to_string(),
                issuer.public_key(),
                subject.public_key(),
                Some(vec!["testcap".to_string()]),
                None,
                false,
                Some(1),
                Some("1.0.0".to_string()),
            ),
            issuer,
        )
    }

    fn extract_token(inv: &Invocation) -> protocol::catalog::Token {
        let delivermsg = messaging::DeliverMessage::decode(inv.msg.as_ref()).unwrap();
        protocol::catalog::Token::decode(delivermsg.message.unwrap().body.as_ref()).unwrap()
    }
}
