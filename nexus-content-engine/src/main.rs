use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Command;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{info, error};
use base64::Engine;

#[derive(Deserialize)]
struct VideoRequest {
    prompt: String,
    tipo: String,
    color: Option<String>,
    image_path: Option<String>,
    audio_path: Option<String>,
    duration: Option<u32>,
}

#[derive(Serialize)]
struct VideoResponse {
    status: String,
    message: String,
    video_url: Option<String>,
}

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
}

#[derive(Serialize)]
struct DownloadResponse {
    status: String,
    message: String,
    media_url: Option<String>,
}

#[derive(Deserialize)]
struct PreviewRequest {
    url: String,
}

#[derive(Serialize)]
struct PreviewResponse {
    status: String,
    preview_url: Option<String>,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SearchResultItem {
    id: Option<String>,
    title: Option<String>,
    channel: Option<String>,
    duration_string: Option<String>,
    webpage_url: Option<String>,
}

#[derive(Serialize)]
struct SearchResponse {
    status: String,
    results: Vec<SearchResultItem>,
}

#[derive(Deserialize)]
struct GenerateImageRequest {
    prompt: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct GenerateImageResponse {
    status: String,
    image_url: Option<String>,
    message: String,
}

#[derive(Deserialize)]
struct UploadMediaRequest {
    file_name: String,
    base64_data: String,
}

#[derive(Serialize)]
struct UploadMediaResponse {
    status: String,
    media_url: Option<String>,
    message: String,
}

#[derive(Deserialize)]
struct SearchImagesRequest {
    query: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ImageSearchResultItem {
    id: String,
    title: String,
    thumb_url: String,
    full_url: String,
    author: String,
    source: String,
}

#[derive(Serialize)]
struct SearchImagesResponse {
    status: String,
    results: Vec<ImageSearchResultItem>,
}

#[derive(Deserialize)]
struct ImportImageUrlRequest {
    url: String,
}

#[derive(Serialize)]
struct ImportImageUrlResponse {
    status: String,
    media_url: Option<String>,
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Iniciando NEXUS Content Engine...");

    std::fs::create_dir_all("workspace")?;

