// Configuration utilities

pub struct Config {
    pub settings: Vec<(String, String)>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            settings: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.settings.retain(|(k, _)| k != &key);
        self.settings.push((key, value));
    }
}
