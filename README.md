# Pecan

A robust and lightweight API server for online judge systems, built with Rust.

Originally developed for use by [Wafflestudio](https://wafflestudio.com/) to power their internal recruiting platform.

[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://choosealicense.com/licenses/mit/)
[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%20v2-yellow.svg)](https://opensource.org/license/apache-2-0)

## Features

- **Efficient sandbox management:**
  An internal sandbox manager maintains a prewarmed pool of isolated environments to improve performance and scalability.

- **Multi-language support:**
  Supports major programming languages including **C, C++, Java, Kotlin, Python, and JavaScript.**

- **Pluggable sandbox backends:**
  Compatible with various sandboxing tools at build time, such as [**Nsjail**](https://github.com/google/nsjail) and [**Isolate**](https://github.com/ioi/isolate).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for ways to get started.

## More information

For more information, please refer to our compilation of documents in the [`docs/ directory`](./docs/README.md).

## Authors

- [@atlasyang](https://www.github.com/AtlasYang)
