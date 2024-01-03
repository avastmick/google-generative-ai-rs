use std::env;

use google_generative_ai_rs::v1::{api::Client, gemini::ResponseType};
use log::info;

/// Simple text request using the public API and an API key for authn
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] RUST_LOG=info cargo run --package google-generative-ai-rs  --example get_models
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::new_from_response_type(
        ResponseType::GetModelList,
        env::var("API_KEY").unwrap().to_string(),
    );

    let response = client.get_model_list(30).await?;

    info!("{:#?}", response);

    Ok(())
}
