#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use actix_web::{
    error, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,middleware::Logger,
};
use core::panic;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::env;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;
use models::SensorData;
use models::SensorPostData;
use std::ops::Deref;
pub struct PoolData {
    db_pool: DbPool,
}

type PoolState = web::Data<PoolData>;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let db_url = env::var("DATABASE_URL");
    let db_url = match db_url {
        Ok(db_url) => db_url,
        Err(_) => {
            panic!("Veri tabanı bağlantısını DATABASE_URL çevre değişkeninde belirtin.")
        }
    };
    println!("{:} veri tabanı kullanılıyor. ", db_url);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL değişkeni belirtilmemiş.");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Bağlantı havuzu oluşturulamadı.");
    let db_pool_state = web::Data::new(PoolData { db_pool: pool });
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT rakam olmalı");
    HttpServer::new(move || {
        App::new()
            .data(db_pool_state.clone())
            .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
            .route("/", web::get().to(index))
            .service(parse_post)
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
    })
    .bind(("0.0.0.0",port))?
    .run()
    .await
}

pub fn insert_data(conn: &PgConnection, sicaklik: f64, nem: f64) -> usize {
    use crate::schema::sensor_data;
    let timestamp = chrono::Utc::now().naive_utc();
    let new_data = models::NewSensorData {
        sicaklik,
        nem,
        timestamp,
    };
    diesel::insert_into(sensor_data::table)
        .values(&new_data)
        .execute(conn)
        .expect("Yeni veri kaydedilirken hata oluştu.")
}

#[post("/sensor")]
async fn parse_post(info: web::Json<SensorPostData>, pool: web::Data<PoolState>) -> impl Responder {
    let sicaklik = info.sicaklik.clone();
    let nem = info.nem.clone();
    let conn = pool.db_pool.get();
    match conn {
        Ok(c) => {
            insert_data(c.deref(), sicaklik, nem);
        }
        Err(e) => panic!("{}", &e.to_string()),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json((nem, sicaklik))
}

pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    use actix_web::error::JsonPayloadError;

    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().body(detail),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            HttpResponse::UnprocessableEntity().body(detail)
        }
        _ => HttpResponse::BadRequest().body(detail),
    };
    error::InternalError::from_response(err, resp).into()
}

pub fn get_all_data(conn: &PgConnection) -> Result<Vec<SensorData>, diesel::result::Error> {
    use crate::schema::sensor_data::dsl::*;
    let results = sensor_data.load::<SensorData>(conn);
    match results {
        Ok(data) => Ok(data),
        Err(e) => Err(e),
    }
}

pub async fn index(pool: web::Data<PoolState>) -> HttpResponse {
    let conn = pool.db_pool.get();
    let conn = match conn {
        Ok(c) => c,
        Err(e) => panic!("{}", &e.to_string()),
    };

    let all_data = get_all_data(conn.deref());
    match all_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Ok().json(["error", &e.to_string()]),
    }
}

