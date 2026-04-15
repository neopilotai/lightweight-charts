use crate::indicator::dsl::SignalNode;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    EMA { period: usize, field: String, out: String },
    RSI { period: usize, field: String, out: String },
    Highest { period: usize, field: String, out: String },
    Lowest { period: usize, field: String, out: String },
}

#[derive(Debug, Clone)]
pub struct CompiledIndicator {
    pub name: String,
    pub inputs: std::collections::HashMap<String, f64>,
    pub ops: Vec<Op>,
    pub outputs: Vec<String>,
    pub signals: Vec<SignalNode>,
}
