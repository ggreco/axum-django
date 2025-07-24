use axum::{routing::{any, get}, Router};
use rust_zola::endpoints::{fallback_handler, rust_handler, users_handler};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

async fn start_django_server() -> Result<tokio::process::Child, Box<dyn std::error::Error>> {
    println!("Starting Django server...");
    
    // Spawn the Django server in the background using Python from virtual environment
    let child = Command::new("python")
        .args(&["manage.py", "runserver", "127.0.0.1:8000"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Give Django a moment to start
    sleep(Duration::from_secs(1)).await;

    // Check if Django is responding
    let client = reqwest::Client::new();
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 10;
    
    while attempts < MAX_ATTEMPTS {
        match client.get("http://127.0.0.1:8000").send().await {
            Ok(response) => {
                if response.status().is_success() || response.status() == 404 {
                    println!("Django server is ready!");
                    return Ok(child);
                }
            }
            Err(_) => {
                attempts += 1;
                println!("Waiting for Django server... (attempt {}/{})", attempts, MAX_ATTEMPTS);
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
    
    return Err("Django server failed to start or is not responding".into());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start Django server in the background and keep the handle
    let _django_child = start_django_server().await?;

    // build our application with a single route and a fallback
    let app = Router::new()
        .route("/users", get(users_handler))
        .route("/rust", get(rust_handler))
        .fallback(any(fallback_handler)); // this will forward to django all the requests that are not handled by the rust code

    println!("Starting Rust server on http://127.0.0.1:3000");
    println!("Django admin available at: http://127.0.0.1:8000/admin/");
    println!("Admin credentials: username=admin, password=admin123");
    
    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
