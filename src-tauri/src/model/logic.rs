use std::{convert::Infallible, path::PathBuf};

use llm::{InferenceFeedback, InferenceResponse, InferenceSession};
use models::NewTokenRes;
use tauri::{Manager, Window};

use super::Model;

static CHARACTER_NAME: &str = "### Assistant";
static USER_NAME: &str = "### Human";

/// Helper function to start the session for a human / assistant chat
fn start_session(model: &Box<dyn llm::Model>) -> InferenceSession {
    let persona = "A chat between a human and an assistant.";
    let history = format!(
        "{CHARACTER_NAME}\n\
        Hello - How may I help you today?\n\n\
        \
        {USER_NAME}\n\
        What is the capital of France?\n\n\
        \
        {CHARACTER_NAME}\n\
        Paris is the capital of France.\n\n\
        \
        "
    );
    let mut session = model.start_session(Default::default());
    session
        .feed_prompt(
            model.as_ref(),
            format!("{persona}\n{history}").as_str(),
            &mut Default::default(),
            llm::feed_prompt_callback(|_| {
                Ok::<llm::InferenceFeedback, Infallible>(llm::InferenceFeedback::Continue)
            }),
        )
        .expect("Failed to ingest initial prompt.");
    session
}

#[tauri::command]
pub async fn predict(
    window: Window,
    text: String,
    state: tauri::State<'_, Model>,
) -> Result<(), String> {
    let model = state.model.lock().unwrap();
    let model = model.as_ref().unwrap();

    let mut lock = state.session.lock().unwrap();
    let mut session = lock.take().unwrap_or_else(|| start_session(model));

    session
        .infer::<Infallible>(
            model.as_ref(),
            &mut rand::thread_rng(),
            &llm::InferenceRequest {
                prompt: format!("{USER_NAME}\n{text}\n\n{CHARACTER_NAME}\n")
                    .as_str()
                    .into(),
                parameters: &llm::InferenceParameters::default(),
                play_back_previous_tokens: false,
                maximum_token_count: None,
            },
            &mut Default::default(),
            conversation_inference_callback(USER_NAME, move |token| {
                window.emit_all("new-token", NewTokenRes { token }).unwrap();
            }),
        )
        .unwrap_or_else(|e| panic!("{e}"));

    *lock = Some(session);
    Ok(())
}

#[tauri::command]
/// Loads the LLM model at the path provided by the frontend. Currently only supporting
/// Llama models
pub fn load_llm_model(path: String, state: tauri::State<'_, Model>) -> Result<(), ()> {
    let path = PathBuf::from(path);
    let model_parameters = llm::ModelParameters {
        prefer_mmap: true,
        context_size: 2048,
        lora_adapters: None,
        use_gpu: true,
        gpu_layers: None,
        rope_overrides: None,
        n_gqa: None,
    };
    match llm::load_dynamic(
        Some(llm::ModelArchitecture::Llama),
        &path,
        llm::TokenizerSource::Embedded,
        model_parameters,
        llm::load_progress_callback_stdout,
    ) {
        Ok(model) => {
            *state.model.lock().unwrap() = Some(model);
            *state.session.lock().unwrap() = None; // Reset session
            Ok(())
        }
        Err(_) => Err(()),
    }
}

/// Copy of the conversation_inference_callback function within the llm crate but fixes an issue
/// where tokens starting with whitespace won't match the stop_sequence
pub fn conversation_inference_callback<'a, E: std::error::Error + Send + Sync + 'static>(
    stop_sequence: &'a str,
    mut callback: impl FnMut(String) + 'a,
) -> impl FnMut(InferenceResponse) -> Result<InferenceFeedback, E> + 'a {
    let mut stop_sequence_buf = String::new();
    move |resp| match resp {
        InferenceResponse::InferredToken(token) => {
            // We've generated a token, so we need to check if it's contained in the stop sequence.
            let mut buf = stop_sequence_buf.clone();
            buf.push_str(&token);

            if !buf.trim().is_empty() {
                if buf.trim().starts_with(stop_sequence) {
                    // We've generated the stop sequence, so we're done.
                    // Note that this will contain the extra tokens that were generated after the stop sequence,
                    // which may affect generation. This is non-ideal, but it's the best we can do without
                    // modifying the model.
                    stop_sequence_buf.clear();
                    return Ok(InferenceFeedback::Halt);
                } else if stop_sequence.starts_with(&buf.trim()) {
                    // We've generated a prefix of the stop sequence, so we need to keep buffering.
                    stop_sequence_buf = buf;
                    return Ok(InferenceFeedback::Continue);
                }
            }

            // We've generated a token that isn't part of the stop sequence, so we can
            // pass it to the callback.
            stop_sequence_buf.clear();
            callback(buf);
            Ok(InferenceFeedback::Continue)
        }
        InferenceResponse::EotToken => Ok(InferenceFeedback::Halt),
        _ => Ok(InferenceFeedback::Continue),
    }
}
