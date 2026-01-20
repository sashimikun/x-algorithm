use crate::candidate_pipeline::candidate::PostCandidate;
use crate::candidate_pipeline::query::ScoredPostsQuery;
use std::collections::HashSet;
use tonic::async_trait;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

/// Deduplicates retweets, keeping only the first occurrence of a tweet
/// (whether as an original or as a retweet).
pub struct RetweetDeduplicationFilter;

#[async_trait]
impl Filter<ScoredPostsQuery, PostCandidate> for RetweetDeduplicationFilter {
    async fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> Result<FilterResult<PostCandidate>, String> {
        let mut seen_tweet_ids: HashSet<u64> = HashSet::new();
        let mut kept = Vec::new();
        let mut removed = Vec::new();

        for candidate in candidates {
            match candidate.retweeted_tweet_id {
                Some(retweeted_id) => {
                    // Remove if we've already seen this tweet (as original or retweet)
                    if seen_tweet_ids.insert(retweeted_id) {
                        kept.push(candidate);
                    } else {
                        removed.push(candidate);
                    }
                }
                None => {
                    // Mark this original tweet ID as seen so retweets of it get filtered
                    if seen_tweet_ids.insert(candidate.tweet_id as u64) {
                        kept.push(candidate);
                    } else {
                        removed.push(candidate);
                    }
                }
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

    #[tokio::test]
    async fn test_retweet_deduplication_filter_removes_original_if_retweet_seen_first() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let retweet_candidate = PostCandidate {
            tweet_id: 100,
            retweeted_tweet_id: Some(200),
            ..Default::default()
        };

        let original_candidate = PostCandidate {
            tweet_id: 200,
            retweeted_tweet_id: None,
            ..Default::default()
        };

        let candidates = vec![retweet_candidate, original_candidate];

        let result = filter.filter(&query, candidates).await.unwrap();

        // Should keep the retweet (first occurrence of 200)
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 100);

        // Should remove the original tweet (second occurrence of 200)
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].tweet_id, 200);
    }

    #[tokio::test]
    async fn test_retweet_deduplication_filter_removes_retweet_if_original_seen_first() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let original_candidate = PostCandidate {
            tweet_id: 200,
            retweeted_tweet_id: None,
            ..Default::default()
        };

        let retweet_candidate = PostCandidate {
            tweet_id: 100,
            retweeted_tweet_id: Some(200),
            ..Default::default()
        };

        let candidates = vec![original_candidate, retweet_candidate];

        let result = filter.filter(&query, candidates).await.unwrap();

        // Should keep the original tweet (first occurrence of 200)
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 200);

        // Should remove the retweet (second occurrence of 200)
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].tweet_id, 100);
    }
}
