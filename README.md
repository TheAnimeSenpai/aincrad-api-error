# aincrad-api-error

Standard Axum API error type with consistent JSON responses.

## Usage

```toml
[dependencies]
aincrad-api-error = { git = "https://github.com/TheAnimeSenpai/aincrad-api-error" }
```

```rust
use aincrad_api_error::ApiError;

async fn get_item(id: Uuid) -> Result<Json<Item>, ApiError> {
    let item = db.find(id).await?;   // sqlx::Error → ApiError::Database
    item.ok_or(ApiError::NotFound)
}
```

## Variants

| Variant | Status | Response body |
|---|---|---|
| `NotFound` | 404 | `{ "error": "not_found" }` |
| `Forbidden` | 403 | `{ "error": "forbidden" }` |
| `BadRequest(msg)` | 400 | `{ "error": "bad_request", "message": "..." }` |
| `Database(sqlx::Error)` | 500 | `{ "error": "internal_error" }` — error logged |
| `Internal(anyhow::Error)` | 500 | `{ "error": "internal_error" }` — error logged |

`From<sqlx::Error>` and `From<anyhow::Error>` are implemented so `?` works on both.

For app-specific variants (domain validation failures, etc.) define your own error enum that wraps or converts from `ApiError`.
