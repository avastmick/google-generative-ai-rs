//! Manages the interaction with the REST API.
use std::fmt;
use std::time::Duration;

use futures::prelude::*;
use gcp_auth::AuthenticationManager;
use reqwest_streams::*;

use crate::v1::errors::GoogleAPIError;
use crate::v1::gemini::request::Request;
use crate::v1::gemini::response::GeminiResponse;
use crate::v1::gemini::Model;

use super::gemini::response::StreamedGeminiResponse;
use super::gemini::{ModelInformation, ModelInformationList, ResponseType};

const PUBLIC_API_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1";
const VERTEX_AI_API_URL_BASE: &str = "https://{region}-aiplatform.googleapis.com/v1";

const GCP_API_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// Enables a streamed or non-streamed response to be returned from the API.
#[derive(Debug)]
pub enum PostResult {
    Rest(GeminiResponse),
    Streamed(StreamedGeminiResponse),
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
}

/// Manages the specific API connection
pub struct Client {
    url: String,
    pub model: Model,
    pub region: Option<String>,
    pub project_id: Option<String>,
    pub response_type: ResponseType,
}
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
    pub fn new_from_model_reponse_type(
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
    /// Create a new private API client using the default model, `Gemini-pro`.
    ///
    /// Note: the current version of the Vertex API only supports streamed responses. A call to a 'generateContent' will return a '404' error.
    ///
    /// Parameters:
    /// * region - the GCP region to use
    /// * project_id - the GCP account project_id to use
    pub fn new_from_region_project_id(region: String, project_id: String) -> Self {
        let url = Url::new_from_region_project_id(
            &Model::default(),
            region.clone(),
            project_id.clone(),
            &ResponseType::StreamGenerateContent,
        );
        Self {
            url: url.url,
            model: Model::default(),
            region: Some(region),
            project_id: Some(project_id),
            response_type: ResponseType::StreamGenerateContent,
        }
    }
    /// Create a new private API client.
    /// Parameters:
    /// * model - the Gemini model to use
    /// * region - the GCP region to use
    /// * project_id - the GCP account project_id to use
    pub fn new_from_model_region_project_id(
        model: Model,
        region: String,
        project_id: String,
    ) -> Self {
        let url = Url::new_from_region_project_id(
            &model,
            region.clone(),
            project_id.clone(),
            &ResponseType::StreamGenerateContent,
        );
        Self {
            url: url.url,
            model,
            region: Some(region),
            project_id: Some(project_id),
            response_type: ResponseType::StreamGenerateContent,
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
        let result = self.get_post_response(client, api_request, None).await;

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
    /// A streamed post request
    async fn get_streamed_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<StreamedGeminiResponse, GoogleAPIError> {
        let token: gcp_auth::Token = self.get_gcp_authn_token().await?;

        let result = self
            .get_post_response(client, api_request, Some(token.as_str()))
            .await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    // Wire to enable introspection on the response stream
                    let mut streamed_reponse = StreamedGeminiResponse {
                        streamed_candidates: vec![],
                    };
                    let mut response_stream = response.json_array_stream::<serde_json::Value>(2048); //TODO what is a good length?
                    while let Some(json_value) =
                        response_stream
                            .try_next()
                            .await
                            .map_err(|e| GoogleAPIError {
                                message: format!("Failed to get JSON stream from request: {}", e),
                                code: None,
                            })?
                    {
                        let res: GeminiResponse = Self::convert_json_value_to_response(&json_value)
                            .map_err(|e| GoogleAPIError {
                                message: format!(
                                    "Failed to deserialize API response into v1::gemini::response::GeminiResponse: {}",
                                    e
                                ),
                                code: None,
                            })?;
                        // TODO trap the "usageMetadata" too at the end of the stream
                        streamed_reponse.streamed_candidates.push(res);
                    }

                    Ok(streamed_reponse)
                }
                _ => Err(self.new_error_from_status_code(response.status())),
            },
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
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
        authn_token: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut request_builder = client
            .post(&self.url)
            .header(reqwest::header::USER_AGENT, env!("CARGO_CRATE_NAME"));

        if let Some(token) = authn_token {
            request_builder = request_builder.bearer_auth(token);
        }

        request_builder.json(&api_request).send().await
    }
    // TODO countTokens - see: "https://ai.google.dev/tutorials/rest_quickstart#count_tokens"

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

    /// Gets a GCP authn token.
    /// See [`AuthenticationManager::new`](https://docs.rs/gcp-auth/0.1.0/gcp_auth/struct.AuthenticationManager.html) for details of approach.
    async fn get_gcp_authn_token(&self) -> Result<gcp_auth::Token, GoogleAPIError> {
        let authentication_manager =
            AuthenticationManager::new()
                .await
                .map_err(|e| GoogleAPIError {
                    message: format!("Failed to create AuthenticationManager: {}", e),
                    code: None,
                })?;
        let scopes = &[GCP_API_AUTH_SCOPE];
        let token = authentication_manager
            .get_token(scopes)
            .await
            .map_err(|e| GoogleAPIError {
                message: format!("Failed to generate authentication token: {}", e),
                code: None,
            })?;
        Ok(token)
    }
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

/// Ensuring there is no leakage of secrets
impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.region.is_some() && self.project_id.is_some() {
            write!(
                f,
                "GenerativeAiClient {{ url: {:?}, model: {:?}, region: {:?}, project_id: {:?} }}",
                self.url, self.model, self.region, self.project_id
            )
        } else {
            write!(
                f,
                "GenerativeAiClient {{ url: {:?}, model: {:?}, region: {:?}, project_id: {:?} }}",
                Url::new(
                    &self.model,
                    "*************".to_string(),
                    &self.response_type
                ),
                self.model,
                self.region,
                self.project_id
            )
        }
    }
}
/// There are two different URLs for the API, depending on whether the model is public or private.
/// Authn for public models is via an API key, while authn for private models is via application default credentials (ADC).
/// The public API URL is in the form of: https://generativelanguage.googleapis.com/v1/models/{model}:{generateContent|streamGenerateContent}
/// The Vertex AI API URL is in the form of: https://{region}-aiplatform.googleapis.com/v1/projects/{project_id}/locations/{region}/publishers/google/models/{model}:{streamGenerateContent}
#[derive(Debug)]
struct Url {
    url: String,
}
impl Url {
    fn new(model: &Model, api_key: String, response_type: &ResponseType) -> Self {
        let base_url = PUBLIC_API_URL_BASE.to_owned();
        match response_type {
            ResponseType::GenerateContent => Self {
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
            _ => panic!("Unsupported response type: {:?}", response_type),
        }
    }
    fn new_from_region_project_id(
        model: &Model,
        region: String,
        project_id: String,
        response_type: &ResponseType,
    ) -> Self {
        let base_url = VERTEX_AI_API_URL_BASE
            .to_owned()
            .replace("{region}", &region);

        let url = format!(
            "{}/projects/{}/locations/{}/publishers/google/models/{}:{}",
            base_url, project_id, region, model, response_type,
        );
        Self { url }
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
    fn test_new_from_region_project_id() {
        let region = String::from("us-central1");
        let project_id = String::from("my-project");
        let client = Client::new_from_region_project_id(region.clone(), project_id.clone());

        assert_eq!(client.region, Some(region));
        assert_eq!(client.project_id, Some(project_id));
    }

    #[test]
    fn test_new_from_model_region_project_id() {
        let model = Model::default();
        let region = String::from("us-central1");
        let project_id = String::from("my-project");
        let client = Client::new_from_model_region_project_id(
            model.clone(),
            region.clone(),
            project_id.clone(),
        );

        assert_eq!(client.model, model);
        assert_eq!(client.region, Some(region));
        assert_eq!(client.project_id, Some(project_id));
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

    #[test]
    fn test_url_new_from_region_project_id() {
        let model = Model::default();
        let region = String::from("us-central1");
        let project_id = String::from("my-project");
        let url = Url::new_from_region_project_id(
            &model,
            region.clone(),
            project_id.clone(),
            &ResponseType::StreamGenerateContent,
        );

        assert_eq!(
            url.url,
            format!(
                "{}/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent",
                VERTEX_AI_API_URL_BASE.replace("{region}", &region),
                project_id,
                region,
                model
            )
        );
    }
}
