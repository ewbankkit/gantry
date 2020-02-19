fn main() {
    prost_build::compile_protos(&["src/catalog.proto", "src/stream.proto"], &["src/"]).unwrap();
}
