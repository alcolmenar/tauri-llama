use leptos::*;

use crate::components::{chat_area::ChatArea, input_area::InputArea, settings_bar::SettingsBar};
use models::Conversation;

#[component]
pub fn App() -> impl IntoView {
    let conversation = create_rw_signal(Conversation::new());
    let valid_model = create_rw_signal(false);

    view! {
        <SettingsBar valid_model conversation />
        <main class="container">
            <Show when=move || valid_model.get() >
                <ChatArea conversation />
                <InputArea conversation />
            </Show>
        </main>
    }
}
