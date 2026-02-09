use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::app::TodoItem;

pub const PROJECT_SEPARATOR: &str = "---";

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectNode {
    pub name: String,
    pub full_path: String,
    pub direct_count: usize,
    pub children: Vec<ProjectNode>,
}

#[derive(Default)]
struct TempNode {
    count: usize,
    children: BTreeMap<String, TempNode>,
}

pub fn build_project_tree(todos: &[TodoItem]) -> Vec<ProjectNode> {
    let mut root = BTreeMap::<String, TempNode>::new();

    for todo in todos {
        for project in &todo.projects {
            let parts: Vec<&str> = project.split(PROJECT_SEPARATOR).collect();
            let len = parts.len();
            let mut current = &mut root;
            for (i, part) in parts.into_iter().enumerate() {
                let node = current
                    .entry(part.to_string())
                    .or_insert_with(TempNode::default);
                if i == len - 1 {
                    node.count += 1;
                }
                current = &mut node.children;
            }
        }
    }

    fn convert(map: &BTreeMap<String, TempNode>, prefix: &str) -> Vec<ProjectNode> {
        map.iter()
            .map(|(name, node)| {
                let full_path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}{}{}", prefix, PROJECT_SEPARATOR, name)
                };
                let children = convert(&node.children, &full_path);
                ProjectNode {
                    name: name.clone(),
                    full_path,
                    direct_count: node.count,
                    children,
                }
            })
            .collect()
    }

    convert(&root, "")
}

pub fn render_project_tree(
    nodes: Vec<ProjectNode>,
    depth: usize,
    active_project_filter: ReadSignal<Option<String>>,
    set_active_project_filter: WriteSignal<Option<String>>,
    collapsed_nodes: ReadSignal<std::collections::HashSet<String>>,
    set_collapsed_nodes: WriteSignal<std::collections::HashSet<String>>,
) -> impl IntoView {
    let pad_class = match depth {
        0 => "pl-0",
        1 => "pl-4",
        2 => "pl-8",
        3 => "pl-12",
        _ => "pl-16",
    };

    nodes
        .into_iter()
        .map(move |node| {
            let full_path = node.full_path.clone();
            let full_path_click = full_path.clone();
            let full_path_toggle = full_path.clone();
            let full_path_active = full_path.clone();
            let full_path_collapsed = full_path.clone();
            let has_children = !node.children.is_empty();
            let children = node.children.clone();
            let name = node.name.clone();
            let count = node.direct_count;

            let on_toggle_collapse = move |ev: leptos::ev::MouseEvent| {
                ev.stop_propagation();
                let mut set = collapsed_nodes.get_untracked();
                if set.contains(&full_path_toggle) {
                    set.remove(&full_path_toggle);
                } else {
                    set.insert(full_path_toggle.clone());
                }
                set_collapsed_nodes.set(set);
            };

            let on_click = move |_| {
                set_active_project_filter.set(Some(full_path_click.clone()));
            };

            view! {
                <div>
                    <div
                        class=format!("flex items-center gap-1 px-2 py-1 cursor-pointer rounded hover:bg-base-200 {}", pad_class)
                        class=(
                            "bg-primary/20",
                            {
                                let fp = full_path_active.clone();
                                move || active_project_filter.get().as_deref() == Some(&fp)
                            },
                        )
                        on:click=on_click
                    >
                        // Chevron for expand/collapse
                        {if has_children {
                            view! {
                                <button
                                    class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4"
                                    on:click=on_toggle_collapse
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-3 w-3 transition-transform"
                                        class=(
                                            "rotate-90",
                                            {
                                                let fp = full_path.clone();
                                                move || !collapsed_nodes.get().contains(&fp)
                                            },
                                        )
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </button>
                            }.into_any()
                        } else {
                            view! { <span class="w-4"></span> }.into_any()
                        }}
                        // Folder icon
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
                        </svg>
                        <span class="text-sm truncate flex-1">{name}</span>
                        {if count > 0 {
                            view! { <span class="badge badge-xs badge-neutral">{count}</span> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                    // Children
                    {if has_children {
                        let fp = full_path_collapsed.clone();
                        view! {
                            <div class=("hidden", move || collapsed_nodes.get().contains(&fp))>
                                {render_project_tree(
                                    children.clone(),
                                    depth + 1,
                                    active_project_filter,
                                    set_active_project_filter,
                                    collapsed_nodes,
                                    set_collapsed_nodes,
                                )}
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            }
        })
        .collect_view()
}
