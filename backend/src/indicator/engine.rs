use crate::indicator::ast::*;
use crate::indicator::ops::*;
use crate::indicator::state::*;
use crate::models::candle::Candle;
use std::collections::HashMap;

#[derive(Clone)]
pub struct IndicatorEngine {
    pub compiled: CompiledIndicator,
    pub state: IndicatorState,
    pub last_outputs: HashMap<String, f64>,
}

impl IndicatorEngine {
    pub fn new(compiled: CompiledIndicator) -> Self {
        Self {
            compiled,
            state: IndicatorState::default(),
            last_outputs: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.last_outputs.clear();
    }

    pub fn update(&mut self, candle: &Candle) -> HashMap<String, f64> {
        let mut outputs = HashMap::new();

        for op in &self.compiled.ops {
            match op {
                Op::EMA { period, field, out } => {
                    let price = get_field(candle, field);
                    let prev = self.state.values.get(out).cloned().unwrap_or(price);
                    let value = ema(prev, price, *period);

                    self.state.values.insert(out.clone(), value);
                    outputs.insert(out.clone(), value);
                }
                Op::RSI { period, field, out } => {
                    let price = get_field(candle, field);
                    let history = self.state.history.entry(field.clone()).or_default();
                    history.push(price);
                    trim_history(history, *period);

                    let value = rsi(history, *period);
                    self.state.values.insert(out.clone(), value);
                    outputs.insert(out.clone(), value);
                }
                Op::Highest { period, field, out } => {
                    let price = get_field(candle, field);
                    let history = self.state.history.entry(field.clone()).or_default();
                    history.push(price);
                    trim_history(history, *period);

                    let start = history.len().saturating_sub(*period);
                    let value = highest(&history[start..]);
                    self.state.values.insert(out.clone(), value);
                    outputs.insert(out.clone(), value);
                }
                Op::Lowest { period, field, out } => {
                    let price = get_field(candle, field);
                    let history = self.state.history.entry(field.clone()).or_default();
                    history.push(price);
                    trim_history(history, *period);

                    let start = history.len().saturating_sub(*period);
                    let value = lowest(&history[start..]);
                    self.state.values.insert(out.clone(), value);
                    outputs.insert(out.clone(), value);
                }
            }
        }

        self.last_outputs = outputs.clone();
        outputs
    }

    pub fn evaluate_signals(&self, candle: &Candle) -> Vec<String> {
        let mut actions = Vec::new();
        for signal in &self.compiled.signals {
            if let Ok(true) = self.evaluate_condition(&signal.condition, candle) {
                actions.push(signal.action.clone());
            }
        }
        actions
    }

    fn evaluate_condition(&self, condition: &str, candle: &Candle) -> Result<bool, String> {
        let operators = ["<=", ">=", "==", "!=", "<", ">"];
        let (lhs, op, rhs) = operators
            .iter()
            .find_map(|operator| {
                if let Some(pos) = condition.find(operator) {
                    let lhs = condition[..pos].trim();
                    let rhs = condition[pos + operator.len()..].trim();
                    Some((lhs, *operator, rhs))
                } else {
                    None
                }
            })
            .ok_or_else(|| "Unsupported condition format".to_string())?;

        let left = self.resolve_value(lhs, candle)?;
        let right = self.resolve_value(rhs, candle)?;

        Ok(match op {
            "<" => left < right,
            ">" => left > right,
            "<=" => left <= right,
            ">=" => left >= right,
            "==" => (left - right).abs() < f64::EPSILON,
            "!=" => (left - right).abs() >= f64::EPSILON,
            _ => false,
        })
    }

    fn resolve_value(&self, token: &str, candle: &Candle) -> Result<f64, String> {
        if let Ok(number) = token.parse::<f64>() {
            return Ok(number);
        }

        if let Some(value) = self.last_outputs.get(token) {
            return Ok(*value);
        }

        match token {
            "open" => Ok(candle.open),
            "high" => Ok(candle.high),
            "low" => Ok(candle.low),
            "close" => Ok(candle.close),
            "volume" => Ok(candle.volume),
            _ => Err(format!("Unknown token in condition: {}", token)),
        }
    }
}

fn trim_history(history: &mut Vec<f64>, period: usize) {
    let max_store = period.saturating_mul(4).max(100);
    if history.len() > max_store {
        let drain = history.len() - max_store;
        history.drain(0..drain);
    }
}

fn get_field(candle: &Candle, field: &str) -> f64 {
    match field {
        "open" => candle.open,
        "high" => candle.high,
        "low" => candle.low,
        "close" => candle.close,
        "volume" => candle.volume,
        _ => candle.close,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator::compiler::compile;
    use crate::indicator::dsl::IndicatorDef;
    use crate::models::candle::Candle;
    use serde_json::json;

    #[test]
    fn parse_compile_and_run_rsi_indicator() {
        let json_def = json!({
            "name": "rsi_test",
            "logic": [
                { "type": "rsi", "period": 3, "field": "close", "output": "rsi" }
            ]
        });

        let def: IndicatorDef = serde_json::from_value(json_def).unwrap();
        let compiled = compile(def).unwrap();
        let mut engine = IndicatorEngine::new(compiled);

        let candles = vec![
            Candle { time: 1, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1.0, rsi: None, ema12: None, ema26: None, macd: None, signal: None, histogram: None, bollinger_upper: None, bollinger_middle: None, bollinger_lower: None, stoch_k: None, stoch_d: None },
            Candle { time: 2, open: 100.0, high: 101.0, low: 99.0, close: 102.0, volume: 1.0, rsi: None, ema12: None, ema26: None, macd: None, signal: None, histogram: None, bollinger_upper: None, bollinger_middle: None, bollinger_lower: None, stoch_k: None, stoch_d: None },
            Candle { time: 3, open: 102.0, high: 103.0, low: 101.0, close: 101.0, volume: 1.0, rsi: None, ema12: None, ema26: None, macd: None, signal: None, histogram: None, bollinger_upper: None, bollinger_middle: None, bollinger_lower: None, stoch_k: None, stoch_d: None },
            Candle { time: 4, open: 101.0, high: 102.0, low: 100.0, close: 103.0, volume: 1.0, rsi: None, ema12: None, ema26: None, macd: None, signal: None, histogram: None, bollinger_upper: None, bollinger_middle: None, bollinger_lower: None, stoch_k: None, stoch_d: None },
        ];

        let outputs = candles.iter().map(|c| engine.update(c)).collect::<Vec<_>>();
        assert_eq!(outputs.last().unwrap().get("rsi").is_some(), true);
    }

    #[test]
    fn condition_signals_are_evaluated_against_last_outputs() {
        let json_def = json!({
            "name": "ema_signal",
            "logic": [
                { "type": "ema", "period": 3, "field": "close", "output": "ema" }
            ],
            "signals": [
                { "condition": "close > ema", "action": "buy" }
            ]
        });

        let def: IndicatorDef = serde_json::from_value(json_def).unwrap();
        let compiled = compile(def).unwrap();
        let mut engine = IndicatorEngine::new(compiled);

        let candle = Candle { time: 1, open: 1.0, high: 2.0, low: 1.0, close: 2.0, volume: 1.0, rsi: None, ema12: None, ema26: None, macd: None, signal: None, histogram: None, bollinger_upper: None, bollinger_middle: None, bollinger_lower: None, stoch_k: None, stoch_d: None };
        engine.update(&candle);

        let actions = engine.evaluate_signals(&candle);
        assert_eq!(actions, vec!["buy".to_string()]);
    }
}
