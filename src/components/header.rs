use leptos::prelude::*;
use web_sys::KeyboardEvent;

use crate::models::{CreateTodo, Todo};

/// Header component with the "todos" title and new-todo input field.
/// On Enter, posts a new todo to the API and refreshes the list.
#[component]
pub fn TodoHeader(set_todos: WriteSignal<Vec<Todo>>) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());

    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            let title = input_value.get().trim().to_string();
            if title.is_empty() {
                return;
            }
            set_input_value.set(String::new());

            let title_clone = title.clone();
            leptos::task::spawn_local(async move {
                let payload = CreateTodo { title: title_clone };
                let result = gloo_net::http::Request::post("/api/todos")
                    .json(&payload)
                    .expect("failed to serialize")
                    .send()
                    .await;

                match result {
                    Ok(resp) if resp.ok() => match resp.json::<Todo>().await {
                        Ok(todo) => {
                            set_todos.update(|todos| todos.push(todo));
                        }
                        Err(e) => {
                            leptos::logging::error!("failed to parse todo: {e}");
                        }
                    },
                    Ok(resp) => {
                        leptos::logging::error!("create todo failed: {}", resp.status());
                    }
                    Err(e) => {
                        leptos::logging::error!("network error: {e}");
                    }
                }
            });
        }
    };

    view! {
        <header class="header">
            <h1>"todos"</h1>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                autofocus=true
                prop:value=input_value
                on:input=move |ev| {
                    set_input_value.set(event_target_value(&ev));
                }
                on:keydown=on_keydown
            />
        </header>
    }
}
