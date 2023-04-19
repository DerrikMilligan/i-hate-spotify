use std::sync::RwLock;

use reqwest;
use sqlite::Connection;
use sqlite::State;

pub struct InnerAppState {
    pub db: Connection,
}

// Needed to be able to aquite the locks for SQLite
unsafe impl Send for InnerAppState {}
unsafe impl Sync for InnerAppState {}

// impl InnerAppState {
//     pub fn reset(&mut self) {
//         // do stuff
//         sqlite::open(":memory:").unwrap();
//     }
// }

pub struct AppState(pub RwLock<InnerAppState>);

// A custom error type that represents all possible in our command
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to make the request: {0}")]
    Reqwest(#[from] reqwest::Error),
}

// we must also implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
async fn hello_command(name: String) -> Result<String, Error> {
    let body: String = reqwest::get("https://randomuser.me/api/")
        .await?
        .text()
        .await?;

    Ok(body.into())
}

#[tauri::command]
async fn create_user(name: String, state: tauri::State<'_, AppState>) -> Result<String, Error> {
    let state= state.0.write().unwrap();

    let _ = state.db.execute("
        CREATE TABLE IF NOT EXISTS Users (
            id INTPRIMARY KEY,
            name VARCHAR(255)
        );
    ");

    let _ = state.db.execute(format!("
        INSERT INTO Users (name) VALUES 
        (\"{name}\")
    "));

    Ok("success".to_string())
}

#[tauri::command]
async fn get_users(state: tauri::State<'_, AppState>) -> Result<Vec<String>, Error> {
    let state = state.0.read().unwrap();

    let mut statement = state.db.prepare("SELECT * FROM Users;").unwrap();

    let mut users : Vec<String> = Vec::new();

    while let Ok(State::Row) = statement.next() {
        let name = statement.read::<String, _>("name").unwrap();
        users.push(name);
    }

    Ok(users)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(RwLock::new(InnerAppState{ db: sqlite::open(":memory:").unwrap() })))
        .invoke_handler(tauri::generate_handler![
            hello_command,
            create_user,
            get_users
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
