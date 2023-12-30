//! Manages the interaction with the REST API.
use std::fmt;
use std::time::Duration;

use futures::prelude::*;
use gcp_auth::AuthenticationManager;
use reqwest_streams::*;

use crate::v1::errors::GoogleAPIError;
use crate::v1::gemini::request::Request;
use crate::v1::gemini::response::Response;
use crate::v1::gemini::Model;

use super::gemini::response::StreamedResponse;

const PUBLIC_API_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1";
const PRIVATE_API_URL_BASE: &str = "https://{region}-aiplatform.googleapis.com/v1";

const GCP_API_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// Enables a streamed or non-streamed response to be returned from the API.
#[derive(Debug)]
pub enum PostResult {
    Rest(Response),
    Streamed(StreamedResponse),
}
impl PostResult {
    pub fn rest(self) -> Option<Response> {
        match self {
            PostResult::Rest(response) => Some(response),
            _ => None,
        }
    }
    pub fn streamed(self) -> Option<StreamedResponse> {
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
}
impl Client {
    /// Creates a default new public API client.
    pub fn new(api_key: String) -> Self {
        let url = Url::new(&Model::default(), api_key);
        Self {
            url: url.url,
            model: Model::default(),
            region: None,
            project_id: None,
        }
    }
    /// Create a new public API client for a specified model.
    pub fn new_from_model(model: Model, api_key: String) -> Self {
        let url = Url::new(&model, api_key);
        Self {
            url: url.url,
            model,
            region: None,
            project_id: None,
        }
    }
    /// Create a new private API client using the default model, `Gemini-pro`.
    ///
    /// Parameters:
    /// * region - the GCP region to use
    /// * project_id - the GCP account project_id to use
    pub fn new_from_region_project_id(region: String, project_id: String) -> Self {
        let url =
            Url::new_from_region_project_id(&Model::default(), region.clone(), project_id.clone());
        Self {
            url: url.url,
            model: Model::default(),
            region: Some(region),
            project_id: Some(project_id),
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
        let url = Url::new_from_region_project_id(&model, region.clone(), project_id.clone());
        Self {
            url: url.url,
            model,
            region: Some(region),
            project_id: Some(project_id),
        }
    }
    // post
    pub async fn post(
        &self,
        timeout: u64,
        api_request: &Request,
    ) -> Result<PostResult, GoogleAPIError> {
        let client: reqwest::Client = self.get_reqwest_client(timeout)?;
        // Test for the type of API request, i.e., public or private
        if self.region.is_some() && self.project_id.is_some() {
            let result = self.get_private_post_result(client, api_request).await?;
            Ok(PostResult::Streamed(result))
        } else {
            let result = self.get_public_post_result(client, api_request).await?;
            Ok(PostResult::Rest(result))
        }
    }

    /// A standard post request to the public API - i.e., not to the Vertex AI private API.
    async fn get_public_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<Response, GoogleAPIError> {
        let result = self.get_post_response(client, api_request, None).await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => Ok(response.json::<Response>().await.map_err(|e|GoogleAPIError {
                message: format!(
                        "Failed to deserialize API response into v1::gemini::response::Response: {}",
                        e
                    ),
                code: None,
            })?),
                _ => Err(self.new_error_from_status_code(response.status())),
            },
            Err(e) => Err(self.new_error_from_reqwest_error(e)),
        }
    }
    /// A standard post request to the Vertex AI private API - i.e., not to the public API.
    async fn get_private_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<StreamedResponse, GoogleAPIError> {
        let token: gcp_auth::Token = self.get_gcp_authn_token().await?;

        let result = self
            .get_post_response(client, api_request, Some(token.as_str()))
            .await;

        match result {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    // Wire to enable introspection on the response stream
                    let mut streamed_reponse = StreamedResponse {
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
                        let res: Response = Self::convert_json_value_to_response(&json_value)
                            .map_err(|e| GoogleAPIError {
                                message: format!(
                                    "Failed to deserialize API response into v1::gemini::response::Candidate: {}",
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
    /// Gets a ['reqwest::Response'] from a post request.
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
    // get

    // function

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
    ) -> Result<Response, serde_json::error::Error> {
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
                Url::new(&self.model, "*************".to_string()),
                self.model,
                self.region,
                self.project_id
            )
        }
    }
}
/// There are two different URLs for the API, depending on whether the model is public or private.
/// Authn for public models is via an API key, while authn for private models is via application default credentials (ADC).
/// The public API URL is in the form of:
/// The private API URL is in the form of: https://{region}-aiplatform.googleapis.com/v1/projects/{project_id}/locations/{region}/publishers/google/models/{model}:streamGenerateContent
#[derive(Debug)]
struct Url {
    url: String,
}
impl Url {
    pub fn new(model: &Model, api_key: String) -> Self {
        let base_url = PUBLIC_API_URL_BASE.to_owned();
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            base_url, model, api_key
        );
        Self { url }
    }
    pub fn new_from_region_project_id(model: &Model, region: String, project_id: String) -> Self {
        let base_url = PRIVATE_API_URL_BASE.to_owned().replace("{region}", &region);

        let url = format!(
            "{}/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent",
            base_url, project_id, region, model
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
        let url = Url::new(&model, api_key.clone());

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
        let url = Url::new_from_region_project_id(&model, region.clone(), project_id.clone());

        assert_eq!(
            url.url,
            format!(
                "{}/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent",
                PRIVATE_API_URL_BASE.replace("{region}", &region),
                project_id,
                region,
                model
            )
        );
    }
}
