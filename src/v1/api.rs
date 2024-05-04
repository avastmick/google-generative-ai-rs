//! Manages the interaction with the REST API for the Gemini API.
use futures::prelude::*;
use futures::stream::StreamExt;
use reqwest_streams::error::StreamBodyError;
use reqwest_streams::*;
use serde_json;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::v1::errors::GoogleAPIError;
use crate::v1::gemini::request::Request;
use crate::v1::gemini::response::GeminiResponse;
use crate::v1::gemini::Model;

use super::gemini::response::{StreamedGeminiResponse, TokenCount};
use super::gemini::{ModelInformation, ModelInformationList, ResponseType};

#[cfg(feature = "beta")]
const PUBLIC_API_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

#[cfg(not(feature = "beta"))]
const PUBLIC_API_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1";

/// Enables a streamed or non-streamed response to be returned from the API.
#[derive(Debug)]
pub enum PostResult {
    Rest(GeminiResponse),
    Streamed(StreamedGeminiResponse),
    Count(TokenCount),
}
impl PostResult {
    pub fn rest(self) -> Option<GeminiResponse> {
        match self {
            PostResult::Rest(response) => Some(response),
            _ => None,
        }
    }
    pub fn streamed(self) -> Option<StreamedGeminiResponse> {
        match self {
            PostResult::Streamed(streamed_response) => Some(streamed_response),
            _ => None,
        }
    }
    pub fn count(self) -> Option<TokenCount> {
        match self {
            PostResult::Count(response) => Some(response),
            _ => None,
        }
    }
}

/// Manages the specific API connection
pub struct Client {
    pub url: String,
    pub model: Model,
    pub region: Option<String>,
    pub project_id: Option<String>,
    pub response_type: ResponseType,
}

/// Implements the functions for the API client.
/// TODO: This is getting unwieldy. We need to refactor this into a more manageable state.
///         See Issue #26 - 'Code tidy and improvement'
impl Client {
    /// Creates a default new public API client.
    pub fn new(api_key: String) -> Self {
        let url = Url::new(&Model::default(), api_key, &ResponseType::GenerateContent);
        Self {
            url: url.url,
            model: Model::default(),
            region: None,
            project_id: None,
            response_type: ResponseType::GenerateContent,
        }
    }

    /// Creates a default new public API client for a specified response type.
    pub fn new_from_response_type(response_type: ResponseType, api_key: String) -> Self {
        let url = Url::new(&Model::default(), api_key, &response_type);
        Self {
            url: url.url,
            model: Model::default(),
            region: None,
            project_id: None,
            response_type,
        }
    }

    /// Create a new public API client for a specified model.
    pub fn new_from_model(model: Model, api_key: String) -> Self {
        let url = Url::new(&model, api_key, &ResponseType::GenerateContent);
        Self {
            url: url.url,
            model,
            region: None,
            project_id: None,
            response_type: ResponseType::GenerateContent,
        }
    }

    /// Create a new public API client for a specified model.
    pub fn new_from_model_response_type(
        model: Model,
        api_key: String,
        response_type: ResponseType,
    ) -> Self {
        let url = Url::new(&model, api_key, &response_type);
        Self {
            url: url.url,
            model,
            region: None,
            project_id: None,
            response_type,
        }
    }

    // post
    pub async fn post(
        &self,
        timeout: u64,
        api_request: &Request,
    ) -> Result<PostResult, GoogleAPIError> {
        let client: reqwest::Client = self.get_reqwest_client(timeout)?;
        match self.response_type {
            ResponseType::GenerateContent => {
                let result = self.get_post_result(client, api_request).await?;
                Ok(PostResult::Rest(result))
            }
            ResponseType::StreamGenerateContent => {
                let result = self.get_streamed_post_result(client, api_request).await?;
                Ok(PostResult::Streamed(result))
            }
            ResponseType::CountTokens => {
                let result = self.get_token_count(client, api_request).await?;
                Ok(PostResult::Count(result))
            }
            _ => Err(GoogleAPIError {
                message: format!("Unsupported response type: {:?}", self.response_type),
                code: None,
            }),
        }
    }

