# Rustub

Rustub is a lightweight HTTP stub application written in Rust. It runs a local HTTP server that responds to configured requests with predefined responses, which makes it useful for mocking APIs, testing client integrations, and simulating backend services without needing a full server implementation.

## What it does

Instead of forwarding traffic to a real service, Rustub listens for incoming HTTP requests and matches them against stub definitions. When a request matches a configured method and URL pattern, it returns the configured status code, headers, and body.

This is helpful when you want to:

- mock third-party APIs during development
- test frontend or client behavior without a backend
- validate error handling or edge cases
- quickly prototype an HTTP contract

## Architecture

Rustub reads stub definitions from JSON files in the `config/default` directory and starts an Axum-based HTTP server from `src/main.rs`.

Each stub defines:

- an HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`)
- a request URL path
- an expected response status
- a response body
- optional response headers

## Getting started

### Run the app

```bash
cargo run
```

By default, Rustub reads:

- `config/default/server.yaml` for server settings
- `config/default/*-stub.json` for stub definitions

### Default server config

```yaml
server:
  address: "127.0.0.1"
  port: 8080
```

### Using a custom profile

Rustub supports selecting a config profile by directory name under `config/`.

- Default profile: `config/default`
- Custom profile: `config/<profile>`

You can run the app with a custom profile like this:

```bash
cargo run -- -p dev
```

or:

```bash
cargo run -- --profile staging
```

This loads:

- `config/dev/server.yaml`
- `config/dev/*-stub.json`

Example custom profile structure:

```text
config/
  default/
    server.yaml
    test-stub.json
  dev/
    server.yaml
    api-stub.json
```

`server.yaml` still follows the same format as the default profile, and the stub files can be any JSON list of stub definitions.

## Example stub

```json
[
  {
    "request": {
      "method": "GET",
      "url": "/test"
    },
    "response": {
      "status": 200,
      "body": "Hello, world!",
      "headers": {
        "Content-Type": "text/plain"
      }
    }
  }
]
```

This stub makes the server return a `200 OK` response when a `GET /test` request is received.

## Example route with path parameter

```json
[
  {
    "request": {
      "method": "GET",
      "url": "/api/user/{id}"
    },
    "response": {
      "status": 200,
      "body": {
        "id": 123,
        "username": "user_123",
        "role": "admin"
      }
    }
  }
]
```

## Notes

Rustub is intentionally simple: it acts as a local HTTP stub server for development and testing. It does not aim to replace a full production API server or service framework.

## CI

The project uses GitHub Actions for continuous integration. The workflow runs tests and builds on push to the `master` branch.

- CI workflow: [`.github/workflows/ci.yml`](.github/workflows/ci.yml)
- Actions tab: [GitHub Actions](https://github.com/robson021/Rustub/actions)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
