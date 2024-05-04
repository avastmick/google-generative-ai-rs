# Google Generative AI API client (unofficial)

[![Rust Check](https://github.com/avastmick/google-generative-ai-rs/actions/workflows/rust-check.yml/badge.svg)](https://github.com/avastmick/google-generative-ai-rs/actions/workflows/rust-check.yml)
An unofficial rust-based client library to interact with the Google generative AI API.

The goal is to emulate the [Google AI Python SDK](https://github.com/google/generative-ai-python) but in Rust.

The initial focus will be on the [newer Gemini models](https://blog.google/technology/ai/google-gemini-ai/), but the more stable and mature models will hopefully also be supported soon.

## Usage

Start point, gain familiarity with Google's Gemini generative AI.

- For the public Gemini endpoint, see the [Gemini API Overview docs](https://ai.google.dev/docs/gemini_api_overview)

- Similarly, for the Vertex AI endpoint, see the [Vertex AI Gemini API docs](https://cloud.google.com/vertex-ai/docs/generative-ai/model-reference/gemini#text_1)

See [examples](examples) and follow the in-comment instructions. The code is (hopefully) easy and readable.

## Contributing

Yes, please!! Create a fork and branch, make your contribution, and raise a PR.

Please see [contributing](CONTRIBUTING.md) for the rules; they are standard though.

## Work status

## Potentially Breaking Changes

Version `0.3.0` may lead to breaking changes. This version adds in some `beta` features and I have now added a feature flag to enable these.

```
google-generative-ai-rs = { version = "0.3.0", features = ["beta"] }
```

Using the `beta` feature will enable the following:

- `gemini-1.5-pro-latest`
- system instructions
- `json_mode`

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
