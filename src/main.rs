extern crate rocket;

use std::path::Path;
// use std::sync::Arc;
// use tokio;
// use tokio::sync::Mutex;

use clap::{App, Arg, ArgMatches};
use std::net::Ipv4Addr;

mod api;
mod database;
mod models;
mod utils;

fn arg_matches() -> ArgMatches {
    App::new("illiad")
        .version("1.0.0")
        .author("Paul Chambaz")
        .about("Illiad is an audiobook server")
        .arg(
            Arg::with_name("version")
                .short('v')
                .long("version")
                .help("Prints the version of the program"),
        )
        .arg(
            Arg::with_name("help")
                .short('h')
                .long("help")
                .help("Prints the help message"),
        )
        .arg(
            Arg::with_name("data")
                .short('d')
                .long("data")
                .value_name("DATA")
                .help("Path to the data directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sql")
                .short('s')
                .long("sql")
                .value_name("DATABASE")
                .help("Path to the sqlite database")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to bind to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("address")
                .short('a')
                .long("address")
                .value_name("ADDRESS")
                .help("Address to bind to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("register")
                .short('r')
                .long("register")
                .help("Whether or not to activate the register endpoint"),
        )
        .arg(
            Arg::with_name("config")
                .short('c')
                .long("config")
                .value_name("CONFIG")
                .help("Path to the config file")
                .takes_value(true),
        )
        .get_matches()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let matches = arg_matches();
    let pool: sqlx::Pool<sqlx::Sqlite>;
    if let Some(sql) = matches.value_of("sql") {
        // need to check that file is valid - but i think that sql does that already
        pool = database::schema::create_pool(sql).await;
    } else {
        pool = database::schema::create_pool("/usr/share/illiad/database.sqlite").await;
    };
    // let shared_pool = Arc::new(Mutex::new(pool));

    database::schema::create_accounts(&pool)
        .await
        .expect("Could not create accounts");

    database::schema::create_positions(&pool)
        .await
        .expect("Could not create positions");

    if let Some(data) = matches.value_of("data") {
        database::schema::scan_audiobooks(Path::new(data), &pool)
            .await
            .expect("Could not scan audiobooks");
    } else {
        // eprintln!("Missing data directory.\nPlease add data: eitherin /etc/illiad/illiadrc, $HOME/.config/illiad/illiadrc, as an argument with -d or --data or in a config file with -c or --config.");
        eprintln!("Missing data directory, please provide:");
        eprintln!("  - as an argument with -d or --data");
        eprintln!("  - in a config file with -c or --config");
        eprintln!("  - in $HOME/etc/illiadrc");
        eprintln!("  - in /etc/illiadrc");
        std::process::exit(1);
    }

    let port = if let Some(port_str) = matches.value_of("port") {
        if let Ok(port) = port_str.parse::<u16>() {
            port
        } else {
            eprintln!("Could not parse port.");
            std::process::exit(1);
        }
    } else {
        15000
    };

    let address = if let Some(address_str) = matches.value_of("address") {
        if let Ok(address) = address_str.parse::<Ipv4Addr>() {
            address
        } else {
            eprintln!("Could not parse address.");
            std::process::exit(1);
        }
    } else {
        Ipv4Addr::new(127, 0, 0, 1)
    };

    let rocket =
        api::routes::create_rocket(port, matches.is_present("register"), address.into(), pool);

    let _ = rocket.launch().await?;
    Ok(())
}
