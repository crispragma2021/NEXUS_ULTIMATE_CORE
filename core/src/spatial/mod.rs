use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub tags: Vec<String>,
}

pub struct SpatialEngine {
    pub root: PathBuf,
    pub threads: usize,
}

impl SpatialEngine {
    pub fn new(root: PathBuf) -> Self {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu_all();
        let total_threads = sys.cpus().len();
        let threads = total_threads.saturating_sub(2).max(1);

        Self {
            root,
            threads, // Optimización dinámica de CPU (reserva 2 hilos para el sistema)
        }
    }

    /// Escaneo multi-hilo de alta velocidad (Remplazo WizTree/QDirStat)
    pub async fn full_scan(&self) -> Vec<FileMetadata> {
        println!("🚀 [SPATIAL] Iniciando escaneo ZENITH en: {:?}", self.root);
        let (tx, rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        let results = Arc::new(Mutex::new(Vec::new()));

        // Semilla: Directorio raíz
        tx.send(self.root.clone()).unwrap();

        let mut handles = vec![];

        for _i in 0..self.threads {
            let tx_clone = tx.clone();
            let rx_clone = rx.clone();
            let results_clone = results.clone();

            let handle = thread::spawn(move || {
                while let Ok(path) = rx_clone.recv_timeout(std::time::Duration::from_millis(100)) {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            let metadata = entry.metadata().ok();
                            let path_entry = entry.path();

                            if let Some(m) = metadata {
                                let size = m.len();
                                let is_dir = m.is_dir();

                                {
                                    let mut res = results_clone.lock().unwrap();
                                    res.push(FileMetadata {
                                        path: path_entry.clone(),
                                        size,
                                        is_dir,
                                        tags: Vec::new(),
                                    });
                                }

                                if is_dir {
                                    let _ = tx_clone.send(path_entry);
                                }
                            }
                        }
                    }
                }
                // println!("  [THREAD {}] Escaneo completado", i);
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let final_results = results.lock().unwrap().clone();
        println!(
            "✅ [SPATIAL] Escaneo completado: {} elementos detectados",
            final_results.len()
        );
        final_results
    }

    /// Implementación de Etiquetado Soberano (Consolidación TagSpaces)
    pub fn tag_file(&self, path: &Path, tag: &str) {
        // Por ahora, simulamos persistencia en consola
        // En fase 2, esto irá a PostgreSQL
        println!("🏷️ [TAG] Marcando {:?} como '{}'", path, tag);
    }
}
