extern crate rocket;
// use std::sync::Arc;
// use tokio;
// use tokio::sync::Mutex;

mod api;
mod database;
mod models;
mod utils;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let config = utils::cli::parse_args();

    let pool = database::schema::create_pool(config.sql).await;
    // let shared_pool = Arc::new(Mutex::new(pool));

    database::schema::create_accounts(&pool)
        .await
        .expect("Could not create accounts");

    database::schema::create_positions(&pool)
        .await
        .expect("Could not create positions");

    database::schema::scan_audiobooks(&config.data, &pool)
        .await
        .expect("Could not scan audiobooks");

    let rocket =
        api::routes::create_rocket(config.port, config.register, config.address.into(), pool);

    let _ = rocket.launch().await?;
    Ok(())
}
