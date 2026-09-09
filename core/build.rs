fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 🐝 COLMENA — Proto de enjambre gRPC
    let colmena_proto = "proto/colmena.proto";
    let colmena_path = if std::path::Path::new(colmena_proto).exists() {
        colmena_proto.to_string()
    } else {
        let workspace = "core/proto/colmena.proto";
        if std::path::Path::new(workspace).exists() {
            workspace.to_string()
        } else {
            panic!("❌ No se encontró 'colmena.proto' en proto/ ni core/proto/");
        }
    };
    tonic_build::compile_protos(&colmena_path)?;

    // 📡 NEXUS TELEMETRY — Proto de telemetría
    let telemetry_proto = "proto/nexus_telemetry.proto";
    let telemetry_path = if std::path::Path::new(telemetry_proto).exists() {
        telemetry_proto.to_string()
    } else {
        let workspace = "core/proto/nexus_telemetry.proto";
        if std::path::Path::new(workspace).exists() {
            workspace.to_string()
        } else {
            panic!("❌ No se encontró 'nexus_telemetry.proto' en proto/ ni core/proto/");
        }
    };
    tonic_build::compile_protos(&telemetry_path)?;

    Ok(())
}
