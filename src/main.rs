extern crate rocket;

use std::path::Path;
// use std::sync::Arc;
// use tokio;
// use tokio::sync::Mutex;

mod api;
mod database;
mod models;
mod utils;

// TODO: main must take a port and a sqlite:// url instead of it being static data should also be
// specified in the arguments - or i a config.toml
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // let pool = database::schema::create_pool("/usr/share/illiad/database.sqlite").await;
    let pool = database::schema::create_pool("database.sqlite").await;

    // let shared_pool = Arc::new(Mutex::new(pool));

    database::schema::create_accounts(&pool)
        .await
        .expect("Could not create accounts");

    database::schema::create_positions(&pool)
        .await
        .expect("Could not create positions");

    database::schema::scan_audiobooks(Path::new("/home/paul/dev/rust/illiad/data"), &pool)
        .await
        .expect("Could not scan audiobooks");

    let rocket = api::routes::create_rocket(15000, pool);

    let _ = rocket.launch().await?;
    Ok(())
}
