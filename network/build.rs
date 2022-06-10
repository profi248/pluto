fn main() {
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(&["src/protos"])
        .inputs(&[
            "src/protos/auth.proto",
            "src/protos/shared.proto"
        ])
        .cargo_out_dir("protos")
        .run_from_script()
}
