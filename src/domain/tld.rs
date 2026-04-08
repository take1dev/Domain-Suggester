/// TLD expansion: given a base name, produce full domain candidates across configured TLDs.

/// The default set of popular TLDs to check.
pub const DEFAULT_TLDS: &[&str] = &[
    ".com", ".io", ".dev", ".app", ".co", ".net", ".org", ".ai", ".xyz", ".tech", ".me",
];

/// Given a list of base names and a list of TLDs, produce all combinations.
///
/// Example: `expand(&["nexora"], &[".com", ".io"])` → `["nexora.com", "nexora.io"]`
pub fn expand(base_names: &[String], tlds: &[String]) -> Vec<String> {
    let mut results = Vec::with_capacity(base_names.len() * tlds.len());
    for name in base_names {
        let clean = name.trim().to_lowercase();
        if clean.is_empty() {
            continue;
        }
        for tld in tlds {
            results.push(format!("{clean}{tld}"));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_basic() {
        let names = vec!["nexora".to_string(), "brindle".to_string()];
        let tlds = vec![".com".to_string(), ".io".to_string()];
        let result = expand(&names, &tlds);
        assert_eq!(
            result,
            vec!["nexora.com", "nexora.io", "brindle.com", "brindle.io"]
        );
    }

    #[test]
    fn test_expand_empty() {
        let names: Vec<String> = vec![];
        let tlds = vec![".com".to_string()];
        assert!(expand(&names, &tlds).is_empty());
    }
}
