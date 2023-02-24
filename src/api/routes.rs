use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
// use rocket::response::status;
use rocket::response::Response;
use rocket::serde::json::Json;
use rocket::{catch, get, post, Build, Rocket, State};
use rocket_db_pools::sqlx;
use std::net::IpAddr;

use crate::api;
use crate::api::controllers;
use crate::database;
use crate::models::account;
use crate::models::audiobook;
use crate::models::position;
use crate::utils::error;

struct AuthToken(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthToken {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth_header) = req.headers().get_one("Auth") {
            return Outcome::Success(AuthToken(auth_header.to_string()));
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

struct AuthHeader;

#[rocket::async_trait]
impl Fairing for AuthHeader {
    fn info(&self) -> Info {
        Info {
            name: "Add Auth Header",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Headers", "Auth"));
    }
}

pub fn create_rocket(
    port: u16,
    register: bool,
    address: IpAddr,
    pool: sqlx::Pool<sqlx::Sqlite>,
) -> Rocket<Build> {
    let config = rocket::Config {
        address: address,
        port: port,
        ..rocket::Config::debug_default()
    };
    rocket::custom(&config)
        .attach(AuthHeader)
        .mount(
            "/",
            if register {
                rocket::routes![
                    get_audiobooks_route,
                    get_audiobook_route,
                    get_audiobook_position_route,
                    post_audiobook_position_route,
                    login_route,
                    register_route,
                ]
            } else {
                rocket::routes![
                    get_audiobooks_route,
                    get_audiobook_route,
                    get_audiobook_position_route,
                    post_audiobook_position_route,
                    login_route,
                ]
            },
        )
        .register("/", rocket::catchers![not_found])
        .manage(pool)
}

#[catch(404)]
fn not_found() -> Json<error::Answer> {
    Json(error::not_found())
}

#[get("/audiobooks")]
async fn get_audiobooks_route(
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Result<Json<audiobook::Audiobooks>, Json<error::Answer>> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    match user {
        Ok(user) => user,
        Err(_) => {
            return Err(Json(error::cant_auth()));
        }
    };
    controllers::get_audiobooks(pool).await
}

#[get("/audiobook/<hash>")]
async fn get_audiobook_route(
    hash: String,
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Result<Vec<u8>, Json<error::Answer>> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    match user {
        Ok(user) => user,
        Err(_) => {
            return Err(Json(error::cant_auth()));
        }
    };
    controllers::get_audiobook(hash, pool).await
}

#[get("/audiobook/<hash>/position")]
async fn get_audiobook_position_route(
    hash: String,
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Result<Json<position::Position>, Json<error::Answer>> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return Err(Json(error::cant_auth()));
        }
    };
    api::controllers::get_audiobook_position(hash, user, pool).await
}

#[post(
    "/audiobook/<hash>/position",
    format = "application/json",
    data = "<position>"
)]
async fn post_audiobook_position_route(
    hash: String,
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    position: Json<position::Position>,
    auth_token: AuthToken,
) -> Json<error::Answer> {
    let user = database::schema::query_user(auth_token.0, pool).await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return Json(error::Answer {
                code: 2,
                msg: String::from("Error, could not authenticate"),
            });
        }
    };
    println!("{}", user);

    api::controllers::post_audiobook_position(
        hash,
        user,
        position.file.clone(),
        position.position,
        pool,
    )
    .await
}

#[post("/register", format = "application/json", data = "<account>")]
async fn register_route(
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    account: Json<account::NewAccount>,
) -> Result<Json<controllers::ApiKey>, Json<error::Answer>> {
    api::controllers::post_account(account.user.clone(), account.password.clone(), pool).await
}

#[post("/login", format = "application/json", data = "<account>")]
async fn login_route(
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    account: Json<account::NewAccount>,
) -> Result<Json<controllers::ApiKey>, Json<error::Answer>> {
    api::controllers::get_account(account.user.clone(), account.password.clone(), pool).await
}
