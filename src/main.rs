#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use actix_web::{
    error, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use core::{f32, panic};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use r2d2;
use r2d2::Pool;
use r2d2_diesel;
use r2d2_diesel::ConnectionManager;
use std::env;
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
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
    env_logger::init();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL");
    let db_url = match db_url {
        Ok(db_url) => db_url,
        Err(_) => {
            panic!("Veri tabanını .env dosyasında belirtin. Örnek: DATABASE_URL=sensordata.db")
        }
    };
    println!("{:} veri tabanı kullanılıyor. ", db_url);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL değişkeni belirtilmemiş.");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Bağlantı havuzu oluşturulamadı.");
    let my_data = web::Data::new(PoolData { db_pool: pool });

    HttpServer::new(move || {
        App::new()
            .data(my_data.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .service(parse_post)
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

pub fn insert_data(conn: &SqliteConnection, sicaklik: f32, nem: f32) -> usize {
    use crate::schema::sensor_veri;
    let timestamp = chrono::Utc::now().naive_utc();
    let new_data = models::NewSensorData {
        sicaklik,
        nem,
        timestamp,
    };
    diesel::insert_into(sensor_veri::table)
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
        Err(e) => panic!(&e.to_string()),
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

pub fn get_all_data(conn: &SqliteConnection) -> Result<Vec<SensorData>, diesel::result::Error> {
    use crate::schema::sensor_veri::dsl::*;
    let results = sensor_veri.load::<SensorData>(conn);
    match results {
        Ok(data) => Ok(data),
        Err(e) => Err(e),
    }
}

pub async fn index(pool: web::Data<PoolState>) -> HttpResponse {
    let conn = pool.db_pool.get();
    let conn = match conn {
        Ok(c) => c,
        Err(e) => panic!(&e.to_string()),
    };

    let all_data = get_all_data(conn.deref());
    match all_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Ok().json(["error", &e.to_string()]),
    }
}
