use std::{sync::OnceLock, time::Duration};

use axum::{body::Body, extract::Request, response::IntoResponse};
use reqwest::StatusCode;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub async fn rust_handler() -> &'static str {
    "Hello, World!"
}

pub fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    })
}

// Fallback handler that forwards the request to another server on port 4000
pub async fn fallback_handler(req: Request<Body>) -> impl IntoResponse {
    // Build the URL to forward to
    let uri = req.uri();
    let method = req.method().clone();
    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let url = format!("http://127.0.0.1:8000{}", path_and_query);

    // Build the reqwest request
    let client = get_http_client();
    let mut builder = client.request(method, &url);

    // Forward headers
    for (name, value) in req.headers().iter() {
        // we may need to add or filter out some header, this is a good place to do it
        builder = builder.header(name, value);
    }

    // Extract the body using axum's Body type (which is re-exported hyper::Body)
    let Ok(bytes) = axum::body::to_bytes(req.into_body(),usize::MAX).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to extract body").into_response();
    };

    // Send the request
    let resp = builder.body(bytes).send().await;

    match resp {
        Ok(r) => {
            let status = r.status();
            let headers = r.headers().clone();
            let body = r.bytes().await.unwrap_or_default();
            let mut response = axum::response::Response::builder()
                .status(status);

            for (name, value) in headers.iter() {
                // Some headers are not allowed to be set manually, skip them
                if name != "content-length" && name != "transfer-encoding" {
                    response = response.header(name, value);
                }
            }

            response
                .body(Body::from(body))
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response())
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "Failed to forward request").into_response(),
    }
}

pub async fn users_handler() -> impl IntoResponse {
    use axum::Json;
    use serde::Serialize;
    use sqlx::sqlite::SqlitePool;

    #[derive(Serialize, sqlx::FromRow)]
    struct User {
        id: i64,
        username: String,
        is_active: bool
    }

    // Create a connection pool (in a real app, you should reuse the pool, not create it per request)
    let pool = match SqlitePool::connect("sqlite://./db.sqlite3").await {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to connect to database: {}", e),
            )
                .into_response();
        }
    };

    // Query all users
    let users: Result<Vec<User>, sqlx::Error> =
        sqlx::query_as::<_, User>("SELECT id, username, is_active FROM auth_user")
            .fetch_all(&pool)
            .await;

    match users {
        Ok(list) => Json(list).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database query failed: {}", e),
        )
            .into_response(),
    }
}