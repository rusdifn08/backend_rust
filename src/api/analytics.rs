use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{sse::{Event, Sse}, IntoResponse},
    Extension,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Instant};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_requests: u64,
    pub active_concurrency: i32,
    pub avg_latency_ms: f64,
    pub get_count: u64,
    pub post_count: u64,
    pub put_count: u64,
    pub delete_count: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            active_concurrency: 0,
            avg_latency_ms: 0.0,
            get_count: 0,
            post_count: 0,
            put_count: 0,
            delete_count: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub latency_ms: f64,
}

#[derive(Clone, Serialize)]
pub struct AnalyticsPayload {
    pub metrics: Metrics,
    pub log: Option<RequestLog>,
}

#[derive(Clone)]
pub struct AnalyticsState {
    pub metrics: Arc<RwLock<Metrics>>,
    pub tx: broadcast::Sender<AnalyticsPayload>,
}

impl AnalyticsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
            tx,
        }
    }
}

pub async fn analytics_middleware(
    Extension(state): Extension<AnalyticsState>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Exclude analytics stream itself from metrics
    if req.uri().path() == "/api/analytics/stream" {
        return Ok(next.run(req).await);
    }

    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_owned();

    // Increment concurrency
    {
        let mut m = state.metrics.write().await;
        m.active_concurrency += 1;
        
        let payload = AnalyticsPayload { metrics: m.clone(), log: None };
        let _ = state.tx.send(payload);
    }

    // Process request
    let response = next.run(req).await;
    let status_code = response.status().as_u16();

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    // Update metrics on completion
    {
        let mut m = state.metrics.write().await;
        m.active_concurrency -= 1;
        m.total_requests += 1;
        
        match method.as_str() {
            "GET" => m.get_count += 1,
            "POST" => m.post_count += 1,
            "PUT" => m.put_count += 1,
            "DELETE" => m.delete_count += 1,
            _ => {}
        }

        // Running average
        m.avg_latency_ms = m.avg_latency_ms + ((elapsed - m.avg_latency_ms) / m.total_requests as f64);

        let log = RequestLog {
            method: method.to_string(),
            path,
            status_code,
            latency_ms: elapsed,
        };

        let payload = AnalyticsPayload { metrics: m.clone(), log: Some(log) };
        let _ = state.tx.send(payload);
    }

    Ok(response)
}

pub async fn stream_analytics(
    State(state): State<AnalyticsState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    
    // Send initial state immediately
    let initial_metrics = {
        let m = state.metrics.read().await;
        m.clone()
    };
    
    let initial_payload = AnalyticsPayload { metrics: initial_metrics, log: None };
    let initial_stream = futures::stream::once(async move {
        Ok(Event::default().data(serde_json::to_string(&initial_payload).unwrap()))
    });

    let broadcast_stream = BroadcastStream::new(rx).filter_map(|res| async {
        match res {
            Ok(payload) => Some(Ok(Event::default().data(serde_json::to_string(&payload).unwrap()))),
            Err(_) => None, // receiver lagged
        }
    });

    let combined_stream = initial_stream.chain(broadcast_stream);
    
    Sse::new(combined_stream).keep_alive(axum::response::sse::KeepAlive::new())
}
