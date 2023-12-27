use std::env;

use google_generative_ai_api_rs::v1::api::Client;

/// Simple text request using the public API and an API key for authn
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] cargo run --package google-generative-ai-api-rs  --example text_request
/// ``
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("API_KEY").unwrap().to_string());

    println!("{}", client);

    Ok(())
}
