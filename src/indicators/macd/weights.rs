use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdWeights {
    pub crossover_weight: f64,
    pub distance_weight: f64,
    pub histogram_momentum_weight: f64,
}

impl Default for MacdWeights {
    fn default() -> Self {
        Self {
            crossover_weight: 0.4,
            distance_weight: 0.3,
            histogram_momentum_weight: 0.3,
        }
    }
}

impl MacdWeights {
    pub fn new(
        crossover_weight: f64,
        distance_weight: f64,
        histogram_momentum_weight: f64,
    ) -> Result<Self, String> {
        let total = crossover_weight + distance_weight + histogram_momentum_weight;
        if (total - 1.0).abs() > 0.001 {
            return Err(format!(
                "Weights must sum to 1.0, got: {}",
                total
            ));
        }
        if crossover_weight < 0.0 || distance_weight < 0.0 || histogram_momentum_weight < 0.0 {
            return Err("All weights must be non-negative".to_string());
        }
        Ok(Self {
            crossover_weight,
            distance_weight,
            histogram_momentum_weight,
        })
    }
}

