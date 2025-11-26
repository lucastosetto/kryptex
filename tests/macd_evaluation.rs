use kryptex::indicators::macd::*;
use kryptex::models::indicators::MacdIndicator;

#[test]
fn test_bullish_crossover() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bullish);
    assert!(evaluation.crossover_score > 0.0);
    assert!(evaluation.distance_score > 0.0);
    assert!(evaluation.histogram_momentum_score > 0.0);
    assert!(evaluation.overall_score > 0.0);
    assert_eq!(evaluation.macd_value, 0.5);
    assert_eq!(evaluation.signal_value, 0.3);
    assert_eq!(evaluation.histogram_value, 0.2);
}

#[test]
fn test_bearish_crossover() {
    let macd = MacdIndicator {
        macd: 0.3,
        signal: 0.5,
        histogram: -0.2,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bearish);
    assert!(evaluation.crossover_score > 0.0);
    assert!(evaluation.distance_score > 0.0);
    assert!(evaluation.histogram_momentum_score > 0.0);
    assert!(evaluation.overall_score > 0.0);
}

#[test]
fn test_no_crossover_equal_lines() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.5,
        histogram: 0.0,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::None);
    assert_eq!(evaluation.crossover_score, 0.0);
    assert_eq!(evaluation.distance_score, 0.0);
    assert_eq!(evaluation.histogram_momentum_score, 0.0);
    assert_eq!(evaluation.overall_score, 0.0);
}

#[test]
fn test_no_crossover_very_close() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.50005,
        histogram: -0.00005,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::None);
    assert_eq!(evaluation.crossover_score, 0.0);
}

#[test]
fn test_strong_bullish_crossover() {
    let macd = MacdIndicator {
        macd: 60.0,
        signal: 10.0,
        histogram: 50.0,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bullish);
    assert_eq!(evaluation.crossover_score, 1.0);
    assert_eq!(evaluation.distance_score, 1.0);
    assert_eq!(evaluation.histogram_momentum_score, 1.0);
}

#[test]
fn test_strong_bearish_crossover() {
    let macd = MacdIndicator {
        macd: -60.0,
        signal: -10.0,
        histogram: -50.0,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bearish);
    assert_eq!(evaluation.crossover_score, 1.0);
    assert_eq!(evaluation.distance_score, 1.0);
    assert_eq!(evaluation.histogram_momentum_score, 1.0);
}

#[test]
fn test_distance_calculation() {
    let macd1 = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    };
    let distance1 = calculate_distance_score(&macd1);
    
    let macd2 = MacdIndicator {
        macd: 1.0,
        signal: 0.5,
        histogram: 0.5,
        period: None,
    };
    let distance2 = calculate_distance_score(&macd2);
    
    assert!(distance2 > distance1);
    assert!(distance1 > 0.0);
    assert!(distance2 <= 1.0);
}

#[test]
fn test_histogram_momentum_calculation() {
    let macd1 = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: None,
    };
    let momentum1 = calculate_histogram_momentum_score(&macd1);
    
    let macd2 = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 30.0,
        period: None,
    };
    let momentum2 = calculate_histogram_momentum_score(&macd2);
    
    assert!(momentum2 > momentum1);
    assert!(momentum1 > 0.0);
    assert_eq!(momentum2, 1.0);
}

#[test]
fn test_negative_histogram_momentum() {
    let macd = MacdIndicator {
        macd: 0.3,
        signal: 0.5,
        histogram: -30.0,
        period: None,
    };
    let momentum = calculate_histogram_momentum_score(&macd);
    
    assert_eq!(momentum, 1.0);
}

#[test]
fn test_custom_weights() {
    let macd = MacdIndicator {
        macd: 10.0,
        signal: 5.0,
        histogram: 20.0,
        period: None,
    };
    
    let weights1 = MacdWeights::new(0.9, 0.05, 0.05).unwrap();
    let evaluation1 = evaluate_macd(&macd, &weights1);
    
    let weights2 = MacdWeights::new(0.05, 0.05, 0.9).unwrap();
    let evaluation2 = evaluate_macd(&macd, &weights2);
    
    assert_ne!(evaluation1.overall_score, evaluation2.overall_score);
    assert!(evaluation2.overall_score > evaluation1.overall_score);
}

