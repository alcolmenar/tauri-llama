use leptos::{html::Input, *};
use models::{Conversation, Message};
use tauri_sys::tauri::invoke;

#[component]
pub fn InputArea(conversation: RwSignal<Conversation>) -> impl IntoView {
    let input_ref = create_node_ref::<Input>();
    let has_input = create_rw_signal(String::new());
    let memo_has_input = create_memo(move |_| !has_input.get().is_empty());

    let send: Action<String, ()> = create_action(move |new_message: &String| {
        let message = Message {
            user: true,
            text: new_message.clone(),
        };

        let message2 = message.clone();

        conversation.update(move |c| {
            c.messages.push(message2);
        });

        async move { invoke("predict", &message).await.unwrap() }
    });

    create_effect(move |_| {
        if send.input().get().is_some() {
            let model_message = Message {
                user: false,
                text: String::from("..."),
            };

            conversation.update(move |c| {
                c.messages.push(model_message);
            });
        }
    });

    view! {
        <div class="h-24 w-full fixed bottom-0 flex justify-center items-center p-5 bg-white border-t border-gray-300">
            <form class="w-full flex justify-center items-center gap-4" on:submit=move |ev| {
                ev.prevent_default();
                let input = input_ref.get().filter(|val| !val.value().is_empty()).expect("input to exist");
                send.dispatch(input.value());
                input.set_value("");
            }>
            <input
                class="w-2/3 p-4 border border-gray-300 rounded-md"
                type="text"
                node_ref=input_ref
                on:input=move |ev| has_input.set(event_target_value(&ev))
            />
            <input
                class="min-w-32 h-full p-4 bg-blue-500 text-white rounded-md cursor-pointer disabled:opacity-75"
                type="submit"
                disabled=move || send.pending().get() || !memo_has_input.get()
                value=move || if send.pending().get() { "Predicting" } else { "Submit" }
            />
            </form>
        </div>
    }
}
