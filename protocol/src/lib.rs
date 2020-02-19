pub mod catalog {
    //! # Gantry catalog protocol
    //!
    //! This module contains data types and traits for use with Gantry's catalog
    //! functionality. Gantry supports the following catalog operations:
    //! * `put` - Adds a token to the catalog
    //! * `query` - Queries the catalog    
    //! * `delete` - Removes an actor from the catalog. This operation _marks an actor as removed_, but does not remove the corresponding entry from underlying storage
    include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

    pub static SUBJECT_CATALOG_PUT_TOKEN: &str = "gantry.catalog.tokens.put";
    pub static SUBJECT_CATALOG_DELETE_TOKEN: &str = "gantry.catalog.tokens.delete";
    pub static SUBJECT_CATALOG_QUERY: &str = "gantry.catalog.tokens.query";
}

pub mod stream {
    //! # Gantry streaming protocol
    //!
    //! This module contains data types and traits for use with Gantry's module streaming
    //! functionality. Gantry separates the streaming of a module's raw bytes from the management
    //! of a module's metadata. The following operations are available for streaming:
    //! * `stream_put` - Send the raw bytes for a module to Gantry, corresponding to a specific public key+revision pair
    //! * `stream_get` - Retrieve the raw bytes for a module to Gantry, corresponding to a specific public key+revision pair
    include!(concat!(env!("OUT_DIR"), "/stream.rs"));

    // Requests to initiate transfers
    pub static SUBJECT_STREAM_DOWNLOAD: &str = "gantry.stream.get";
    pub static SUBJECT_STREAM_UPLOAD: &str = "gantry.stream.put";

    // Topics on which actual transfers occur
    pub static SUBJECT_STREAM_DOWNLOAD_PREFIX: &str = "gantry.stream.download.";
    pub static SUBJECT_STREAM_UPLOAD_PREFIX: &str = "gantry.stream.upload.";
}

pub mod token {
    pub enum TokenType {
        Actor,
        Account,
        Operator,
    }
}
