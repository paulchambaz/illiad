use clap::{App, Arg};
use dirs::home_dir;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::exit;

#[derive(serde::Deserialize)]
struct OptionConfig {
    data: Option<PathBuf>,
    sql: Option<PathBuf>,
    port: Option<u16>,
    address: Option<Ipv4Addr>,
    register: Option<bool>,
}

impl OptionConfig {
    fn new() -> Self {
        Self {
            data: None,
            sql: None,
            port: None,
            address: None,
            register: None,
        }
    }
}

pub struct Config {
    pub data: PathBuf,
    pub sql: PathBuf,
    pub port: u16,
    pub address: Ipv4Addr,
    pub register: bool,
}

impl Config {
    fn from(config: OptionConfig) -> Self {
        Self {
            data: config.data.unwrap(),
            sql: config.sql.unwrap(),
            port: config.port.unwrap(),
            address: config.address.unwrap(),
            register: config.register.unwrap(),
        }
    }
}

pub fn parse_args() -> Config {
    let matches = App::new("illiad")
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
        .get_matches();

    let mut config = OptionConfig::new();

    let toml = std::fs::read_to_string("/etc/illiad/illiadrc");
    match toml {
        Ok(toml) => match toml::from_str::<OptionConfig>(&toml) {
            Ok(_config) => {
                config = _config;
            }
            Err(_) => {
                eprintln!("Error, /etc/illiad/illiadrc has invalid fields");
                exit(1);
            }
        },
        Err(_) => {}
    };

    let home_dir = match home_dir() {
        Some(home_dir) => home_dir,
        None => {
            eprintln!("Error, could not find your home directory");
            exit(1);
        }
    };

    let toml = std::fs::read_to_string(home_dir.join(".config").join("illiad/illiadrc"));
    match toml {
        Ok(toml) => match toml::from_str::<OptionConfig>(&toml) {
            Ok(_config) => {
                config = _config;
            }
            Err(_) => {
                eprintln!("Error, ~/.config/illiad/illiadrc has invalid fields");
                exit(1);
            }
        },
        Err(_) => {}
    };

    if let Some(config_str) = matches.value_of("config") {
        let toml = std::fs::read_to_string(PathBuf::from(config_str));

        match toml {
            Ok(toml) => match toml::from_str::<OptionConfig>(&toml) {
                Ok(_config) => {
                    config = _config;
                }
                Err(_) => {
                    eprintln!("Error, '{}' has invalid fields", config_str);
                    exit(1);
                }
            },
            Err(_) => {
                eprintln!("Error, could not open '{}'", config_str);
                exit(1);
            }
        };
    }

    if let Some(data) = matches.value_of("data") {
        config.data = Some(PathBuf::from(data));
    }

    if !config.data.is_some() {
        eprintln!("Missing data directory, please provide:");
        eprintln!("  - as an argument with -d or --data");
        eprintln!("  - in a config file with -c or --config");
        eprintln!("  - in $HOME/etc/illiadrc");
        eprintln!("  - in /etc/illiadrc");
        std::process::exit(1);
    }

    if let Some(sql) = matches.value_of("sql") {
        config.sql = Some(PathBuf::from(sql));
    }

    if let Some(port) = matches.value_of("port") {
        if let Ok(port) = port.parse::<u16>() {
            config.port = Some(port);
        } else {
            eprintln!("Could not parse port.");
            std::process::exit(1);
        }
    }

    if !config.port.is_some() {
        config.port = Some(8080);
    }

    if let Some(address) = matches.value_of("address") {
        if let Ok(address) = address.parse::<Ipv4Addr>() {
            config.address = Some(address);
        } else {
            eprintln!("Could not parse address.");
            std::process::exit(1);
        }
    }

    if !config.address.is_some() {
        config.address = Some(Ipv4Addr::new(127, 0, 0, 1));
    }

    if matches.is_present("register") {
        config.register = Some(true);
    }

    if !config.register.is_some() {
        config.register = Some(false);
    }

    Config::from(config)
}
