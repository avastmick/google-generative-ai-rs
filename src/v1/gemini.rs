//! Handles the text interaction with the API
use core::fmt;
use serde::{Deserialize, Serialize};

use self::request::{FileData, InlineData, VideoMetadata};
/// Defines the type of response expected from the API.
/// Used at the end of the API URL for the Gemini API.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ResponseType {
    #[default]
    GenerateContent,
    StreamGenerateContent,
    GetModel,
    GetModelList,
    CountTokens,
    EmbedContent,
    BatchEmbedContents,
}
impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResponseType::GenerateContent => f.write_str("generateContent"),
            ResponseType::StreamGenerateContent => f.write_str("streamGenerateContent"),
            ResponseType::GetModel => f.write_str(""), // No display as its already in the URL
            ResponseType::GetModelList => f.write_str(""), // No display as its already in the URL
            ResponseType::CountTokens => f.write_str("countTokens"),
            ResponseType::EmbedContent => f.write_str("embedContent"),
            ResponseType::BatchEmbedContents => f.write_str("batchEmbedContents"),
        }
    }
}
/// Captures the information for a specific Google generative AI model.
///
/// ```json
/// {
///    "name": "models/gemini-pro",
///    "version": "001",
///    "displayName": "Gemini Pro",
///    "description": "The best model for scaling across a wide range of tasks",
///    "inputTokenLimit": 30720,
///    "outputTokenLimit": 2048,
///    "supportedGenerationMethods": [
///        "generateContent",
///        "countTokens"
///    ],
///    "temperature": 0.9,
///    "topP": 1,
///    "topK": 100,
/// }
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "model")]
pub struct ModelInformation {
    pub name: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: i32,
    pub output_token_limit: i32,
    pub supported_generation_methods: Vec<String>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}
/// Lists the available models for the Gemini API.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "models")]
pub struct ModelInformationList {
    pub models: Vec<ModelInformation>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[default]
    GeminiPro,
    #[cfg(feature = "beta")]
    #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
    Gemini1_5Pro,
    GeminiProVision,
    // TODO Embedding001
}
impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Model::GeminiPro => write!(f, "gemini-pro"),

            #[cfg(feature = "beta")]
            Model::Gemini1_5Pro => write!(f, "gemini-1.5-pro-latest"),

            Model::GeminiProVision => write!(f, "gemini-pro-vision"),
            // TODO Model::Embedding001 => write!(f, "embedding-001"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Content {
    pub role: Role,
    #[serde(default)]
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<InlineData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<FileData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_metadata: Option<VideoMetadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Model,
}

/// The request format follows the following structure:
/// ```json
/// {
///   "contents": [
///     {
///       "role": string,
///       "parts": [
///         {
///           /// Union field data can be only one of the following:
///           "text": string,
///           "inlineData": {
///             "mimeType": string,
///             "data": string
///           },
///           "fileData": {
///             "mimeType": string,
///             "fileUri": string
///           },
///           /// End of list of possible types for union field data.
///           "videoMetadata": {
///             "startOffset": {
///               "seconds": integer,
///               "nanos": integer
///             },
///             "endOffset": {
///               "seconds": integer,
///               "nanos": integer
///             }
///           }
///         }
///       ]
///     }
///   ],
///   "tools": [
///     {
///       "functionDeclarations": [
///         {
///           "name": string,
///           "description": string,
///           "parameters": {
///             object (OpenAPI Object Schema)
///           }
///         }
///       ]
///     }
///   ],
///   "safetySettings": [
///     {
///       "category": enum (HarmCategory),
///       "threshold": enum (HarmBlockThreshold)
///     }
///   ],
///   "generationConfig": {
///     "temperature": number,
///     "topP": number,
///     "topK": number,
///     "candidateCount": integer,
///     "maxOutputTokens": integer,
///     "stopSequences": [
///       string
///     ]
///   }
/// }
/// ```
/// See https://cloud.google.com/vertex-ai/docs/generative-ai/model-reference/gemini
pub mod request {
    use serde::{Deserialize, Serialize};

