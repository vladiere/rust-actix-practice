use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use argonautica::{Hasher, Verifier};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{self, FromRow};

use crate::appstate::AppState;
use crate::middleware::TokenClaims;

#[derive(Deserialize)]
struct RegisterUserBody {
    firstname: String,
    lastname: String,
    email_address: String,
    password: String,
}

#[derive(Serialize, FromRow)]
struct UserNoPassword {
    admin_id: i32,
    firstname: String,
    lastname: String,
    email_address: String,
}

#[derive(Serialize, FromRow)]
struct AdminId {
    admin_id: i32,
}

#[derive(Serialize, FromRow)]
struct AuthUser {
    admin_id: i32,
    email_address: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginUser {
    email_address: String,
    password: String,
}

#[derive(Serialize, Debug)]
struct ResponseToken {
    access_token: String,
}

#[derive(Serialize, Debug)]
struct ErrorStatus {
    message: String,
    status: u32,
}

#[post("/api/register")]
pub async fn register_user(state: Data<AppState>, body: Json<RegisterUserBody>) -> impl Responder {
    let search_query = "select admin_id from admin_table where email_address = $1";
    let query =
        "insert into admin_table (firstname,lastname,email_address,password) values ($1,$2,$3,$4) returning admin_id, firstname, lastname, email_address";
    let user: RegisterUserBody = body.into_inner();

    match sqlx::query_as::<_, AdminId>(search_query)
        .bind(&user.email_address)
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => HttpResponse::InternalServerError().json(format!("Email address already exist")),
        Err(_) => {
            let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET is not set");
            let mut hasher = Hasher::default();
            let hash = hasher
                .with_password(user.password)
                .with_secret_key(hash_secret)
                .hash()
                .unwrap();

            match sqlx::query_as::<_, UserNoPassword>(query)
                .bind(user.firstname)
                .bind(user.lastname)
                .bind(user.email_address)
                .bind(hash)
                .fetch_one(&state.db)
                .await
            {
                Ok(user) => HttpResponse::Ok().json(user),
                Err(error) => {
                    println!("{:?}", error);
                    HttpResponse::InternalServerError().json(format!("{:?}", error))
                }
            }
        }
    }
}

#[post("/api/login")]
pub async fn basic_auth(state: Data<AppState>, credentials: Json<LoginUser>) -> impl Responder {
    let query =
        "select admin_id, email_address, password from admin_table where email_address = $1";
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET is not set")
            .as_bytes(),
    )
    .unwrap();

    let user: LoginUser = credentials.into_inner();
    let LoginUser {
        email_address: email,
        password: pass,
    } = user;

    if !email.is_empty() && !pass.is_empty() {
        if !email.is_empty() {
            if !pass.is_empty() {
                match sqlx::query_as::<_, AuthUser>(query)
                    .bind(email)
                    .fetch_one(&state.db)
                    .await
                {
                    Ok(user) => {
                        let hash_secret =
                            std::env::var("HASH_SECRET").expect("HASH_SECRET is not set");
                        let mut verifier = Verifier::default();
                        let is_valid = verifier
                            .with_hash(user.password)
                            .with_password(pass)
                            .with_secret_key(hash_secret)
                            .verify()
                            .unwrap();

                        if is_valid {
                            let claims = TokenClaims {
                                id: user.admin_id,
                                email: user.email_address,
                            };
                            let token_str = ResponseToken {
                                access_token: claims.sign_with_key(&jwt_secret).unwrap(),
                            };

                            HttpResponse::Ok().json(token_str)
                        } else {
                            let error = ErrorStatus {
                                message: String::from("Incorrect username or password"),
                                status: 401,
                            };
                            HttpResponse::Unauthorized().json(error)
                        }
                    }
                    Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
                }
            } else {
                let error = ErrorStatus {
                    message: String::from("Password is required"),
                    status: 401,
                };
                HttpResponse::Unauthorized().json(error)
            }
        } else {
            let error = ErrorStatus {
                message: String::from("Email address is required"),
                status: 401,
            };
            HttpResponse::Unauthorized().json(error)
        }
    } else {
        let error = ErrorStatus {
            message: String::from("Email address and password are required"),
            status: 401,
        };
        HttpResponse::Unauthorized().json(error)
    }
}
