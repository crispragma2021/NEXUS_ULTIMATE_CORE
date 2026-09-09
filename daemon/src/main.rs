use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

fn get_socket_path() -> PathBuf {
    if let Ok(custom) = env::var("NEXUS_SOCKET_PATH") {
        return PathBuf::from(custom);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".nexus_host.sock")
}

/// Devuelve true si el directorio existe (o se puede crear) y es escribible.
fn dir_es_usable(path: &Path) -> bool {
    if path.as_os_str().is_empty() {
        return false;
    }
    if let Err(_) = fs::create_dir_all(path) {
        return false;
    }
    path.is_dir()
}

/// Raíz temporal portable, equivalente a `nexo_plataforma.temp_root()` en Python.
///
/// Orden de preferencia: `$NEXUS_TMPDIR` -> `$TMPDIR` -> `$PREFIX/tmp` ->
/// `$HOME/tmp`. En Android/Termux eso resuelve bajo el prefijo de la app; en un
/// PC de escritorio se conserva el comportamiento original.
fn get_temp_root() -> PathBuf {
    let candidatos: Vec<PathBuf> = ["NEXUS_TMPDIR", "TMPDIR"]
        .iter()
        .filter_map(|nombre| env::var(nombre).ok())
        .filter(|valor| !valor.trim().is_empty())
        .map(PathBuf::from)
        .chain(
            env::var("PREFIX")
                .ok()
                .filter(|valor| !valor.trim().is_empty())
                .map(|prefix| PathBuf::from(prefix).join("tmp")),
        )
        .chain(
            env::var("HOME")
                .ok()
                .filter(|valor| !valor.trim().is_empty())
                .map(|home| PathBuf::from(home).join("tmp")),
        )
        .collect();

    for candidato in candidatos {
        if candidato.is_absolute() && dir_es_usable(&candidato) {
            return candidato;
        }
    }

    // Último recurso: el temporal del sistema y, si falla, el directorio actual.
    for ultimo in [env::temp_dir(), env::current_dir().unwrap_or_else(|_| PathBuf::from("."))] {
        if dir_es_usable(&ultimo) {
            return ultimo;
        }
    }
    PathBuf::from(".")
}

