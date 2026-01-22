use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::env;

/// Processes templates by substituting variables
pub struct TemplateProcessor {
    variables: HashMap<String, String>,
}

impl TemplateProcessor {
    /// Create a new template processor with the given variables
    pub fn new(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }

    /// Substitute ${VAR_NAME} placeholders in the template string
    pub fn substitute(&self, template: &str) -> Result<String> {
        let re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        let mut result = template.to_string();

        for cap in re.captures_iter(template) {
            let var_name = &cap[1];
            let value = self
                .get_variable(var_name)
                .with_context(|| format!("Variable ${{{}}} not found", var_name))?;

            result = result.replace(&format!("${{{}}}", var_name), &value);
        }

        Ok(result)
    }

    /// Get a variable value, checking manual variables first, then environment
    fn get_variable(&self, name: &str) -> Option<String> {
        // Priority: 1. Manual variables, 2. Environment variables
        self.variables
            .get(name)
            .cloned()
            .or_else(|| env::var(name).ok())
    }

    /// Substitute variables in all values of a HashMap
    pub fn substitute_map(&self, map: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        for (key, value) in map {
            result.insert(key.clone(), self.substitute(value)?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_simple() {
        let mut vars = HashMap::new();
        vars.insert("API_KEY".to_string(), "secret123".to_string());
        let processor = TemplateProcessor::new(vars);

        let result = processor.substitute("Bearer ${API_KEY}").unwrap();
        assert_eq!(result, "Bearer secret123");
    }

    #[test]
    fn test_substitute_multiple() {
        let mut vars = HashMap::new();
        vars.insert("HOST".to_string(), "localhost".to_string());
        vars.insert("PORT".to_string(), "8080".to_string());
        let processor = TemplateProcessor::new(vars);

        let result = processor.substitute("http://${HOST}:${PORT}/api").unwrap();
        assert_eq!(result, "http://localhost:8080/api");
    }

    #[test]
    fn test_substitute_missing_var() {
        let processor = TemplateProcessor::new(HashMap::new());
        let result = processor.substitute("${MISSING}");
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_env_var() {
        env::set_var("TEST_ENV_VAR", "from_env");
        let processor = TemplateProcessor::new(HashMap::new());

        let result = processor.substitute("Value: ${TEST_ENV_VAR}").unwrap();
        assert_eq!(result, "Value: from_env");

        env::remove_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_manual_overrides_env() {
        env::set_var("TEST_VAR", "from_env");
        let mut vars = HashMap::new();
        vars.insert("TEST_VAR".to_string(), "from_manual".to_string());
        let processor = TemplateProcessor::new(vars);

        let result = processor.substitute("${TEST_VAR}").unwrap();
        assert_eq!(result, "from_manual");

        env::remove_var("TEST_VAR");
    }
}
