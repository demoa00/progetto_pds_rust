use druid::im::Vector;
use druid::{keyboard_types::Key, Data, HotKey, SysMods};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, read_dir, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use strum_macros::EnumIter;

const CONFIG_SHORTCUT_FILE_PATH: &str = "./conf/shortcut_config.toml";
const CONFIG_SHORTCUT_FILE_NAME: &str = "shortcut_config.toml";

#[derive(
    Debug, Data, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, Deserialize, Serialize,
)]
pub enum Action {
    NewScreenshot,
    Save,
    SaveAs,
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::NewScreenshot => String::from_str("New screenshot").unwrap(),
            Action::Save => String::from_str("Save").unwrap(),
            Action::SaveAs => String::from_str("Save as").unwrap(),
        }
    }

    pub fn from_string(action: String) -> Self {
        match action.as_str() {
            "New screenshot" => Action::NewScreenshot,
            "Save" => Action::Save,
            "Save as" => Action::SaveAs,
            _ => panic!("Could not translate string to enum Action!"),
        }
    }
}

trait ToFromString {
    fn to_string(key: SysMods) -> Option<String>;
    fn from_string(key: String) -> Option<SysMods>;
}

impl ToFromString for SysMods {
    fn to_string(key: SysMods) -> Option<String> {
        match key {
            SysMods::Shift => Some(format!("Shift")),
            #[cfg(not(target_os = "macos"))]
            SysMods::Cmd => Some(format!("Ctrl")),
            #[cfg(not(target_os = "macos"))]
            SysMods::AltCmd => Some(format!("Alt + Ctrl")),
            #[cfg(not(target_os = "macos"))]
            SysMods::CmdShift => Some(format!("Ctrl + Shift")),
            #[cfg(target_os = "macos")]
            SysMods::Cmd => Some(format!("Cmd")),
            #[cfg(target_os = "macos")]
            SysMods::AltCmd => Some(format!("Alt + Cmd")),
            #[cfg(target_os = "macos")]
            SysMods::CmdShift => Some(format!("Cmd + Shift")),
            _ => None,
        }
    }

    fn from_string(key: String) -> Option<SysMods> {
        match key.as_str() {
            "Shift" => Some(SysMods::Shift),
            #[cfg(not(target_os = "macos"))]
            "Ctrl" => Some(SysMods::Cmd),
            #[cfg(not(target_os = "macos"))]
            "Alt + Ctrl" => Some(SysMods::AltCmd),
            #[cfg(not(target_os = "macos"))]
            "Ctrl + Shift" => Some(SysMods::CmdShift),
            #[cfg(target_os = "macos")]
            "Cmd" => Some(SysMods::Cmd),
            #[cfg(target_os = "macos")]
            "Alt + Cmd" => Some(SysMods::AltCmd),
            #[cfg(target_os = "macos")]
            "Cmd + Shift" => Some(SysMods::CmdShift),
            _ => None,
        }
    }
}

trait ToSysMods {
    fn to_sysmods(key: Key) -> Option<SysMods>;
    fn to_sysmods_combination(key: (Key, Key)) -> Option<SysMods>;
}

impl ToSysMods for Key {
    fn to_sysmods(key: Key) -> Option<SysMods> {
        match key {
            Key::Shift => Some(SysMods::Shift),
            #[cfg(not(target_os = "macos"))]
            Key::Control => Some(SysMods::Cmd),
            #[cfg(target_os = "macos")]
            Key::Meta => Some(SysMods::Cmd),
            _ => None,
        }
    }

    fn to_sysmods_combination(keys: (Key, Key)) -> Option<SysMods> {
        match keys {
            #[cfg(not(target_os = "macos"))]
            (Key::Alt, Key::Control) => Some(SysMods::AltCmd),
            #[cfg(not(target_os = "macos"))]
            (Key::Control, Key::Shift) => Some(SysMods::CmdShift),
            #[cfg(target_os = "macos")]
            (Key::Alt, Key::Meta) => Some(SysMods::AltCmd),
            #[cfg(target_os = "macos")]
            (Key::Meta, Key::Shift) => Some(SysMods::CmdShift),
            _ => None,
        }
    }
}

trait ToFromCode {
    fn to_code(key: SysMods) -> Option<usize>;
    fn from_code(code: usize) -> Option<SysMods>;
}

impl ToFromCode for SysMods {
    fn to_code(key: SysMods) -> Option<usize> {
        match key {
            SysMods::Shift => Some(0),
            SysMods::Cmd => Some(1),
            SysMods::AltCmd => Some(2),
            SysMods::CmdShift => Some(3),
            _ => None,
        }
    }

    fn from_code(code: usize) -> Option<SysMods> {
        match code {
            0 => Some(SysMods::Shift),
            1 => Some(SysMods::Cmd),
            2 => Some(SysMods::AltCmd),
            3 => Some(SysMods::CmdShift),
            _ => None,
        }
    }
}

