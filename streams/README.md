# Gantry Streams

The Gantry server requires a number of actor modules to comprise the bulk of its business logic. The _Gantry Streams_ actor is responsible for managing the raw bytes associated with the actors stored within the registry, including storing them in a supporting blob store and streaming those bytes into and out of the registry.

## Building

To build and sign _Gantry Streams_ use the `make build` command. This command assumes that you have an `account.nk` and a `module.nk` file in your `.keys/` directory. In order to ensure that the official version of this actor always has the same subject and issuer, we maintain these keys offline. To build your own, you'll have to generate your own keys. Also note that the official Gantry waSCC host binary expects the `catalog` and `streams` actors to have specific public keys, so you will also need to modify the host if you're building your own version of these actors.