    let app = Router::new()
        .route("/api/health", get(|| async { "NEXUS Content Engine Operativo" }))
        .route("/api/generate", post(generate_video))
        .route("/api/generate-image", post(generate_ai_image))
        .route("/api/download", post(download_media))
        .route("/api/preview", post(get_preview_stream))
        .route("/api/search", post(search_tracks))
        .route("/api/search-images", post(search_images))
        .route("/api/import-image-url", post(import_image_url))
        .route("/api/upload-media", post(upload_media))
        .nest_service("/media", ServeDir::new("workspace"))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 43211));
    info!("🎧 Motor escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn upload_media(Json(payload): Json<UploadMediaRequest>) -> Json<UploadMediaResponse> {
    let clean_base64 = if let Some(pos) = payload.base64_data.find(',') {
        &payload.base64_data[pos + 1..]
    } else {
        &payload.base64_data
    };

    let timestamp = chrono::Utc::now().timestamp();
    let file_ext = std::path::Path::new(&payload.file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin");

    let saved_filename = format!("upload_{}_{}.{}", timestamp, rand::random::<u16>(), file_ext);
    let file_path = format!("workspace/{}", saved_filename);
    let media_url = format!("http://localhost:43211/media/{}", saved_filename);

    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(clean_base64) {
        if let Ok(_) = std::fs::write(&file_path, bytes) {
            info!("📁 Archivo subido y guardado en workspace: {}", media_url);
            return Json(UploadMediaResponse {
                status: "success".to_string(),
                media_url: Some(media_url),
                message: "Archivo subido correctamente".to_string(),
            });
        }
    }

    Json(UploadMediaResponse {
        status: "error".to_string(),
        media_url: None,
        message: "Error al procesar el archivo subido".to_string(),
    })
}

async fn generate_ai_image(Json(payload): Json<GenerateImageRequest>) -> Json<GenerateImageResponse> {
    info!("🎨 Generando imagen IA con prompt: '{}'", payload.prompt);
    let timestamp = chrono::Utc::now().timestamp();
    let file_path = format!("workspace/bg_{}.jpg", timestamp);
    let media_url = format!("http://localhost:43211/media/bg_{}.jpg", timestamp);

    let encoded_prompt = urlencoding::encode(&payload.prompt);
    let seed = rand::random::<u32>();
    let pollinations_url = format!(
        "https://image.pollinations.ai/prompt/{}?width={}&height={}&seed={}&nologo=true",
        encoded_prompt, payload.width, payload.height, seed
    );

    let res = tokio::task::spawn_blocking(move || {
        Command::new("curl")
            .args(&["-s", "-L", "-o", &file_path, &pollinations_url])
            .output()
    }).await;

    if let Ok(Ok(output)) = res {
        if output.status.success() {
            info!("✅ Imagen de fondo IA descargada y guardada localmente: {}", media_url);
            return Json(GenerateImageResponse {
                status: "success".to_string(),
                image_url: Some(media_url),
                message: "Imagen de fondo generada exitosamente".to_string(),
            });
        }
    }

    Json(GenerateImageResponse {
        status: "error".to_string(),
        image_url: None,
        message: "Falló la descarga del motor de IA".to_string(),
    })
}

async fn get_preview_stream(Json(payload): Json<PreviewRequest>) -> Json<PreviewResponse> {
    info!("🎧 Solicitando stream URL para previsualización: {}", payload.url);
    let ytdlp_bin = "/home/nexus/.local/bin/yt-dlp";

    let res = tokio::task::spawn_blocking(move || {
        Command::new(ytdlp_bin)
            .args(&["-g", "-f", "ba/b", &payload.url])
            .output()
    }).await;

    if let Ok(Ok(output)) = res {
        if output.status.success() {
            let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !stream_url.is_empty() {
                return Json(PreviewResponse {
                    status: "success".to_string(),
                    preview_url: Some(stream_url),
                });
            }
        }
    }

    Json(PreviewResponse {
        status: "error".to_string(),
        preview_url: None,
    })
}

async fn search_tracks(Json(payload): Json<SearchRequest>) -> Json<SearchResponse> {
    info!("🔍 Buscando canciones para la consulta: {}", payload.query);
    let ytdlp_bin = "/home/nexus/.local/bin/yt-dlp";
    let search_arg = format!("ytsearch6:{}", payload.query);

    let res = tokio::task::spawn_blocking(move || {
        Command::new(ytdlp_bin)
            .args(&["--flat-playlist", "-j", &search_arg])
            .output()
    }).await;

    let mut items = Vec::new();

    if let Ok(Ok(output)) = res {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(item) = serde_json::from_str::<SearchResultItem>(line) {
                    items.push(item);
                }
            }
            return Json(SearchResponse {
                status: "success".to_string(),
                results: items,
            });
        }
    }

