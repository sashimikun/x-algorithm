## 2025-10-18 - Optimizing Error Recovery in Rust Pipelines
**Learning:** Preemptive cloning for error recovery (e.g., `let backup = candidates.clone()`) is a major performance bottleneck in hot paths.
**Action:** Redesign traits to return ownership of the data on failure (e.g., `Result<Success, (Error, Data)>`), allowing zero-cost error recovery without cloning.
