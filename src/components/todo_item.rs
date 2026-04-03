use leptos::prelude::*;

use crate::models::{Todo, UpdateTodo};

/// Renders a single todo item with checkbox toggle and destroy button.
#[component]
pub fn TodoItem(todo: Todo, set_todos: WriteSignal<Vec<Todo>>) -> impl IntoView {
    let todo_id = todo.id;
    let initial_completed = todo.completed;
    let (completed, set_completed) = signal(initial_completed);

    let on_toggle = move |_| {
        let new_completed = !completed.get();
        set_completed.set(new_completed);

        let payload = UpdateTodo {
            completed: Some(new_completed),
            title: None,
            display_order: None,
        };

        leptos::task::spawn_local(async move {
            let result = gloo_net::http::Request::patch(&format!("/api/todos/{todo_id}"))
                .json(&payload)
                .expect("failed to serialize")
                .send()
                .await;

            match result {
                Ok(resp) if resp.ok() => {
                    if let Ok(updated) = resp.json::<Todo>().await {
                        set_todos.update(|todos| {
                            if let Some(t) = todos.iter_mut().find(|t| t.id == todo_id) {
                                *t = updated;
                            }
                        });
                    }
                }
                Ok(resp) => {
                    leptos::logging::error!("toggle failed: {}", resp.status());
                    set_completed.set(!new_completed);
                }
                Err(e) => {
                    leptos::logging::error!("network error toggling: {e}");
                    set_completed.set(!new_completed);
                }
            }
        });
    };

    let on_destroy = move |_| {
        leptos::task::spawn_local(async move {
            let result = gloo_net::http::Request::delete(&format!("/api/todos/{todo_id}"))
                .send()
                .await;

            match result {
                Ok(resp) if resp.ok() || resp.status() == 204 => {
                    set_todos.update(|todos| {
                        todos.retain(|t| t.id != todo_id);
                    });
                }
                Ok(resp) => {
                    leptos::logging::error!("delete failed: {}", resp.status());
                }
                Err(e) => {
                    leptos::logging::error!("network error deleting: {e}");
                }
            }
        });
    };

    view! {
        <li class:completed=move || completed.get()>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=move || completed.get()
                    on:change=on_toggle
                />
                <label>{todo.title.clone()}</label>
                <button class="destroy" on:click=on_destroy></button>
            </div>
        </li>
    }
}