    Json(SearchResponse {
        status: "error".to_string(),
        results: vec![],
    })
}

async fn download_media(Json(payload): Json<DownloadRequest>) -> Json<DownloadResponse> {
    info!("📥 Orden de descarga recibida para URL: {}", payload.url);
    
    let timestamp = chrono::Utc::now().timestamp();
    let filename_base = format!("workspace/media_{}", timestamp);
    let ytdlp_bin = "/home/nexus/.local/bin/yt-dlp";

    let args = vec![
        "-x".to_string(),
        "--audio-format".to_string(), "mp3".to_string(),
        "-o".to_string(), format!("{}.%(ext)s", filename_base),
        payload.url.clone()
    ];

    let output_file = format!("workspace/media_{}.mp3", timestamp);
    let media_url = format!("http://localhost:43211/media/media_{}.mp3", timestamp);

    let res = tokio::task::spawn_blocking(move || {
        Command::new(ytdlp_bin)
            .args(&args)
            .output()
    }).await;

    match res {
        Ok(Ok(output)) => {
            if output.status.success() {
                info!("✅ Audio descargado con éxito: {}", output_file);
                Json(DownloadResponse {
                    status: "success".to_string(),
                    message: "Media descargado exitosamente vía yt-dlp".to_string(),
                    media_url: Some(media_url),
                })
            } else {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                error!("❌ Error en yt-dlp: {}", err_msg);
                Json(DownloadResponse {
                    status: "error".to_string(),
                    message: format!("Error en yt-dlp: {}", err_msg),
                    media_url: None,
                })
            }
        },
        _ => {
            error!("❌ Falló la ejecución del proceso de descarga");
            Json(DownloadResponse {
                status: "error".to_string(),
                message: "Error ejecutando el proceso interno de descarga".to_string(),
                media_url: None,
            })
        }
    }
}

async fn generate_video(Json(payload): Json<VideoRequest>) -> Json<VideoResponse> {
    info!("🎬 Nueva orden recibida: Generar video tipo '{}' basado en: {}", payload.tipo, payload.prompt);
    
    // Resolve paths if media URLs were passed
    let img_path = if let Some(ref url) = payload.image_path {
        if url.contains("/media/") {
            let filename = url.split("/media/").last().unwrap_or("dummy.jpg");
            format!("workspace/{}", filename)
        } else {
            if let Ok(entries) = std::fs::read_dir("workspace") {
                let mut found = "workspace/dummy.jpg".to_string();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "jpg" || ext == "png" {
                            found = path.to_string_lossy().to_string();
                            break;
                        }
                    }
                }
                found
            } else {
                "workspace/dummy.jpg".to_string()
            }
        }
    } else {
        "workspace/dummy.jpg".to_string()
    };

    let audio_path = if let Some(ref url) = payload.audio_path {
        if url.contains("/media/") {
            let filename = url.split("/media/").last().unwrap_or("dummy.mp3");
            format!("workspace/{}", filename)
        } else {
            if let Ok(entries) = std::fs::read_dir("workspace") {
                let mut found = "workspace/dummy.mp3".to_string();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "mp3" || ext == "m4a" || ext == "wav" {
                            found = path.to_string_lossy().to_string();
                            break;
                        }
                    }
                }
                found
            } else {
                "workspace/dummy.mp3".to_string()
            }
        }
    } else {
        "workspace/dummy.mp3".to_string()
    };

    let timestamp = chrono::Utc::now().timestamp();
    let output_filename = format!("output_{}.mp4", timestamp);
    let output_path = format!("workspace/{}", output_filename);
    let video_url = format!("http://localhost:43211/media/{}", output_filename);

    let wave_color = payload.color.unwrap_or_else(|| "cyan".to_string());
    
    let ffmpeg_color = match wave_color.as_str() {
        "#00ffff" => "cyan",
        "#a855f7" => "purple",
        "#f43f5e" => "red",
        "#10b981" => "green",
        "#ffffff" => "white",
        other => "cyan",
    };

    let (res_w, res_h, wave_w, wave_h) = match payload.tipo.to_lowercase().as_str() {
        "tiktok" | "reels" | "shorts" => (1080, 1920, 1080, 320),
        "instagram" => (1080, 1080, 1080, 260),
        _ => (1920, 1080, 1920, 300),
    };

    let filter_arg = format!(
        "[0:v]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}[bg];[1:a]showwaves=s={}x{}:mode=cline:colors={}[wave];[bg][wave]overlay=0:H-h[outv]",
        res_w, res_h, res_w, res_h, wave_w, wave_h, ffmpeg_color
    );

    let mut ffmpeg_args: Vec<String> = vec![
        "-y".to_string(),
        "-loop".to_string(), "1".to_string(),
        "-i".to_string(), img_path,
        "-i".to_string(), audio_path,
        "-filter_complex".to_string(), filter_arg,
        "-map".to_string(), "[outv]".to_string(),
        "-map".to_string(), "1:a".to_string(),
        "-c:v".to_string(), "libx264".to_string(),
        "-c:a".to_string(), "aac".to_string(),
        "-pix_fmt".to_string(), "yuv420p".to_string(),
    ];

    if let Some(dur) = payload.duration {
        if dur > 0 {
            ffmpeg_args.push("-t".to_string());
            ffmpeg_args.push(dur.to_string());
        } else {
            ffmpeg_args.push("-shortest".to_string());
        }
    } else {
        ffmpeg_args.push("-shortest".to_string());
    }

    ffmpeg_args.push(output_path.clone());

    let output_path_clone = output_path.clone();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .output();
            
        match output {
            Ok(o) => {
                if o.status.success() {
                    info!("✅ Video renderizado exitosamente: {}", output_path_clone);
                    let downloads_dir = "/home/nexus/Downloads";
                    let _ = std::fs::create_dir_all(downloads_dir);
                    let dest = format!("{}/{}", downloads_dir, output_filename);
                    if let Err(e) = std::fs::copy(&output_path_clone, &dest) {
                        error!("⚠️ No se pudo copiar a Downloads: {}", e);
                    } else {
                        info!("📁 Video copiado automáticamente a {}", dest);
                    }
                } else {
                    error!("❌ Error en ffmpeg: {}", String::from_utf8_lossy(&o.stderr));
                }
            },
            Err(e) => error!("❌ Falló al ejecutar ffmpeg: {}", e),
        }
    });

    Json(VideoResponse {
        status: "processing".to_string(),
        message: format!("Generación de video enviada al motor FFMPEG con color: {}. Plataforma: {}", wave_color, payload.tipo),
        video_url: Some(video_url),
    })
}

