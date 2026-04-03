use leptos::prelude::*;

use crate::models::Todo;

/// Filter state for the todo list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/active" => Filter::Active,
            "#/completed" => Filter::Completed,
            _ => Filter::All,
        }
    }

    pub fn hash(self) -> &'static str {
        match self {
            Filter::All => "#/",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
    }
}

/// Footer showing active item count, filter links, and clear-completed button.
#[component]
pub fn TodoFooter(
    todos: ReadSignal<Vec<Todo>>,
    set_todos: WriteSignal<Vec<Todo>>,
    filter: ReadSignal<Filter>,
    set_filter: WriteSignal<Filter>,
) -> impl IntoView {
    let active_count = move || todos.get().iter().filter(|t| !t.completed).count();
    let has_completed = move || todos.get().iter().any(|t| t.completed);

    let on_clear_completed = move |_| {
        leptos::task::spawn_local(async move {
            let result = gloo_net::http::Request::delete("/api/todos/completed")
                .send()
                .await;

            match result {
                Ok(resp) if resp.ok() || resp.status() == 204 => {
                    set_todos.update(|todos| {
                        todos.retain(|t| !t.completed);
                    });
                }
                Ok(resp) => {
                    leptos::logging::error!("clear completed failed: {}", resp.status());
                }
                Err(e) => {
                    leptos::logging::error!("network error clearing completed: {e}");
                }
            }
        });
    };

    let filter_link = move |f: Filter, label: &'static str| {
        let is_selected = move || filter.get() == f;
        view! {
            <li>
                <a
                    href=f.hash()
                    class:selected=is_selected
                    on:click=move |_| set_filter.set(f)
                >
                    {label}
                </a>
            </li>
        }
    };

    view! {
        <footer class="footer">
            <span class="todo-count">
                <strong>{active_count}</strong>
                {move || if active_count() == 1 { " item left" } else { " items left" }}
            </span>
            <ul class="filters">
                {filter_link(Filter::All, "All")}
                {filter_link(Filter::Active, "Active")}
                {filter_link(Filter::Completed, "Completed")}
            </ul>
            {move || {
                if has_completed() {
                    Some(view! {
                        <button class="clear-completed" on:click=on_clear_completed>
                            "Clear completed"
                        </button>
                    })
                } else {
                    None
                }
            }}
        </footer>
    }
}
