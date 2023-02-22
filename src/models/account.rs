// use openssl::base64::encode_block;
// use openssl::rand::rand_bytes;
use rand::Rng;

#[derive(serde::Serialize)]
pub struct Account {
    pub user: String,
    pub password: String,
    pub key: String,
}

#[derive(serde::Deserialize)]
pub struct NewAccount {
    pub user: String,
    pub password: String,
}

/// Computes the api key for the account
pub fn generate_key() -> String {
    let mut rng = rand::thread_rng();
    let key: String = (0..16)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    key
}
