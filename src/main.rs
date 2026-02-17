mod config;
mod device;
mod frame;
mod imgops;
mod solid;
mod templates;

use axum::{self, RequestExt};
use axum::body::{Body, Bytes};
use axum::extract::{Json, OptionalFromRequestParts, Path, Request, State};
use axum::http;
use axum::response::{IntoResponse, Response};
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter};

use std::path::PathBuf;
use std::time::Duration;

const FRAME_COLS: u8 = 30;
const FRAME_ROWS: u8 = 32;
const FRAME_DIMS: (u8, u8) = (FRAME_COLS, FRAME_ROWS);
const MILLIS_PER_FRAME: u64 = 30;

enum FramesCmd {
    Empty,
    Loop(frame::Frames),
    Transition(frame::Frames),
}

#[derive(Clone)]
struct AppState {
    frames_tx: tokio::sync::mpsc::Sender<FramesCmd>,
    templates: PathBuf,
}

fn respond_binary(payload: Vec<u8>) -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [
            (http::header::CONTENT_TYPE, "application/octet-stream"),
        ],
        payload
    )
}

fn respond_error(status_code: http::StatusCode, message: String) -> impl IntoResponse {
    (
        status_code,
        [
            (http::header::CONTENT_TYPE, "text/plain"),
        ],
        message
    )
}

fn respond_html(content: String) -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [
            (http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        ],
        content
    )
}

fn respond_ok() -> impl IntoResponse {
    http::StatusCode::OK
}

fn respond_json(json: String) -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [
            (http::header::CONTENT_TYPE, "application/json")
        ],
        json
    )
}

fn serve_html_file(path: &str) -> Response<Body> {
    match std::fs::read_to_string(path) {
        Ok(content) => respond_html(content).into_response(),
        Err(_) => respond_error(http::StatusCode::NOT_FOUND, String::new()).into_response(),
    }
}

async fn route_index() -> Response<Body> {
    serve_html_file("web/index.html")
}

