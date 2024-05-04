use actix_web::{
    get, patch, post, web::Data, web::Json, web::Path, App, HttpResponse, HttpServer, Responder,
};
mod db;
mod models;
use crate::db::Database;
use crate::models::users::{AddUserRequest, EditUserURL};
use validator::Validate;

#[get("/users")]
async fn get_users(db: Data<Database>) -> impl Responder {
    let users = db.get_all_users().await;
    match users {
        Some(found_users) => HttpResponse::Ok().body(format!("{:?}", found_users)),
        None => HttpResponse::Ok().body("Error!"),
    }
}

#[post("/adduser")]
async fn add_users(body: Json<AddUserRequest>) -> impl Responder {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let fullname = body.fullname.clone();
            HttpResponse::Ok().body(format!("User's fullname {fullname}"))
        }
        Err(_) => HttpResponse::Ok().body("Fullname required!"),
    }
}

#[patch("/edituser/{uuid}")]
async fn edit_user(edit_user_url: Path<EditUserURL>) -> impl Responder {
    let uuid = edit_user_url.into_inner().uuid;
    HttpResponse::Ok().body(format!("Use editting {uuid}"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::init()
        .await
        .expect("error conencting to database");

    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(get_users)
            .service(add_users)
            .service(edit_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
