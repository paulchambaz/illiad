use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;
use rocket::response::Response;
use rocket::serde::json::Json;
use rocket::{get, post, Build, Rocket, State};
use rocket_db_pools::sqlx;

use crate::api;
use crate::database;
use crate::models::account;
use crate::models::audiobook;
use crate::models::position;

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

pub fn create_rocket(port: u16, pool: sqlx::Pool<sqlx::Sqlite>) -> Rocket<Build> {
    let config = rocket::Config {
        address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
        port: port,
        ..rocket::Config::debug_default()
    };
    rocket::custom(&config)
        .attach(AuthHeader)
        .mount(
            "/",
            rocket::routes![
                get_audiobooks_route,
                get_audiobook_route,
                get_audiobook_position_route,
                post_audiobook_position_route,
                register_route,
                login_route,
            ],
        )
        .manage(pool)
}

#[get("/audiobooks")]
async fn get_audiobooks_route(
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Json<Vec<audiobook::AudiobookFmt>> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    match user {
        Ok(user) => user,
        Err(_) => {
            return Json(Vec::new());
        }
    };
    api::controllers::get_audiobooks(pool).await
}

#[get("/audiobook/<hash>")]
async fn get_audiobook_route(
    hash: String,
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Result<Vec<u8>, status::Custom<String>> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    match user {
        Ok(user) => user,
        Err(_) => {
            return Err(status::Custom(
                Status::NotFound,
                format!("Bad authentification token"),
            ));
        }
    };
    api::controllers::get_audiobook(hash, pool).await
}

#[get("/audiobook/<hash>/position")]
async fn get_audiobook_position_route(
    hash: String,
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    auth_token: AuthToken,
) -> Json<position::Position> {
    let user = database::schema::query_user(auth_token.0, pool).await;
    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return Json(position::Position {
                file: String::new(),
                position: 0,
            });
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
) -> Json<api::controllers::Answer> {
    let user = database::schema::query_user(auth_token.0, pool).await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return Json(api::controllers::Answer { code: 1 });
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
) -> String {
    api::controllers::post_account(account.user.clone(), account.password.clone(), pool).await
}

#[post("/login", format = "application/json", data = "<account>")]
async fn login_route(
    pool: &State<sqlx::Pool<sqlx::Sqlite>>,
    account: Json<account::NewAccount>,
) -> String {
    api::controllers::get_account(account.user.clone(), account.password.clone(), pool).await
}
