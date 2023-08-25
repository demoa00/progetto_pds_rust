use bincode;
use druid::{
    im::{OrdMap, Vector},
    keyboard_types::Key,
    Data,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    mem::size_of,
};
use strum_macros::EnumIter;
use Action::*;

/// Simple struct that represent keys combination
/// of an shortcut.
#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Data)]
pub struct ShortcutKey(String, String);

impl ShortcutKey {
    pub fn new(keys: &Vector<Key>) -> Self {
        let mut keys_clone = keys.clone();

        let key_0 = match keys_clone.pop_front().unwrap() {
            Key::Character(c) => c,
            _ => keys[0].to_string(),
        };

        let key_1 = match keys_clone.pop_front().unwrap() {
            Key::Character(c) => c,
            _ => keys[1].to_string(),
        };

        return ShortcutKey(key_0, key_1);
    }

    pub fn to_string(&self) -> String {
        return format!("{}+{}", self.0, self.1);
    }
}

/// This emun represent in verbose mode, all the possible actions
/// available:
/// - new screenshot
/// - save
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Data, EnumIter)]
pub enum Action {
    NewScreenshot,
    Save,
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            NewScreenshot => "New screenshot".to_string(),
            Save => "Save".to_string(),
        }
    }

    pub fn from_str(str: &str) -> Option<Action> {
        match str {
            "New screenshot" => Some(NewScreenshot),
            "Save" => Some(Save),
            _ => None,
        }
    }
}

/// This struct contains the collection of all shortcut
/// available, each entry contains keys combination and
/// the correlated action:
/// - new screenshot -> ctrl+n
/// - save -> ctrl+s
#[derive(Clone, Debug, PartialEq, Eq, Data)]
pub struct Shortcuts {
    map: OrdMap<ShortcutKey, Action>,
}

impl Shortcuts {
    /// Debug and inizialization function for writing
    /// shortcut on binary file
    pub fn new_shortcut_to_file(keys: &Vector<Key>) {
        let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("./shortcuts")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        file.write_all(
            &bincode::serialize(&(
                ShortcutKey::new(keys),
                Action::Save, /* Mettere enum corrispondente all'azione voluta */
            ))
            .unwrap(),
        )
        .expect("File writing failed!");

        file.flush().expect("File writing failed!");
    }

    /// Initialize and return the collection of shortcuts
    /// stored in a binary file ( ___./shortcuts___ )
    pub fn from_file() -> Self {
        let mut shortcuts: OrdMap<ShortcutKey, Action> = OrdMap::new();

        let mut file = match OpenOptions::new().read(true).open("./shortcuts") {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        loop {
            let mut buf: [u8; size_of::<(ShortcutKey, Action)>()] =
                [0; size_of::<(ShortcutKey, Action)>()];
            match file.read_exact(&mut buf) {
                Ok(_) => {
                    let s: (ShortcutKey, Action) = bincode::deserialize(&buf).unwrap();
                    shortcuts.insert(s.0, s.1);
                }
                Err(_) => break,
            }
        }

        return Self { map: shortcuts };
    }

    /// Extract and return the list of available shortcut keys combination
    pub fn extract_keys(&mut self) -> Vector<String> {
        let keys: Vector<String> = self.map.keys().cloned().map(|k| k.to_string()).collect();
        return keys;
    }

    /// Extract and return the list of available shortcut names
    pub fn extract_values(&mut self) -> Vector<String> {
        let values: Vector<String> = self.map.values().cloned().map(|a| a.to_string()).collect();
        return values;
    }

    /// Allows you to change a key combination for
    /// a given shortcut and save it in the shortcut
    /// configuration file ( ___./shortcuts___ )
    pub fn edit_shortcut(&mut self, old_shortcut: ShortcutKey, pressed_keys: &Vector<Key>) -> bool {
        match self.map.contains_key(&old_shortcut) {
            true => {
                let new_shortcut = ShortcutKey::new(pressed_keys);

                if self.map.contains_key(&new_shortcut) == true {
                    return false;
                }

                let function = self.map.remove(&old_shortcut).unwrap();

                self.map.insert(new_shortcut, function);

                let mut file = match OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open("./shortcuts")
                {
                    Ok(f) => f,
                    Err(e) => panic!("{}", e),
                };

                let map_clone = self.map.clone();
                for s in map_clone.into_iter() {
                    file.write_all(&bincode::serialize(&s).unwrap())
                        .expect("File writing failed!");
                }

                file.flush().expect("File writing failed!");

                return true;
            }
            false => {
                panic!("Shortcut doesn't exist!");
            }
        }
    }

    /// Given a key combination, it return the associeted
    /// action of the shortcut, if it exists
    pub fn extract_value(&self, keys: &Vector<Key>) -> Option<Action> {
        let s = ShortcutKey::new(keys);

        match self.map.get(&s).cloned() {
            Some(a) => Some(a),
            None => None,
        }
    }

    pub fn to_string(&self) -> Vec<(String, String)> {
        let mut result = vec![];
        for shortcut in &self.map {
            result.push((shortcut.0.to_string(), shortcut.1.to_string()));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    /* use super::*;

    #[test]
    fn it_works() {
        /* Write here */
    } */
}
