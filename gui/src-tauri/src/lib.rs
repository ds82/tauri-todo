use serde::Serialize;
use todotxt::TodoList;

#[derive(Serialize)]
struct TodoResponse {
    id: usize,
    subject: String,
    finished: bool,
    priority: u8,
    contexts: Vec<String>,
    projects: Vec<String>,
}

#[tauri::command]
fn get_todos() -> Result<Vec<TodoResponse>, String> {
    let todo_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../todo.txt");
    let list = TodoList::from_file(todo_path).map_err(|e| e.to_string())?;

    Ok(list
        .items()
        .iter()
        .map(|item| TodoResponse {
            id: item.id,
            subject: item.subject().to_string(),
            finished: item.finished(),
            priority: item.priority(),
            contexts: item.contexts().to_vec(),
            projects: item.projects().to_vec(),
        })
        .collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_todos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