    use super::{
        safety::{HarmBlockThreshold, HarmCategory},
        Content,
    };

    /// Holds the data to be used for a specific text request
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Request {
        pub contents: Vec<Content>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub tools: Vec<Tools>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default, rename = "safetySettings")]
        pub safety_settings: Vec<SafetySettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, rename = "generationConfig")]
        pub generation_config: Option<GenerationConfig>,

        #[cfg(feature = "beta")]
        #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, rename = "system_instruction")]
        pub system_instruction: Option<SystemInstructionContent>,
    }
    impl Request {
        pub fn new(
            contents: Vec<Content>,
            tools: Vec<Tools>,
            safety_settings: Vec<SafetySettings>,
            generation_config: Option<GenerationConfig>,
        ) -> Self {
            Request {
                contents,
                tools,
                safety_settings,
                generation_config,
                #[cfg(feature = "beta")]
                system_instruction: None,
            }
        }

        #[cfg(feature = "beta")]
        #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
        pub fn set_system_instruction(&mut self, instruction: SystemInstructionContent) {
            self.system_instruction = Some(instruction);
        }

        /// Gets the total character count of the prompt.
        /// As per the Gemini API, "Text input is charged by every 1,000 characters of input (prompt).
        ///     Characters are counted by UTF-8 code points and white space is excluded from the count."
        /// See: https://cloud.google.com/vertex-ai/pricing
        ///
        /// Returns the total character count of the prompt as per the Gemini API.
        pub fn get_prompt_character_count(&self) -> usize {
            let mut text_count = 0;
            for content in &self.contents {
                for part in &content.parts {
                    if let Some(text) = &part.text {
                        // Exclude white space from the count
                        let num_chars = bytecount::num_chars(text.as_bytes());
                        let num_spaces = bytecount::count(text.as_bytes(), b' ');
                        text_count += num_chars - num_spaces;
                    }
                }
            }
            text_count
        }
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InlineData {
        pub mime_type: String,
        pub data: String,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileData {
        pub mime_type: String,
        pub file_uri: String,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct VideoMetadata {
        pub start_offset: StartOffset,
        pub end_offset: EndOffset,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct StartOffset {
        pub seconds: i32,
        pub nanos: i32,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct EndOffset {
        pub seconds: i32,
        pub nanos: i32,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Tools {
        #[serde(rename = "functionDeclarations")]
        pub function_declarations: Vec<FunctionDeclaration>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct FunctionDeclaration {
        pub name: String,
        pub description: String,
        pub parameters: serde_json::Value,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SafetySettings {
        pub category: HarmCategory,
        pub threshold: HarmBlockThreshold,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GenerationConfig {
        pub temperature: Option<f32>,
        pub top_p: Option<f32>,
        pub top_k: Option<i32>,
        pub candidate_count: Option<i32>,
        pub max_output_tokens: Option<i32>,
        pub stop_sequences: Option<Vec<String>>,

        #[cfg(feature = "beta")]
        #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
        pub response_mime_type: Option<String>,
    }

    #[cfg(feature = "beta")]
    #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SystemInstructionContent {
        #[serde(default)]
        pub parts: Vec<SystemInstructionPart>,
    }

    #[cfg(feature = "beta")]
    #[cfg_attr(docsrs, doc(cfg(feature = "beta")))]
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SystemInstructionPart {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
    }
}

/// The response format follows the following structure:
/// ```json
/// {
///   "candidates": [
///     {
///       "content": {
///         "parts": [
///           {
///             "text": string
///           }
///         ]
///       },
///       "finishReason": enum (FinishReason),
///       "safetyRatings": [
///         {
///           "category": enum (HarmCategory),
///           "probability": enum (HarmProbability),
///           "blocked": boolean
///         }
///       ],
///       "citationMetadata": {
///         "citations": [
///           {
///             "startIndex": integer,
///             "endIndex": integer,
///             "uri": string,
///             "title": string,
///             "license": string,
///             "publicationDate": {
///               "year": integer,
///               "month": integer,
///               "day": integer
///             }
///           }
///         ]
///       }
///     }
///   ],
///   "usageMetadata": {
///     "promptTokenCount": integer,
///     "candidatesTokenCount": integer,
///     "totalTokenCount": integer
///   }
/// }
/// ```
pub mod response {
    use core::fmt;
    use futures::Stream;
    use reqwest_streams::error::StreamBodyError;
    use serde::Deserialize;
    use std::pin::Pin;

    use super::{
        safety::{HarmCategory, HarmProbability},
        Content,
    };

    impl fmt::Debug for StreamedGeminiResponse {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "StreamedGeminiResponse {{ /* stream values */ }}")
        }
    }

    type ResponseJsonStream =
        Pin<Box<dyn Stream<Item = Result<serde_json::Value, StreamBodyError>> + Send>>;

    /// The token count for a given prompt.
    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TokenCount {
        pub total_tokens: u64,
    }

    // The streamGenerateContent response
    #[derive(Default)]
    pub struct StreamedGeminiResponse {
        pub response_stream: Option<ResponseJsonStream>,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GeminiResponse {
        pub candidates: Vec<Candidate>,
        pub prompt_feedback: Option<PromptFeedback>,
        pub usage_metadata: Option<UsageMetadata>,
    }
    impl GeminiResponse {
        /// Returns the total character count of the response as per the Gemini API.
        pub fn get_response_character_count(&self) -> usize {
            let mut text_count = 0;
            for candidate in &self.candidates {
                for content in &candidate.content.parts {
                    if let Some(text) = &content.text {
                        // Exclude white space from the count
                        let num_chars = bytecount::num_chars(text.as_bytes());
                        let num_spaces = bytecount::count(text.as_bytes(), b' ');
                        text_count += num_chars - num_spaces;
                    }
                }
            }
            text_count
        }
    }
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Candidate {
        pub content: Content,
        pub finish_reason: Option<String>,
        pub index: Option<i32>,
        pub safety_ratings: Vec<SafetyRating>,
    }
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UsageMetadata {
        pub prompt_token_count: u64,
        pub candidates_token_count: u64,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct PromptFeedback {
        #[serde(rename = "safetyRatings")]
        pub safety_ratings: Vec<SafetyRating>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct SafetyRating {
        pub category: HarmCategory,
        pub probability: HarmProbability,
        #[serde(default)]
        pub blocked: bool,
    }

    /// The reason why the model stopped generating tokens. If empty, the model has not stopped generating the tokens.
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum FinishReason {
        FinishReasonUnspecified, // The finish reason is unspecified.
        FinishReasonStop,        // Natural stop point of the model or provided stop sequence.
        FinishReasonMaxTokens, // The maximum number of tokens as specified in the request was reached.
        FinishReasonSafety, // The token generation was stopped as the response was flagged for safety reasons. Note that [`Candidate`].content is empty if content filters block the output.
        FinishReasonRecitation, // The token generation was stopped as the response was flagged for unauthorized citations.
        FinishReasonOther,      // All other reasons that stopped the token
    }
    #[cfg(test)]
    mod tests {}
}

/// The safety data for HarmCategory, HarmBlockThreshold and HarmProbability
pub mod safety {
    use serde::{Deserialize, Serialize};

    /// The safety category to configure a threshold for.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum HarmCategory {
        HarmCategorySexuallyExplicit,
        HarmCategoryHateSpeech,
        HarmCategoryHarassment,
        HarmCategoryDangerousContent,
    }
    /// For a request: the safety category to configure a threshold for. For a response: the harm probability levels in the content.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum HarmProbability {
        HarmProbabilityUnspecified,
        Negligible,
        Low,
        Medium,
        High,
    }
    /// The threshold for blocking responses that could belong to the specified safety category based on probability.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum HarmBlockThreshold {
        BlockNone,
        BlockLowAndAbove,
        BlockMedAndAbove,
        BlockHighAndAbove,
    }
}
