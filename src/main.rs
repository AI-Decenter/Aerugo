use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::env;

mod routes;
mod handlers;
mod models;
mod utils;
mod middleware;

#[tokio::main]
async fn main() {
    // Load environment variables from .env
    dotenv().ok();

    // Initialize tracing (logging)
    let subscriber = utils::logging::setup_logging();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Setup database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    // Build our application with routes and middleware
    let app = Router::new()
        .route("/health", get(handlers::health::check))
        .route("/users", post(handlers::user::create_user))
        .route("/users/:id", get(handlers::user::get_user))
        .route("/users/:id", put(handlers::user::update_user))
        .route("/users/:id", delete(handlers::user::delete_user))
        .layer(tower_http::trace::TraceLayer::new_for_http())
// correlation_id middleware doesn't need state => use from_fn instead of from_fn_with_state        .layer(axum::middleware::from_fn(middleware::correlation::correlation_id))
        .with_state(pool);

    // Run it
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    tracing::info!("listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    )
    .await
    .unwrap();
}
