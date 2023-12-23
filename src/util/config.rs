use std::collections::HashMap;

pub struct Config {
    kv: HashMap<String, String>,
}

impl Config {
    pub fn new_no_validation() -> Self {
        let mut kv = HashMap::new();
        kv.insert("run_validation_override".to_owned(), "false".to_owned());

        Self {
            kv,
        }
    }

    pub fn get_opt_bool(&self, key: &str) -> Option<bool> {
        match self.get_kv_opt(key) {
            Some(v) => {
                match v.as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => panic!("cannot parse string to bool: {}", v),
                }
            }
            None => None,
        }
    }

    fn get_kv_opt(&self, key: &str) -> Option<&String> {
        self.kv.get(key)
    }
}