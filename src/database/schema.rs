extern crate rocket;

use rocket_db_pools::{self, sqlx};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

// use chrono;

use crate::models::account;
use crate::models::audiobook;
use crate::models::position;

#[derive(sqlx::FromRow, Debug)]
struct AudiobookFmtRow {
    hash: String,
    title: String,
    author: String,
}

#[derive(sqlx::FromRow, Debug)]
struct AudiobookPathRow {
    path: String,
}

#[derive(sqlx::FromRow, Debug)]
struct PositionPathRow {
    file: String,
    position: u32,
}

#[derive(sqlx::FromRow, Debug)]
struct UserRow {
    user: String,
}

#[derive(sqlx::FromRow, Debug)]
struct KeyRow {
    key: String,
}

pub async fn scan_audiobooks(
    dir: &PathBuf,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), sqlx::Error> {
    match sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS audiobooks (
    hash TEXT PRIMARY KEY,
    title TEXT,
    author TEXT,
    path TEXT)"#,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };

    let audiobooks = scan_audiobook_direcories(Path::new(dir));

    for audiobook in audiobooks {
        match insert_audiobook(audiobook, pool).await {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    }

    Ok(())
}

pub async fn create_positions(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    match sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS positions (
        hash TEXT,
        user TEXT,
        file TEXT,
        position NUMBER, 
        last_modified DATE)"#,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    }

    Ok(())
}

pub async fn create_accounts(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
    match sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS accounts (
        user TEXT PRIMARY KEY,
        password TEXT,
        key TEXT)"#,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    }

    Ok(())
}

/// queries a list of all audiobooks
pub async fn query_audiobooks(
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<audiobook::Audiobooks, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, AudiobookFmtRow>(r#"SELECT hash, title, author FROM audiobooks"#)
            .fetch_all(pool)
            .await?;

    let audiobooks: Vec<audiobook::AudiobookFmt> = rows
        .into_iter()
        .map(|row| audiobook::AudiobookFmt {
            hash: row.hash,
            title: row.title,
            author: row.author,
        })
        .collect();

    Ok(audiobook::Audiobooks {
        audiobooks: audiobooks,
    })
}

pub async fn query_audiobook(
    hash: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<String, sqlx::Error> {
    let row =
        sqlx::query_as::<_, AudiobookPathRow>(r#"SELECT path FROM audiobooks WHERE hash = ?"#)
            .bind(hash)
            .fetch_one(pool)
            .await?;

    Ok(row.path)
}

pub async fn create_pool(address: PathBuf) -> sqlx::Pool<sqlx::Sqlite> {
    match sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(address),
    )
    .await
    {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}

pub async fn insert_audiobook(
    audiobook: audiobook::Audiobook,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), sqlx::Error> {
    let hash = audiobook::compute_hash(audiobook.title.clone(), audiobook.author.clone());
    let title = audiobook.title;
    let author = audiobook.author;
    let path = audiobook.path;

    match sqlx::query(
        r#"INSERT OR REPLACE INTO audiobooks (hash, title, author, path)
        VALUES (?, ?, ?, ?)"#,
    )
    .bind(hash)
    .bind(title)
    .bind(author)
    .bind(path)
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };

    Ok(())
}

pub async fn insert_position(
    hash: String,
    user: String,
    file: String,
    position: u32,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), sqlx::Error> {
    // updating positions for the ones that already exist

    // let now = chrono::Local::now();
    // let iso_date_time = now.to_rcf3339();

    match sqlx::query(
        r#"UPDATE positions
        SET file = ?, position = ?
        WHERE hash = ? AND user = ?"#,
    )
    .bind(file.clone())
    .bind(position.clone())
    // .bind(iso_date_time)
    .bind(hash.clone())
    .bind(user.clone())
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };
    match sqlx::query(
        r#"INSERT INTO positions (hash, user, file, position)
        SELECT ?, ?, ?, ?
        WHERE NOT EXISTS (
        SELECT 1 FROM positions WHERE hash = ? AND user = ?)"#,
    )
    .bind(hash.clone())
    .bind(user.clone())
    .bind(file.clone())
    .bind(position.clone())
    .bind(hash.clone())
    .bind(user.clone())
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };
    // add new rows when a user has never had information about a book
    Ok(())
}
pub async fn select_position(
    hash: String,
    user: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<position::Position, sqlx::Error> {
    let rows = match sqlx::query_as::<_, PositionPathRow>(
        r#"SELECT file, position FROM positions
        WHERE hash = ? AND user = ?"#,
    )
    .bind(hash)
    .bind(user)
    .fetch_one(pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return Err(err);
        }
    };

    let position = position::Position {
        file: rows.file,
        position: rows.position,
    };

    Ok(position)
}

pub async fn insert_user(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<String, sqlx::Error> {
    let key = account::generate_key();

    match sqlx::query(
        r#"INSERT INTO accounts (user, password, key)
        VALUES (?, ?, ?)"#,
    )
    .bind(user)
    .bind(password)
    .bind(key.clone())
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };

    Ok(key)
}

pub async fn select_user(
    user: String,
    password: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<String, sqlx::Error> {
    let row = match sqlx::query_as::<_, KeyRow>(
        r#"SELECT key FROM accounts
        WHERE user = ? AND password = ?"#,
    )
    .bind(user)
    .bind(password)
    .fetch_one(pool)
    .await
    {
        Ok(row) => row,
        Err(err) => {
            return Err(err);
        }
    };
    Ok(row.key)
}

pub async fn query_user(
    key: String,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<String, sqlx::Error> {
    let row = match sqlx::query_as::<_, UserRow>(
        r#"SELECT user FROM accounts
        WHERE key = ?"#,
    )
    .bind(key)
    .fetch_one(pool)
    .await
    {
        Ok(row) => row,
        Err(err) => {
            return Err(err);
        }
    };

    Ok(row.user)
}

pub fn scan_audiobook_direcories(dir: &Path) -> Vec<audiobook::Audiobook> {
    let mut audiobooks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let dir_path = entry.path();
                if dir_path.is_dir() {
                    if let Some(audiobook) = scan_audiobook_directory(&dir_path) {
                        audiobooks.push(audiobook);
                    }
                }
            }
        }
    }
    audiobooks
}

pub fn scan_audiobook_directory(path: &PathBuf) -> Option<audiobook::Audiobook> {
    let info_file_path = path.join("info.toml");
    let metadata_str = fs::read_to_string(&info_file_path).ok()?;
    let metadata: toml::Value = metadata_str.parse().ok()?;
    let title = metadata.get("title").and_then(|v| v.as_str())?.to_owned();
    let author = metadata.get("author").and_then(|v| v.as_str())?.to_owned();
    let path = path.to_string_lossy().to_string();

    // let cover_path = ["cover.jpg", "cover.jpeg", "cover.png"]
    //     .iter()
    //     .map(|name| path.join(name))
    //     .filter(|path| path.exists())
    //     .next();

    // let audio_exts = ["ogg", "mp3", "webm"];
    // let audio_paths = fs::read_dir(&path)
    //     .ok()?
    //     .filter_map(Result::ok)
    //     .filter(|entry| entry.file_type().ok().map_or(false, |ft| ft.is_file()))
    //     .filter_map(|entry| {
    //         let path = entry.path();
    //         let ext = path.extension()?.to_str()?;
    //         if audio_exts.contains(&ext) {
    //             Some(path)
    //         } else {
    //             None
    //         }
    //     })
    //     .collect::<Vec<_>>();

    let audiobook = audiobook::Audiobook {
        title,
        author,
        path,
    };
    Some(audiobook)
}
