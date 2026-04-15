pub struct JsonBuffer {
    buffer: String,
}

impl JsonBuffer {
    pub fn new(capacity: usize) -> Self {
        let buffer = String::with_capacity(capacity);
        Self { buffer }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.buffer
    }
}

impl Default for JsonBuffer {
    fn default() -> Self {
        Self::new(512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_buffer_reuse() {
        let mut buffer = JsonBuffer::new(128);
        buffer.as_mut_string().push_str("{\"foo\":1}");
        assert_eq!(buffer.as_str(), "{\"foo\":1}");

        buffer.clear();
        assert_eq!(buffer.as_str(), "");

        buffer.as_mut_string().push_str("{\"bar\":2}");
        assert_eq!(buffer.as_str(), "{\"bar\":2}");
        assert!(buffer.as_mut_string().capacity() >= 128);
    }
}
