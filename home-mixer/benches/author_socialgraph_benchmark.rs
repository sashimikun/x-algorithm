use std::time::Instant;

// Mock structures to replicate the production environment
#[derive(Clone, Debug, Default)]
pub struct PostCandidate {
    pub author_id: u64,
    // other fields are not relevant for this specific benchmark
}

#[derive(Clone, Debug, Default)]
pub struct UserFeatures {
    pub blocked_user_ids: Vec<i64>,
    pub muted_user_ids: Vec<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct ScoredPostsQuery {
    pub user_features: UserFeatures,
}

pub struct FilterResult {
    pub kept: Vec<PostCandidate>,
    pub removed: Vec<PostCandidate>,
}

// The "Before" implementation: Uses cloning
#[allow(dead_code)]
fn filter_before(
    query: &ScoredPostsQuery,
    candidates: Vec<PostCandidate>,
) -> FilterResult {
    // OLD CODE: Cloning vectors
    let viewer_blocked_user_ids = query.user_features.blocked_user_ids.clone();
    let viewer_muted_user_ids = query.user_features.muted_user_ids.clone();

    if viewer_blocked_user_ids.is_empty() && viewer_muted_user_ids.is_empty() {
        return FilterResult {
            kept: candidates,
            removed: Vec::new(),
        };
    }

    let mut kept: Vec<PostCandidate> = Vec::new();
    let mut removed: Vec<PostCandidate> = Vec::new();

    for candidate in candidates {
        let author_id = candidate.author_id as i64;
        let muted = viewer_muted_user_ids.contains(&author_id);
        let blocked = viewer_blocked_user_ids.contains(&author_id);
        if muted || blocked {
            removed.push(candidate);
        } else {
            kept.push(candidate);
        }
    }

    FilterResult { kept, removed }
}

// The "After" implementation: Uses references
#[allow(dead_code)]
fn filter_after(
    query: &ScoredPostsQuery,
    candidates: Vec<PostCandidate>,
) -> FilterResult {
    // NEW CODE: Using references
    let viewer_blocked_user_ids = &query.user_features.blocked_user_ids;
    let viewer_muted_user_ids = &query.user_features.muted_user_ids;

    if viewer_blocked_user_ids.is_empty() && viewer_muted_user_ids.is_empty() {
        return FilterResult {
            kept: candidates,
            removed: Vec::new(),
        };
    }

    let mut kept: Vec<PostCandidate> = Vec::new();
    let mut removed: Vec<PostCandidate> = Vec::new();

    for candidate in candidates {
        let author_id = candidate.author_id as i64;
        let muted = viewer_muted_user_ids.contains(&author_id);
        let blocked = viewer_blocked_user_ids.contains(&author_id);
        if muted || blocked {
            removed.push(candidate);
        } else {
            kept.push(candidate);
        }
    }

    FilterResult { kept, removed }
}

fn main() {
    println!("Running AuthorSocialgraphFilter Benchmark...");

    let iterations = 5000;
    let num_candidates = 1000;
    let num_blocked = 2000;
    let num_muted = 2000;

    println!("Configuration:");
    println!("  Iterations: {}", iterations);
    println!("  Candidates per iteration: {}", num_candidates);
    println!("  Blocked users size: {}", num_blocked);
    println!("  Muted users size: {}", num_muted);

    // Setup data
    let mut blocked_ids = Vec::with_capacity(num_blocked);
    for i in 0..num_blocked {
        blocked_ids.push(i as i64);
    }
    let mut muted_ids = Vec::with_capacity(num_muted);
    for i in 0..num_muted {
        muted_ids.push((i + num_blocked) as i64);
    }

    let query = ScoredPostsQuery {
        user_features: UserFeatures {
            blocked_user_ids: blocked_ids,
            muted_user_ids: muted_ids,
        },
    };

    let mut candidates_template = Vec::with_capacity(num_candidates);
    for i in 0..num_candidates {
        candidates_template.push(PostCandidate { author_id: i as u64 });
    }

    // Warmup
    print!("Warming up...");
    for _ in 0..100 {
        filter_before(&query, candidates_template.clone());
        filter_after(&query, candidates_template.clone());
    }
    println!(" Done.");

    // Benchmark "Before"
    let start_before = Instant::now();
    for _ in 0..iterations {
        filter_before(&query, candidates_template.clone());
    }
    let duration_before = start_before.elapsed();
    println!("Before (Clone): {:.4}s", duration_before.as_secs_f64());

    // Benchmark "After"
    let start_after = Instant::now();
    for _ in 0..iterations {
        filter_after(&query, candidates_template.clone());
    }
    let duration_after = start_after.elapsed();
    println!("After (Ref):    {:.4}s", duration_after.as_secs_f64());

    // Results
    let improvement = duration_before.as_secs_f64() - duration_after.as_secs_f64();
    let percent = (improvement / duration_before.as_secs_f64()) * 100.0;

    println!("--------------------------------------------------");
    println!("Improvement: {:.4}s ({:.2}%)", improvement, percent);

    if duration_after < duration_before {
        println!("SUCCESS: Optimization is faster.");
    } else {
        println!("FAILURE: Optimization is slower or same.");
    }
}
