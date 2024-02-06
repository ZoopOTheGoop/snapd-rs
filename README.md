# Snapd-rs

[actions-image]: https://github.com/canonical/snapd-rs/actions/workflows/push.yaml/badge.svg
[actions-url]: https://github.com/canonical/snapd-rs/actions/workflows/push.yaml

[license-image]: https://img.shields.io/badge/License-MIT-blue.svg

[![Code quality][actions-image]][actions-url]
[![License][license-image]](LICENSE)

This repository contains Rust packages for handling the REST API for `snapd`. This is current manually maintained, due to a lack of a way to auto-generate a wrapper, and may not work on new `snapd` releases if the semi-stable API has changed. 

For general details, including installation, getting started and setting up a development environment, head over to our section on [Contributing to the code](CONTRIBUTING.md#contributing-to-the-code).

## Setup

To test this, you'll need Rust installed (use [https://rustup.rs]), as well as a local, current of `snapd`. All tests can be run via `cargo test`.

## Implementation guidelines

- For endpoints with multiple static values with their own request/response formats (the best example is `assertions`), implement each as its own separate request/response pair, instead of a giant one for each. It makes more sense to abstract away the endpoint that way.
- For responses that return unknown values, using a `HashMap` keyed with the corresponding `Cow` or newtype is fine.
- Try to, where possible, implement all fields as a custom `struct`, and avoid allocations when Deserializing. Similarly, Requests should not be required to own their values.
- It is okay to implement "fake" requests for convenience's sake, that don't match `snapd`'s actual endpoints, but all of `snapd`'s endpoints should be findeable.
- Where possible, valid requests and responses should be statically guaranteed (e.g. authorized endpoints should require an authorized client).

## Get involved

This is an [open source](LICENSE) project and we warmly welcome community contributions, suggestions, and constructive feedback. If you're interested in contributing, please take a look at our [Contribution guidelines](CONTRIBUTING.md) first.

- To report an issue, please file a bug report against our repository.
- For suggestions and constructive feedback, please file a feature request or a bug report.

## Get in touch

We're friendly! We have a community forum at [https://discourse.ubuntu.com](https://discourse.ubuntu.com) where we discuss feature plans, development news, issues, updates and troubleshooting.

For news and updates, follow the [Ubuntu twitter account](https://twitter.com/ubuntu) and on [Facebook](https://www.facebook.com/ubuntu).
