const PROTO: &str = "proto";
/// GEN_PROTO=1 cargo build
/// cargo fmt
fn main() {
    if std::env::var("GEN_PROTO").is_err() {
        println!("cargo:warning=Skipping proto generation (GEN_PROTO not set)");
        return;
    }
    execute_build();
}

#[allow(dead_code)]
fn execute_build() {
    // 递归收集 proto 下所有 .proto 文件
    let mut proto_files = Vec::new();
    for entry in walkdir::WalkDir::new(PROTO) {
        let entry = entry.unwrap();
        if entry.path().extension().is_some_and(|ext| ext == PROTO) {
            proto_files.push(entry.path().to_owned());
        }
    }
    tonic_prost_build::configure()
        .out_dir("src/proto")
        .compile_protos(&proto_files, &[PROTO.into()])
        .unwrap();
}
