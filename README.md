# Gantry

Gantry is a registry service for managing secure WebAssembly modules signed with the [wascap](https://github.com/wascc/wascap) command line tool or library. It is a **waSCC** runtime host that loads waSCC actors responsible for the following:

* [catalog](./catalog/README.md) - Maintain a catalog of JSON Web Tokens (JWTs) for actors, operators, and accounts. Uses the `wascc:messaging` and `wascc:keyvalue` capabilities.
* [streams](./streams/README.md) - Manage the storage of actor binary (`.wasm`) files and the streaming of those files to and from the registry. Uses the `wascc:messaging` and `wascc:blobstore` capabilities, as well as consumes the `catalog` actor via actor-to-actor comms.

## Additional Components

The following additional components are part of the overall **Gantry** project:

* [protocol](./protocol/README.md) - A shared protocol describing the public API for interacting with Gantry
* [server](./server/README.md) - The Gantry waSCC host runtime
* [client](./client/README.md) - The Gantry client Rust crate and CLI tool

## WARNING

Gantry is in a very preliminary state and all of its components are likely to change radically in the near future.