    /// A standard post request, i.e., not streamed
    async fn get_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<GeminiResponse, GoogleAPIError> {
        let token_option = self.get_auth_token_option().await?;

        let result = self
            .get_post_response(client, api_request, token_option)
            .await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => Ok(response.json::<GeminiResponse>().await.map_err(|e|GoogleAPIError {
                message: format!(
                        "Failed to deserialize API response into v1::gemini::response::GeminiResponse: {}",
                        e
                    ),
                code: None,
            })?),
                _ => Err(self.new_error_from_status_code(response.status())),
            },
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }
    // Define the function that accepts the stream and the consumer
    /// A streamed post request
    async fn get_streamed_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<StreamedGeminiResponse, GoogleAPIError> {
        let token_option = self.get_auth_token_option().await?;

        let result = self
            .get_post_response(client, api_request, token_option)
            .await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    // Wire to enable introspection on the response stream
                    let json_stream = response.json_array_stream::<serde_json::Value>(2048); //TODO what is a good length?;

                    Ok(StreamedGeminiResponse {
                        response_stream: Some(json_stream),
                    })
                }
                _ => Err(self.new_error_from_status_code(response.status())),
            },
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }

    /// Applies an asynchronous operation to each item in a stream, potentially concurrently.
    ///
    /// This function retrieves each item from the provided stream, processes it using the given
    /// consumer callback, and awaits the futures produced by the consumer. The concurrency level
    /// is unbounded, meaning items will be processed as soon as they are ready without a limit.
    ///
    /// # Type Parameters
    ///
    /// - `F`: The type of the consumer closure. It must accept a `GeminiResponse` and return a future.
    /// - `Fut`: The future type returned by the `consumer` closure. It must resolve to `()`.
    ///
    /// # Parameters
    ///
    /// - `stream`: A `Pin<Box<dyn Stream>>` that produces items of type `Result<serde_json::Value, StreamBodyError>`.
    ///   The stream already needs to be pinned and boxed when passed into this function.
    /// - `consumer`: A mutable closure that is called for each `GeminiResponse`. The results of the
    ///   closure are futures which will be awaited to completion. This closure needs to be `Send` and
    ///   `'static` to allow for concurrent and potentially multi-threaded execution.
    pub async fn for_each_async<F, Fut>(
        stream: Pin<Box<dyn Stream<Item = Result<serde_json::Value, StreamBodyError>> + Send>>,
        consumer: F,
    ) where
        F: FnMut(GeminiResponse) -> Fut + Send + 'static,
        Fut: Future<Output = ()>,
    {
        // Since the stream is already boxed and pinned, you can directly use it
        let consumer = Arc::new(Mutex::new(consumer));

        // Use the for_each_concurrent method to apply the consumer to each item
        // in the stream, handling each item as it's ready. Set `None` for unbounded concurrency,
        // or set a limit with `Some(n)`

        stream
            .for_each_concurrent(None, |item: Result<serde_json::Value, StreamBodyError>| {
                let consumer = Arc::clone(&consumer);
                async move {
                    let res = match item {
                        Ok(result) => {
                            Client::convert_json_value_to_response(&result).map_err(|e| {
                                GoogleAPIError {
                                    message: format!(
                                        "Failed to get JSON stream from request: {}",
                                        e
                                    ),
                                    code: None,
                                }
                            })
                        }
                        Err(e) => Err(GoogleAPIError {
                            message: format!("Failed to get JSON stream from request: {}", e),
                            code: None,
                        }),
                    };

                    if let Ok(response) = res {
                        let mut consumer = consumer.lock().await;
                        consumer(response).await;
                    }
                }
            })
            .await;
    }

    /// Gets a ['reqwest::GeminiResponse'] from a post request.
    /// Parameters:
    /// * client - the ['reqwest::Client'] to use
    /// * api_request - the ['Request'] to send
    /// * authn_token - an optional authn token to use
    async fn get_post_response(
        &self,
        client: reqwest::Client,
        api_request: &Request,
        authn_token: Option<String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut request_builder = client
            .post(&self.url)
            .header(reqwest::header::USER_AGENT, env!("CARGO_CRATE_NAME"))
            .header(reqwest::header::CONTENT_TYPE, "application/json");

        // If a GCP authn token is provided, use it
        if let Some(token) = authn_token {
            request_builder = request_builder.bearer_auth(token);
        }

        request_builder.json(&api_request).send().await
    }
    // Count Tokens - see: "https://ai.google.dev/tutorials/rest_quickstart#count_tokens"
    //
    /// Parameters:
    /// * timeout - the timeout in seconds
    /// * api_request - the request to send to check token count
    pub async fn get_token_count(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<TokenCount, GoogleAPIError> {
        let token_option = self.get_auth_token_option().await?;

        let result = self
            .get_post_response(client, api_request, token_option)
            .await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => Ok(response.json::<TokenCount>().await.map_err(|e|GoogleAPIError {
                message: format!(
                        "Failed to deserialize API response into v1::gemini::response::TokenCount: {}",
                        e
                    ),
                code: None,
            })?),
                _ => Err(self.new_error_from_status_code(response.status())),
            },
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }

    /// Get for the url specified in 'self'
    async fn get(
        &self,
        timeout: u64,
    ) -> Result<Result<reqwest::Response, reqwest::Error>, GoogleAPIError> {
        let client: reqwest::Client = self.get_reqwest_client(timeout)?;
        let result = client
            .get(&self.url)
            .header(reqwest::header::USER_AGENT, env!("CARGO_CRATE_NAME"))
            .send()
            .await;
        Ok(result)
    }
    /// Gets a model - see: "https://ai.google.dev/tutorials/rest_quickstart#get_model"
    /// Parameters:
    /// * timeout - the timeout in seconds
    pub async fn get_model(&self, timeout: u64) -> Result<ModelInformation, GoogleAPIError> {
        let result = self.get(timeout).await?;

        match result {
            Ok(response) => {
                match response.status() {
                    reqwest::StatusCode::OK => Ok(response
                        .json::<ModelInformation>()
                        .await
                        .map_err(|e| GoogleAPIError {
                            message: format!(
                        "Failed to deserialize API response into v1::gemini::ModelInformation: {}",
                        e
                    ),
                            code: None,
                        })?),
                    _ => Err(self.new_error_from_status_code(response.status())),
                }
            }
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }
    /// Gets a list of models - see: "https://ai.google.dev/tutorials/rest_quickstart#list_models"
    /// Parameters:
    /// * timeout - the timeout in seconds
    pub async fn get_model_list(
        &self,
        timeout: u64,
    ) -> Result<ModelInformationList, GoogleAPIError> {
        let result = self.get(timeout).await?;

        match result {
            Ok(response) => {
                match response.status() {
                    reqwest::StatusCode::OK => Ok(response
                        .json::<ModelInformationList>()
                        .await
                        .map_err(|e| GoogleAPIError {
                            message: format!(
                        "Failed to deserialize API response into Vec<v1::gemini::ModelInformationList>: {}",
                        e
                    ),
                        code: None,
                    })?),
                    _ => Err(self.new_error_from_status_code(response.status())),
                }
            }
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }

    // TODO function - see "https://cloud.google.com/vertex-ai/docs/generative-ai/multimodal/function-calling"

    // TODO embedContent - see: "https://ai.google.dev/tutorials/rest_quickstart#embedding"

    /// The current version of the Vertex API only supports streamed responses, so
    /// in order to handle any issues we use a serde_json::Value and then convert to a Gemini [`Candidate`].
    fn convert_json_value_to_response(
        json_value: &serde_json::Value,
    ) -> Result<GeminiResponse, serde_json::error::Error> {
        serde_json::from_value(json_value.clone())
    }
    fn get_reqwest_client(&self, timeout: u64) -> Result<reqwest::Client, GoogleAPIError> {
        let client: reqwest::Client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| self.new_error_from_reqwest_error(e.without_url()))?;
        Ok(client)
    }
    /// Creates a new error from a status code.
    fn new_error_from_status_code(&self, code: reqwest::StatusCode) -> GoogleAPIError {
        let status_text = code.canonical_reason().unwrap_or("Unknown Status");
        let message = format!("HTTP Error: {}: {}", code.as_u16(), status_text);

        GoogleAPIError {
            message,
            code: Some(code),
        }
    }
    /// Creates a new error from a reqwest error.
    fn new_error_from_reqwest_error(&self, mut e: reqwest::Error) -> GoogleAPIError {
        if let Some(url) = e.url_mut() {
            // Remove the API key from the URL, if any
            url.query_pairs_mut().clear();
        }

        GoogleAPIError {
            message: format!("{}", e),
            code: e.status(),
        }
    }
}

