use actix_files::NamedFile;
use actix_web::{App, HttpResponse, HttpServer, Result, web};
use rand::seq::SliceRandom;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::SystemTime;

static IMAGE_DIR: LazyLock<String> =
    LazyLock::new(|| env::var("IMAGE_DIR").unwrap_or_else(|_| "/app/images".to_string()));

// Cache for image files
struct ImageCache {
    files: Vec<PathBuf>,
    last_modified: SystemTime,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            last_modified: SystemTime::UNIX_EPOCH,
        }
    }
}

// Get all image files from the directory
fn get_image_files(cache: &Arc<Mutex<ImageCache>>) -> Result<Vec<PathBuf>, std::io::Error> {
    let image_dir = &*IMAGE_DIR;

    // Check directory modification time
    let metadata = fs::metadata(&image_dir)?;
    let dir_modified = metadata.modified()?;

    {
        let cache_guard = cache.lock().unwrap();

        // Return cached files if directory hasn't been modified
        if dir_modified == cache_guard.last_modified && !cache_guard.files.is_empty() {
            return Ok(cache_guard.files.clone());
        }
    }

    // Scan directory for image files
    let mut images = Vec::new();
    let entries = fs::read_dir(&image_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
                ) {
                    images.push(path);
                }
            }
        }
    }

    // Update cache
    let mut cache_guard = cache.lock().unwrap();
    cache_guard.files = images.clone();
    cache_guard.last_modified = dir_modified;

    Ok(images)
}

// Handler for getting a random image
async fn random_image(cache: web::Data<Arc<Mutex<ImageCache>>>) -> Result<HttpResponse> {
    let images = get_image_files(&cache).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to read directory: {}", e))
    })?;

    if images.is_empty() {
        return Err(actix_web::error::ErrorNotFound(format!(
            "No images found in {} directory",
            &*IMAGE_DIR
        )));
    }

    let mut rng = rand::thread_rng();
    let random_image = images.choose(&mut rng).ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Failed to select random image")
    })?;

    println!("Serving random image: {:?}", random_image);

    // Read the file
    let image_data = fs::read(random_image).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to read image: {}", e))
    })?;

    // Determine content type from file extension
    let content_type = random_image
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            _ => "application/octet-stream",
        })
        .unwrap_or("application/octet-stream");

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .insert_header(("Pragma", "no-cache"))
        .insert_header(("Expires", "0"))
        .body(image_data))
}

// Health check endpoint
async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

// Serve favicon.ico from IMAGE_DIR
async fn favicon() -> Result<NamedFile> {
    let image_dir = &*IMAGE_DIR;
    let favicon_path = PathBuf::from(image_dir).join("favicon.ico");

    Ok(NamedFile::open(favicon_path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let image_dir = &*IMAGE_DIR;

    println!("Starting random image server...");
    println!("Serving images from: {}", image_dir);

    // Check if directory exists
    match fs::metadata(&image_dir) {
        Ok(metadata) if metadata.is_dir() => {
            println!("✓ Directory exists");
        }
        Ok(_) => {
            eprintln!("⚠ Warning: {} exists but is not a directory", image_dir);
        }
        Err(e) => {
            eprintln!("⚠ Warning: Cannot access {}: {}", image_dir, e);
        }
    }

    // Initialize the image cache
    let cache = Arc::new(Mutex::new(ImageCache::new()));

    let bind_addr = "0.0.0.0:8080";
    println!("Server running at http://{}", bind_addr);
    println!("Endpoints:");
    println!("  GET /random - Get a random image");
    println!("  GET /health - Health check");
    println!("  GET /favicon.ico - Empty favicon");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .route("/", web::get().to(random_image))
            .route("/health", web::get().to(health))
            .route("/favicon.ico", web::get().to(favicon))
    })
    .bind(bind_addr)?
    .run()
    .await
}
