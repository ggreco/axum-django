# Rust-Django Migration Boilerplate

The goal is to migrate a Django project to Axum, step by step. This is the boilerplate that shows the approach that can be followed.

## Overview

This project demonstrates a hybrid approach for gradually migrating from Django to Rust's Axum web framework. The Rust application serves as a reverse proxy, handling specific routes directly while forwarding unhandled requests to the Django backend.

## Architecture

- **Rust (Axum)**: Handles `/rust` test endpoint and other future ones
- **Rust (Axum)**: Handles `/users` endpoint that shows the users configured in the django admin db
- **Django**: Continues to handle all other routes, including admin interface
- **Gradual Migration**: New features can be implemented in Rust while existing Django functionality remains intact

## Configure the application

Install the prerequirements: `cargo` and `uv`

Do the initial setup of the django application:

```bash
uv sync
uv run manage.py migrate
uv run manage.py createsuperuser
```

(the last step is interactive)

## Getting Started

1. Start the application:

   ```bash
   cargo run
   ```

   This will automatically start both the Django server (port 8000) and the Rust server (port 3000).

2. Some interesting routes:
   - Rust endpoint: http://127.0.0.1:3000/rust
   - Django ninja endpoint: http://127.0.0.1:3000/api/hello
   - Django admin: http://127.0.0.1:3000/admin/ (use the user created before) **served through axum**
   - All other routes are forwarded from Axum to Django

## Migration Strategy

Routes can be migrated from Django to Axum incrementally by:

1. Implementing the route handler in Rust (`src/endpoints.rs`)
2. Adding the route to the Axum router (`src/main.rs`)
3. Removing or deprecating the corresponding Django ninja API
