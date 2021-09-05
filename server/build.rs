fn main() {
    prost_build::compile_protos(&["../protobuf/sample.proto"], &["../protobuf"]).unwrap();
}
