use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

use crate::appstate::AppState;
use crate::middleware::TokenClaims;

#[derive(Deserialize)]
struct ProductBody {
    product_id: i32,
    product_qty: i32,
    product_price: f32,
    operation: String,
}

#[derive(Serialize, FromRow)]
struct ProductId {
    product_id: i32,
}

enum Optional<T> {
    Some(T),
    None,
}

#[post("/api/update")]
pub async fn update_product(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<ProductBody>,
) -> impl Responder {
    let select_product = "select product_id from products_table where product_id = $1";
    let mut update_product =
        String::from("update products_table set product_price = $1, product_qty = product_qty");

    let product: ProductBody = body.into_inner();
    let prod_id: Optional<i32> = Optional::Some(product.product_id);

    if product.operation == "plus" {
        update_product.push_str(" + $2 where product_id = $3");
    } else {
        update_product.push_str(" - $2 where product_id = $3");
    }

    match req_user {
        Some(_) => {
            match prod_id {
                Optional::Some(id) => {
                    // search if the product exist
                    match sqlx::query_as::<_, ProductId>(select_product)
                        .bind(id)
                        .fetch_one(&state.db)
                        .await
                    {
                        Ok(_) => {
                            match sqlx::query(&update_product.to_string())
                                .bind(product.product_price)
                                .bind(product.product_qty)
                                .bind(product.product_id)
                                .execute(&state.db)
                                .await
                            {
                                Ok(_) => {
                                    HttpResponse::Created().json("Product updated successfullt")
                                }
                                Err(error) => HttpResponse::InternalServerError()
                                    .json(format!("Updating error: {:?}", error)),
                            }
                        }
                        Err(error) => HttpResponse::InternalServerError()
                            .json(format!("Product not found: {:?}", error)),
                    }
                }
                Optional::None => {
                    HttpResponse::InternalServerError().json("No product id is present")
                }
            }
        }
        _ => HttpResponse::Unauthorized().json("Unauthorized access"),
    }
}
