use directories::UserDirs;
use druid::im::Vector;
use druid::{keyboard_types::Key, Data, HotKey, SysMods};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, create_dir, read_dir, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use strum_macros::EnumIter;

const CONF_DIR_PATH: &str = "./conf";

const CONF_SHORTCUT_FILE_PATH: &str = "./conf/shortcut_conf.toml";
const CONF_SHORTCUT_FILE_NAME: &str = "shortcut_conf.toml";

const CONF_SAVEPATH_FILE_PATH: &str = "./conf/save_path_conf.toml";
const CONF_SAVEPATH_FILE_NAME: &str = "save_path_conf.toml";

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
            .truncate(true)
            .open(CONF_SHORTCUT_FILE_PATH)
            .expect("Unable to open shortcut_conf file");

        let mut new_shortcuts = Shortcuts::default();
        let comment = "# AUTO GENERATED FILE - EDIT ONLY `code` AND `character` FIELDS\n\n# Possible value for `code`\n# Shift => 0\n# Cmd => 1\n# AltCmd => 2\n# CmdShift => 3\n\n";

        file.write_all(comment.as_bytes())
            .expect("Could not write to shortcut_conf file");

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
            .expect("Could not write to shortcut_conf file");

        file.flush().expect("Could not write to shortcut_conf file");
    }

    fn from_toml() -> Self {
        let contents =
            fs::read_to_string(CONF_SHORTCUT_FILE_PATH).expect("Could not read shortcut_conf file");

        let new_shortcuts: Shortcuts =
            toml::from_str(&contents).expect("Unable to decode data from toml");

        let mut error = false;
        new_shortcuts.shortcuts.iter().for_each(|e1| {
            if new_shortcuts
                .shortcuts
                .iter()
                .filter(|e2| e2.1 == e1.1)
                .count()
                != 1
            {
                error = true
            }
        });

        if error {
            Shortcuts::create_toml();
            return Shortcuts::new();
        }

        return new_shortcuts;
    }

    pub fn new() -> Self {
        let read_dir = match read_dir(CONF_DIR_PATH) {
            Ok(r) => r,
            Err(_) => {
                create_dir(CONF_DIR_PATH).expect("Unable to create conf dir");
                read_dir(CONF_DIR_PATH).expect("Unable to read conf dir")
            }
        };

        let mut found = false;
        for e in read_dir {
            if e.unwrap().file_name() == CONF_SHORTCUT_FILE_NAME {
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

    pub fn extract_value_for_menu(&self, key: Action) -> Option<HotKey> {
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

    pub fn extract_value_for_gui(&self, key: Action) -> Option<Shortcut> {
        let shortcut = match self.shortcuts.get(&key) {
            Some(s) => s.clone(),
            None => panic!("Unable to extract HotKey, Action does not exist"),
        };

        return Some(shortcut);
    }

    pub fn update_value(&mut self, key: Action, new_value: (SysMods, char)) {
        if !self.shortcuts.contains_key(&key) {
            panic!("Unable to update shortcut, Action does not exist")
        }

        self.shortcuts.remove(&key);

        self.shortcuts
            .insert(key, Shortcut::new(new_value.0, new_value.1));

        let toml_string = toml::to_string(&self).expect("Unable to encode data to toml format");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(CONF_SHORTCUT_FILE_PATH)
            .expect("Unable to open shortcut_conf file");

        file.write(toml_string.as_bytes())
            .expect("Could not write to shortcut_conf file");

        file.flush().expect("Could not write to shortcut_conf file");
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

    pub fn extract_value_string(&self, key: &Action) -> Option<String> {
        Some("Alt + F4".to_string())
    }
}

#[derive(Debug, Data, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SavePath {
    #[data(eq)]
    save_path: PathBuf,
}

impl SavePath {
    fn create_toml() {
        let user_dirs = match UserDirs::new() {
            Some(d) => d,
            None => panic!("Unable to find user dir path"),
        };

        let img_dir = match user_dirs.picture_dir() {
            Some(dir) => dir.to_owned(),
            Option::None => panic!("Unable to find image dir path!"),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(CONF_SAVEPATH_FILE_PATH)
            .expect("Unable to open save_path_conf file");

        let mut new_save_path = SavePath::default();
        let comment = "# AUTO GENERATED FILE - EDIT ONLY `save_path` FIELD\n\n";

        file.write_all(comment.as_bytes())
            .expect("Could not write to save_path_conf file");

        new_save_path.save_path = img_dir;

        let toml_string =
            toml::to_string(&new_save_path).expect("Unable to encode data to toml format");

        file.write(toml_string.as_bytes())
            .expect("Could not write to save_path_conf file");

        file.flush()
            .expect("Could not write to save_path_conf file");
    }

    fn from_toml() -> Self {
        let contents = fs::read_to_string(CONF_SAVEPATH_FILE_PATH)
            .expect("Could not read save_path_conf file");

        let new_save_path: SavePath =
            toml::from_str(&contents).expect("Unable to decode data from toml");

        match read_dir(&new_save_path.save_path) {
            Ok(_) => {}
            Err(_) => {
                SavePath::create_toml();
                return SavePath::new();
            }
        }

        return new_save_path;
    }

    pub fn new() -> Self {
        let read_dir = match read_dir(CONF_DIR_PATH) {
            Ok(r) => r,
            Err(_) => {
                create_dir(CONF_DIR_PATH).expect("Unable to create conf dir");
                read_dir(CONF_DIR_PATH).expect("Unable to read conf dir")
            }
        };

        let mut found = false;
        for e in read_dir {
            if e.unwrap().file_name() == CONF_SAVEPATH_FILE_NAME {
                found = true;
                break;
            }
        }

        if !found {
            SavePath::create_toml();
        }

        return SavePath::from_toml();
    }

    pub fn get_save_path(&self) -> PathBuf {
        return self.save_path.clone();
    }

    pub fn update_save_path(&mut self, new_save_path: PathBuf) {
        read_dir(&new_save_path).expect("Unable to find dir");

        self.save_path = new_save_path;

        let toml_string = toml::to_string(&self).expect("Unable to encode data to toml format");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(CONF_SAVEPATH_FILE_PATH)
            .expect("Unable to open save_path_conf file");

        file.write(toml_string.as_bytes())
            .expect("Could not write to save_path_conf file");

        file.flush()
            .expect("Could not write to save_path_conf file");
    }

    pub fn to_string(&self) -> String {
        return String::from_str(self.save_path.clone().to_str().unwrap()).unwrap();
    }
}
