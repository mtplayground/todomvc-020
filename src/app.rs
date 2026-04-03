use leptos::prelude::*;

use crate::components::footer::{Filter, TodoFooter};
use crate::components::header::TodoHeader;
use crate::components::todo_item::TodoItem;
use crate::models::{Todo, ToggleAll};

/// Root application component for the TodoMVC app.
#[component]
pub fn App() -> impl IntoView {
    let (todos, set_todos) = signal(Vec::<Todo>::new());

    // Initialize filter from URL hash
    let initial_hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    let (filter, set_filter) = signal(Filter::from_hash(&initial_hash));

    // Listen for hashchange events
    let set_filter_hash = set_filter;
    leptos::task::spawn_local(async move {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        if let Some(window) = web_sys::window() {
            let closure = Closure::<dyn Fn()>::new(move || {
                if let Some(w) = web_sys::window() {
                    if let Ok(hash) = w.location().hash() {
                        set_filter_hash.set(Filter::from_hash(&hash));
                    }
                }
            });
            let _ = window
                .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    // Fetch initial todos from API
    let set_todos_init = set_todos;
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get("/api/todos").send().await {
            Ok(resp) if resp.ok() => match resp.json::<Vec<Todo>>().await {
                Ok(fetched) => set_todos_init.set(fetched),
                Err(e) => leptos::logging::error!("failed to parse todos: {e}"),
            },
            Ok(resp) => leptos::logging::error!("failed to fetch todos: {}", resp.status()),
            Err(e) => leptos::logging::error!("network error fetching todos: {e}"),
        }
    });

    // Derived signals
    let has_todos = move || !todos.get().is_empty();
    let all_completed = move || {
        let t = todos.get();
        !t.is_empty() && t.iter().all(|todo| todo.completed)
    };

    // Filtered todos based on current filter
    let filtered_todos = move || {
        let all = todos.get();
        match filter.get() {
            Filter::All => all,
            Filter::Active => all.into_iter().filter(|t| !t.completed).collect(),
            Filter::Completed => all.into_iter().filter(|t| t.completed).collect(),
        }
    };

    // Toggle-all handler
    let on_toggle_all = move |_| {
        let new_completed = !all_completed();
        let payload = ToggleAll {
            completed: new_completed,
        };

        leptos::task::spawn_local(async move {
            let result = gloo_net::http::Request::patch("/api/todos/toggle-all")
                .json(&payload)
                .expect("failed to serialize")
                .send()
                .await;

            match result {
                Ok(resp) if resp.ok() => match resp.json::<Vec<Todo>>().await {
                    Ok(updated_todos) => {
                        set_todos.set(updated_todos);
                    }
                    Err(e) => {
                        leptos::logging::error!("failed to parse toggle-all response: {e}");
                    }
                },
                Ok(resp) => {
                    leptos::logging::error!("toggle-all failed: {}", resp.status());
                }
                Err(e) => {
                    leptos::logging::error!("network error toggle-all: {e}");
                }
            }
        });
    };

    view! {
        <section class="todoapp">
            <TodoHeader set_todos=set_todos />
            <section class="main" style:display=move || if has_todos() { "" } else { "none" }>
                <input
                    id="toggle-all"
                    class="toggle-all"
                    type="checkbox"
                    prop:checked=all_completed
                    on:change=on_toggle_all
                />
                <label for="toggle-all">"Mark all as complete"</label>
                <ul class="todo-list">
                    <For
                        each=filtered_todos
                        key=|todo| todo.id
                        let:todo
                    >
                        <TodoItem todo=todo set_todos=set_todos />
                    </For>
                </ul>
            </section>
            <TodoFooter todos=todos set_todos=set_todos filter=filter set_filter=set_filter />
        </section>
        <footer class="info">
            <p>"Double-click to edit a todo"</p>
            <p>"Created with Leptos + Axum"</p>
        </footer>
    }
}