#[test]
fn test_weights_validation() {
    assert!(MacdWeights::new(0.4, 0.3, 0.3).is_ok());
    assert!(MacdWeights::new(1.0, 0.0, 0.0).is_ok());
    assert!(MacdWeights::new(0.5, 0.5, 0.0).is_ok());
}

#[test]
fn test_weights_invalid_sum() {
    assert!(MacdWeights::new(0.5, 0.3, 0.3).is_err());
    assert!(MacdWeights::new(0.4, 0.3, 0.4).is_err());
}

#[test]
fn test_weights_negative() {
    assert!(MacdWeights::new(-0.1, 0.5, 0.6).is_err());
    assert!(MacdWeights::new(0.4, -0.1, 0.7).is_err());
}

#[test]
fn test_weights_default() {
    let weights = MacdWeights::default();
    assert_eq!(weights.crossover_weight, 0.4);
    assert_eq!(weights.distance_weight, 0.3);
    assert_eq!(weights.histogram_momentum_weight, 0.3);
    
    let total = weights.crossover_weight + weights.distance_weight + weights.histogram_momentum_weight;
    assert!((total - 1.0).abs() < 0.001);
}

#[test]
fn test_small_distance() {
    let macd = MacdIndicator {
        macd: 0.501,
        signal: 0.5,
        histogram: 0.001,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bullish);
    assert!(evaluation.distance_score < 0.1);
    assert!(evaluation.histogram_momentum_score < 0.1);
}

#[test]
fn test_large_distance() {
    let macd = MacdIndicator {
        macd: 100.0,
        signal: 50.0,
        histogram: 50.0,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.distance_score, 1.0);
    assert_eq!(evaluation.histogram_momentum_score, 1.0);
}

#[test]
fn test_zero_values() {
    let macd = MacdIndicator {
        macd: 0.0,
        signal: 0.0,
        histogram: 0.0,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::None);
    assert_eq!(evaluation.crossover_score, 0.0);
    assert_eq!(evaluation.distance_score, 0.0);
    assert_eq!(evaluation.histogram_momentum_score, 0.0);
    assert_eq!(evaluation.overall_score, 0.0);
}

#[test]
fn test_negative_macd_values() {
    let macd = MacdIndicator {
        macd: -0.5,
        signal: -0.3,
        histogram: -0.2,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bearish);
    assert!(evaluation.distance_score > 0.0);
    assert!(evaluation.histogram_momentum_score > 0.0);
}

#[test]
fn test_mixed_positive_negative() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: -0.3,
        histogram: 0.8,
        period: None,
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bullish);
    assert!(evaluation.distance_score > 0.0);
    assert!(evaluation.histogram_momentum_score > 0.0);
}

#[test]
fn test_evaluation_with_period() {
    let macd = MacdIndicator {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
        period: Some((12, 26, 9)),
    };
    let weights = MacdWeights::default();
    let evaluation = evaluate_macd(&macd, &weights);
    
    assert_eq!(evaluation.crossover_type, CrossoverType::Bullish);
    assert!(evaluation.overall_score > 0.0);
}

#[test]
fn test_score_bounds() {
    let test_cases = vec![
        MacdIndicator {
            macd: 0.5,
            signal: 0.3,
            histogram: 0.2,
            period: None,
        },
        MacdIndicator {
            macd: 100.0,
            signal: 50.0,
            histogram: 50.0,
            period: None,
        },
        MacdIndicator {
            macd: 0.0,
            signal: 0.0,
            histogram: 0.0,
            period: None,
        },
    ];
    
    let weights = MacdWeights::default();
    for macd in test_cases {
        let evaluation = evaluate_macd(&macd, &weights);
        assert!(evaluation.overall_score >= 0.0);
        assert!(evaluation.overall_score <= 1.0);
        assert!(evaluation.crossover_score >= 0.0);
        assert!(evaluation.crossover_score <= 1.0);
        assert!(evaluation.distance_score >= 0.0);
        assert!(evaluation.distance_score <= 1.0);
        assert!(evaluation.histogram_momentum_score >= 0.0);
        assert!(evaluation.histogram_momentum_score <= 1.0);
    }
}

