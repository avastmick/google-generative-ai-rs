//! Contains logic and types specific to the Vertex AI endpoint (opposed to the public Gemini API endpoint)
use std::{fmt, sync::Arc};

use super::{
    api::{Client, Url},
    gemini::{Model, ResponseType},
};
use crate::v1::errors::GoogleAPIError;

const VERTEX_AI_API_URL_BASE: &str = "https://{region}-aiplatform.googleapis.com/v1";

const GCP_API_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

impl Client {
    /// Create a new private API client (Vertex AI) using the default model, `Gemini-pro`.
    ///
    /// Parameters:
    /// * region - the GCP region to use
    /// * project_id - the GCP account project_id to use
    pub fn new_from_region_project_id(region: String, project_id: String) -> Self {
        Client::new_from_region_project_id_response_type(
            region,
            project_id,
            ResponseType::StreamGenerateContent,
        )
    }
    pub fn new_from_region_project_id_response_type(
        region: String,
        project_id: String,
        response_type: ResponseType,
    ) -> Self {
        let url = Url::new_from_region_project_id(
            &Model::default(),
            region.clone(),
            project_id.clone(),
            &response_type,
        );
        Self {
            url: url.url,
            model: Model::default(),
            region: Some(region),
            project_id: Some(project_id),
            response_type,
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

    /// If this is a Vertex AI request, get the token from the GCP authn library, if it is correctly configured, else None.
    pub(crate) async fn get_auth_token_option(&self) -> Result<Option<String>, GoogleAPIError> {
        let token_option = if self.project_id.is_some() && self.region.is_some() {
            let token = self.get_gcp_authn_token().await?.as_str().to_string();
            Some(token)
        } else {
            None
        };
        Ok(token_option)
    }
    /// Gets a GCP authn token.
    async fn get_gcp_authn_token(&self) -> Result<Arc<gcp_auth::Token>, GoogleAPIError> {
        let provider = gcp_auth::provider().await.map_err(|e| GoogleAPIError {
            message: format!("Failed to create AuthenticationManager: {}", e),
            code: None,
        })?;
        let scopes = &[GCP_API_AUTH_SCOPE];
        let token = provider.token(scopes).await.map_err(|e| GoogleAPIError {
            message: format!("Failed to generate authentication token: {}", e),
            code: None,
        })?;
        Ok(token)
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

impl Url {
    pub(crate) fn new_from_region_project_id(
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
    use crate::v1::{
        api::{Client, Url},
        gemini::{Model, ResponseType},
    };

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
