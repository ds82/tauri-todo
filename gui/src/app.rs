use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
struct AddTodoArgs<'a> {
    text: &'a str,
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
    let (dialog_open, set_dialog_open) = signal(false);
    let (new_todo, set_new_todo) = signal(String::new());

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

    let on_add_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let text = new_todo.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&AddTodoArgs { text: &text }).unwrap();
            let result = invoke("add_todo", args).await;
            match serde_wasm_bindgen::from_value::<Vec<TodoItem>>(result) {
                Ok(items) => {
                    set_error.set(None);
                    set_todos.set(items);
                    set_new_todo.set(String::new());
                    set_dialog_open.set(false);
                }
                Err(e) => set_error.set(Some(format!("Failed to add todo: {e}"))),
            }
        });
    };

    view! {
        <main class="min-h-screen bg-base-200 p-8">
            <div class="max-w-2xl mx-auto">
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-3xl font-bold">"Todo.txt"</h1>
                    <button
                        class="btn btn-primary"
                        on:click=move |_| set_dialog_open.set(true)
                    >
                        "+"
                    </button>
                </div>

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

        <dialog class="modal" class:modal-open=move || dialog_open.get()>
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Add Todo"</h3>
                <form on:submit=on_add_submit>
                    <div class="form-control mt-4">
                        <input
                            type="text"
                            placeholder="e.g. (A) Buy milk @errands +shopping"
                            class="input input-bordered w-full"
                            prop:value=move || new_todo.get()
                            on:input=move |ev| set_new_todo.set(event_target_value(&ev))
                        />
                        <p class="label text-xs opacity-60">
                            "Use todo.txt format: (A) priority, @context, +project"
                        </p>
                    </div>
                    <div class="modal-action">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| {
                                set_new_todo.set(String::new());
                                set_dialog_open.set(false);
                            }
                        >
                            "Cancel"
                        </button>
                        <button type="submit" class="btn btn-primary">"Add"</button>
                    </div>
                </form>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button
                    type="button"
                    on:click=move |_| {
                        set_new_todo.set(String::new());
                        set_dialog_open.set(false);
                    }
                />
            </form>
        </dialog>
    }
}
