//! Category-based aggregation logic

use crate::indicators::registry::IndicatorCategory;
use crate::models::signal::SignalReason;
use crate::signals::categories::CategoryWeights;

/// Indicator score with metadata
#[derive(Debug, Clone)]
pub struct IndicatorScore {
    pub name: String,
    pub score: f64,
    pub category: IndicatorCategory,
    pub weight: f64,
}

/// Aggregate scores by category
pub struct Aggregator;

impl Aggregator {
    /// Aggregate indicator scores into category scores
    pub fn aggregate_by_category(scores: &[IndicatorScore]) -> Vec<(IndicatorCategory, f64)> {
        let mut category_scores: std::collections::HashMap<IndicatorCategory, (f64, usize)> =
            std::collections::HashMap::new();

        for score in scores {
            let entry = category_scores
                .entry(score.category)
                .or_insert((0.0, 0));
            entry.0 += score.score * score.weight;
            entry.1 += 1;
        }

        category_scores
            .iter()
            .map(|(&category, &(sum, count))| {
                let avg_score = if count > 0 { sum / count as f64 } else { 0.0 };
                (category, avg_score)
            })
            .collect()
    }

    /// Calculate global score from category scores
    pub fn calculate_global_score(category_scores: &[(IndicatorCategory, f64)]) -> f64 {
        category_scores
            .iter()
            .map(|(category, score)| {
                let weight = CategoryWeights::get(*category);
                score * weight
            })
            .sum()
    }

    /// Generate explainability breakdown
    pub fn generate_reasons(
        indicator_scores: &[IndicatorScore],
        category_scores: &[(IndicatorCategory, f64)],
        _global_score: f64,
    ) -> Vec<SignalReason> {
        let mut reasons = Vec::new();

        // Add category-level reasons
        for (category, score) in category_scores {
            let category_name = match category {
                IndicatorCategory::Momentum => "Momentum",
                IndicatorCategory::Trend => "Trend",
                IndicatorCategory::Volatility => "Volatility",
                IndicatorCategory::MarketStructure => "Market Structure",
            };
            let weight = CategoryWeights::get(*category);
            reasons.push(SignalReason {
                description: format!("{}: {:.2}%", category_name, score * 100.0),
                weight: weight * score.abs(),
            });
        }

        // Add indicator-level reasons (top contributors)
        let mut indicator_reasons: Vec<_> = indicator_scores
            .iter()
            .map(|s| SignalReason {
                description: format!("{}: {:.2}%", s.name, s.score * 100.0),
                weight: s.weight * s.score.abs(),
            })
            .collect();
        indicator_reasons.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        
        // Add top 3 indicator reasons
        for reason in indicator_reasons.iter().take(3) {
            reasons.push(reason.clone());
        }

        reasons
    }
}


