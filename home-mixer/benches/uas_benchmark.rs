use std::time::Instant;

// Mock structures to simulate the actual Thrift data structures
#[derive(Clone)]
struct AggregatedUserAction {
    pub impressed_time_ms: Option<i64>,
    pub data: [u8; 128], // Simulate some payload size
}

impl Default for AggregatedUserAction {
    fn default() -> Self {
        Self {
            impressed_time_ms: None,
            data: [0; 128],
        }
    }
}

#[derive(Clone, Default)]
struct UserActionSequenceMeta {
    pub last_modified_epoch_ms: Option<i64>,
    pub last_kafka_publish_epoch_ms: Option<i64>,
}

#[derive(Clone, Default)]
struct ThriftUserActionSequence {
    pub user_actions: Option<Vec<AggregatedUserAction>>,
    pub metadata: Option<UserActionSequenceMeta>,
}

// -----------------------------------------------------------------------------
// "Before" Implementation (The logic we replaced)
// -----------------------------------------------------------------------------
fn aggregate_user_action_sequence_old(
    user_id: i64,
    uas_thrift: ThriftUserActionSequence,
) -> Result<usize, String> {
    // OLD: .clone().unwrap_or_default()
    // This performs a deep copy of the vector
    let thrift_user_actions = uas_thrift.user_actions.clone().unwrap_or_default();

    if thrift_user_actions.is_empty() {
        return Err(format!("No user actions found for user {}", user_id));
    }

    // Simulate access to metadata
    let _original_metadata = uas_thrift.metadata.clone().unwrap_or_default();

    // Simulate aggregation work (just counting here)
    Ok(thrift_user_actions.len())
}

// -----------------------------------------------------------------------------
// "After" Implementation (The optimized logic)
// -----------------------------------------------------------------------------
fn aggregate_user_action_sequence_new(
    user_id: i64,
    uas_thrift: ThriftUserActionSequence,
) -> Result<usize, String> {
    // NEW: .unwrap_or_default()
    // This moves the vector out of the Option, no allocation
    let thrift_user_actions = uas_thrift.user_actions.unwrap_or_default();

    if thrift_user_actions.is_empty() {
        return Err(format!("No user actions found for user {}", user_id));
    }

    // Simulate access to metadata
    let _original_metadata = uas_thrift.metadata.unwrap_or_default();

    // Simulate aggregation work
    Ok(thrift_user_actions.len())
}

fn main() {
    let iterations = 20_000;
    let sequence_length = 2_000;

    println!("Preparing data...");
    println!("  Iterations: {}", iterations);
    println!("  Sequence Length: {} items", sequence_length);
    println!("  Payload per item: 128 bytes");

    // Create meaningful data
    let actions: Vec<AggregatedUserAction> = (0..sequence_length)
        .map(|_| AggregatedUserAction::default())
        .collect();

    let meta = UserActionSequenceMeta {
        last_modified_epoch_ms: Some(123456789),
        last_kafka_publish_epoch_ms: Some(987654321),
    };

    let input_template = ThriftUserActionSequence {
        user_actions: Some(actions),
        metadata: Some(meta),
    };

    println!("\nRunning Warmup...");
    for _ in 0..100 {
        let _ = aggregate_user_action_sequence_old(1, input_template.clone());
        let _ = aggregate_user_action_sequence_new(1, input_template.clone());
    }

    println!("\nBenchmark: 'Before' (With Clone)");
    let start_old = Instant::now();
    for _ in 0..iterations {
        let _ = aggregate_user_action_sequence_old(1, input_template.clone());
    }
    let duration_old = start_old.elapsed();
    println!("  Time: {:?}", duration_old);

    println!("\nBenchmark: 'After' (Without Clone)");
    let start_new = Instant::now();
    for _ in 0..iterations {
        let _ = aggregate_user_action_sequence_new(1, input_template.clone());
    }
    let duration_new = start_new.elapsed();
    println!("  Time: {:?}", duration_new);

    // Calculate stats
    let old_secs = duration_old.as_secs_f64();
    let new_secs = duration_new.as_secs_f64();
    let improvement = (old_secs - new_secs) / old_secs * 100.0;

    println!("\n--------------------------------------------------");
    println!("RESULTS");
    println!("--------------------------------------------------");
    println!("Before: {:.4}s", old_secs);
    println!("After:  {:.4}s", new_secs);
    println!("Improvement: {:.2}%", improvement);

    if improvement > 10.0 {
        println!("\n✅ SUCCESS: Significant performance improvement verified.");
    } else {
        println!("\n⚠️ WARNING: Improvement is negligible.");
    }
}
