# Contributing to Pecan

We welcome contributions: bug reports, feature requests, documentation updates, and pull requests.

## How to Contribute

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/wafflestudio/pecan.git
   cd pecan
   ```

2. Create a branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. Install dependencies and run tests:
   Rust is required. See [docs](./docs/dev-env-using-devcontainer.md) for setup.

4. Commit changes with clear messages:
   ```bash
   git commit -m "feat: add new authentication flow"
   ```

5. Push and open a Pull Request on GitHub.

## Guidelines

- Keep changes focused — one purpose per PR.
- Follow existing code style and linting rules.
- Include or update tests if relevant.
- Update documentation if your change affects users or developers.
- Reference related issues using `Closes #123` or `Fixes #456`.

## AI Usage

Generative AI (LLM) tools may be used for documentation, but exercise caution when using them for code contributions. Review and verify all AI-generated code for correctness, security, and adherence to project standards before submitting.

## Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new login feature
fix: correct typo in README
docs: update contributing guide
```

## License

By contributing, you agree that your contributions will be licensed under the same license as this project: **[MIT](./LICENSE-MIT)**, **[Apache 2.0](./LICENSE-APACHE)**
