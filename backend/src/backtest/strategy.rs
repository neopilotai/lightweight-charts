pub fn check_signal(
    condition: &str,
    values: &std::collections::HashMap<String, f64>,
) -> bool {
    // example: "close > ema" or "rsi < 30"

    let parts: Vec<&str> = condition.split_whitespace().collect();
    if parts.len() != 3 {
        return false;
    }

    let left_str = parts[0];
    let op = parts[1];
    let right_str = parts[2];

    let left = if let Ok(val) = left_str.parse::<f64>() {
        val
    } else {
        values.get(left_str).cloned().unwrap_or(0.0)
    };

    let right = if let Ok(val) = right_str.parse::<f64>() {
        val
    } else {
        values.get(right_str).cloned().unwrap_or(0.0)
    };

    match op {
        ">" => left > right,
        "<" => left < right,
        ">=" => left >= right,
        "<=" => left <= right,
        "==" => left == right,
        "!=" => left != right,
        _ => false,
    }
}