async fn search_images(Json(payload): Json<SearchImagesRequest>) -> Json<SearchImagesResponse> {
    info!("🖼️ Buscando imágenes libres de derechos para: '{}'", payload.query);
    let encoded_query = urlencoding::encode(&payload.query);
    let openverse_url = format!("https://api.openverse.org/v1/images/?q={}&page_size=12", encoded_query);

    let res = tokio::task::spawn_blocking(move || {
        Command::new("curl")
            .args(&["-s", "-H", "User-Agent: Mozilla/5.0", &openverse_url])
            .output()
    }).await;

    let mut items = Vec::new();

    if let Ok(Ok(output)) = res {
        if output.status.success() {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(results) = v.get("results").and_then(|r| r.as_array()) {
                    for (idx, item) in results.iter().enumerate() {
                        let id = item.get("id").and_then(|s| s.as_str()).unwrap_or(&format!("img_{}", idx)).to_string();
                        let title = item.get("title").and_then(|s| s.as_str()).unwrap_or("Stock Image HD").to_string();
                        let thumb_url = item.get("thumbnail").and_then(|s| s.as_str())
                            .or_else(|| item.get("url").and_then(|s| s.as_str()))
                            .unwrap_or("")
                            .to_string();
                        let full_url = item.get("url").and_then(|s| s.as_str()).unwrap_or("").to_string();
                        let creator = item.get("creator").and_then(|s| s.as_str()).unwrap_or("Creative Commons Artist").to_string();
                        let license = item.get("license").and_then(|s| s.as_str()).unwrap_or("CC Free").to_string();

                        if !thumb_url.is_empty() && !full_url.is_empty() {
                            items.push(ImageSearchResultItem {
                                id,
                                title,
                                thumb_url,
                                full_url,
                                author: creator,
                                source: format!("Openverse / {} ({})", item.get("provider").and_then(|s| s.as_str()).unwrap_or("CC"), license.to_uppercase()),
                            });
                        }
                    }
                }
            }
        }
    }

    if items.is_empty() {
        let fallback_terms = vec!["cyberpunk", "landscape", "neon", "nature", "galaxy", "abstract"];
        for (i, term) in fallback_terms.iter().enumerate() {
            items.push(ImageSearchResultItem {
                id: format!("unsplash_fb_{}", i),
                title: format!("HD Stock {}", term),
                thumb_url: format!("https://picsum.photos/seed/{}/400/225", term),
                full_url: format!("https://picsum.photos/seed/{}/1920/1080", term),
                author: "Unsplash / CC0".to_string(),
                source: "Unsplash CC0 HD".to_string(),
            });
        }
    }

    Json(SearchImagesResponse {
        status: "success".to_string(),
        results: items,
    })
}

async fn import_image_url(Json(payload): Json<ImportImageUrlRequest>) -> Json<ImportImageUrlResponse> {
    info!("📥 Importando imagen Stock desde URL: {}", payload.url);
    let timestamp = chrono::Utc::now().timestamp();
    let file_path = format!("workspace/stock_{}.jpg", timestamp);
    let media_url = format!("http://localhost:43211/media/stock_{}.jpg", timestamp);

    let url_clone = payload.url.clone();
    let res = tokio::task::spawn_blocking(move || {
        Command::new("curl")
            .args(&["-s", "-L", "-o", &file_path, &url_clone])
            .output()
    }).await;

    if let Ok(Ok(output)) = res {
        if output.status.success() {
            info!("✅ Imagen stock descargada y guardada en workspace: {}", media_url);
            return Json(ImportImageUrlResponse {
                status: "success".to_string(),
                media_url: Some(media_url),
                message: "Imagen importada correctamente".to_string(),
            });
        }
    }

    Json(ImportImageUrlResponse {
        status: "error".to_string(),
        media_url: None,
        message: "Falló la importación de la imagen stock".to_string(),
    })
}
