# Gantry Catalog

Gantry is a WebAssembly module registry that manages [wascap](https://github.com/wascc/wascap)-signed `.wasm` files. The Gantry server is a waSCC host runtime that loads several waSCC actors that make up the bulk of its functionality.

This actor is responsible for managing the _catalog_, the storage and retrieval of account, operator, and actor JWTs. It does _not_ manage the storage or retrieval of actor binary files, as that is the responsibility of the [streams](../streams/README.md) actor.

## Building

To build and sign _Gantry Catalog_ use the `make build` command. This command assumes that you have an `account.nk` and a `module.nk` file in your `.keys/` directory. In order to ensure that the official version of this actor always has the same subject and issuer, we maintain these keys offline. To build your own, you'll have to generate your own keys. Also note that the official Gantry waSCC host binary expects the `catalog` and `streams` actors to have specific public keys, so you will also need to modify the host if you're building your own version of these actors.
