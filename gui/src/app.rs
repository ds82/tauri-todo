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

#[derive(Serialize)]
struct ToggleTodoArgs {
    id: usize,
}

#[derive(Serialize)]
struct DeleteTodoArgs {
    id: usize,
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
        <div class="flex h-screen">
            // Sidebar navigation
            <nav class="fixed left-0 top-0 h-full w-16 bg-base-300 flex flex-col items-center py-4 z-50">
                <ul class="menu menu-vertical gap-2">
                    <li>
                        <a class="menu-active tooltip tooltip-right" data-tip="Todos">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"/>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a class="tooltip tooltip-right" data-tip="Add Todo"
                            on:click=move |_| set_dialog_open.set(true)
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                            </svg>
                        </a>
                    </li>
                    <li>
                        <a class="tooltip tooltip-right" data-tip="Settings">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                            </svg>
                        </a>
                    </li>
                </ul>
            </nav>

            // Main content
            <main class="ml-16 flex-1 overflow-y-auto bg-base-200 p-8">
                <div class="max-w-5xl mx-auto">
                    <h1 class="text-3xl font-bold mb-6">"Inbox"</h1>

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
                                        let id = item.id;
                                        let finished = item.finished;
                                        let subject = item.subject.clone();
                                        let priority = item.priority;
                                        let contexts = item.contexts.clone();
                                        let projects = item.projects.clone();

                                        let on_toggle = move |_| {
                                            spawn_local(async move {
                                                let args = serde_wasm_bindgen::to_value(&ToggleTodoArgs { id }).unwrap();
                                                let result = invoke("toggle_todo", args).await;
                                                match serde_wasm_bindgen::from_value::<Vec<TodoItem>>(result) {
                                                    Ok(items) => {
                                                        set_error.set(None);
                                                        set_todos.set(items);
                                                    }
                                                    Err(e) => set_error.set(Some(format!("Failed to toggle todo: {e}"))),
                                                }
                                            });
                                        };

                                        let on_delete = move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            spawn_local(async move {
                                                let args = serde_wasm_bindgen::to_value(&DeleteTodoArgs { id }).unwrap();
                                                let result = invoke("delete_todo", args).await;
                                                match serde_wasm_bindgen::from_value::<Vec<TodoItem>>(result) {
                                                    Ok(items) => {
                                                        set_error.set(None);
                                                        set_todos.set(items);
                                                    }
                                                    Err(e) => set_error.set(Some(format!("Failed to delete todo: {e}"))),
                                                }
                                            });
                                        };

                                        view! {
                                            <li class="list-row group cursor-pointer hover:bg-base-300 transition-colors" >
                                                    <input
                                                        type="checkbox"
                                                        class="checkbox checkbox-accent"
                                                        prop:checked=finished
                                                        on:click=on_toggle
                                                    />
                                                    <div class="">
                                                        <span
                                                            class=("line-through", finished)
                                                            class=("opacity-50", finished)
                                                        >
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


                                                    <button
                                                        class="btn btn-ghost btn-sm opacity-0 group-hover:opacity-80 transition-opacity"
                                                        on:click=on_delete
                                                    >
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                        </svg>
                                                    </button>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </div>
                    </div>
                </div>
            </main>
        </div>

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
