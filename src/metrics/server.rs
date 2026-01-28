//! HTTP server for Prometheus metrics endpoint.

use crate::metrics::MetricsRegistry;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur during metrics server operations.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("failed to bind to address: {0}")]
    Bind(#[from] std::io::Error),

    #[error("server error: {0}")]
    Server(String),
}

/// Configuration for the metrics server.
#[derive(Debug, Clone)]
pub struct MetricsServerConfig {
    /// Address to bind the server to.
    pub bind_addr: SocketAddr,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: ([0, 0, 0, 0], 9090).into(),
        }
    }
}

impl MetricsServerConfig {
    /// Creates a config with a custom port.
    pub fn with_port(port: u16) -> Self {
        Self {
            bind_addr: ([0, 0, 0, 0], port).into(),
        }
    }
}

/// Shared state for the metrics server.
pub struct MetricsState {
    registry: MetricsRegistry,
}

/// HTTP server for exposing Prometheus metrics.
pub struct MetricsServer {
    config: MetricsServerConfig,
    state: Arc<RwLock<MetricsState>>,
}

impl MetricsServer {
    /// Creates a new metrics server.
    pub fn new(
        config: MetricsServerConfig,
        registry: MetricsRegistry,
    ) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(MetricsState { registry })),
        }
    }

    /// Returns a reference to the shared state for updating metrics.
    pub fn state(&self) -> Arc<RwLock<MetricsState>> {
        Arc::clone(&self.state)
    }

    /// Starts the HTTP server.
    ///
    /// This method runs the server until it is shut down.
    pub async fn run(self) -> Result<(), ServerError> {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .with_state(self.state);

        let listener = tokio::net::TcpListener::bind(self.config.bind_addr).await?;

        tracing::info!(
            addr = %self.config.bind_addr,
            "Metrics server listening"
        );

        axum::serve(listener, app)
            .await
            .map_err(|e| ServerError::Server(e.to_string()))?;

        Ok(())
    }
}

impl MetricsState {
    /// Updates the metrics from a snapshot.
    pub fn update(&self, snapshot: &super::MetricsSnapshot) {
        self.registry.update(snapshot);
    }
}

/// Handler for the /metrics endpoint.
async fn metrics_handler(
    State(state): State<Arc<RwLock<MetricsState>>>,
) -> impl IntoResponse {
    let state = state.read().await;

    match state.registry.encode() {
        Ok(output) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
            output,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "text/plain; charset=utf-8")],
            format!("Failed to encode metrics: {}", e),
        ),
    }
}

/// Handler for the /health endpoint.
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MetricsServerConfig::default();
        assert_eq!(config.bind_addr.port(), 9090);
    }

    #[test]
    fn test_config_with_port() {
        let config = MetricsServerConfig::with_port(8080);
        assert_eq!(config.bind_addr.port(), 8080);
    }
}
