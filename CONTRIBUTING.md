# Contributing to `snapd`

A big welcome and thank you for considering contributing to `snapd` and Ubuntu! It’s people like you that make it a reality for users in our community.

Reading and following these guidelines will help us make the contribution process easy and effective for everyone involved. It also communicates that you agree to respect the time of the developers managing and developing this project. In return, we will reciprocate that respect by addressing your issue, assessing changes, and helping you finalize your pull requests.

These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Quicklinks

- [Contributing to `snapd`](#contributing-to-snapd)
  - [Quicklinks](#quicklinks)
  - [Code of Conduct](#code-of-conduct)
  - [Getting Started](#getting-started)
    - [Issues](#issues)
    - [Pull Requests](#pull-requests)
  - [Contributing to the code](#contributing-to-the-code)
    - [Code style](#code-style)
  - [Contributor License Agreement](#contributor-license-agreement)
  - [Getting Help](#getting-help)

## Code of Conduct

We take our community seriously and hold ourselves and other contributors to high standards of communication. By participating and contributing to this project, you agree to uphold our [Code of Conduct](https://ubuntu.com/community/code-of-conduct).

## Getting Started

Contributions are made to this project via Issues and Pull Requests (PRs). A few general guidelines that cover both:

* Search for existing Issues and PRs on this repository before creating your own.
* We work hard to makes sure issues are handled in a timely manner but, depending on the impact, it could take a while to investigate the root cause. A friendly ping in the comment thread to the submitter or a contributor can help draw attention if your issue is blocking.
* If you've never contributed before, see [this Ubuntu discourse post](https://discourse.ubuntu.com/t/contribute/26) for resources and tips on how to get started.

### Issues

Issues should be used to report problems with the software, request a new feature, or to discuss potential changes before a PR is created. When you create a new Issue, a template will be loaded that will guide you through collecting and providing the information we need to investigate.

If you find an Issue that addresses the problem you're having, please add your own reproduction information to the existing issue rather than creating a new one. Adding a [reaction](https://github.blog/2016-03-10-add-reactions-to-pull-requests-issues-and-comments/) can also help be indicating to our maintainers that a particular problem is affecting more than just the reporter.

### Pull Requests

PRs are always welcome and can be a quick way to get your fix or improvement slated for the next release. In general, PRs should:

* Only fix/add the functionality in question **OR** address wide-spread style issues, not both.
* Add unit or integration tests for fixed or changed functionality.
* Address a single concern with the least invasive changes possible.

For changes that address core functionality or would require breaking changes (e.g. a major release), it's best to open an Issue to discuss your proposal first. This is not required but can save time creating and reviewing changes.

In general, we follow the ["fork-and-pull" Git workflow](https://github.com/susam/gitpr)

1. Fork the repository to your own Github account
2. Clone the project to your machine
3. Create a branch locally with a succinct but descriptive name
4. Commit changes to the branch
5. Following any formatting and testing guidelines specific to this repo
6. Push changes to your fork
7. Open a PR in our repository and follow the PR template so that we can efficiently review the changes.

> PRs will trigger unit and integration tests, linting and formatting validations, static and security checks. All the tests must pass before merging in main branch.


## Contributing to the code

The test suite must pass before merging the PR to our main branch. Any new feature, change or fix must be covered by corresponding tests.

### Code style

This project follow the [rust style-guide](https://doc.rust-lang.org/1.0.0/style/README.html).

## Contributor License Agreement

It is required to sign the [Contributor License Agreement](https://ubuntu.com/legal/contributors) in order to contribute to this project.

An automated test is executed on PRs to check if it has been accepted.

This project is covered by [THIS LICENSE](LICENSE).

## Getting Help

Join us in the [Ubuntu Community](https://discourse.ubuntu.com/c/desktop/8) and post your question there with a descriptive tag.
