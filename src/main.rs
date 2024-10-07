use actix_web::{
    web, error, get, guard, App, HttpResponse, HttpServer, Responder,
    http::{header::ContentType, StatusCode},
    middleware::Logger,
};
use actix_cors::Cors;
use env_logger::Env;
use std::{io, sync::{Arc, Mutex}};
use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use derive_more::derive::Display;

mod model;
mod db;
mod api;

mod util;
pub use util::{IdType, TimeType, now};

// SHARED STATE EXAMPLES //

struct AppState {
    app_name: String
}

struct AppStateWithCounter {
    counter: Mutex<usize>,
}

//-------

// USER FACING ERRORS
#[derive(Debug, Display)]
enum AppError {
    #[display("Validation error on field: {field}")]
    ValidationError { field: String },
    #[display("Not found")]
    NotFound,
    #[display("Database error has occurred: {error_msg}. Please try again later.")]
    DBError { error_msg: String },
    #[display("An internal error has occurred. Please try again later.")]
    InternalError,
    #[display("Not implemented yet.")]
    NotImplemented,
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::DBError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
        }
    }
}

//----------


#[get("/")]
async fn index(data: web::Data<AppState>, data_with_counter: web::Data<AppStateWithCounter>) -> impl Responder {
    let app_name = &data.app_name;
    let mut counter = data_with_counter.counter.lock().unwrap();
    *counter += 1;

    format!("Welcome to the app: {}, request count: {}", app_name, counter)
}

#[get("/info")]
async fn info() -> impl Responder {
    format!{"Welcome to the app! Here we will provide useful info for debugging the server."}
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // construct the data only once
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0)
    });

    let json_config = Arc::new(web::JsonConfig::default()
        .limit(4096) // 4kb
        // create custom error response
        .error_handler(|err, _req| {
            error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                .into()
        }));

    // database connection
    let conn = Connection::open("game_of_life.db").expect("error connecting to database");
    conn.execute("PRAGMA foreign_keys = ON;", ()).expect("cannot set pragma foreign_keys to ON");
    db::create_db(&conn).expect("cannot create db");

    let manager = SqliteConnectionManager::file("game_of_life.db");

    // pool to make db requests, this will be shared
    let pool = db::Pool::new(manager).expect("error creating connection pool");

    // setup logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        let logger = Logger::new("%a %r %s Req: Content-Type=%{Content-Type}i");
        //let logger = Logger::default();
        let cors = Cors::default()
            .allowed_origin("http://localhost:4000")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(logger)
            .wrap(cors)
            // prepare shared states
            .app_data(web::Data::new(pool.clone()))
            .app_data(json_config.clone())
            .app_data(
                web::Data::new(AppState {
                    app_name: String::from("Game of Life API")
                })
            )
            .app_data(counter.clone())
            // configure api
            .configure(api::config)
            // configure index and info routes
            .service(
                web::scope("")
                    .guard(guard::Host("localhost").scheme("http"))
                    .service(index)
                    .service(info)
            )
    })
        //.keep_alive(None)
        .workers(2)
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
