use std::collections::HashMap;
use zeroize::Zeroize;

pub struct Credentials {
    data: HashMap<String, String>,
}

impl Drop for Credentials {
    /// Wipes secret values from memory when the store is dropped, so plaintext
    /// secrets don't linger in freed heap allocations.
    fn drop(&mut self) {
        for value in self.data.values_mut() {
            value.zeroize();
        }
    }
}

impl Credentials {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn from_map(data: HashMap<String, String>) -> Self {
        Self { data }
    }

    pub fn to_map(&self) -> &HashMap<String, String> {
        &self.data
    }

    #[allow(unused)]
    pub fn to_map_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.data
    }

    pub fn add(&mut self, name: String, secret: String) -> Result<(), String> {
        if self.data.contains_key(&name) {
            return Err(format!("'{}' already exists.", name));
        }
        self.data.insert(name, secret);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.data.get(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.data.remove(name).is_some()
    }

    pub fn list(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::new()
    }
}
