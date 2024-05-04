use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse, Responder,
};
// use actix_web_httpauth::extractors::basic::BasicAuth;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

use crate::appstate::AppState;
use crate::middleware::TokenClaims;

#[derive(Deserialize)]
struct ProductBodyId {
    product_id: i32,
}

#[derive(Serialize, FromRow)]
struct ProductTable {
    product_id: i32,
    product_name: String,
    product_type: String,
    product_cat: String,
    product_price: f32,
    product_qty: i32,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

enum Optional<T> {
    Some(T),
    None,
}

#[post("/api/get")]
pub async fn retrieve_product(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<ProductBodyId>,
) -> impl Responder {
    let select_all = "select 
                        pt.product_id,
                        pt.product_name,
                        tt.type_name as product_type,
                        ct.category_name as product_cat,
                        pt.product_price,
                        pt.product_qty,
                        pt.created_at,
                        pt.updated_at 
                    from
                        products_table pt 
                    left join
                        type_table tt on tt.type_id = pt.product_type_id 
                    left join 
                        category_table ct on ct.category_id = pt.product_cat_id 
                        order by
                        pt.updated_at
                        desc";

    let select_one = "select 
                        pt.product_id,
                        pt.product_name,
                        tt.type_name as product_type,
                        ct.category_name as product_cat,
                        pt.product_price,
                        pt.product_qty,
                        pt.created_at,
                        pt.updated_at 
                    from
                        products_table pt 
                    left join
                        type_table tt on tt.type_id = pt.product_type_id 
                    left join 
                        category_table ct on ct.category_id = pt.product_cat_id 
                    where pt.product_id = $1
                        order by
                        pt.updated_at
                        desc";

    let prod_id: ProductBodyId = body.into_inner();
    let product_id: Optional<i32> = Optional::Some(prod_id.product_id);

    match req_user {
        Some(_) => match product_id {
            Optional::Some(id) => {
                if id == 0 {
                    match sqlx::query_as::<_, ProductTable>(select_all)
                        .fetch_all(&state.db)
                        .await
                    {
                        Ok(products) => HttpResponse::Ok().json(products),
                        Err(error) => {
                            HttpResponse::InternalServerError().json(format!("{:?}", error))
                        }
                    }
                } else {
                    match sqlx::query_as::<_, ProductTable>(select_one)
                        .bind(id)
                        .fetch_one(&state.db)
                        .await
                    {
                        Ok(product) => HttpResponse::Ok().json(product),
                        Err(error) => {
                            HttpResponse::InternalServerError().json(format!("{:?}", error))
                        }
                    }
                }
            }
            Optional::None => HttpResponse::InternalServerError().json("No product id is present"),
        },
        _ => HttpResponse::Unauthorized().json("Unauthorized access"),
    }
}
