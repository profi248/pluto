fn main() {
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(&["src/protos"])
        .inputs(&[
            "src/protos/auth.proto",
            "src/protos/_status.proto"
        ])
        .cargo_out_dir("protos")
        .run_from_script()
}
