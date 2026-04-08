/// Prompt construction and LLM response parsing for domain name generation.

use super::SuggestionRequest;
use anyhow::{Context, Result};

/// Build the system prompt that establishes the LLM's role and output format.
pub fn build_system_prompt() -> String {
    r#"You are a world-class startup branding consultant. Your specialty is creating short, memorable, brandable domain names.

RULES:
- Suggest ONLY the base name (no TLD like .com, .io, etc.)
- Names must be easy to spell and pronounce
- Avoid hyphens and numbers
- Prefer invented/portmanteau words, clever combinations, or evocative single words
- Each name should be unique and distinct from the others

OUTPUT FORMAT:
You MUST respond with a valid JSON object and nothing else. No markdown, no explanation.
The JSON must have exactly one key "suggestions" containing an array of strings.

Example:
{"suggestions": ["nexora", "brindle", "vertiq", "cloudpeak", "luminary"]}"#
        .to_string()
}

/// Build the user prompt from a `SuggestionRequest`.
pub fn build_user_prompt(req: &SuggestionRequest) -> String {
    format!(
        r#"Generate {count} brandable domain name ideas for the following:

KEYWORDS: {keywords}
INDUSTRY: {industry}
BRAND PERSONALITY: {personality}
MAX CHARACTERS PER NAME: {max_len}

Remember: output ONLY a JSON object with a "suggestions" array. No other text."#,
        count = req.count,
        keywords = req.keywords,
        industry = req.industry,
        personality = req.personality.label(),
        max_len = req.max_length,
    )
}

/// Parse the LLM's response text into a list of domain name suggestions.
///
/// Tries structured JSON first, then falls back to regex extraction.
pub fn parse_suggestions(response: &str) -> Result<Vec<String>> {
    // Try direct JSON parse
    if let Ok(parsed) = try_parse_json(response) {
        if !parsed.is_empty() {
            return Ok(parsed);
        }
    }

    // Try to find JSON embedded in the response (e.g. markdown code blocks)
    if let Some(json_str) = extract_json_block(response) {
        if let Ok(parsed) = try_parse_json(&json_str) {
            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }
    }

    // Fallback: try to extract quoted strings
    let fallback = extract_quoted_strings(response);
    if !fallback.is_empty() {
        return Ok(fallback);
    }

    anyhow::bail!("Could not parse any domain suggestions from LLM response:\n{response}")
}

/// Attempt to parse a JSON response with a "suggestions" key.
fn try_parse_json(text: &str) -> Result<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct Response {
        suggestions: Vec<String>,
    }

    let trimmed = text.trim();
    let resp: Response =
        serde_json::from_str(trimmed).context("Failed to deserialize suggestions JSON")?;

    // Filter and clean
    let names: Vec<String> = resp
        .suggestions
        .into_iter()
        .map(|s| s.trim().to_lowercase().replace(' ', ""))
        .filter(|s| !s.is_empty() && s.len() <= 30)
        .collect();

    Ok(names)
}

/// Extract a JSON block from markdown-style ```json ... ``` fences or bare { ... }.
fn extract_json_block(text: &str) -> Option<String> {
    // Look for ```json ... ```
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            return Some(text[content_start..content_start + end].trim().to_string());
        }
    }
    // Look for ``` ... ```
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        if let Some(end) = text[content_start..].find("```") {
            let inner = text[content_start..content_start + end].trim();
            if inner.starts_with('{') {
                return Some(inner.to_string());
            }
        }
    }
    // Look for bare { ... }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return Some(text[start..=end].to_string());
            }
        }
    }
    None
}

/// Last-resort extraction: find all double-quoted strings.
fn extract_quoted_strings(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut word = String::new();
            for inner in chars.by_ref() {
                if inner == '"' {
                    break;
                }
                word.push(inner);
            }
            let cleaned = word.trim().to_lowercase().replace(' ', "");
            if !cleaned.is_empty()
                && cleaned.len() <= 30
                && cleaned != "suggestions"
                && cleaned.chars().all(|c| c.is_alphanumeric())
            {
                results.push(cleaned);
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_json() {
        let input = r#"{"suggestions": ["nexora", "brindle", "vertiq"]}"#;
        let result = parse_suggestions(input).unwrap();
        assert_eq!(result, vec!["nexora", "brindle", "vertiq"]);
    }

    #[test]
    fn test_parse_json_in_markdown() {
        let input = r#"Here are some suggestions:
```json
{"suggestions": ["cloudpeak", "mintflow"]}
```"#;
        let result = parse_suggestions(input).unwrap();
        assert_eq!(result, vec!["cloudpeak", "mintflow"]);
    }

    #[test]
    fn test_parse_fallback_quoted() {
        let input = r#"I suggest: "nexora", "brindle", "vertiq""#;
        let result = parse_suggestions(input).unwrap();
        assert_eq!(result, vec!["nexora", "brindle", "vertiq"]);
    }
}
