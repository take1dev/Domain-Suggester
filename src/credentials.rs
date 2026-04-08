/// Basic credential storage using a local credentials.json file.
/// This bypasses OS-level keyring issues on Windows for unpackaged binaries.

use std::collections::HashMap;
use std::fs;

const FILE_PATH: &str = "credentials.json";

/// Credential keys.
pub const KEY_OPENROUTER: &str = "openrouter_api_key";
pub const KEY_WHOISFREAKS: &str = "whoisfreaks_api_key";

fn load_map() -> HashMap<String, String> {
    if let Ok(data) = fs::read_to_string(FILE_PATH) {
        if let Ok(map) = serde_json::from_str(&data) {
            return map;
        }
    }
    HashMap::new()
}

fn save_map(map: &HashMap<String, String>) -> anyhow::Result<()> {
    let data = serde_json::to_string_pretty(map)?;
    fs::write(FILE_PATH, data)?;
    Ok(())
}

/// Save a credential to the local file.
pub fn save_credential(key: &str, value: &str) -> anyhow::Result<()> {
    let mut map = load_map();
    map.insert(key.to_string(), value.to_string());
    save_map(&map)?;
    tracing::info!("Saved credential: {key}");
    Ok(())
}

/// Load a credential from the local file.
pub fn load_credential(key: &str) -> Option<String> {
    let map = load_map();
    map.get(key).cloned().filter(|s| !s.is_empty())
}

/// Delete a credential from the local file.
pub fn delete_credential(key: &str) -> anyhow::Result<()> {
    let mut map = load_map();
    map.remove(key);
    save_map(&map)?;
    tracing::info!("Deleted credential: {key}");
    Ok(())
}
