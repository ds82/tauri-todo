use serde::Serialize;
use todotxt::TodoList;

const TODO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../todo.txt");

#[derive(Serialize)]
struct TodoResponse {
    id: usize,
    subject: String,
    finished: bool,
    priority: u8,
    contexts: Vec<String>,
    projects: Vec<String>,
}

fn to_response(list: &TodoList) -> Vec<TodoResponse> {
    list.items()
        .iter()
        .map(|item| TodoResponse {
            id: item.id,
            subject: item.subject().to_string(),
            finished: item.finished(),
            priority: item.priority(),
            contexts: item.contexts().to_vec(),
            projects: item.projects().to_vec(),
        })
        .collect()
}

#[tauri::command]
fn get_todos() -> Result<Vec<TodoResponse>, String> {
    let list = TodoList::from_file(TODO_PATH).map_err(|e| e.to_string())?;
    Ok(to_response(&list))
}

#[tauri::command]
fn add_todo(text: &str) -> Result<Vec<TodoResponse>, String> {
    let mut list = TodoList::from_file(TODO_PATH).map_err(|e| e.to_string())?;
    list.add(text);
    list.save().map_err(|e| e.to_string())?;
    Ok(to_response(&list))
}

#[tauri::command]
fn toggle_todo(id: usize) -> Result<Vec<TodoResponse>, String> {
    let mut list = TodoList::from_file(TODO_PATH).map_err(|e| e.to_string())?;
    let item = list.get(id).ok_or("Todo not found")?;
    if item.finished() {
        list.uncomplete(id);
    } else {
        list.complete(id);
    }
    list.save().map_err(|e| e.to_string())?;
    Ok(to_response(&list))
}

#[tauri::command]
fn delete_todo(id: usize) -> Result<Vec<TodoResponse>, String> {
    let mut list = TodoList::from_file(TODO_PATH).map_err(|e| e.to_string())?;
    list.remove(id).ok_or("Todo not found")?;
    list.save().map_err(|e| e.to_string())?;
    Ok(to_response(&list))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_todos, add_todo, toggle_todo, delete_todo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
