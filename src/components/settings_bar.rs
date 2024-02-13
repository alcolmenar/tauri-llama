use leptos::{html::Input, *};
use leptos_heroicons::size_24::outline::{Cog, XMark};

use models::{Conversation, ModelPath};
use tauri_sys::{dialog::FileDialogBuilder, tauri::invoke};

#[component]
pub fn SettingsBar(
    valid_model: RwSignal<bool>,
    conversation: RwSignal<Conversation>,
) -> impl IntoView {
    let input_ref = create_node_ref::<Input>();
    let expanded = create_rw_signal(false);
    let show = Signal::derive(move || expanded.get() || !valid_model.get());
    let show_str = Signal::derive(move || show.get().to_string()); // data-expanded requires a str

    let file_chooser = create_action(|_task: &String| async move {
        FileDialogBuilder::new()
            .add_filter("Models", &["bin"])
            .pick_file()
            .await
            .unwrap()
            .unwrap()
    });

    let load_model = create_action(|path: &String| {
        let path = path.clone();
        async move {
            invoke::<ModelPath, ()>("load_llm_model", &ModelPath { path })
                .await
                .map(|_| ())
                .map_err(|_| ())
        }
    });

    create_effect(move |_| {
        if let Some(res) = load_model.value().get() {
            match res {
                Ok(_) => {
                    valid_model.set(true);
                    conversation.set(Conversation::new());
                }
                Err(_) => {
                    let input = input_ref.get().unwrap();
                    input.set_value("");
                    valid_model.set(false);
                }
            }
        }
    });

    create_effect(move |_| {
        if let Some(file) = file_chooser.value().get() {
            let input = input_ref.get().unwrap();
            input.set_value(file.to_str().unwrap());
        }
    });

    view! {
        <div data-expanded=show_str class=SETTINGS_BAR_CLASS>
            <SettingsIcon expanded show />
            <label class=LABEL_CLASS>Model file</label>
            <input class=PATH_CLASS type="text"
                node_ref=input_ref
                on:click=move |_| file_chooser.dispatch("".to_string())
            />
            <div class="flex justify-center">
                <input
                    class=LOAD_MODEL_CLASS
                    on:click=move |_| {
                        let input = input_ref.get().unwrap();
                        load_model.dispatch(input.value());
                    }
                    type="submit"
                    value="Load model"
                />
            </div>
        </div>
    }
}

#[component]
pub fn SettingsIcon(expanded: RwSignal<bool>, show: Signal<bool>) -> impl IntoView {
    let toggle = move |_| {
        expanded.update(|prev| *prev = !*prev);
    };

    view! {
        <div
            on:click=toggle
            class=ICON_CLASS>
            { move || {
                if show.get() {
                    view! { <XMark /> }
                } else {
                    view! { <Cog /> }
                }
            }}
        </div>
    }
}

pub const SETTINGS_BAR_CLASS: &str = "absolute w-1/2 transition-expanded ease-in-out \
duration-300 bg-gray-200 border-t border-r border-b border-gray-300 rounded-r-lg z-10 p-4 \
justify-center

data-[expanded=true]:left-0
data-[expanded=false]:-left-1/2";

pub const ICON_CLASS: &str = "relative float-right h-10 w-10 -right-14 -top-4";
pub const LABEL_CLASS: &str = "block mb-2 text-sm font-medium text-gray-900 dark:text-white";

pub const PATH_CLASS: &str =
    "block w-full mb-5 text-xs text-gray-900 border border-gray-300 rounded-lg cursor-pointer \
        bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 \
        dark:placeholder-gray-400";
pub const LOAD_MODEL_CLASS: &str = "p-2 bg-blue-500 text-white rounded-md cursor-pointer";
