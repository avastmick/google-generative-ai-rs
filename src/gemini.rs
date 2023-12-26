//! Handles the text interaction with the API

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
    use serde::Serialize;

    use super::safety::{HarmBlockThreshold, HarmCategory};

    #[derive(Debug, Clone, Serialize)]
    pub struct Part {
        pub text: Option<String>,
        #[serde(rename = "inlineData")]
        pub inline_data: Option<InlineData>,
        #[serde(rename = "fileData")]
        pub file_data: Option<FileData>,
        #[serde(rename = "videoMetadata")]
        pub video_metadata: Option<VideoMetadata>,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct InlineData {
        #[serde(rename = "mimeType")]
        pub mime_type: String,
        pub data: String,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct FileData {
        #[serde(rename = "mimeType")]
        pub mime_type: String,
        #[serde(rename = "fileUri")]
        pub file_uri: String,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct VideoMetadata {
        #[serde(rename = "startOffset")]
        pub start_offset: StartOffset,
        #[serde(rename = "endOffset")]
        pub end_offset: EndOffset,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct StartOffset {
        pub seconds: i32,
        pub nanos: i32,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct EndOffset {
        pub seconds: i32,
        pub nanos: i32,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Tools {
        #[serde(rename = "functionDeclarations")]
        pub function_declarations: Vec<FunctionDeclaration>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct FunctionDeclaration {
        pub name: String,
        pub description: String,
        pub parameters: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct SafetySettings {
        pub category: HarmCategory,
        pub threshold: HarmBlockThreshold,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct GenerationConfig {
        pub temperature: Option<f32>,
        pub top_p: Option<f32>,
        pub top_k: Option<i32>,
        #[serde(rename = "candidateCount")]
        pub candidate_count: Option<i32>,
        #[serde(rename = "maxOutputTokens")]
        pub max_output_tokens: Option<i32>,
        #[serde(rename = "stopSequences")]
        pub stop_sequences: Option<Vec<String>>,
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
    use serde::Deserialize;

    use super::safety::{HarmCategory, HarmProbability};

    #[derive(Debug, Clone, Deserialize)]
    pub struct TextResponse {
        pub model: Option<String>,
        pub candidates: Vec<Candidate>,
        #[serde(rename = "promptFeedback")]
        pub prompt_feedback: Option<PromptFeedback>,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct Candidate {
        pub content: Content,
        #[serde(rename = "finishReason")]
        pub finish_reason: Option<String>,
        pub index: Option<i32>,
        #[serde(rename = "safetyRatings")]
        pub safety_ratings: Vec<SafetyRating>,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct Content {
        pub parts: Option<Vec<Part>>,
        pub role: Role,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct Part {
        pub text: String,
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
    }
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Role {
        User,
        Model,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum FinishReason {
        FinishReasonUnspecified,
        FinishReasonLength,
        FinishReasonStopSequence,
        FinishReasonMaxTokens,
        FinishReasonTimeout,
        FinishReasonPromptSuggestion,
        FinishReasonEos,
        FinishReasonNumCandidates,
        FinishReasonUserInitiated,
        FinishReasonMaxChars,
        FinishReasonMaxExamples,
        FinishReasonMaxTime,
        FinishReasonMaxTokensPerExample,
        FinishReasonMaxTokensPerPass,
        FinishReasonMaxTokensTotal,
        FinishReasonMaxTokensPerResponse,
        FinishReasonMaxTokensPerPrompt,
        FinishReasonMaxTokensPerInput,
        FinishReasonMaxTokensPerInputPrefix,
        FinishReasonMaxTokensPerInputSuffix,
        FinishReasonMaxTokensPerInputPass,
        FinishReasonMaxTokensPerInputTotal,
        FinishReasonMaxTokensPerInputResponse,
        FinishReasonMaxTokensPerInputPrompt,
        FinishReasonMaxCharsPerExample,
        FinishReasonMaxCharsPerPass,
        FinishReasonMaxCharsTotal,
        FinishReasonMaxCharsPerResponse,
        FinishReasonMaxCharsPerPrompt,
        FinishReasonMaxCharsPerInput,
        FinishReasonMaxCharsPerInputPrefix,
        FinishReasonMaxCharsPerInputSuffix,
        FinishReasonMaxCharsPerInputPass,
        FinishReasonMaxCharsPerInputTotal,
        FinishReasonMaxCharsPerInputResponse,
        FinishReasonMaxCharsPerInputPrompt,
        FinishReasonMaxExamplesPerPass,
        FinishReasonMaxExamplesTotal,
        FinishReasonMaxExamplesPerResponse,
        FinishReasonMaxExamplesPerPrompt,
        FinishReasonMaxExamplesPerInput,
        FinishReasonMaxExamplesPerInputPrefix,
        FinishReasonMaxExamplesPerInputSuffix,
        FinishReasonMaxExamplesPerInputPass,
        FinishReasonMaxExamplesPerInputTotal,
        FinishReasonMaxExamplesPerInputResponse,
        FinishReasonMaxExamplesPerInputPrompt,
        FinishReasonMaxTimePerPass,
        FinishReasonMaxTimeTotal,
        FinishReasonMaxTimePerResponse,
        FinishReasonMaxTimePerPrompt,
        FinishReasonMaxTimePerInput,
        FinishReasonMaxTimePerInputPrefix,
        FinishReasonMaxTimePerInputSuffix,
        FinishReasonMaxTimePerInputPass,
        FinishReasonMaxTimePerInputTotal,
        FinishReasonMaxTimePerInputResponse,
        FinishReasonMaxTimePerInputPrompt,
        FinishReasonMaxPasses,
        FinishReasonMaxPassesPerResponse,
        FinishReasonMaxPassesPerPrompt,
        FinishReasonMaxPassesPerInput,
        FinishReasonMaxPass,
    }

    #[cfg(test)]
    mod tests {}
}

/// The safety data for HarmCategory, HarmBlockThreshold and HarmProbability
pub mod safety {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum HarmCategory {
        HarmCategoryUnspecified,
        HarmCategorySevere,
        HarmCategorySevereRecurring,
        HarmCategorySeverePersistent,
        HarmCategoryModerate,
        HarmCategoryModerateRecurring,
        HarmCategoryModeratePersistent,
        HarmCategoryMild,
        HarmCategoryMildRecurring,
        HarmCategoryMildPersistent,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct HarmProbability {
        pub value: f32,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum HarmBlockThreshold {
        HarmBlockThresholdUnspecified,
        HarmBlockThresholdLow,
        HarmBlockThresholdMedium,
        HarmBlockThresholdHigh,
    }
}
