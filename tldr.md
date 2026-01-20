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

## How to Hack the Algo (Legally)
Based on `WeightedScorer.rs`, here is what boosts your score:
*   **The Big 3:** Likes, Reposts, and Replies are the core positive signals.
*   **Dwell Time:** `DWELL_WEIGHT` and `CONT_DWELL_TIME_WEIGHT` are real. If people stop scrolling to read your thread, you win.
*   **Visuals:** `PHOTO_EXPAND_WEIGHT` and `VQV_WEIGHT` (Video Quality View) exist.
    *   *Tip:* Videos must exceed a minimum duration to qualify for the boost.
*   **Shares:** Sharing via DM or Copy Link are tracked explicitly.
*   **Don't Spam:** The `AuthorDiversityScorer` applies a decay factor to multiple posts from the same author in a single feed session.

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
