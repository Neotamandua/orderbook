fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/command.proto")?;
    tonic_build::compile_protos("proto/query.proto")?;
    tonic_build::compile_protos("proto/stream.proto")?;
    Ok(())
}
