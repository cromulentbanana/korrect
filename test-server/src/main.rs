use actix_web::{get, web, App, HttpResponse, HttpServer, Result};
use clap::Parser;
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing kubectl binaries
    #[arg(short, long, default_value_os_t = env::current_dir().unwrap_or_default())]
    directory: PathBuf,
}

#[get("/release/stable.txt")]
async fn stable_version() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("v1.31.3"))
}

#[get("/release/{version}/bin/{_x}/{_y}/kubectl")]
async fn serve_kubectl(
    path: web::Path<(String, String, String)>,
    data: web::Data<Args>,
) -> Result<HttpResponse> {
    let (version, _, _) = path.into_inner();
    let file_name = format!("kubectl-{}", version);
    let file_path = data.directory.join(&file_name);

    if !file_path.exists() {
        return Ok(HttpResponse::NotFound().body(format!("File {} not found", file_name)));
    }

    match std::fs::read(&file_path) {
        Ok(contents) => Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"kubectl\""),
            ))
            .body(contents)),
        Err(e) => {
            Ok(HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Verify directory exists and is actually a directory
    if !args.directory.exists() || !args.directory.is_dir() {
        eprintln!(
            "Error: {} is not a valid directory",
            args.directory.display()
        );
        std::process::exit(1);
    }

    println!(
        "Starting server with directory: {}",
        args.directory.display()
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(args.clone()))
            .service(serve_kubectl)
            .service(stable_version)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
