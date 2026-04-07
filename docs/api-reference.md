# API Reference

This document provides a comprehensive reference for the Pecan API endpoints.

## Base URL

The API base URL is configurable via environment variables:
- **Host**: `HOST` (default: `0.0.0.0`)
- **Port**: `PORT` (default: `8080`)

All API endpoints are prefixed with `/v1`.

## Content Type

All requests and responses use `application/json` content type.

## CORS

The API supports Cross-Origin Resource Sharing (CORS) with `Allow-Origin: *` for all endpoints.

## Error Handling

The API uses standard HTTP status codes. When an error occurs, the response body contains a JSON object with error details:

```json
{
  "error": "Error message description"
}
```

### Error Types

The API may return the following error types:

- `NotSupportedLanguage`: The specified programming language is not supported
- `CompileError`: Code compilation failed
- `RuntimeError`: Code execution failed at runtime
- `TimeLimitExceeded`: Execution exceeded the time limit
- `MemoryLimitExceeded`: Execution exceeded the memory limit
- `AllocatingTaskError`: Failed to allocate a sandbox for task execution
- `InternalError`: An internal server error occurred

All errors return HTTP status code `500 Internal Server Error`.

---

## Endpoints

### Health Check

#### `GET /v1/health`

Check if the API server is running.

**Response**

- **Status Code**: `200 OK`
- **Body**: Plain text `"OK"`

**Example Request**

```bash
curl http://localhost:8080/v1/health
```

**Example Response**

```
OK
```

---

### Judge Endpoints

#### `POST /v1/judge/judge-single`

Execute and judge a single code submission.

**Request Body**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | string | Yes | Source code to execute |
| `language` | string | Yes | Programming language identifier |
| `stdin` | string | Yes | Standard input for the program |
| `desired_stdout` | string | Yes | Expected standard output |
| `time_limit` | number | Yes | Time limit in seconds (e.g., 1.0) |
| `memory_limit` | number | Yes | Memory limit in KB (e.g., 262144.0 for 256 MB) |

**Example Request**

```json
{
  "code": "#include <iostream>\nint main() { std::cout << \"Hello\"; return 0; }",
  "language": "cpp",
  "stdin": "",
  "desired_stdout": "Hello",
  "time_limit": 1.0,
  "memory_limit": 262144.0
}
```

**Response**

- **Status Code**: `200 OK` on success
- **Status Code**: `500 Internal Server Error` on error

**Response Body**

| Field | Type | Description |
|-------|------|-------------|
| `code` | number | Status code (0-6) |
| `status` | string | Status enum value |
| `stdout` | string | Actual standard output from execution |
| `stderr` | string | Standard error output from execution |
| `time` | number | Execution time in seconds |
| `memory` | number | Memory usage in KB |

**Status Codes**

| Code | Status | Description |
|------|--------|-------------|
| 0 | `Accepted` | Code executed successfully and output matches |
| 1 | `WrongAnswer` | Code executed but output does not match |
| 2 | `CompileError` | Code failed to compile |
| 3 | `RuntimeError` | Code crashed during execution |
| 4 | `TimeLimitExceeded` | Execution exceeded time limit |
| 5 | `MemoryLimitExceeded` | Execution exceeded memory limit |
| 6 | `InternalError` | Internal server error occurred |

**Example Response**

```json
{
  "code": 0,
  "status": "Accepted",
  "stdout": "Hello",
  "stderr": "",
  "time": 0.05,
  "memory": 12800.0
}
```

**Example cURL Request**

```bash
curl -X POST http://localhost:8080/v1/judge/judge-single \
  -H "Content-Type: application/json" \
  -d '{
    "code": "#include <iostream>\nint main() { std::cout << \"Hello\"; return 0; }",
    "language": "cpp",
    "stdin": "",
    "desired_stdout": "Hello",
    "time_limit": 1.0,
    "memory_limit": 262144.0
  }'
```

---

#### `POST /v1/judge/judge-single-async`

Submit a code submission for asynchronous judging. The result is delivered to the specified `webhook_url` via a POST request once execution completes.

**Request Body**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | string | Yes | Source code to execute |
| `language` | string | Yes | Programming language identifier |
| `stdin` | string | Yes | Standard input for the program |
| `desired_stdout` | string | Yes | Expected standard output |
| `time_limit` | number | Yes | Time limit in seconds (e.g., 1.0) |
| `memory_limit` | number | Yes | Memory limit in KB (e.g., 262144.0 for 256 MB) |
| `webhook_url` | string | Yes | URL to receive the result via POST |

**Response**

- **Status Code**: `200 OK` — request accepted and queued
- **Status Code**: `500 Internal Server Error` — queue full or internal error

**Response Body**

| Field | Type | Description |
|-------|------|-------------|
| `request_id` | string (UUID) | Unique identifier for tracking this submission |

**Example Request**

```bash
curl -X POST http://localhost:8080/v1/judge/judge-single-async \
  -H "Content-Type: application/json" \
  -d '{
    "code": "#include <iostream>\nint main() { std::cout << \"Hello\"; return 0; }",
    "language": "cpp",
    "stdin": "",
    "desired_stdout": "Hello",
    "time_limit": 1.0,
    "memory_limit": 262144.0,
    "webhook_url": "http://localhost:9000"
  }'
```

**Example Response**

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Webhook Callback

When an async judge submission finishes execution, the server sends a `POST` request to the `webhook_url` provided in the original request.

**Webhook Payload**

| Field | Type | Description |
|-------|------|-------------|
| `request_id` | string (UUID) | The same identifier returned by `judge-single-async` |
| `res` | object | Judge result (same schema as `judge-single` response) |
| `res.code` | number | Status code (0–6) |
| `res.status` | string | Status enum value |
| `res.stdout` | string | Actual standard output |
| `res.stderr` | string | Standard error output |
| `res.time` | number | Execution time in seconds |
| `res.memory` | number | Memory usage in KB |

**Example Webhook Payload**

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "res": {
    "code": 0,
    "status": "Accepted",
    "stdout": "Hello",
    "stderr": "",
    "time": 0.05,
    "memory": 12800.0
  }
}
```

> The webhook endpoint should respond with any `2xx` status code. The server currently does not retry on delivery failure.

---

### Manager Endpoints

#### `GET /v1/manager/sandbox-status`

Get the current status of sandbox resources.

**Response**

- **Status Code**: `200 OK` on success
- **Status Code**: `500 Internal Server Error` on error

**Response Body**

| Field | Type | Description |
|-------|------|-------------|
| `available_sandboxes` | number | Total number of available sandboxes |
| `idle_sandboxes` | number | Number of sandboxes currently idle |
| `running_sandboxes` | number | Number of sandboxes currently running tasks |
| `error_sandboxes` | number | Number of sandboxes in error state |

**Example Response**

```json
{
  "available_sandboxes": 10,
  "idle_sandboxes": 8,
  "running_sandboxes": 2,
  "error_sandboxes": 0
}
```

**Example cURL Request**

```bash
curl http://localhost:8080/v1/manager/sandbox-status
```

---

## Notes

- All endpoints support CORS and can be called from browser-based applications
- The API uses async processing and may queue requests if all sandboxes are busy
- Time and memory limits are specified in seconds and kilobytes respectively
- The `stdout` field in the judge response contains the actual output, which may differ from `desired_stdout` if the submission is incorrect
