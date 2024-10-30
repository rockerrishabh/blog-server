use actix_cors::Cors;
use actix_web::{get, http::header, web::Data, App, HttpResponse, HttpServer, Responder};
use db::connection::{establish_pool, AppState};
use middlewares::auth::Authentication;
use rust_server::*;
use services::{
    posts::{create_post, delete_post, get_post, get_posts, update_post},
    users::{check_auth, login, logout, register},
};
use std::env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = establish_pool(database_url);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://rishabhportfolio.site")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".rishabhportfolio.site")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600)
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(Authentication)
            .app_data(Data::new(AppState { pool: pool.clone() }))
            .service(hello)
            .service(login)
            .service(check_auth)
            .service(logout)
            .service(register)
            .service(get_posts)
            .service(get_post)
            .service(create_post)
            .service(update_post)
            .service(delete_post)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
