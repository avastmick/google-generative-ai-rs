# Google Generative AI API client (unofficial)

> [!CAUTION]
> NO LONGER MAINTAINED.

> Gemini (Public and Vertex) is now accessible and compatible with the [OpenAI API format](https://ai.google.dev/gemini-api/docs/openai), there is little point in dusting this off - Thanks. 

[![Rust Check](https://github.com/avastmick/google-generative-ai-rs/actions/workflows/rust-check.yml/badge.svg)](https://github.com/avastmick/google-generative-ai-rs/actions/workflows/rust-check.yml)
An unofficial rust-based client library to interact with the Google generative AI API.

The goal is to emulate the [Google AI Python SDK](https://github.com/google/generative-ai-python) but in Rust.


## Usage

Start point, gain familiarity with Google's Gemini generative AI.

- For the public Gemini endpoint, see the [Gemini API Overview docs](https://ai.google.dev/docs/gemini_api_overview)

- Similarly, for the Vertex AI endpoint, see the [Vertex AI Gemini API docs](https://cloud.google.com/vertex-ai/docs/generative-ai/model-reference/gemini#text_1)

See [examples](examples) and follow the in-comment instructions. The code is (hopefully) easy and readable.

## Contributing

Yes, please!! Create a fork and branch, make your contribution, and raise a PR.

Please see [contributing](CONTRIBUTING.md) for the rules; they are standard though.

## Work status

```
google-generative-ai-rs = { version = "0.3.4", features = ["beta"] }
```

Using the `beta` feature will enable the following:

- `gemini-1.5-pro-latest`
- `gemini-1.0-pro`
- `gemini-1.5-pro-latest")`
- `gemini-1.5-flash")`
- `"gemini-1.5-flash-8b")`
- `gemini-2.0-flash-exp")`
- or custom `Model::Custom(name)`
- system instructions
- `json_mode`

Note: `gemini-1.0-pro` is deprecated and will be unavailable from 15th February 2025.

I do my best to release working code.

Status today is: *"Happy path for both public and Vertex AI endpoints work for Gemini."*

## Outline tasks

- [X] Create request and response structs
- [X] Create the public API happy path for Gemini
- [X] Create the Vertex AI (private) API happy path for Gemini
- [X] Create basic error handling
- [X] get - see: "<https://ai.google.dev/tutorials/rest_quickstart#get_model>" and "<https://ai.google.dev/tutorials/rest_quickstart#list_models>"
- [X] countTokens - see: "<https://ai.google.dev/tutorials/rest_quickstart#count_tokens>"
- [ ] function - see "<https://cloud.google.com/vertex-ai/docs/generative-ai/multimodal/function-calling>"
- [ ] embedContent - see: "<https://ai.google.dev/tutorials/rest_quickstart#embedding>"
