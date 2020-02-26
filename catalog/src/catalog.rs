use actor::prelude::*;
use gantry_protocol as protocol;
use protocol::catalog::*;
use protocol::token::TokenType;

pub(crate) fn put_token(
    ctx: &CapabilitiesContext,
    token: &Token,
) -> Result<CatalogQueryResult, Box<dyn std::error::Error>> {
    ctx.log(&format!("Request to put token: {:?}", token));
    let claims: serde_json::Value = serde_json::from_str(&token.decoded_token_json)?;
    let subject = claims["sub"].as_str().unwrap();
    write_token(ctx, subject, token, &claims)
}

pub(crate) fn query_catalog(
    ctx: &CapabilitiesContext,
    query: &CatalogQuery,
) -> Result<CatalogQueryResults, Box<dyn std::error::Error>> {
    ctx.log(&format!("Querying catalog: {:?}", query));
    let set_key = {
        match QueryType::from_i32(query.query_type).unwrap() {
            QueryType::Actor => "gantry:actors",
            QueryType::Operator => "gantry:operators",
            QueryType::Account => "gantry:accounts",
        }
        .to_string()
    };
    let results_raw = ctx.kv().set_members(&set_key)?;
    ctx.log(&format!("Done quering - {} results", results_raw.len()));

    let items = results_raw
        .iter()
        .map(|r| {
            let raw = ctx.kv().get(&format!("gantry:tokens:{}:0", r)).unwrap();
            let details: serde_json::Value = serde_json::from_str(&raw.unwrap()).unwrap();
            CatalogQueryResult {
                actor: None,
                issuer: details["iss"].as_str().unwrap_or("??").to_string(),
                name: details["wascap"]["name"]
                    .as_str()
                    .unwrap_or("??")
                    .to_string(),
                subject: r.to_string(),
            }
        })
        .collect();

    Ok(CatalogQueryResults { results: items })
}

/// Places decoded token in gantry:tokens:{subject}:{revision}
/// puts revision into gantry:actors:{subject}:revisions
/// Puts subject into list gantry:actors, gantry:operators, or gantry:accounts depending on subject type
/// Puts the raw (encoded) token in gantry:tokens:{subject}:{revision}:raw
fn write_token(
    ctx: &CapabilitiesContext,
    subject: &str,
    token: &Token,
    claims: &serde_json::Value,
) -> Result<CatalogQueryResult, Box<dyn ::std::error::Error>> {
    if !token.validation_result.as_ref().unwrap().signature_valid {
        return Err("Cannot store token - invalid signature".into());
    }
    if token.validation_result.as_ref().unwrap().expired {
        return Err("Cannot store token - expired".into());
    }

    ctx.kv()
        .set(&token_key(subject, claims), &token.decoded_token_json, None)?;
    ctx.kv()
        .set(&token_raw_key(subject, claims), &token.raw_token, None)?;
    ctx.kv()
        .set_add(&revisions_key(subject), &format!("{}", revision(claims)))?;

    match token_type(subject) {
        TokenType::Actor => ctx.kv().set_add("gantry:actors", subject)?,
        TokenType::Operator => ctx.kv().set_add("gantry:operators", subject)?,
        TokenType::Account => ctx.kv().set_add("gantry:accounts", subject)?,
    };

    Ok(CatalogQueryResult {
        subject: claims["sub"].as_str().unwrap_or("??").to_string(),
        issuer: claims["iss"].as_str().unwrap_or("??").to_string(),
        name: claims["wascap"]["name"]
            .as_str()
            .unwrap_or("Anonymous")
            .to_string(),
        actor: None,
    })
}

fn token_type(subject: &str) -> TokenType {
    if subject.starts_with('A') {
        TokenType::Account
    } else if subject.starts_with('M') {
        TokenType::Actor
    } else {
        TokenType::Operator
    }
}

fn revisions_key(subject: &str) -> String {
    format!("gantry:tokens:{}:revisions", subject)
}

fn token_key(subject: &str, claims: &serde_json::Value) -> String {
    format!("gantry:tokens:{}:{}", subject, revision(claims))
}

fn token_raw_key(subject: &str, claims: &serde_json::Value) -> String {
    format!("gantry:tokens:{}:{}:raw", subject, revision(claims))
}

fn revision(claims: &serde_json::Value) -> u64 {
    claims["metadata"]["revision"].as_u64().unwrap_or(0)
}
