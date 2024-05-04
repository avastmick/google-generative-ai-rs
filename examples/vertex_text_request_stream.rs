use std::env;
use std::io::{stdout, Write};

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, response::GeminiResponse, Content, Part, Role},
};

/// Streamed text request using Vertex AI API endpoint and GCP application default credentials (ADC) authn
///
/// You'll need to install the GCP cli tools and set up your GCP project and region.
///
/// The ensure you locally authenticated with GCP using the following commands:
/// ```
/// gcloud init
/// gcloud auth application-default login
/// ```
/// To run:
/// ```
/// GCP_REGION_NAME=[THE REGION WHERE YOUR ENDPOINT IS HOSTED] GCP_PROJECT_ID=[YOUR GCP PROJECT_ID] RUST_LOG=info cargo run --package google-generative-ai-rs --example vertex_text_request
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let region = env::var("GCP_REGION_NAME").unwrap().to_string();
    let project_id = env::var("GCP_PROJECT_ID").unwrap().to_string();

    let client = Client::new_from_region_project_id(region.to_string(), project_id.to_string());

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

        #[cfg(feature = "beta")]
        system_instruction: None,
    };

    let response = client.post(30, &txt_request).await?;

    println!("output streaming content");

    if let Some(stream_response) = response.streamed() {
        if let Some(json_stream) = stream_response.response_stream {
            Client::for_each_async(json_stream, move |response: GeminiResponse| async move {
                let mut lock = stdout().lock();
                write!(
                    lock,
                    "{}",
                    response.candidates[0].content.parts[0]
                        .text
                        .clone()
                        .unwrap()
                        .as_str()
                )
                .unwrap();
            })
            .await
        }
    }

    Ok(())
}
