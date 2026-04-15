use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndicatorDef {
    pub name: String,
    pub inputs: Option<HashMap<String, f64>>,
    pub logic: Vec<LogicNode>,
    pub signals: Option<Vec<SignalNode>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LogicNode {
    EMA {
        period: usize,
        field: String,
        output: String,
    },
    RSI {
        period: usize,
        field: String,
        output: String,
    },
    Highest {
        period: usize,
        field: String,
        output: String,
    },
    Lowest {
        period: usize,
        field: String,
        output: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignalNode {
    pub condition: String,
    pub action: String,
}
