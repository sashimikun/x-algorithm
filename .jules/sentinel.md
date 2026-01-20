## 2024-10-12 - [Hardcoded Empty String as Env Var Name]
**Vulnerability:** `std::env::var("")` used to retrieve sensitive secrets.
**Learning:** Empty strings as environment variable names always fail to retrieve values, potentially causing applications to fall back to insecure defaults or crash unpredictably. This pattern often indicates a placeholder that was missed during review.
**Prevention:** Ensure all `std::env::var` calls use explicit, documented environment variable names (e.g., `KAFKA_SASL_PASSWORD`). Use linting tools that check for empty strings in such contexts.
