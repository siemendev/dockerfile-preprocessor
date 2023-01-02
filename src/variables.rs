use std::collections::HashMap;

#[derive(Clone)]
pub struct VariableCollection {
    variables: HashMap<String, String>
}

impl VariableCollection {
    pub fn new() -> VariableCollection {
        VariableCollection {
            variables: HashMap::new()
        }
    }

    pub fn set(mut self, key: &str, value: &str) -> VariableCollection {
        self.variables.insert(String::from(key), String::from(value));

        self
    }

    pub fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = vec![];

        for key in self.variables.keys() {
            keys.push(key);
        }

        keys
    }

    pub fn has(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    pub fn get(&self, key: &str) -> &str {
        if self.has(key) {
            return self.variables.get(key).expect("Key not found, make sure you check the variable existence first!").as_str();
        }

        ""
    }
}