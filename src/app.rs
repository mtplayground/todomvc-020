use leptos::prelude::*;

/// Root application component for the TodoMVC app.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <section class="min-h-screen bg-gray-100 flex flex-col items-center">
            <header class="mt-8">
                <h1 class="text-7xl font-thin text-red-300">"todos"</h1>
            </header>
        </section>
        <footer class="text-center text-gray-500 text-sm mt-4">
            <p>"Double-click to edit a todo"</p>
            <p>"Created with Leptos + Axum"</p>
        </footer>
    }
}
