use std::env;

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, Content, Part, ResponseType, Role},
};
use log::info;

/// Counts the tokens used in a prompt using the public API and an API key for authn
/// See: `https://ai.google.dev/tutorials/rest_quickstart#count_tokens`
/// You'll need to install the GCP cli tools and set up your GCP project and region.
///
/// The ensure you locally authenticated with GCP using the following commands:
/// ```
/// gcloud init
/// gcloud auth application-default login
/// ```
///
/// To run:
/// ```
/// GCP_REGION_NAME=[THE REGION WHERE YOUR ENDPOINT IS HOSTED] GCP_PROJECT_ID=[YOUR GCP PROJECT_ID] RUST_LOG=info cargo run --package google-generative-ai-rs  --example vertex_count_tokens
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let region = env::var("GCP_REGION_NAME").unwrap().to_string();
    let project_id = env::var("GCP_PROJECT_ID").unwrap().to_string();

    let client = Client::new_from_region_project_id_response_type(
        region.to_string(),
        project_id.to_string(),
        ResponseType::CountTokens,
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
