//! Manages the interaction with the REST API.
use std::fmt;
use std::time::Duration;

use crate::v1::errors::GoogleAPIError;
use crate::v1::gemini::request::Request;
use crate::v1::gemini::response::Response;
use crate::v1::gemini::Model;

const PUBLIC_API_URL_BASE: &str = "https://generativelanguage.googleapis.com/v1";
const PRIVATE_API_URL_BASE: &str = "https://{region}-aiplatform.googleapis.com/v1/";

/// Manages the specific API connection
pub struct Client {
    _url: String,
    pub model: Model,
    pub region: Option<String>,
    pub project_id: Option<String>,
}
impl Client {
    /// Creates a default new public API client.
    pub fn new(api_key: String) -> Self {
        let url = Url::new(&Model::default(), api_key);
        Self {
            _url: url.url,
            model: Model::default(),
            region: None,
            project_id: None,
        }
    }
    /// Create a new public API client for a specified model.
    pub fn new_from_model(model: Model, api_key: String) -> Self {
        let url = Url::new(&model, api_key);
        Self {
            _url: url.url,
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
            _url: url.url,
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
            _url: url.url,
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
    ) -> Result<Response, GoogleAPIError> {
        let client = self.get_reqwest_client(timeout)?;

        if self.region.is_some() && self.project_id.is_some() {
            // Not implemented yet
            Err(GoogleAPIError {
                message: "The client not implemented for the VertexAI API.".to_owned(),
                code: None,
            })
        } else {
            self.get_public_post_result(client, api_request).await
        }
    }

    /// A standard post request to the public API - i.e., not to the Vertex AI private API.
    async fn get_public_post_result(
        &self,
        client: reqwest::Client,
        api_request: &Request,
    ) -> Result<Response, GoogleAPIError> {
        let result = client
            .post(&self._url)
            .header(reqwest::header::USER_AGENT, env!("CARGO_CRATE_NAME"))
            .json(&api_request)
            .send()
            .await;

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

    // get

    // function

    fn get_reqwest_client(&self, timeout: u64) -> Result<reqwest::Client, GoogleAPIError> {
        let client: reqwest::Client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| self.new_error_from_reqwest_error(e.without_url()))?;
        Ok(client)
    }
    /// Creates a new error from a status code.
    /// TODO manage all the potential status codes
    fn new_error_from_status_code(&self, code: reqwest::StatusCode) -> GoogleAPIError {
        match code {
            reqwest::StatusCode::UNAUTHORIZED => GoogleAPIError {
                message: "Authorization error.".to_owned(),
                code: Some(code),
            },
            reqwest::StatusCode::BAD_REQUEST => GoogleAPIError {
                message: "API request format not correctly formed.".to_owned(),
                code: Some(code),
            },
            reqwest::StatusCode::FORBIDDEN => GoogleAPIError {
                message: "Forbidden. Check API permissions.".to_owned(),
                code: Some(code),
            },
            _ => GoogleAPIError {
                message: format!("An unexpected HTTP error code: {:?}", code),
                code: Some(code),
            },
        }
    }
    /// Creates a new error from a reqwest error.
    fn new_error_from_reqwest_error(&self, e: reqwest::Error) -> GoogleAPIError {
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
                self._url, self.model, self.region, self.project_id
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
