use std::env;

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, Content, Part, ResponseType, Role},
};
use log::info;

/// Counts the tokens used in a prompt using the public API and an API key for authn
/// See: `https://ai.google.dev/tutorials/rest_quickstart#count_tokens`
///
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] RUST_LOG=info cargo run --package google-generative-ai-rs  --example count_tokens
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::new_from_response_type(
        ResponseType::CountTokens,
        env::var("API_KEY").unwrap().to_string(),
    );

    let txt_request = Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: Some("Write a story about a magic backpack.".to_string()),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
        }],
        tools: vec![],
        safety_settings: vec![],
        generation_config: None,

        #[cfg(feature = "beta")]
        system_instruction: None,
    };

    let response = client.post(30, &txt_request).await?;

    info!("{:#?}", response);

    Ok(())
}