/// Ruta donde se guardan las capturas de pantalla.
///
/// Sustituye la ruta fija de almacenamiento compartido que había antes: esa
/// ruta no existe en un PC y en Android puede no tener permisos de escritura.
fn screenshot_path() -> PathBuf {
    if let Ok(custom) = env::var("NEXUS_SCREENSHOT_PATH") {
        return PathBuf::from(custom);
    }
    get_temp_root().join("nexus_shot.png")
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action", content = "payload")]
enum ActionRequest {
    Tap { x: u32, y: u32 },
    Type { text: String },
    Screenshot,
    Exec { command: String },
    GetScreenSize,
}

#[derive(Serialize)]
struct ActionResponse {
    ok: bool,
    result: String,
}

fn get_rish_binary() -> String {
    let termux_rish = "/data/data/com.termux/files/usr/bin/rish";
    if Path::new(termux_rish).exists() {
        return termux_rish.to_string();
    }
    "rish".to_string()
}

fn run_with_shizuku(cmd_str: &str) -> Result<String, String> {
    let rish_bin = get_rish_binary();

    let output = Command::new("sh")
        .env("PATH", "/data/data/com.termux/files/usr/bin:/system/bin:/system/xbin")
        .env("RISH_APPLICATION_ID", "com.termux")
        .env("LD_LIBRARY_PATH", "/data/data/com.termux/files/usr/lib")
        .arg("-c")
        .arg(format!("{} -c '{}'", rish_bin, cmd_str))
        .output();

    match output {
        Ok(out) => {
            let stdout_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let stderr_str = String::from_utf8_lossy(&out.stderr).trim().to_string();

            if out.status.success() {
                if !stdout_str.is_empty() {
                    Ok(stdout_str)
                } else if !stderr_str.is_empty() {
                    Ok(stderr_str)
                } else {
                    Ok("OK".to_string())
                }
            } else {
                let err_detail = if !stderr_str.is_empty() { stderr_str } else { stdout_str };
                Err(format!("Command failed with exit code {:?}: {}", out.status.code(), err_detail))
            }
        }
        Err(e) => Err(format!("Failed to spawn process: {}", e)),
    }
}

fn handle_client(mut stream: UnixStream) {
    let reader = BufReader::new(stream.try_clone().expect("Cannot clone stream"));
    for line in reader.lines() {
        if let Ok(req_str) = line {
            let res = match serde_json::from_str::<ActionRequest>(&req_str) {
                Ok(ActionRequest::Tap { x, y }) => {
                    let cmd = format!("input tap {} {}", x, y);
                    match run_with_shizuku(&cmd) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Type { text }) => {
                    let cmd = format!("input text '{}'", text.replace("'", "'\\''"));
                    match run_with_shizuku(&cmd) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Screenshot) => {
                    let destino = screenshot_path();
                    let destino_str = destino.to_string_lossy().to_string();
                    // Se entrecomilla la ruta por si el temporal contiene espacios.
                    let cmd = format!("screencap -p '{}'", destino_str.replace('\'', "'\\''"));
                    match run_with_shizuku(&cmd) {
                        Ok(_) => ActionResponse { ok: true, result: destino_str },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Exec { command }) => {
                    match run_with_shizuku(&command) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::GetScreenSize) => {
                    match run_with_shizuku("wm size") {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Err(e) => ActionResponse {
                    ok: false,
                    result: format!("JSON parse error: {}", e),
                },
            };

            if let Ok(resp_json) = serde_json::to_string(&res) {
                let _ = writeln!(stream, "{}", resp_json);
            }
        }
    }
}

fn main() {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    println!("Iniciando Nexus Host Daemon en {:?}", socket_path);
    let listener = UnixListener::bind(&socket_path).expect("No se pudo enlazar al socket Unix");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Error en conexión entrante: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Limpia las variables que influyen en la resolución de rutas.
    fn limpiar_env() {
        for nombre in [
            "NEXUS_TMPDIR",
            "TMPDIR",
            "PREFIX",
            "HOME",
            "NEXUS_SCREENSHOT_PATH",
            "NEXUS_SOCKET_PATH",
        ] {
            env::remove_var(nombre);
        }
    }

    fn tempdir_unico(etiqueta: &str) -> PathBuf {
        let base = env::temp_dir().join(format!(
            "nexus_test_{}_{}_{}",
            etiqueta,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&base).expect("no se pudo crear el directorio de prueba");
        base
    }

    #[test]
    fn tmpdir_tiene_prioridad() {
        limpiar_env();
        let base = tempdir_unico("tmpdir");
        env::set_var("TMPDIR", &base);
        assert_eq!(get_temp_root(), base);
    }

    #[test]
    fn override_explicito_gana_siempre() {
        limpiar_env();
        let base = tempdir_unico("override");
        env::set_var("NEXUS_TMPDIR", &base);
        env::set_var("TMPDIR", "/ruta/que/no/debe/usarse");
        assert_eq!(get_temp_root(), base);
    }

    #[test]
    fn cae_a_prefix_tmp_estilo_termux() {
        limpiar_env();
        let prefix = tempdir_unico("prefix");
        env::set_var("PREFIX", &prefix);
        assert_eq!(get_temp_root(), prefix.join("tmp"));
    }

    #[test]
    fn tmpdir_inusable_se_descarta() {
        limpiar_env();
        let prefix = tempdir_unico("inusable");
        env::set_var("TMPDIR", "/proc/ruta/no/escribible/nexus");
        env::set_var("PREFIX", &prefix);
        assert_eq!(get_temp_root(), prefix.join("tmp"));
    }

    #[test]
    fn rutas_relativas_se_descartan() {
        limpiar_env();
        let prefix = tempdir_unico("relativa");
        env::set_var("TMPDIR", "tmp_relativo");
        env::set_var("PREFIX", &prefix);
        assert_eq!(get_temp_root(), prefix.join("tmp"));
    }

    #[test]
    fn temp_root_nunca_devuelve_tmp_duro_en_termux() {
        limpiar_env();
        let prefix = tempdir_unico("noduro");
        env::set_var("PREFIX", &prefix);
        let resuelto = get_temp_root();
        assert!(resuelto.is_absolute());
        // Se construye el literal para no dejar una ruta dura en el fuente.
        let tmp_duro: PathBuf = ["/", "tmp"].iter().collect();
        assert_ne!(resuelto, tmp_duro);
    }

    #[test]
    fn temp_root_siempre_devuelve_directorio_absoluto_util() {
        limpiar_env();
        let resuelto = get_temp_root();
        assert!(resuelto.is_absolute(), "{:?} no es absoluta", resuelto);
        assert!(resuelto.is_dir(), "{:?} no es un directorio", resuelto);
    }

    #[test]
    fn screenshot_va_bajo_la_raiz_temporal() {
        limpiar_env();
        let base = tempdir_unico("shot");
        env::set_var("TMPDIR", &base);
        assert_eq!(screenshot_path(), base.join("nexus_shot.png"));
    }

    #[test]
    fn screenshot_es_sobreescribible() {
        limpiar_env();
        env::set_var("NEXUS_SCREENSHOT_PATH", "/ruta/personal/captura.png");
        assert_eq!(screenshot_path(), PathBuf::from("/ruta/personal/captura.png"));
    }

    #[test]
    fn socket_path_es_sobreescribible() {
        limpiar_env();
        env::set_var("NEXUS_SOCKET_PATH", "/ruta/personal/nexus.sock");
        assert_eq!(get_socket_path(), PathBuf::from("/ruta/personal/nexus.sock"));
    }

    #[test]
    fn no_queda_sdcard_duro_en_el_codigo() {
        let fuente = include_str!("main.rs");
        // Se construye la aguja: si se escribiera literal, este propio test
        // (que include_str! incorpora al fuente) se haria fallar a si mismo.
        let aguja = ["/", "sdcard", "/"].concat();
        let produccion = fuente.split("#[cfg(test)]").next().unwrap_or(fuente);
        assert!(
            !produccion.contains(&aguja),
            "queda una ruta dura de almacenamiento compartido en main.rs"
        );
    }
}
