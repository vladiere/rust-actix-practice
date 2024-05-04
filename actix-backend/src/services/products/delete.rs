use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx;

use crate::appstate::AppState;
use crate::middleware::TokenClaims;

#[derive(Deserialize)]
struct ProductBody {
    product_id: i32,
}

#[post("/api/delete")]
pub async fn delete_product(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<ProductBody>,
) -> impl Responder {
    let query = "delete from products_table where product_id = $1";

    let product = body.into_inner();

    match req_user {
        Some(_) => {
            match sqlx::query(query)
                .bind(product.product_id)
                .execute(&state.db)
                .await
            {
                Ok(_) => HttpResponse::Ok().json("Product remove successfull"),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unauthorized access"),
    }
}
