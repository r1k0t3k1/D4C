use std::time::Duration;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use anyhow::Result;

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub struct AppState {
    db: PgPool,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from(value: DatabaseConfig) -> Self {
        Self::new()
            .host(&value.host)
            .port(value.port)
            .username(&value.username)
            .password(&value.password)
            .database(&value.database)
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn health_check_db(data: web::Data<AppState>) -> impl Responder {
    let connection_result = sqlx::query("SELECT 1;").fetch_one(&data.db).await;

    match connection_result {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("{}", e);
            HttpResponse::InternalServerError()
        }
    }
}

fn connect_database() -> PgPool {
    let database_config = DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        username: "api".into(),
        password: "api".into(),
        database: "api".into(),
    };

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(database_config.into())
}

#[actix_web::main]
async fn main() -> Result<()> {
    let port = 9000;

    let connection_pool = connect_database();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: connection_pool.clone(),
            }))
            .route("/health", web::get().to(health_check))
            .route("/health/db", web::get().to(health_check_db))
    })
    .bind(("127.0.0.1", port))?
    .run();

    println!("API server listening on port {}...", port);
    server.await.expect("Failed to run api server.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{connect_database, health_check, health_check_db, AppState};
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app =
            test::init_service(App::new().route("/health", web::get().to(health_check))).await;
        let request = test::TestRequest::get().uri("/health").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_check_db() {
        let connection_pool = connect_database();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    db: connection_pool.clone(),
                }))
                .route("/health/db", web::get().to(health_check_db)),
        )
        .await;

        let request = test::TestRequest::get().uri("/health/db").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
