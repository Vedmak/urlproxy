use axum::body::StreamBody;
use axum::extract::Path;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use clap::Parser;
use tower_http::trace::TraceLayer;

#[derive(Parser)]
#[command()]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:8000")]
    listen: String,
    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.debug {
        std::env::set_var("RUST_LOG", "info,tower_http=debug");
    }

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/:encoded_url", get(get_content))
        .layer(TraceLayer::new_for_http().on_request(()));

    tracing::info!("listening on {}", cli.listen);
    axum::Server::bind(&cli.listen.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_content(Path(encoded_url): Path<String>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Ok(decoded_bytes) = base64::decode_config(encoded_url, base64::URL_SAFE) else {
        return Err((StatusCode::BAD_REQUEST, "Failed to decode base64 url param".to_string()));
    };

    let Ok(url) = String::from_utf8(decoded_bytes) else {
        return Err((StatusCode::BAD_REQUEST, "Failed to convert bytes to utf8 string".to_string()));
    };

    let Ok(url_response) = reqwest::get(url).await else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Request url failed".to_string()));
    };

    if !url_response.status().is_success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Request url status code is not success".to_string(),
        ));
    }

    let mut headers = HeaderMap::new();
    if let Some(content_type) = url_response.headers().get(header::CONTENT_TYPE) {
        headers.insert(header::CONTENT_TYPE, content_type.to_owned());
    }

    let body = StreamBody::new(url_response.bytes_stream());

    Ok((headers, body))
}
