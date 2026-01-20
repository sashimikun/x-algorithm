# I read the code so you don't have to

This repo contains the X "For You" feed recommendation system.

## The Gist
It's a **hybrid** system:
1.  **Thunder**: In-memory, real-time store for tweets from people you follow.
2.  **Phoenix**: Vector database & ML engine for tweets from people you *don't* follow.
3.  **Home Mixer**: The conductor that queries both, merges them, and ranks them.

## The "Special Sauce"
-   **No manual features.** They deleted all the hand-engineered heuristics. The ranking is done by a **Grok-based transformer** (`PhoenixScorer`) that looks at your interaction history and the tweet to predict engagement (like, reply, repost, etc.).
-   **Weighted Scoring.** The final score is just a linear combination of those predicted probabilities (e.g., `10 * P(Like) + 20 * P(Repost) - 50 * P(Report)`).
-   **Fail-Open.** The pipeline is designed to keep serving a feed even if individual filters or components crash.

## Architecture Breakdown

### 1. Home Mixer (Rust)
The orchestration server. It builds a `PhoenixCandidatePipeline` which:
-   **Fetches:**
    -   `ThunderSource` (In-network)
    -   `PhoenixSource` (Out-of-network)
-   **Filters:**
    -   Safety (Violence, Gore, etc.)
    -   De-duplication
    -   "Previously seen" checks
-   **Scores:**
    -   `PhoenixScorer` (The heavy ML lifter)
    -   `AuthorDiversityScorer` (Prevent one person from flooding your feed)
    -   `WeightedScorer` (Combines the signals)

### 2. Thunder (Rust)
A custom in-memory database built on `DashMap`.
-   Listens to Kafka for new tweets/deletes.
-   Keeps tweets in RAM for super-fast "People you follow" lookups.
-   Separates "Original posts" from "Replies" to make feed construction easier.

### 3. Phoenix (Python/JAX)
The ML brain.
-   **Retrieval:** Uses Two-Tower models to find relevant candidates from the global firehose.
-   **Ranking:** Runs the Grok transformer to score candidates.

## TL;DR of the TL;DR
Rust handles the plumbing and speed (Thunder/Mixer), while a massive Grok model (Phoenix) decides what's actually interesting.