#[derive(Debug, Data, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Shortcut {
    code: usize,
    character: char,
}

impl Shortcut {
    pub fn new(key: SysMods, character: char) -> Self {
        return Self {
            code: SysMods::to_code(key).unwrap(),
            character: character,
        };
    }

    pub fn to_string(&self) -> String {
        let s = self.clone();

        return format!(
            "{} {}",
            SysMods::to_string(SysMods::from_code(s.code).unwrap()).unwrap(),
            s.character
        );
    }
}

#[derive(Debug, Data, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Shortcuts {
    #[data(eq)]
    shortcuts: BTreeMap<Action, Shortcut>,
}

impl Shortcuts {
    fn create_toml() {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(CONFIG_SHORTCUT_FILE_PATH)
            .expect("Unable to open shortcut config file");

        let mut new_shortcuts = Shortcuts::default();
        let comment = "# AUTO GENERATED CODE - EDIT ONLY `code` AND `character` FIELDS\n\n# Possible value for `code`\n# Shift => 0\n# Cmd => 1\n# AltCmd => 2\n# CmdShift => 3\n\n";

        file.write_all(comment.as_bytes())
            .expect("Could not write to shortcut config file");

        new_shortcuts
            .shortcuts
            .insert(Action::NewScreenshot, Shortcut::new(SysMods::Cmd, 'n'));
        new_shortcuts
            .shortcuts
            .insert(Action::Save, Shortcut::new(SysMods::Cmd, 's'));
        new_shortcuts
            .shortcuts
            .insert(Action::SaveAs, Shortcut::new(SysMods::CmdShift, 's'));

        let toml_string =
            toml::to_string(&new_shortcuts).expect("Unable to encode data to toml format");

        file.write(toml_string.as_bytes())
            .expect("Could not write to shortcut config file");
    }

    fn from_toml() -> Self {
        let contents = match fs::read_to_string(CONFIG_SHORTCUT_FILE_PATH) {
            Ok(c) => c,
            Err(_) => panic!("Could not read file {:?}", CONFIG_SHORTCUT_FILE_PATH),
        };

        let new_shortcuts: Shortcuts = match toml::from_str(&contents) {
            Ok(s) => s,
            Err(_) => panic!("Unable to parse data from {:?}", CONFIG_SHORTCUT_FILE_PATH),
        };

        new_shortcuts.shortcuts.iter().for_each(|e1|{
            if new_shortcuts.shortcuts.iter().filter(|e2| e2.1 == e1.1).count() != 1 {
                panic!("Found duplicate values in shortcut config file");
            }
        });

        return new_shortcuts;
    }

    pub fn new() -> Self {
        let read_dir = match read_dir("./conf") {
            Ok(r) => r,
            Err(_) => panic!("Unable to read conf dir"),
        };

        let mut found = false;
        for e in read_dir {
            if e.unwrap().file_name() == CONFIG_SHORTCUT_FILE_NAME {
                found = true;
                break;
            }
        }

        if !found {
            Shortcuts::create_toml();
        }

        return Shortcuts::from_toml();
    }

    pub fn reset(self) -> Self {
        Shortcuts::create_toml();
        return Shortcuts::new();
    }

    pub fn extract_value(&self, key: Action) -> Option<HotKey> {
        let shortcut = match self.shortcuts.get(&key) {
            Some(s) => s.clone(),
            None => panic!("Unable to extract HotKey, Action does not exist"),
        };

        let sysmod = match SysMods::from_code(shortcut.code) {
            Some(m) => m,
            None => panic!("Unable to translate code to SysMods"),
        };

        return Some(HotKey::new(sysmod, shortcut.character.to_string().as_str()));
    }

    pub fn update_value(&mut self, key: Action, new_value: (SysMods, char)) {
        if !self.shortcuts.contains_key(&key) {
            panic!("Unable to update shortcut, Action does not exist")
        }

        self.shortcuts.remove(&key);

        self.shortcuts
            .insert(key, Shortcut::new(new_value.0, new_value.1));

        let toml_string =
            toml::to_string(&self.shortcuts).expect("Unable to encode data to toml format");

        fs::write(CONFIG_SHORTCUT_FILE_PATH, toml_string)
            .expect("Could not write to shortcut config file");
    }

    pub fn extract_actions(&self) -> Vector<Action> {
        let mut actions = Vector::new();

        for s in self.shortcuts.clone() {
            actions.push_back(s.0);
        }

        return actions;
    }

    pub fn extract_values(&self) -> Vector<Shortcut> {
        let mut keys = Vector::new();

        for s in self.shortcuts.clone() {
            keys.push_back(s.1);
        }

        return keys;
    }
}
