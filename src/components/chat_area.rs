use futures::StreamExt;
use leptos::{html::Div, *};
use models::{Conversation, NewTokenRes};
use tauri_sys::event;

const USER_MESSAGE_CLASS: &str = "max-w-md p-4 mb-5 rounded-lg self-end bg-blue-500 text-white";
const MODEL_MESSAGE_CLASS: &str = "max-w-md p-4 mb-5 rounded-lg self-start bg-gray-200 text-black";
const CHAT_AREA_CLASS: &str =
    "h-screen pb-24 w-full flex flex-col overflow-y-auto border border-gray-300 rounded p-5 \
        bg-gray-100";

#[component]
pub fn ChatArea(conversation: RwSignal<Conversation>) -> impl IntoView {
    let chat_div_ref = create_node_ref::<Div>();
    let (token, set_token) = create_signal(String::new());

    create_effect(move |_| {
        conversation.get();
        if let Some(div) = chat_div_ref.get() {
            div.set_scroll_top(div.scroll_height());
        }
    });

    let memo = create_memo(move |_| token.get());
    create_effect(move |_| {
        if let Some(response) = memo.try_get() {
            conversation.update(move |c| {
                if let Some(last_message) = c.messages.last_mut() {
                    if last_message.text == "..." {
                        last_message.text = response;
                    } else {
                        last_message.text.push_str(&response);
                    }
                }
            });
        }
    });

    create_local_resource(move || set_token, listen_for_tokens);

    view! {
        <div class=CHAT_AREA_CLASS node_ref=chat_div_ref>
        {
            move || conversation.get().messages.iter().map(
                move |message| {
                   let class_str = if message.user { USER_MESSAGE_CLASS } else { MODEL_MESSAGE_CLASS };
                   view! {
                       <div class={ class_str }>
                           {message.text.clone()}
                       </div>
                   }
                }
            ).collect::<Vec<_>>()
        }
        </div>
    }
}

/// Listens for new NewTokenRes and sets the token
async fn listen_for_tokens(writer: WriteSignal<String>) {
    let mut events = event::listen::<NewTokenRes>("new-token").await.unwrap();

    while let Some(event) = events.next().await {
        writer.set(event.payload.token);
    }
}
