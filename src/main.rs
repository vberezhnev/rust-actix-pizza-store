mod db;
mod error;
mod models;

use crate::db::Database;
use crate::error::PizzaError;
use crate::models::{BuyPizzaRequest, Pizza, UpdatePizzaURL};
use actix_web::{delete, get, patch, post, web::Data, web::Json, web::Path, App, HttpServer};
use uuid::Uuid;
use validator::Validate;

#[get("/pizzas")]
async fn get_pizzas(db: Data<Database>) -> Result<Json<Vec<Pizza>>, PizzaError> {
    let pizzas = db.get_all_pizzas().await;
    match pizzas {
        Some(found_pizzas) => Ok(Json(found_pizzas)),
        None => Err(PizzaError::NoPizzasFound),
    }
}

#[post("/buypizza")]
async fn buy_pizza(
    body: Json<BuyPizzaRequest>,
    db: Data<Database>,
) -> Result<Json<Pizza>, PizzaError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let pizza_name = body.pizza_name.clone();
            let mut buffer = Uuid::encode_buffer();
            let new_uuid = Uuid::new_v4().simple().encode_lower(&mut buffer);

            let new_pizza = db
                .add_pizza(Pizza::new(String::from(new_uuid), pizza_name))
                .await;

            match new_pizza {
                Some(created) => Ok(Json(created)),
                None => Err(PizzaError::PizzaCreationFailure),
            }
        }
        Err(_) => Err(PizzaError::PizzaCreationFailure),
    }
}

#[patch("/updatepizza/{uuid}")]
async fn update_pizza(
    update_pizza_url: Path<UpdatePizzaURL>,
    db: Data<Database>,
) -> Result<Json<Pizza>, PizzaError> {
    let uuid = update_pizza_url.into_inner().uuid;
    let update_result = db.update_pizza(uuid).await;

    match update_result {
        Some(update_pizza) => Ok(Json(update_pizza)),
        None => Err(PizzaError::NoSuchPizzaFound),
    }
}

#[delete("/deletepizza/{uuid}")]
async fn delete_pizza(
    delete_pizza_url: Path<UpdatePizzaURL>,
    db: Data<Database>,
) -> Result<Json<Pizza>, PizzaError> {
    let uuid = delete_pizza_url.into_inner().uuid;
    let delete_result = db.delete_pizza(uuid).await;

    match delete_result {
        Some(delete_pizza) => Ok(Json(delete_pizza)),
        None => Err(PizzaError::NoSuchPizzaFound),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // enable logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = Database::init()
        .await
        .expect("Error connecting to database");
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(get_pizzas)
            .service(buy_pizza)
            .service(update_pizza)
            .service(delete_pizza)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
