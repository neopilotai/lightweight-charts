use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct IndicatorState {
    pub values: HashMap<String, f64>,
    pub history: HashMap<String, Vec<f64>>,
}

impl IndicatorState {
    pub fn reset(&mut self) {
        self.values.clear();
        self.history.clear();
    }
}
