use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use std::path::{Path, PathBuf};

use rocket_db_pools::sqlx;

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

use crate::database;
use crate::models::audiobook;
use crate::models::position;

#[derive(serde::Serialize)]
pub struct Answer {
    pub code: u32,
}

pub async fn get_audiobooks(pool: &sqlx::Pool<sqlx::Sqlite>) -> Json<Vec<audiobook::AudiobookFmt>> {
    let audiobooks = database::schema::query_audiobooks(pool)
        .await
        .expect("Could not query the audiobooks");
    Json(audiobooks)
}

pub async fn archive_directory(dir: &Path) -> Result<Vec<u8>, sqlx::Error> {
    // pub async fn archive_directory(dir: &Path) {
    let dir = std::fs::read_dir(dir).expect("Could not read dir");
    let mut data = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut data, Compression::default());
        let mut builder = Builder::new(&mut encoder);

        for entry in dir {
            let entry = entry.expect("Could not get entry");
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_owned();
                builder
                    .append_path_with_name(&path, &file_name)
                    .expect("Could not append path during archive/compression");
            }
        }

        builder.finish().expect("Could not build data");
    }

    Ok(data)
}

pub async fn get_audiobook(
    hash: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Vec<u8>, status::Custom<String>> {
    let path = database::schema::query_audiobook(hash, pool).await;

    let path = match path {
        Ok(path) => path,
        Err(_) => {
            return Err(status::Custom(Status::NotFound, format!("Hash not found")));
        }
    };

    let binary_data = archive_directory(PathBuf::from(path).as_path()).await;

    let binary_data = match binary_data {
        Ok(binary_data) => binary_data,
        Err(_) => {
            return Err(status::Custom(
                Status::NotFound,
                format!("Could not create binary data"),
            ));
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
) -> Json<Answer> {
    let res = database::schema::insert_position(hash, user, file, position, pool).await;

    match res {
        Ok(_) => {
            return Json(Answer { code: 0 });
        }
        Err(_) => {
            return Json(Answer { code: 1 });
        }
    };
}

pub async fn get_audiobook_position(
    hash: String,
    user: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Json<position::Position> {
    let position = database::schema::select_position(hash, user, pool).await;

    match position {
        Ok(position) => {
            return Json(position::Position {
                file: position.file,
                position: position.position,
            });
        }
        Err(_) => {
            return Json(position::Position {
                file: String::new(),
                position: 0,
            });
        }
    };
}

pub async fn post_account(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> String {
    let key = database::schema::insert_user(user, password, pool).await;

    match key {
        Ok(key) => {
            return key;
        }
        Err(_) => {
            return String::new();
        }
    };
}

pub async fn get_account(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> String {
    let key = database::schema::select_user(user, password, pool).await;

    match key {
        Ok(key) => {
            return key;
        }
        Err(_) => {
            return String::new();
        }
    };
}
