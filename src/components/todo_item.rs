use leptos::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent};

use crate::models::{Todo, UpdateTodo};

/// Renders a single todo item with checkbox toggle, destroy button, and inline editing.
/// Double-click the label to edit. Enter saves, Escape cancels, blank save deletes.
#[component]
pub fn TodoItem(todo: Todo, set_todos: WriteSignal<Vec<Todo>>) -> impl IntoView {
    let todo_id = todo.id;
    let initial_title = todo.title.clone();
    let (completed, set_completed) = signal(todo.completed);
    let (title, set_title) = signal(initial_title.clone());
    let (editing, set_editing) = signal(false);
    let (edit_text, set_edit_text) = signal(initial_title);
    let edit_input_ref = NodeRef::<leptos::html::Input>::new();

    // Toggle completed
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

    // Delete todo
    let delete_todo = move || {
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

    let on_destroy = move |_| {
        delete_todo();
    };

    // Double-click to enter edit mode with focus management
    let on_dblclick = move |_| {
        set_edit_text.set(title.get());
        set_editing.set(true);
        // Focus the edit input after it renders
        request_animation_frame(move || {
            if let Some(input) = edit_input_ref.get() {
                let el: &HtmlInputElement = &input;
                let _ = el.focus();
                // Move cursor to end of text
                let len = el.value().len() as u32;
                let _ = el.set_selection_range(len, len);
            }
        });
    };

    // Save edit: if blank, delete; otherwise PATCH title
    let save_edit = move || {
        let trimmed = edit_text.get().trim().to_string();
        set_editing.set(false);

        if trimmed.is_empty() {
            delete_todo();
            return;
        }

        if trimmed == title.get() {
            return;
        }

        set_title.set(trimmed.clone());

        let payload = UpdateTodo {
            title: Some(trimmed),
            completed: None,
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
                        set_title.set(updated.title.clone());
                        set_todos.update(|todos| {
                            if let Some(t) = todos.iter_mut().find(|t| t.id == todo_id) {
                                *t = updated;
                            }
                        });
                    }
                }
                Ok(resp) => {
                    leptos::logging::error!("edit save failed: {}", resp.status());
                }
                Err(e) => {
                    leptos::logging::error!("network error saving edit: {e}");
                }
            }
        });
    };

    // Cancel edit
    let cancel_edit = move || {
        set_edit_text.set(title.get());
        set_editing.set(false);
    };

    let on_keydown = move |ev: KeyboardEvent| match ev.key().as_str() {
        "Enter" => save_edit(),
        "Escape" => cancel_edit(),
        _ => {}
    };

    let on_blur = move |_| {
        if editing.get() {
            save_edit();
        }
    };

    view! {
        <li
            class:completed=move || completed.get()
            class:editing=move || editing.get()
        >
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=move || completed.get()
                    on:change=on_toggle
                />
                <label on:dblclick=on_dblclick>{move || title.get()}</label>
                <button class="destroy" on:click=on_destroy></button>
            </div>
            {move || {
                if editing.get() {
                    Some(view! {
                        <input
                            class="edit"
                            node_ref=edit_input_ref
                            prop:value=move || edit_text.get()
                            on:input=move |ev| {
                                set_edit_text.set(event_target_value(&ev));
                            }
                            on:keydown=on_keydown
                            on:blur=on_blur
                        />
                    })
                } else {
                    None
                }
            }}
        </li>
    }
}
