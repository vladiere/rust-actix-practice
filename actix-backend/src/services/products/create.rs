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
struct CreateProductBody {
    product_name: String,
    product_type: String,
    product_category: String,
    product_price: f32,
    product_qty: i32,
}

#[derive(Serialize, FromRow)]
struct ProductTable {
    product_id: i32,
    product_name: String,
    created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, FromRow)]
struct CategoryTable {
    category_id: i32,
}

#[derive(Serialize, FromRow)]
struct TypeTable {
    type_id: i32,
}

#[derive(Serialize, FromRow)]
struct ProductId {
    product_id: i32,
}

#[post("/api/add")]
pub async fn create_product(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<CreateProductBody>,
) -> impl Responder {
    let product_query = "insert into products_table (product_name,product_type_id,product_cat_id,product_price,product_qty) values ($1,$2,$3,$4,$5) returning product_id, product_name, created_at";
    let select_product = "select product_id from products_table where product_name = $1";
    let select_cat = "select category_id from category_table where category_name = $1";
    let create_cat = "insert into category_table (category_name) values ($1) returning category_id";
    let select_type = "select type_id from type_table where type_name = $1";
    let create_type = "insert into type_table (type_name) values ($1) returning type_id";

    let products: CreateProductBody = body.into_inner();

    match req_user {
        Some(_) => {
            // search if the product already exist
            match sqlx::query_as::<_, ProductId>(select_product)
                .bind(&products.product_name)
                .fetch_one(&state.db)
                .await
            {
                Err(_) => {
                    // Search of the type of product exist
                    match sqlx::query_as::<_, TypeTable>(select_type)
                        .bind(&products.product_type)
                        .fetch_one(&state.db)
                        .await
                    {
                        // if not found then create on
                        Err(_) => {
                            // connecting to the database to create
                            match sqlx::query_as::<_, TypeTable>(create_type)
                                .bind(&products.product_type)
                                .fetch_one(&state.db)
                                .await
                            {
                                // database error
                                Err(error) => {
                                    HttpResponse::InternalServerError().json(format!("{:?}", error))
                                }
                                // if created successfully
                                Ok(types) => {
                                    // search for category if exist
                                    match sqlx::query_as::<_, CategoryTable>(select_cat)
                                        .bind(&products.product_category)
                                        .fetch_one(&state.db)
                                        .await
                                    {
                                        // error not found category
                                        Err(_) => {
                                            // if not found then create on
                                            match sqlx::query_as::<_, CategoryTable>(create_cat)
                                                .bind(&products.product_category)
                                                .fetch_one(&state.db)
                                                .await
                                            {
                                                // database error
                                                Err(error) => HttpResponse::InternalServerError()
                                                    .json(format!("{:?}", error)),
                                                // if success
                                                Ok(category) => {
                                                    // create the products assocaited with the types and
                                                    // category ids
                                                    match sqlx::query_as::<_, ProductTable>(
                                                        product_query,
                                                    )
                                                    .bind(&products.product_name)
                                                    .bind(types.type_id)
                                                    .bind(category.category_id)
                                                    .bind(&products.product_price)
                                                    .bind(&products.product_qty)
                                                    .fetch_one(&state.db)
                                                    .await
                                                    {
                                                        Err(error) => {
                                                            HttpResponse::InternalServerError()
                                                                .json(format!("{:?}", error))
                                                        }
                                                        Ok(product) => {
                                                            HttpResponse::Created().json(product)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // if found category
                                        Ok(category) => {
                                            // create the products assocaited with the types and category
                                            // ids
                                            match sqlx::query_as::<_, ProductTable>(product_query)
                                                .bind(&products.product_name)
                                                .bind(types.type_id)
                                                .bind(category.category_id)
                                                .bind(&products.product_price)
                                                .bind(&products.product_qty)
                                                .fetch_one(&state.db)
                                                .await
                                            {
                                                Err(error) => HttpResponse::InternalServerError()
                                                    .json(format!("{:?}", error)),
                                                Ok(product) => {
                                                    HttpResponse::Created().json(product)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // if the type is found
                        Ok(types) => {
                            // search if there is a category exist with the name
                            match sqlx::query_as::<_, CategoryTable>(select_cat)
                                .bind(&products.product_category)
                                .fetch_one(&state.db)
                                .await
                            {
                                // if not found
                                Err(_) => {
                                    // create category
                                    match sqlx::query_as::<_, CategoryTable>(create_cat)
                                        .bind(&products.product_category)
                                        .fetch_one(&state.db)
                                        .await
                                    {
                                        // database error when creating category
                                        Err(error) => HttpResponse::InternalServerError()
                                            .json(format!("{:?}", error)),
                                        // if created successfully
                                        Ok(category) => {
                                            // create the products assocaited with the types and category
                                            // ids
                                            match sqlx::query_as::<_, ProductTable>(product_query)
                                                .bind(&products.product_name)
                                                .bind(types.type_id)
                                                .bind(category.category_id)
                                                .bind(&products.product_price)
                                                .bind(&products.product_qty)
                                                .fetch_one(&state.db)
                                                .await
                                            {
                                                Err(error) => HttpResponse::InternalServerError()
                                                    .json(format!("{:?}", error)),
                                                Ok(product) => {
                                                    HttpResponse::Created().json(product)
                                                }
                                            }
                                        }
                                    }
                                }
                                // if created successfully
                                Ok(category) => {
                                    // create the products assocaited with the types and category
                                    // ids
                                    match sqlx::query_as::<_, ProductTable>(product_query)
                                        .bind(&products.product_name)
                                        .bind(types.type_id)
                                        .bind(category.category_id)
                                        .bind(&products.product_price)
                                        .bind(&products.product_qty)
                                        .fetch_one(&state.db)
                                        .await
                                    {
                                        Err(error) => HttpResponse::InternalServerError()
                                            .json(format!("{:?}", error)),
                                        Ok(product) => HttpResponse::Created().json(product),
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => HttpResponse::Conflict().json("Product already exist why not updating it"),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
