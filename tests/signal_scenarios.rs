use kryptex::config::Config;
use kryptex::signals::{IndicatorInput, MacdSignal, SignalDirection, SignalGenerator};

struct Scenario {
    input: IndicatorInput,
}

impl Scenario {
    fn new(macd: f64, signal: f64, histogram: f64, rsi: f64, price: f64) -> Self {
        let input = IndicatorInput {
            macd: MacdSignal {
                macd,
                signal,
                histogram,
            },
            rsi,
            funding_rate: None,
            price,
            symbol: Some("BTC".into()),
        };

        Self { input }
    }

    fn with_funding(mut self, funding: f64) -> Self {
        self.input.funding_rate = Some(funding);
        self
    }
}

fn run_scenario(scenario: Scenario, assertion: impl Fn(&SignalDirection, f64, f64, f64, &[String])) {
    let generator = SignalGenerator::new(Config::default());
    let output = generator.generate_signal(&scenario.input);

    let direction = output.direction;
    let conf = output.confidence;
    let sl = output.recommended_sl_pct;
    let tp = output.recommended_tp_pct;
    let reason_labels: Vec<String> = output.reasons.iter().map(|r| r.description.clone()).collect();

    assertion(&direction, conf, sl, tp, &reason_labels);
}

fn approx_range(value: f64, min: f64, max: f64) -> bool {
    value >= min && value <= max
}

fn contains_reason(reasons: &[String], needle: &str) -> bool {
    reasons.iter().any(|desc| desc.contains(needle))
}

#[test]
fn strong_bullish_signal() {
    let scenario = Scenario::new(80.0, 10.0, 30.0, 15.0, 40_000.0);

    run_scenario(scenario, |direction, confidence, sl, tp, reasons| {
        assert_eq!(*direction, SignalDirection::Long, "direction mismatch");
        assert!(
            approx_range(confidence, 0.7, 1.0),
            "confidence out of strong range ({confidence})"
        );
        assert!(tp > 0.0 && sl < 0.02, "SL/TP scaling unexpected");
        assert!(contains_reason(reasons, "MACD"), "missing MACD reason");
        assert!(contains_reason(reasons, "RSI"), "missing RSI reason");
        assert!(contains_reason(reasons, "Histogram"), "missing histogram reason");
    });
}

#[test]
fn strong_bearish_signal() {
    let scenario = Scenario::new(-80.0, -10.0, -30.0, 85.0, 38_000.0);

    run_scenario(scenario, |direction, confidence, sl, tp, reasons| {
        assert_eq!(*direction, SignalDirection::Short, "direction mismatch");
        assert!(
            approx_range(confidence, 0.7, 1.0),
            "confidence out of strong range ({confidence})"
        );
        assert!(tp > 0.0 && sl < 0.02, "SL/TP scaling unexpected");
        assert!(contains_reason(reasons, "MACD"), "missing MACD reason");
        assert!(contains_reason(reasons, "RSI"), "missing RSI reason");
        assert!(contains_reason(reasons, "Histogram"), "missing histogram reason");
    });
}

#[test]
fn borderline_long_signal() {
    let scenario = Scenario::new(30.0, 10.0, 15.0, 33.0, 39_000.0);

    run_scenario(scenario, |direction, confidence, sl, tp, reasons| {
        assert_eq!(*direction, SignalDirection::Long, "direction mismatch");
        assert!(
            approx_range(confidence, 0.3, 0.5),
            "confidence not moderate ({confidence})"
        );
        assert!(tp > 0.0 && sl < 0.02, "SL/TP scaling unexpected");
        assert!(contains_reason(reasons, "MACD"), "missing MACD reason");
        assert!(contains_reason(reasons, "Histogram"), "missing histogram reason");
    });
}

#[test]
fn borderline_short_signal() {
    let scenario = Scenario::new(-30.0, -10.0, -15.0, 67.0, 39_500.0);

    run_scenario(scenario, |direction, confidence, sl, tp, reasons| {
        assert_eq!(*direction, SignalDirection::Short, "direction mismatch");
        assert!(
            approx_range(confidence, 0.3, 0.5),
            "confidence not moderate ({confidence})"
        );
        assert!(tp > 0.0 && sl < 0.02, "SL/TP scaling unexpected");
        assert!(contains_reason(reasons, "MACD"), "missing MACD reason");
        assert!(contains_reason(reasons, "Histogram"), "missing histogram reason");
    });
}

#[test]
fn neutral_signal_expectations() {
    let scenario = Scenario::new(2.0, 1.5, 0.2, 50.0, 40_500.0);

    run_scenario(scenario, |direction, confidence, sl, tp, reasons| {
        assert_eq!(*direction, SignalDirection::None, "direction should be neutral");
        assert!(
            confidence < 0.1,
            "confidence should stay low for neutral scenario ({confidence})"
        );
        assert_eq!(sl, 0.02, "SL should remain default when neutral");
        assert_eq!(tp, 0.04, "TP should remain default when neutral");
        assert!(reasons.is_empty() || !contains_reason(reasons, "RSI oversold"), "unexpected reasons");
    });
}

#[test]
fn macd_only_bullish_signal() {
    let scenario = Scenario::new(18.0, 10.0, 1.0, 48.0, 40_800.0);

    run_scenario(scenario, |direction, confidence, _, _, reasons| {
        assert!(
            matches!(*direction, SignalDirection::Long | SignalDirection::None),
            "direction should be long or neutral"
        );
        assert!(
            approx_range(confidence, 0.05, 0.2),
            "confidence should be partial ({confidence})"
        );
        assert!(contains_reason(reasons, "MACD"), "MACD reason required");
    });
}

#[test]
fn macd_only_bearish_signal() {
    let scenario = Scenario::new(-18.0, -10.0, -1.0, 52.0, 40_900.0);

    run_scenario(scenario, |direction, confidence, _, _, reasons| {
        assert!(
            matches!(*direction, SignalDirection::Short | SignalDirection::None),
            "direction should be short or neutral"
        );
        assert!(
            approx_range(confidence, 0.05, 0.2),
            "confidence should be partial ({confidence})"
        );
        assert!(contains_reason(reasons, "MACD"), "MACD reason required");
    });
}

#[test]
fn extreme_value_scaling() {
    let scenario = Scenario::new(500.0, -200.0, 150.0, 5.0, 41_000.0).with_funding(-0.1);

    run_scenario(scenario, |direction, confidence, _, _, reasons| {
        assert_eq!(*direction, SignalDirection::Long, "direction mismatch");
        assert!(
            approx_range(confidence, 0.9, 1.0),
            "confidence should cap near 1.0 ({confidence})"
        );
        assert!(contains_reason(reasons, "MACD"), "MACD reason required");
        assert!(contains_reason(reasons, "RSI"), "RSI reason required");
        assert!(contains_reason(reasons, "Histogram"), "Histogram reason required");
        assert!(
            contains_reason(reasons, "Funding rate"),
            "Funding reason expected"
        );
    });
}

