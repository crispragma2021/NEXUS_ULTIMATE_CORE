fn main() {
    // Configuración específica para eBPF
    println!("cargo:rustc-cfg=ebpf");
    println!("cargo:rustc-env=PROFILE=ebpf");
}