/// There are two different URLs for the API, depending on whether the model is public or private.
/// Authn for public models is via an API key, while authn for private models is via application default credentials (ADC).
/// The public API URL is in the form of: https://generativelanguage.googleapis.com/v1/models/{model}:{generateContent|streamGenerateContent}
/// The Vertex AI API URL is in the form of: https://{region}-aiplatform.googleapis.com/v1/projects/{project_id}/locations/{region}/publishers/google/models/{model}:{streamGenerateContent}
#[derive(Debug)]
pub(crate) struct Url {
    pub url: String,
}
impl Url {
    pub(crate) fn new(model: &Model, api_key: String, response_type: &ResponseType) -> Self {
        let base_url = PUBLIC_API_URL_BASE.to_owned();
        match response_type {
            ResponseType::GenerateContent => Self {
                url: format!(
                    "{}/models/{}:{}?key={}",
                    base_url, model, response_type, api_key
                ),
            },
            ResponseType::StreamGenerateContent => Self {
                url: format!(
                    "{}/models/{}:{}?key={}",
                    base_url, model, response_type, api_key
                ),
            },
            ResponseType::GetModel => Self {
                url: format!("{}/models/{}?key={}", base_url, model, api_key),
            },
            ResponseType::GetModelList => Self {
                url: format!("{}/models?key={}", base_url, api_key),
            },
            ResponseType::CountTokens => Self {
                url: format!(
                    "{}/models/{}:{}?key={}",
                    base_url, model, response_type, api_key
                ),
            },
            _ => panic!("Unsupported response type: {:?}", response_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn test_new_error_from_status_code() {
        let client = Client::new("my-api-key".to_string());
        let status_code = StatusCode::BAD_REQUEST;

        let error = client.new_error_from_status_code(status_code);

        assert_eq!(error.message, "HTTP Error: 400: Bad Request");
        assert_eq!(error.code, Some(status_code));
    }

    #[test]
    fn test_url_new() {
        let model = Model::default();
        let api_key = String::from("my-api-key");
        let url = Url::new(&model, api_key.clone(), &ResponseType::GenerateContent);

        assert_eq!(
            url.url,
            format!(
                "{}/models/{}:generateContent?key={}",
                PUBLIC_API_URL_BASE, model, api_key
            )
        );
    }
}
