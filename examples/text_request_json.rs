#[cfg(feature = "beta")]
use std::env;

use google_generative_ai_rs::v1::gemini::request::GenerationConfig;

#[cfg(feature = "beta")]
use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, Content, Model, Part, Role},
};

/// JSON-based text request using the public API and an API key for authn
///
/// NOTE: Currently, only available on the v1beta API.
///
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] RUST_LOG=info cargo run -- features "beta" --package google-generative-ai-rs  --example text_request_json
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    #[cfg(not(feature = "beta"))]
    {
        log::error!("JSON-mode only works currently on Gemini 1.5 Pro and on 'beta'");

        Ok(())
    }

    #[cfg(feature = "beta")]
    {
        // Either run as a standard text request or a stream generate content request
        let client = Client::new_from_model(
            Model::Gemini1_5Pro,
            env::var("API_KEY").unwrap().to_string(),
        );

        let prompt = r#"List 5 popular cookie recipes using this JSON schema: 
                        { "type": "object", "properties": { "recipe_name": { "type": "string" }}}"#
            .to_string();

        log::info!("Prompt: {:#?}", prompt);

        let txt_request = Request {
            contents: vec![Content {
                role: Role::User,
                parts: vec![Part {
                    text: Some(prompt),
                    inline_data: None,
                    file_data: None,
                    video_metadata: None,
                }],
            }],
            tools: vec![],
            safety_settings: vec![],
            generation_config: Some(GenerationConfig {
                temperature: None,
                top_p: None,
                top_k: None,
                candidate_count: None,
                max_output_tokens: None,
                stop_sequences: None,
                response_mime_type: Some("application/json".to_string()),
            }),

            system_instruction: None,
        };

        let response = client.post(30, &txt_request).await?;

        log::info!("{:#?}", response);

        Ok(())
    }
}
