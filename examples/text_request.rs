use log::info;
use std::env;

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, Content, Part, Role},
};

/// Simple text request using the public API and an API key for authn
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] RUST_LOG=info cargo run --package google-generative-ai-rs  --example text_request
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Either run as a standard text request or a stream generate content request
    let client = Client::new(env::var("API_KEY").unwrap().to_string());

    // See Issue #12 - currently this gives a '403 Forbidden' error
    // let client = Client::new_from_model_reponse_type(
    //     google_generative_ai_rs::v1::gemini::Model::GeminiPro,
    //     env::var("API_KEY").unwrap().to_string(),
    //     google_generative_ai_rs::v1::gemini::ResponseType::StreamGenerateContent,
    // );

    let txt_request = Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: Some("Give me a recipe for banana bread.".to_string()),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
        }],
        tools: vec![],
        safety_settings: vec![],
        generation_config: None,
    };

    let response = client.post(30, &txt_request).await?;

    info!("{:#?}", response);

    Ok(())
}
