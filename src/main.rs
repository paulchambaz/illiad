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

    match database::schema::create_accounts(&pool).await {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Could not create account");
            std::process::exit(1);
        }
    };

    match database::schema::create_positions(&pool).await {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Could not create positions");
            std::process::exit(1);
        }
    };

    match database::schema::scan_audiobooks(&config.data, &pool).await {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Could not create positions");
            std::process::exit(1);
        }
    };

    let rocket =
        api::routes::create_rocket(config.port, config.register, config.address.into(), pool);

    let _ = rocket.launch().await?;
    Ok(())
}
