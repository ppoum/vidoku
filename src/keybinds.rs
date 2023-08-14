use std::{collections::HashMap, hash::Hash};
use thiserror::Error;
use wasm_bindgen::prelude::*;

use crate::{
    actions::{self, Action},
    key::{self, Key},
};

#[derive(Debug, Eq, Clone, Copy)]
pub struct Keybind {
    pub key: Key,
    pub modifier: Option<Key>,
}

impl PartialEq for Keybind {
    /// Returns true if the keybinds are equivalent. This is the case when both the key and the
    /// modifier key are the same, with one exception. `Key::Alt` and `Key::Meta` are
    /// interchangeable, and two keybinds that have the same key, but either `Alt` or `Meta` will
    /// still equal eachother.
    fn eq(&self, other: &Self) -> bool {
        if self.key != other.key {
            return false;
        }

        // Same key, check if same modifier
        if self.modifier == other.modifier {
            return true; // Full equality
        }

        // Last chance, check if one has meta and the other alt
        (self.modifier == Some(Key::Alt) && other.modifier == Some(Key::Meta))
            || (self.modifier == Some(Key::Meta) && other.modifier == Some(Key::Alt))
    }
}

impl Hash for Keybind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Since Meta and Alt modifiers must have the same hash, use
        // Key::Alt for hashing if our modifier is Key::Meta
        self.key.hash(state);
        match self.modifier {
            Some(Key::Meta) => Some(Key::Alt).hash(state),
            _ => self.modifier.hash(state),
        }
    }
}

impl TryFrom<String> for Keybind {
    type Error = KeybindParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Keys follow Javascript's `keydown` event naming scheme
        // Modifier keys follow vim syntax:
        // S- = Shift, C- = Control, A- or M- = Meta
        match value.chars().filter(|&c| c == '-').count() {
            0 => {
                // No modifier keys, whole string should just be the character
                match value.try_into() {
                    Ok(key) => Ok(Keybind {
                        key,
                        modifier: None,
                    }),
                    Err(e) => Err(KeybindParsingError::Key(e)),
                }
            }
            1 => {
                // Modifier key + key, formatted as Mod-key (with some optional spaces)
                let mut iter = value.split('-').map(|s| s.trim()).map(String::from);
                let modifier = iter.next();
                let key = iter.next();
                if modifier.is_none() || key.is_none() {
                    return Err(KeybindParsingError::Format(value.clone()));
                }

                // Parse string into modifier key using vim syntax
                let modifier = match modifier.unwrap().as_ref() {
                    "S" => Key::Shift,
                    "C" => Key::Control,
                    "A" => Key::Alt,
                    "M" => Key::Meta,
                    _ => return Err(KeybindParsingError::ModifierKey(value)),
                };

                let key = match key.unwrap().try_into() {
                    Ok(k) => k,
                    Err(e) => return Err(KeybindParsingError::Key(e)),
                };

                // Make sure that modifier key is actually a modifier key to avoid
                // invalid binds such as V-x (the V key cannot be a modifier key)
                if !modifier.is_modifier() {
                    return Err(KeybindParsingError::ModifierKey(value.clone()));
                }

                Ok(Keybind {
                    key,
                    modifier: Some(modifier),
                })
            }
            _ => {
                // Invalid format 1-2-3...
                Err(KeybindParsingError::Format(value.clone()))
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum KeybindManagerError {
    #[error(transparent)]
    ActionParsingError(#[from] actions::ActionParsingError),

    #[error(transparent)]
    KeybindParsingError(#[from] KeybindParsingError),
    #[error("Wrongly formatted line: {0}")]
    Format(String),
}

// Allow since we only ever need to send this error type to JS, never receive it from JS
#[allow(clippy::from_over_into)]
impl Into<JsValue> for KeybindManagerError {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}

#[derive(Error, Debug)]
pub enum KeybindParsingError {
    #[error(transparent)]
    Key(#[from] key::KeyParseError),
    #[error("Invalid key used as modifier: {0}")]
    ModifierKey(String),
    #[error("Invalid keybind format: {0}")]
    Format(String),
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct KeybindManager {
    // TODO replace value with resulting action type
    binds: HashMap<Keybind, Action>,
}

// Methods exported to JS
#[wasm_bindgen]
impl KeybindManager {
    /// Generates a `KeybindManager` from the specified config's contents.
    pub fn with_config(config: &str) -> Result<KeybindManager, KeybindManagerError> {
        let mut binds = HashMap::new();
        for line in config.lines() {
            // Lines should have the following format:
            // Key = <action> or Mod-key = <action>
            // Case and spaces are ignored

            // Skip comment lines
            if line.starts_with("//") {
                continue;
            }

            let mut iter = line.split('=').map(|s| s.trim()).map(String::from);
            let bind = iter.next();
            let action = iter.next();
            // Should only have one equal sign (both vars not none, next iter element should be none)
            if bind.is_none() || action.is_none() || iter.next().is_some() {
                return Err(KeybindManagerError::Format(line.to_owned()));
            }
            let bind: Keybind = match bind.unwrap().try_into() {
                Ok(k) => k,
                Err(e) => return Err(KeybindManagerError::KeybindParsingError(e)),
            };
            let action: Action = match action.unwrap().try_into() {
                Ok(a) => a,
                Err(e) => return Err(KeybindManagerError::ActionParsingError(e)),
            };
            binds.insert(bind, action);
        }
        Ok(KeybindManager { binds })
    }
}

impl KeybindManager {
    pub fn get_action(&self, keybind: &Keybind) -> Option<&Action> {
        self.binds.get(keybind)
    }
}
