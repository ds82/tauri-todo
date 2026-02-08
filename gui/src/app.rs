use leptos::task::spawn_local;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Deserialize)]
struct TodoItem {
    id: usize,
    subject: String,
    finished: bool,
    priority: u8,
    contexts: Vec<String>,
    projects: Vec<String>,
}

fn priority_label(p: u8) -> Option<&'static str> {
    match p {
        0 => Some("A"),
        1 => Some("B"),
        2 => Some("C"),
        _ => None,
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (todos, set_todos) = signal(Vec::<TodoItem>::new());
    let (error, set_error) = signal(Option::<String>::None);

    let load_todos = move || {
        spawn_local(async move {
            let result = invoke("get_todos", JsValue::NULL).await;
            match serde_wasm_bindgen::from_value::<Vec<TodoItem>>(result) {
                Ok(items) => {
                    set_error.set(None);
                    set_todos.set(items);
                }
                Err(e) => set_error.set(Some(format!("Failed to load todos: {e}"))),
            }
        });
    };

    load_todos();

    view! {
        <main class="min-h-screen bg-base-200 p-8">
            <div class="max-w-2xl mx-auto">
                <h1 class="text-3xl font-bold mb-6">"Todo.txt"</h1>

                {move || error.get().map(|e| view! {
                    <div class="alert alert-error mb-4">
                        <span>{e}</span>
                    </div>
                })}

                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body p-0">
                        <ul class="list">
                            <For
                                each=move || todos.get()
                                key=|item| item.id
                                children=move |item| {
                                    let finished = item.finished;
                                    let subject = item.subject.clone();
                                    let priority = item.priority;
                                    let contexts = item.contexts.clone();
                                    let projects = item.projects.clone();

                                    view! {
                                        <li class="list-row">
                                            <div class="flex items-center gap-3 w-full">
                                                <input
                                                    type="checkbox"
                                                    class="checkbox"
                                                    checked=finished
                                                    disabled=true
                                                />
                                                <div class="flex-1">
                                                    <span class=("line-through opacity-50", finished)>
                                                        {subject.clone()}
                                                    </span>
                                                    <div class="flex gap-1 mt-1">
                                                        {priority_label(priority).map(|p| view! {
                                                            <span class="badge badge-primary badge-sm">{p}</span>
                                                        })}
                                                        {projects.into_iter().map(|p| view! {
                                                            <span class="badge badge-secondary badge-sm">{"+"}{p}</span>
                                                        }).collect::<Vec<_>>()}
                                                        {contexts.into_iter().map(|c| view! {
                                                            <span class="badge badge-accent badge-sm">{"@"}{c}</span>
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            </div>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    </div>
                </div>
            </div>
        </main>
    }
}
