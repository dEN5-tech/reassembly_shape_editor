// Fixed for WASM support
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;
use std::sync::RwLock;
use once_cell::sync::Lazy;

type TranslationMap = HashMap<String, HashMap<String, String>>;

/// Current application language
static CURRENT_LANGUAGE: Lazy<RwLock<String>> = Lazy::new(|| {
    RwLock::new(String::from("en"))
});

/// Loaded translations
static TRANSLATIONS: Lazy<RwLock<TranslationMap>> = Lazy::new(|| {
    RwLock::new(load_translations().unwrap_or_else(|_| {
        eprintln!("Failed to load translations, using empty map");
        HashMap::new()
    }))
});

/// Load translations from the JSON file
#[cfg(not(target_arch = "wasm32"))]
fn load_translations() -> Result<TranslationMap, Box<dyn std::error::Error>> {
    let mut file = File::open("assets/translations.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let translations: TranslationMap = serde_json::from_str(&contents)?;
    Ok(translations)
}

/// Load translations for WebAssembly target
#[cfg(target_arch = "wasm32")]
fn load_translations() -> Result<TranslationMap, Box<dyn std::error::Error>> {
    // Include translations directly in the WASM binary
    let translations_json = include_str!("../assets/translations.json");
    let translations: TranslationMap = serde_json::from_str(translations_json)?;
    Ok(translations)
}

/// Get a translation for the given key in the current language
pub fn t(key: &str) -> String {
    let lang = CURRENT_LANGUAGE.read().unwrap().clone();
    
    if let Ok(translations) = TRANSLATIONS.read() {
        if let Some(lang_map) = translations.get(&lang) {
            if let Some(value) = lang_map.get(key) {
                return value.clone();
            }
        }
    }
    
    key.to_string()
}

/// Set the current language
pub fn set_language(lang: &str) -> bool {
    if let Ok(translations) = TRANSLATIONS.read() {
        if translations.contains_key(lang) {
            if let Ok(mut current_lang) = CURRENT_LANGUAGE.write() {
                *current_lang = lang.to_string();
                return true;
            }
        }
    }
    false
}

/// Get list of available languages
pub fn available_languages() -> Vec<String> {
    if let Ok(translations) = TRANSLATIONS.read() {
        translations.keys().cloned().collect()
    } else {
        vec![]
    }
}

/// Get the current language
pub fn get_current_language() -> String {
    CURRENT_LANGUAGE.read().unwrap().clone()
}

/// Reload translations from the file
#[cfg(not(target_arch = "wasm32"))]
pub fn reload_translations() -> Result<(), Box<dyn std::error::Error>> {
    let new_translations = load_translations()?;
    if let Ok(mut translations) = TRANSLATIONS.write() {
        *translations = new_translations;
    }
    Ok(())
}

/// Reload translations for WebAssembly target (no-op since we use embedded data)
#[cfg(target_arch = "wasm32")]
pub fn reload_translations() -> Result<(), Box<dyn std::error::Error>> {
    // In WASM, translations are embedded, so no need to reload from file
    Ok(())
}