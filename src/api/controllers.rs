use rocket::serde::json::Json;
use std::path::{Path, PathBuf};

use rocket_db_pools::sqlx;

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

use crate::database;
use crate::models::audiobook;
use crate::models::position;
use crate::utils::error;

#[derive(serde::Serialize)]
pub struct Answer {
    pub code: u32,
    pub msg: String,
}

#[derive(serde::Serialize)]
pub struct ApiKey {
    pub key: String,
}

pub async fn get_audiobooks(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Json<audiobook::Audiobooks>, Json<error::Answer>> {
    let audiobooks = match database::schema::query_audiobooks(pool).await {
        // TODO - this is probably bad.. right?
        Ok(audiobooks) => audiobooks,
        Err(_) => {
            return Err(Json(error::audiobooks_cant_query()));
        }
    };
    Ok(Json(audiobooks))
}

pub async fn archive_directory(dir: &Path) -> Result<Vec<u8>, std::io::Error> {
    let dir = match std::fs::read_dir(dir) {
        Ok(dir) => dir,
        Err(err) => {
            return Err(err);
        }
    };
    let mut data = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut data, Compression::default());
        let mut builder = Builder::new(&mut encoder);

        for entry in dir {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    return Err(err);
                }
            };
            let path = entry.path();
            if path.is_file() {
                // let file_name = path.file_name().unwrap().to_owned();
                let file_name = match path.file_name() {
                    Some(file_name) => file_name.to_owned(),
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Could not get file name",
                        ));
                    }
                };
                match builder.append_path_with_name(&path, &file_name) {
                    Ok(_) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }

        match builder.finish() {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    }

    Ok(data)
}

pub async fn get_audiobook(
    hash: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Vec<u8>, Json<error::Answer>> {
    let path = database::schema::query_audiobook(hash, pool).await;

    let path = match path {
        Ok(path) => path,
        Err(_) => {
            return Err(Json(error::hash_cant_query()));
        }
    };

    let binary_data = archive_directory(PathBuf::from(path).as_path()).await;

    let binary_data = match binary_data {
        Ok(binary_data) => binary_data,
        Err(_) => {
            return Err(Json(error::binary_cant_create()));
        }
    };

    Ok(binary_data)
}

pub async fn post_audiobook_position(
    hash: String,
    user: String,
    file: String,
    position: u32,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Json<error::Answer> {
    let res = database::schema::insert_position(hash, user, file, position, pool).await;

    match res {
        Ok(_) => {
            return Json(error::success());
        }
        Err(_) => {
            return Json(error::position_cant_update());
        }
    };
}

pub async fn get_audiobook_position(
    hash: String,
    user: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Json<position::Position>, Json<error::Answer>> {
    let position = database::schema::select_position(hash, user, pool).await;

    match position {
        Ok(position) => {
            return Ok(Json(position::Position {
                file: position.file,
                position: position.position,
            }));
        }
        Err(_) => {
            return Err(Json(error::position_cant_query()));
        }
    };
}

pub async fn post_account(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Json<ApiKey>, Json<error::Answer>> {
    let key = database::schema::insert_user(user, password, pool).await;

    match key {
        Ok(key) => {
            return Ok(Json(ApiKey { key: key }));
        }
        Err(_) => {
            return Err(Json(error::cant_register()));
        }
    };
}

pub async fn get_account(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Json<ApiKey>, Json<error::Answer>> {
    let key = database::schema::select_user(user, password, pool).await;

    match key {
        Ok(key) => {
            return Ok(Json(ApiKey { key: key }));
        }
        Err(_) => {
            return Err(Json(error::cant_login()));
        }
    };
}
