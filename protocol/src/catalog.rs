//! # Gantry catalog protocol
//!
//! This module contains data types and traits for use with Gantry's catalog
//! functionality. Gantry supports the following catalog operations:
//! * `put` - Adds a token to the catalog
//! * `query` - Queries the catalog
//! * `delete` - Removes an actor from the catalog. This operation _marks an actor as removed_, but does not remove the corresponding entry from underlying storage

pub static SUBJECT_CATALOG_PUT_TOKEN: &str = "gantry.catalog.tokens.put";
pub static SUBJECT_CATALOG_DELETE_TOKEN: &str = "gantry.catalog.tokens.delete";
pub static SUBJECT_CATALOG_QUERY: &str = "gantry.catalog.tokens.query";

/// A token contains the raw string for a JWT signed with the ed25519 signature
/// format. Actors, Accounts, Operators are all identified by tokens
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Token {
    pub raw_token: String,
    pub decoded_token_json: String,
    pub validation_result: Option<TokenValidation>,
}

/// A protocol-specific message version of the validation result that the wascap
/// library provides
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TokenValidation {
    pub expired: bool,
    pub expires_human: String,
    pub not_before_human: String,
    pub cannot_use_yet: bool,
    pub signature_valid: bool,
}

/// Represents the metadata on file for a given actor. This metadata is roughly
/// the same as the information contained in the actor's embedded and signed JWT, and
/// does NOT include the actor's raw bytes.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ActorSummary {
    pub public_key: String,
    pub capabilities: Vec<String>,
    pub provider: bool,
    pub tags: Vec<String>,
    pub version: String,
    pub revision: u64,
    pub account: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountSummary {
    pub public_key: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CatalogQuery {
    pub query_type: QueryType,
    pub issuer: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CatalogQueryResults {
    pub results: Vec<CatalogQueryResult>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CatalogQueryResult {
    pub subject: String,
    pub issuer: String,
    pub name: String,
    pub actor: Option<ActorSummary>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum QueryType {
    Actor,
    Account,
    Operator,
}