async fn route_resample(request: Request) -> Response<Body> {
    let body = match request.extract::<Bytes, _>().await {
        Ok(body) => body,
        Err(e) => return respond_error(http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    match imgops::resample_image(FRAME_DIMS, &body) {
        Ok(resampled_image) => respond_binary(resampled_image).into_response(),
        Err(e) => respond_error(http::StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn route_upload_image(
    State(state): State<AppState>,
    request: Request
) -> Response<Body> {
    let body = match request.extract::<Bytes, _>().await {
        Ok(body) => body,
        Err(e) => return respond_error(http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let frames = match frame::frames_from_image(FRAME_DIMS, &body) {
        Ok(frames) => frames,
        Err(e) => return respond_error(http::StatusCode::BAD_REQUEST, e).into_response(),
    };

    match state.frames_tx.send(FramesCmd::Loop(frames)).await {
        Ok(_) => respond_ok().into_response(),
        Err(e) => {
            error!("Failed to push image to device queue: {}", e);
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to push image to device queue")).into_response()
        },
    }
}

async fn route_solid_color() -> Response<Body> {
    serve_html_file("web/solid-color.html")
}

async fn route_solid_color_instant(
    State(state): State<AppState>,
    Path((r, g, b)): Path<(u8, u8, u8)>
) -> Response<Body> {
    let frame = solid::make_frame(FRAME_DIMS, r, g, b);

    match state.frames_tx.send(FramesCmd::Transition(vec![frame])).await {
        Ok(_) => respond_ok().into_response(),
        Err(e) => {
            error!("Failed to push frames to device queue: {}", e);
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to push image to device queue")).into_response()
        },
    }
}

async fn route_solid_color_smooth(
    State(state): State<AppState>,
    Json(transitions): Json<solid::SmoothSolidColor>
) -> Response<Body> {
    let mut frames = Vec::new();

    for transition in transitions {
        let [r, g, b] = transition;
        let frame = solid::make_frame(FRAME_DIMS, r, g, b);
        frames.push(frame);
    }

    match state.frames_tx.send(FramesCmd::Transition(frames)).await {
        Ok(()) => respond_ok().into_response(),
        Err(e) => {
            error!("Failed to push frames to device queue: {}", e);
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to push frames to device queue: {}", e)).into_response()
        }
    }
}

async fn route_template() -> Response<Body> {
    serve_html_file("web/template.html")
}

async fn route_template_delete(
    State(state): State<AppState>,
    Path(template_name): Path<String>
) -> Response<Body> {
    if template_name.is_empty() {
        return respond_error(http::StatusCode::BAD_REQUEST, "Template name cannot be empty".to_string()).into_response();
    }

    match templates::delete_template(&state.templates, template_name) {
        Ok(()) => respond_ok().into_response(),
        Err(e) => {
            error!("{}", e);
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        }
    }
}

async fn route_template_list(
    State(state): State<AppState>
) -> Response<Body> {
    match templates::list_templates(&state.templates) {
        Ok(templates) => {
            let json = serde_json::to_string(&templates).unwrap();
            respond_json(json).into_response()
        },
        Err(e) => {
            error!("Failed to retrieve list of templates: {}", e);
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        },
    }

}

async fn route_template_load(
    State(state): State<AppState>,
    Path(template_name): Path<String>
) -> Response<Body> {
    if template_name.is_empty() {
        return respond_error(http::StatusCode::BAD_REQUEST, "Template name cannot be empty".to_string()).into_response();
    }

    match templates::read_template(&state.templates, template_name) {
        Ok(image_data) => {
            respond_binary(image_data).into_response()
        },
        Err(e) => {
            error!("Failed to load template: {}", e);
            respond_error(http::StatusCode::BAD_REQUEST, format!("Failed to load template: {}", e)).into_response()
        },
    }
}


async fn route_template_save(
    State(state): State<AppState>,
    request: Request,
) -> Response<Body> {
    let (mut parts, body) = request.into_parts();
    let template_name = match Path::<String>::from_request_parts(&mut parts, &state).await {
        Ok(x) => match x {
            Some(path) => path,
            None => return respond_error(http::StatusCode::BAD_REQUEST, "Invalid URL".to_string()).into_response(),
        },
        Err(_) => return respond_error(http::StatusCode::BAD_REQUEST, "Invalid URL".to_string()).into_response(),
    };
    if template_name.is_empty() {
        return respond_error(http::StatusCode::BAD_REQUEST, String::from("Invalid template name")).into_response();
    }

    let orig_image = match axum::body::to_bytes(body, 32 * 1024 * 1024).await {
        Ok(body) => body,
        Err(e) => return respond_error(http::StatusCode::BAD_REQUEST, format!("Invalid request body: {}", e)).into_response(),
    };

    let resampled_image = match imgops::resample_image(FRAME_DIMS, &orig_image) {
        Ok(resampled_image) => resampled_image,
        Err(e) => {
            return respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to resample image: {}", e)).into_response();
        },
    };

    match templates::write_template(&state.templates, template_name.clone(), &resampled_image) {
        Ok(()) => respond_ok().into_response(),
        Err(e) => {
            respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        }
    }
}

async fn route_template_upload(
    State(state): State<AppState>,
    Path(template_name): Path<String>
) -> Response<Body> {
    if template_name.is_empty() {
        return respond_error(http::StatusCode::BAD_REQUEST, "Template name cannot be empty".to_string()).into_response();
    }

    match templates::read_template(&state.templates, template_name) {
        Ok(template_bytes) => {
            match frame::frames_from_image(FRAME_DIMS, &template_bytes) {
                Ok(frames) => {
                    match state.frames_tx.send(FramesCmd::Loop(frames)).await {
                        Ok(_) => respond_ok().into_response(),
                        Err(e) => respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to push frames to device queue: {}", e)).into_response()
                    }
                },
                Err(e) => {
                    respond_error(http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to upload template to device: {}", e)).into_response()
                },
            }
        },
        Err(e) => {
            respond_error(http::StatusCode::BAD_REQUEST, format!("Failed to load template: {}", e)).into_response()
        },
    }
}

fn init_tracing() {
    fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    let cfg = match config::read_config(&PathBuf::from("./config.json")) {
        Ok(cfg) => cfg,
        Err(e) => {
            panic!("Startup error: {}", e);
        },
    };

    let templates = PathBuf::from(cfg.templates.clone());
    if !templates.is_dir() {
        panic!("Templates directory {} does not exist", cfg.templates);
    }

    let mut device = match cfg.device {
        Some(device) => match device::open_device(device.as_str()) {
            Ok(device) => Some(device),
            Err(e) => panic!("Failed to open device: {}", e),
        },
        None => None,
    };

    let listener = tokio::net::TcpListener::bind(cfg.host)
        .await
        .expect("Could not bind listening socket");

    let (frames_tx, mut frames_rx) = tokio::sync::mpsc::channel(16);
    tokio::spawn(async move {
        let mut cmd = FramesCmd::Empty;
        let mut frame_idx = 0;

        loop {
            let timeout = match &cmd{
                FramesCmd::Empty => Duration::from_secs(10_000),
                FramesCmd::Loop(frames) => {
                    if frames.len() > 1 || (frames.len() == 1 && frame_idx == 0) {
                        Duration::from_millis(MILLIS_PER_FRAME)
                    } else {
                        Duration::from_secs(10_000)
                    }
                },
                FramesCmd::Transition(frames) => {
                    if frame_idx == frames.len() {
                        Duration::from_secs(10_000)
                    } else {
                        Duration::from_millis(MILLIS_PER_FRAME)
                    }
                },
            };

            tokio::select! {
                result = tokio::time::timeout(timeout, frames_rx.recv()) => {
                    match result {
                        Ok(maybe_new_cmd) => {
                            if let Some(new_cmd) = maybe_new_cmd {
                                cmd = new_cmd;
                                frame_idx = 0;
                            }
                        },
                        Err(_) => {
                            if let Some(device) = &mut device {
                                match &cmd {
                                    FramesCmd::Loop(frames) => {
                                        let frame = &frames[frame_idx];

                                        match device::upload_frame(device, frame) {
                                            Ok(()) => {
                                                frame_idx = (frame_idx + 1) % frames.len();
                                            },
                                            Err(e) => {
                                                error!("Failed to upload frame to device: {}", e);

                                                cmd = FramesCmd::Empty;
                                            }
                                        }
                                    },
                                    FramesCmd::Transition(frames) => {
                                        if frame_idx < frames.len() {
                                            let frame = &frames[frame_idx];

                                            match device::upload_frame(device, frame) {
                                                Ok(()) => {
                                                    frame_idx += 1;
                                                },
                                                Err(e) => {
                                                    error!("Failed to upload frame to device: {}", e);

                                                    cmd = FramesCmd::Empty;
                                                }
                                            }
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let app_state = AppState{
        frames_tx,
        templates,
    };

    let app = axum::Router::new()
        .route("/", axum::routing::get(route_index))
        .route("/resample-image", axum::routing::post(route_resample))
        .route("/upload-image", axum::routing::post(route_upload_image))
        .route("/solid-color", axum::routing::get(route_solid_color))
        .route("/solid-color/instant/{r}/{g}/{b}", axum::routing::post(route_solid_color_instant))
        .route("/solid-color/smooth", axum::routing::post(route_solid_color_smooth))
        .route("/template", axum::routing::get(route_template))
        .route("/template/delete/{name}", axum::routing::post(route_template_delete))
        .route("/template/list", axum::routing::get(route_template_list))
        .route("/template/load/{name}", axum::routing::get(route_template_load))
        .route("/template/save/{name}", axum::routing::post(route_template_save))
        .route("/template/upload/{name}", axum::routing::post(route_template_upload))
        .nest_service("/static", tower_http::services::ServeDir::new("web/static"))
        .layer(axum::extract::DefaultBodyLimit::max(32 * 1024 * 1024))
        .layer(TraceLayer::new_for_http()
            .make_span_with(tower_http::trace::DefaultMakeSpan::new()
                .level(tracing::Level::INFO))
            .on_response(
                tower_http::trace::DefaultOnResponse::new()
                .level(tracing::Level::INFO),
            )
        )
        .with_state(app_state);

    info!("Server starting");

    axum::serve(listener, app).await.unwrap();
}
