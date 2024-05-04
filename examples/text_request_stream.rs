use std::env;
use std::io::{stdout, Write};

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, response::GeminiResponse, Content, Part, ResponseType, Role},
};

/// Simple text request using the public API and an API key for authn
/// To run:
/// ```
/// API_KEY=[YOUR_API_KEY] RUST_LOG=info cargo run --package google-generative-ai-rs  --example text_request
/// ``
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let token = match env::var("API_KEY") {
        Ok(v) => v,
        Err(e) => {
            let msg = "$API_KEY not found".to_string();
            panic!("{e:?}:{msg}");
        }
    };

    // Either run as a standard text request or a stream generate content request
    let client = Client::new_from_model_response_type(
        google_generative_ai_rs::v1::gemini::Model::GeminiPro,
        token.clone(),
        ResponseType::StreamGenerateContent,
    );

    println!("token {:#?}", token);

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
