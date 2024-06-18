use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    propagate_header::PropagateHeaderLayer,
    set_header::SetRequestHeaderLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::{Level, Span};
use uuid::Uuid;

pub struct Application {}

impl Application {
    pub fn new() -> Application {
        Application {}
    }

    /// Run the application
    ///
    /// * `listener`: A valid tokio TcpListener
    /// * `block`: Whether the application should block the thread it's on or spawn in a tokio task
    pub async fn run(
        &self,
        listener: tokio::net::TcpListener,
        block: bool,
    ) -> Result<std::net::SocketAddr, Box<dyn std::error::Error>> {
        let socket_addr = listener
            .local_addr()
            .map_err(|err| format!("Failed to get socket address, error: {err}"))?;
        let header_x_request_id = HeaderName::from_static("x-request-id");
        let router = Router::new()
            .route("/", get(|| async { "API Online" }))
            .layer(
                ServiceBuilder::new()
                    .layer(SetRequestHeaderLayer::overriding(
                        header_x_request_id.clone(),
                        |_: &Request<Body>| {
                            Some(HeaderValue::from_str(&format!("{}", Uuid::new_v4())).unwrap())
                        },
                    ))
                    .layer(PropagateHeaderLayer::new(header_x_request_id))
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(|request: &Request<Body>| {
                                let request_id = request
                                    .headers()
                                    .get("x-request-id")
                                    .unwrap()
                                    .to_str()
                                    .unwrap();

                                tracing::span!(
                                    tracing::Level::INFO,
                                    "HTTP-Request",
                                    method = %request.method(),
                                    uri = %request.uri(),
                                    version = ?request.version(),
                                    request_id = request_id

                                )
                            })
                            .on_request(|request: &Request<_>, _span: &Span| {
                                tracing::event!(
                                    Level::INFO,
                                    method = %request.method(),
                                    uri = %request.uri(),
                                    version = ?request.version(),
                                )
                            })
                            .on_response(DefaultOnResponse::new()),
                    ),
            );
        tracing::info!(
            "Starting application on {}:{}",
            socket_addr.ip(),
            socket_addr.port()
        );
        if block {
            tracing::info!("Starting in blocking mode");
            axum::serve(listener, router)
                .await
                .expect("Failed to run API!");
        } else {
            tracing::info!("Starting in nonblocking mode");
            tokio::spawn(async move {
                axum::serve(listener, router)
                    .await
                    .expect("Failed to run API!");
            });
        }
        Ok(socket_addr)
    }
}
