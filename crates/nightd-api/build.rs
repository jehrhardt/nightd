fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("PROTOC").is_none() {
        // SAFETY: build scripts run in a single-threaded process.
        unsafe {
            std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
        }
    }

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/nightd/v1/nightd.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/nightd/v1/nightd.proto");

    Ok(())
}
