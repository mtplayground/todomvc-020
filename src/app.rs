use leptos::prelude::*;

use crate::components::header::TodoHeader;
use crate::components::todo_item::TodoItem;
use crate::models::Todo;

/// Root application component for the TodoMVC app.
#[component]
pub fn App() -> impl IntoView {
    let (todos, set_todos) = signal(Vec::<Todo>::new());

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

    view! {
        <section class="todoapp">
            <TodoHeader set_todos=set_todos />
            <section class="main">
                <ul class="todo-list">
                    <For
                        each=move || todos.get()
                        key=|todo| todo.id
                        let:todo
                    >
                        <TodoItem todo=todo set_todos=set_todos />
                    </For>
                </ul>
            </section>
        </section>
        <footer class="info">
            <p>"Double-click to edit a todo"</p>
            <p>"Created with Leptos + Axum"</p>
        </footer>
    }
}
