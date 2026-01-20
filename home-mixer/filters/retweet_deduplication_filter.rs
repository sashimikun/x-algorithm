use crate::candidate_pipeline::candidate::PostCandidate;
use crate::candidate_pipeline::query::ScoredPostsQuery;
use std::collections::{HashMap, HashSet};
use tonic::async_trait;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

/// Deduplicates retweets, keeping only the best version of a tweet
/// (e.g., preferring a valid Retweet over an Original).
pub struct RetweetDeduplicationFilter;

#[async_trait]
impl Filter<ScoredPostsQuery, PostCandidate> for RetweetDeduplicationFilter {
    async fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> Result<FilterResult<PostCandidate>, String> {
        // Build sets for fast lookup of muted/blocked users
        let blocked_ids: HashSet<i64> = query
            .user_features
            .blocked_user_ids
            .iter()
            .cloned()
            .collect();
        let muted_ids: HashSet<i64> = query
            .user_features
            .muted_user_ids
            .iter()
            .cloned()
            .collect();

        let is_valid_author = |author_id: u64| -> bool {
            let id = author_id as i64;
            !blocked_ids.contains(&id) && !muted_ids.contains(&id)
        };

        // Rank function:
        // 0: Invalid (Muted/Blocked)
        // 1: Original
        // 2: Retweet (Valid)
        let get_rank = |c: &PostCandidate| -> u8 {
            if !is_valid_author(c.author_id) {
                0
            } else if c.retweeted_tweet_id.is_some() {
                2
            } else {
                1
            }
        };

        // Map: Canonical Tweet ID -> Index in `kept` vector
        let mut best_candidates: HashMap<u64, usize> = HashMap::new();
        let mut kept: Vec<PostCandidate> = Vec::new();
        let mut removed: Vec<PostCandidate> = Vec::new();

        for candidate in candidates {
            let canonical_id = candidate
                .retweeted_tweet_id
                .unwrap_or(candidate.tweet_id as u64);

            let current_rank = get_rank(&candidate);

            if let Some(&existing_idx) = best_candidates.get(&canonical_id) {
                let existing_rank = get_rank(&kept[existing_idx]);

                if current_rank > existing_rank {
                    // Current is better: Swap
                    // Move the previously kept candidate to removed
                    let previous = std::mem::replace(&mut kept[existing_idx], candidate);
                    removed.push(previous);
                } else {
                    // Existing is better or equal: Drop current
                    removed.push(candidate);
                }
            } else {
                // New canonical ID
                best_candidates.insert(canonical_id, kept.len());
                kept.push(candidate);
            }
        }

        Ok(FilterResult { kept, removed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate_pipeline::candidate::PostCandidate;
    use crate::candidate_pipeline::query::ScoredPostsQuery;
    use crate::candidate_pipeline::query_features::UserFeatures;

    #[tokio::test]
    async fn test_prefers_retweet_over_original() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let original = PostCandidate {
            tweet_id: 200,
            retweeted_tweet_id: None,
            author_id: 1,
            ..Default::default()
        };

        let retweet = PostCandidate {
            tweet_id: 100,
            retweeted_tweet_id: Some(200),
            author_id: 2,
            ..Default::default()
        };

        // Case 1: Original then Retweet
        let candidates = vec![original.clone(), retweet.clone()];
        let result = filter.filter(&query, candidates).await.unwrap();
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 100); // Kept Retweet (swapped)

        // Case 2: Retweet then Original
        let candidates = vec![retweet.clone(), original.clone()];
        let result = filter.filter(&query, candidates).await.unwrap();
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 100); // Kept Retweet (original dropped)
    }

    #[tokio::test]
    async fn test_prefers_original_over_muted_retweet() {
        let filter = RetweetDeduplicationFilter;
        let mut query = ScoredPostsQuery::default();
        // Mute User 2 (Retweeter)
        query.user_features.muted_user_ids = vec![2];

        let original = PostCandidate {
            tweet_id: 200,
            retweeted_tweet_id: None,
            author_id: 1, // Valid
            ..Default::default()
        };

        let muted_retweet = PostCandidate {
            tweet_id: 100,
            retweeted_tweet_id: Some(200),
            author_id: 2, // Muted
            ..Default::default()
        };

        // Case 1: Original then Muted Retweet
        let candidates = vec![original.clone(), muted_retweet.clone()];
        let result = filter.filter(&query, candidates).await.unwrap();
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 200); // Kept Original (Rank 1 > Rank 0)

        // Case 2: Muted Retweet then Original
        let candidates = vec![muted_retweet.clone(), original.clone()];
        let result = filter.filter(&query, candidates).await.unwrap();
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 200); // Kept Original (Swapped Rank 0 with Rank 1)
    }
}
