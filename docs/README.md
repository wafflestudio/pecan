# Pecan Documentation

## Overview

Pecan is a high-performance API server designed for online judge systems, providing secure code execution and evaluation capabilities. Built with Rust and leveraging asynchronous I/O, Pecan delivers low-latency code execution through a prewarmed sandbox pool architecture. The system supports multiple programming languages and integrates with various sandboxing backends, making it suitable for production-grade online judge platforms.

## Architecture

Pecan follows a layered architecture pattern, organized into three primary components:

### Component Structure

**pecan-api** - HTTP API Layer

- RESTful API server built on Axum
- Exposes endpoints for code execution (`/v1/judge`) and sandbox management (`/v1/manager`)
- Handles request validation, error handling, and response serialization
- Implements service layer pattern for business logic separation

**pecan-core** - Execution Service Layer

- Core orchestration service managing code execution lifecycle
- Language-agnostic execution interface with pluggable toolchain support
- Coordinates between API requests and sandbox resources
- Provides unified execution model across supported languages (C, C++, Java, Kotlin, Python, JavaScript, TypeScript, Go, Rust)

**pecan-sandbox** - Sandbox Management Layer

- Maintains a prewarmed pool of isolated execution environments
- Implements sandbox lifecycle management (creation, execution, cleanup)
- Supports multiple sandbox backends (Nsjail, Isolate) via trait-based abstraction
- Manages resource allocation through semaphore-based concurrency control
- Background worker loop for automatic error recovery and pool maintenance

### Execution Flow

1. **Request Reception**: HTTP request arrives at `pecan-api`, validated and transformed into domain models
2. **Service Dispatch**: Request forwarded to `pecan-core` service, which constructs execution parameters
3. **Sandbox Acquisition**: Core service requests sandbox from `pecan-sandbox` manager via semaphore-protected pool
4. **Code Execution**: Sandbox manager executes code within isolated environment with resource limits
5. **Result Processing**: Execution results (stdout, stderr, metrics) returned through service layer to API
6. **Resource Release**: Sandbox returned to idle pool for subsequent requests

### Key Design Patterns

- **Pool Pattern**: Prewarmed sandbox pool eliminates cold-start latency
- **Trait-based Abstraction**: `ISandboxTool` trait enables pluggable sandbox backends
- **Async/Await**: Full async architecture using Tokio for high concurrency
- **Resource Management**: Semaphore-based concurrency limiting prevents resource exhaustion
- **Error Recovery**: Background loop automatically replaces failed sandboxes

## For developers

- [Build your own development environment using devcontainer](./dev-env-using-devcontainer.md)
- [Deployment guide](./deployment-guide.md)

## Further reading

- [How to add new language support](./how-to-add-language-toolkit.md)
