use axum::extract::State;
use axum::Json;
use clap::Parser;
use db_init::DatabasePool;
use hyper::StatusCode;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::api_init::{axum_init, AxumConfig};
use crate::db_init::{db_init, DatabaseConfig};

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    static_dir: String,

    #[clap(long = "db-url", default_value = "sqlite://sqlite.db")]
    db_url: String,
}

#[tokio::main]
async fn main() {
    let mut opt = Opt::parse();
    opt.log_level = String::from("trace");

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    // enable console logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "server=trace,tower_http=trace,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_config = DatabaseConfig {
        connection_url: opt.db_url.clone(),
    };
    let db = db_init(db_config).await;

    let axum_config = AxumConfig {
        database: db,
        addr: opt.addr,
        port: opt.port,
    };
    let server = axum_init(axum_config);

    // Awaiting the server will last for as long as the server lives.
    // If the future resolves something went wrong...
    if let Err(err) = server.await {
        tracing::error!("Server Error: {}", err)
    }
}

#[derive(Deserialize, Debug)]
struct Location {
    lat: f64,
    lng: f64,
}

async fn add_location(
    State(database_pool): State<DatabasePool>,
    Json(payload): Json<Location>,
) -> StatusCode {
    tracing::trace!(request_payload = ?payload, "Incoming add_location request");

    let result = sqlx::query(
        "\
    INSERT INTO Locations (latitude, longitude, date_added)
    VALUES ($1, $2, DATETIME())",
    )
    .bind(payload.lat)
    .bind(payload.lng)
    .execute(&database_pool)
    .await;

    match result {
        Ok(result) => {
            tracing::info!(?result, ?payload, "Created new location");
            StatusCode::OK
        }
        Err(error) => {
            tracing::error!(?error, "Error while adding new location");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

mod api_init {
    use crate::{add_location, db_init::DatabasePool, Opt};
    use axum::{
        routing::{get, post},
        Router,
    };
    use hyper::Method;
    use std::{
        error::Error,
        net::{IpAddr, Ipv6Addr, SocketAddr},
        str::FromStr as _,
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        trace::TraceLayer,
    };

    pub struct AxumConfig {
        pub database: DatabasePool,
        pub addr: String,
        pub port: u16,
    }

    pub async fn axum_init(config: AxumConfig) -> Result<(), impl Error> {
        // TODO: Update allowed origin to be mangroves.report after updates
        let cors = CorsLayer::new()
            // allow `GET` and `POST` when accessing the resource
            .allow_methods([Method::GET, Method::POST])
            // allow requests from any origin
            .allow_origin(Any);

        let hello = || async { "hello from server!" };

        let app = Router::new()
            .route("/api/v1/hello", get(hello))
            .route(
                "/api/v1/add-location",
                post(add_location).with_state(config.database.clone()),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(cors),
            );

        let sock_addr = SocketAddr::from((
            IpAddr::from_str(config.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            config.port,
        ));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

        tracing::info!("listening on http://{}", listener.local_addr().unwrap());

        axum::serve(listener, app).await
    }
}

mod db_init {
    use std::env::VarError;

    use sqlx::{
        migrate::{MigrateDatabase as _, MigrateError, Migrator},
        Pool, Sqlite, SqlitePool,
    };
    use thiserror::Error;

    pub struct DatabaseConfig {
        pub(crate) connection_url: String,
    }

    pub type DatabasePool = Pool<Sqlite>;

    type SQLxError = sqlx::Error;

    #[derive(Error, Debug)]
    pub enum DatabaseError {
        #[error("Failed to load environment error")]
        Environment(#[from] VarError),
        #[error("Failed to migrate database")]
        Migration(#[from] MigrateError),
        #[error("Failed to create dummy data")]
        DummyData(#[from] SQLxError),
    }

    pub async fn db_init(db_config: DatabaseConfig) -> DatabasePool {
        tracing::info!("Initializing database...");
        let DatabaseConfig { connection_url } = db_config;
        let database_exists = match Sqlite::database_exists(&connection_url).await {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(?error, "Errored while checking if database exists. Panicking to preserve data integrity.");
                panic!("error: {error}");
            }
        };

        if !database_exists {
            match Sqlite::create_database(&connection_url).await {
                Ok(_) => tracing::trace!("Create db success"),
                Err(error) => {
                    tracing::error!(
                        ?error,
                        connection_url,
                        "Errored while creating a database. Panicking to preserve data integrity."
                    );
                    panic!("error: {error}");
                }
            }
        } else {
            tracing::trace!("Database already exists.");
        }

        let db = match SqlitePool::connect(&connection_url).await {
            Ok(pool) => {
                tracing::trace!(connection_url, "Connected to database.");
                pool
            }
            Err(error) => {
                // TODO: Implement a retry system for DB connection
                tracing::error!(
                    ?error,
                    "Errored while connecting to database. Panicking to preserve data integrity."
                );
                panic!("error: {error}");
            }
        };

        apply_migrations(db.clone()).await;

        tracing::info!("Database initialization complete.");

        db
    }

    async fn apply_migrations(db: DatabasePool) {
        let crate_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(
                    ?error,
                    env_var = "CARGO_MANIFEST_DIR",
                    "Error occurred while looking up environment variable."
                );
                panic!("error: {}", DatabaseError::Environment(error));
            }
        };
        let migrations = std::path::Path::new(&crate_dir).join("./migrations");

        let migrator = match Migrator::new(migrations).await {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(
                    ?error,
                    "Error while creating database migrator. Panicking to preserve data integrity."
                );
                panic!("error: {}", DatabaseError::Migration(error));
            }
        };

        match migrator.run(&db).await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(?error, "Error while applying database migrations. Panicking to preserve data integrity.");
                panic!("error: {}", DatabaseError::Migration(error));
            }
        }
    }
}
