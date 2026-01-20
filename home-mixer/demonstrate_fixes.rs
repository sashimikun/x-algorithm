fn main() {
    println!("=== Demonstration of Fixes for Ranking Algorithms ===\n");

    demonstrate_diversity_fix();
    println!("\n-----------------------------------------------------\n");
    demonstrate_weighted_scorer_fix();
}

fn demonstrate_diversity_fix() {
    println!("1. Author Diversity Scorer Fix");
    println!("Scenario: A candidate has a negative score (e.g., -100.0) due to negative signals.");
    println!("          It is the 2nd post from the same author, so a decay factor (0.5) applies.");

    let score = -100.0;
    let multiplier = 0.5;

    // Old Logic
    let old_score = score * multiplier;

    // New Logic
    let new_score = if score > 0.0 {
        score * multiplier
    } else {
        score
    };

    println!("\nInput Score: {}", score);
    println!("Diversity Multiplier: {}", multiplier);
    println!("Before Fix (Old Logic): {} (BOOSTED! -50 is better than -100)", old_score);
    println!("After Fix (New Logic):  {} (CORRECT - score remains low)", new_score);
}

fn demonstrate_weighted_scorer_fix() {
    println!("2. Weighted Scorer Fix");
    println!("Scenario: Comparing scores just above and below zero to check for continuity.");
    println!("          Constants: WEIGHTS_SUM=200, NEGATIVE_SCORES_OFFSET=50, NEGATIVE_WEIGHTS_SUM=-100");

    let score_neg = -0.001;
    let score_pos = 0.001;

    // Constants mocking the params module
    let weights_sum = 200.0;
    let negative_weights_sum = -100.0;
    let negative_scores_offset = 50.0;

    // Old Logic Function
    let old_logic = |s: f64| -> f64 {
        if s < 0.0 {
            (s + negative_weights_sum) / weights_sum * negative_scores_offset
        } else {
            s + negative_scores_offset
        }
    };

    // New Logic Function
    let new_logic = |s: f64| -> f64 {
        s + negative_scores_offset
    };

    println!("\n-- Before Fix (Discontinuity) --");
    println!("Score -0.001 -> {:.4}", old_logic(score_neg));
    println!("Score +0.001 -> {:.4}", old_logic(score_pos));
    println!("Gap: {:.4} (Huge jump at zero!)", (old_logic(score_pos) - old_logic(score_neg)).abs());

    println!("\n-- After Fix (Continuous) --");
    println!("Score -0.001 -> {:.4}", new_logic(score_neg));
    println!("Score +0.001 -> {:.4}", new_logic(score_pos));
    println!("Gap: {:.4} (Smooth transition)", (new_logic(score_pos) - new_logic(score_neg)).abs());
}
