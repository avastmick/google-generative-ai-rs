# Contributing to cosmonaut-code

Whether you want to report a bug, suggest a new feature, or contribute code, I appreciate your input.

Before you start contributing, please take a moment to review the following guidelines.

## Code of Conduct

I expect all contributors to abide by the [Code of Conduct](CODE_OF_CONDUCT.md) in all project-related interactions.

## Reporting Bugs

If you encounter a bug while using cosmonaut-code, please [open an issue](https://github.com/avastmick/google-generative-ai-api-rs/issues/new) and provide as much information as possible, including:

- A clear and descriptive title
- A detailed description of the bug and the expected behavior
- Steps to reproduce the bug
- Any relevant error messages or screenshots

## Suggesting Features

If you have an idea for a new feature or improvement, please [open an issue](https://github.com/avastmick/google-generative-ai-api-rs/issues/new) and provide as much information as possible, including:

- A clear and descriptive title
- A detailed description of the proposed feature or improvement
- Any relevant examples or use cases

## Contributing Code

If you want to contribute code to cosmonaut-code, please follow these steps:

1. [Fork](https://docs.github.com/en/get-started/quickstart/fork-a-repo) the repository to your Github account
2. Clone the forked repository to your local machine
3. Create a new branch for your changes
4. Make your changes, following our [code style guide](CODE_STYLE_GUIDE.md)
5. Commit your changes and push them to your forked repository
6. Ensure that the pipeline runs successfully
7. [Create a pull request](https://docs.github.com/en/github/collaborating-with-pull-requests/creating-a-pull-request) to the main repository

When creating your pull request, please include:

- A clear and descriptive title
- A detailed description of the changes you made and the reasoning behind them
- Any relevant screenshots or examples

I will review your pull request as soon as possible and provide feedback. If your pull request requires any changes, we will let you know what needs to be done.

## Code Style Guide

I follow the [Rust code style guide](https://doc.rust-lang.org/1.0.0/style/README.html) for all code contributed.

Please ensure you run `cargo fmt --all --` frequently.

Additionally, run `cargo clippy --all-targets --all-features -- -D warnings` similarly and resolve any issues you find with your code.

I suggest using a pre-commit hook to do this automatically.

## License

By contributing to google-generative-ai-rs, you agree that your contributions will be licensed under the [MIT License](LICENSE).
