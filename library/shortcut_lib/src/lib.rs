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

/// This trait is used for conversion of
/// `SysMods` type to `String` and vice versa.
/// This trait hold in consideration the different
/// representation of `Ctrl` key on different OSs
pub trait ToFromString {
    /// Translate `SysMods` type to `String`
    fn to_string(key: SysMods) -> Option<String>;
    /// Translate `String` to `SysMods` type
    fn from_string(key: String) -> Option<SysMods>;
}

impl ToFromString for SysMods {
    /// Translate `SysMods` type to `String`
    ///
    /// Table conversion for target OS different from macos is:
    /// - SysMods::Shift => Some(format!("Shift"))
    /// - SysMods::Cmd => Some(format!("Ctrl"))
    /// - SysMods::AltCmd => Some(format!("Ctrl + Alt"))
    /// - SysMods::CmdShift => Some(format!("Ctrl + Shift"))
    ///
    /// Table conversion for macos is:
    /// - SysMods::Shift => Some(format!("Shift"))
    /// - SysMods::Cmd => Some(format!("Cmd"))
    /// - SysMods::AltCmd => Some(format!("Cmd + Alt"))
    /// - SysMods::CmdShift => Some(format!("Cmd + Shift"))
    fn to_string(key: SysMods) -> Option<String> {
        match key {
            SysMods::Shift => Some(format!("Shift")),
            #[cfg(not(target_os = "macos"))]
            SysMods::Cmd => Some(format!("Ctrl")),
            #[cfg(not(target_os = "macos"))]
            SysMods::AltCmd => Some(format!("Ctrl + Alt")),
            #[cfg(not(target_os = "macos"))]
            SysMods::CmdShift => Some(format!("Ctrl + Shift")),
            #[cfg(target_os = "macos")]
            SysMods::Cmd => Some(format!("Cmd")),
            #[cfg(target_os = "macos")]
            SysMods::AltCmd => Some(format!("Cmd + Alt")),
            #[cfg(target_os = "macos")]
            SysMods::CmdShift => Some(format!("Cmd + Shift")),
            _ => None,
        }
    }

    /// Translate `SysMods` type to `String`
    ///
    /// Table conversion for target OS different from macos is:
    /// - "Shift" => Some(SysMods::Shift)
    /// - "Ctrl" => Some(SysMods::Cmd)
    /// - "Ctrl + Alt" => Some(SysMods::AltCmd)
    /// - "Ctrl + Shift" => Some(SysMods::CmdShift)
    ///
    /// Table conversion for macos is:
    /// - "Shift" => Some(SysMods::Shift)
    /// - "Cmd" => Some(SysMods::Cmd)
    /// - "Cmd + Alt" => Some(SysMods::AltCmd)
    /// - "Cmd + Shift" => Some(SysMods::CmdShift)
    fn from_string(key: String) -> Option<SysMods> {
        match key.as_str() {
            "Shift" => Some(SysMods::Shift),
            #[cfg(not(target_os = "macos"))]
            "Ctrl" => Some(SysMods::Cmd),
            #[cfg(not(target_os = "macos"))]
            "Ctrl + Alt" => Some(SysMods::AltCmd),
            #[cfg(not(target_os = "macos"))]
            "Ctrl + Shift" => Some(SysMods::CmdShift),
            #[cfg(target_os = "macos")]
            "Cmd" => Some(SysMods::Cmd),
            #[cfg(target_os = "macos")]
            "Cmd + Alt" => Some(SysMods::AltCmd),
            #[cfg(target_os = "macos")]
            "Cmd + Shift" => Some(SysMods::CmdShift),
            _ => None,
        }
    }
}

/// This trait is used for conversion of
/// `Key` type to `SysMods` type.
/// This trait hold in consideration the different
/// representation of `Ctrl` key on different OSs
pub trait ToSysMods {
    /// Translate `Key` type to `SysMods` type variant
    /// that are composed from one Key value
    fn to_sysmods(key: Key) -> Option<SysMods>;
    /// Translate `Key` type to `SysMods` type variant
    /// that are composed from a tuple of Key values
    fn to_sysmods_combination(key: (Key, Key)) -> Option<SysMods>;
}

impl ToSysMods for Key {
    /// Translate `Key` type to `SysMods` type variant
    /// that are composed from one Key value
    ///
    /// Table conversion for target OS different from macos is:
    /// - Key::Shift => Some(SysMods::Shift)
    /// - Key::Control => Some(SysMods::Cmd)
    ///
    /// Table conversion for macos is:
    /// - Key::Shift => Some(SysMods::Shift)
    /// - Key::Meta => Some(SysMods::Cmd)
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

    /// Translate `Key` type to `SysMods` type variant
    /// that are composed from a tuple of Key values
    ///
    /// Table conversion for target OS different from macos is:
    /// - (Key::Control, Key::Alt) => Some(SysMods::AltCmd)
    /// - (Key::Control, Key::Shift) => Some(SysMods::CmdShift)
    ///
    /// Table conversion for macos is:
    /// - (Key::Meta, Key::Alt) => Some(SysMods::AltCmd)
    /// - (Key::Meta, Key::Shift) => Some(SysMods::CmdShift)
    fn to_sysmods_combination(keys: (Key, Key)) -> Option<SysMods> {
        match keys {
            #[cfg(not(target_os = "macos"))]
            (Key::Control, Key::Alt) => Some(SysMods::AltCmd),
            #[cfg(not(target_os = "macos"))]
            (Key::Control, Key::Shift) => Some(SysMods::CmdShift),
            #[cfg(target_os = "macos")]
            (Key::Meta, Key::Alt) => Some(SysMods::AltCmd),
            #[cfg(target_os = "macos")]
            (Key::Meta, Key::Shift) => Some(SysMods::CmdShift),
            _ => None,
        }
    }
}

/// This trait is used for conversion of
/// `SysMods` type to `usize` and vice versa
pub trait ToFromCode {
    /// Translate `SysMods` type to `usize`
    fn to_code(key: SysMods) -> Option<usize>;
    /// Translate `usize` type to `SysMods`
    fn from_code(code: usize) -> Option<SysMods>;
}

impl ToFromCode for SysMods {
    /// Translate `SysMods` type to `usize`
    ///
    /// The table conversion for all target OS:
    /// - SysMods::Shift => Some(0)
    /// - SysMods::Cmd => Some(1)
    /// - SysMods::AltCmd => Some(2)
    /// - SysMods::CmdShift => Some(3)
    fn to_code(key: SysMods) -> Option<usize> {
        match key {
            SysMods::Shift => Some(0),
            SysMods::Cmd => Some(1),
            SysMods::AltCmd => Some(2),
            SysMods::CmdShift => Some(3),
            _ => None,
        }
    }

    /// Translate `usize` type to `SysMods`
    ///
    /// The table conversion for all target OS:
    /// - 0 => Some(SysMods::Shift)
    /// - 1 => Some(SysMods::Cmd)
    /// - 2 => Some(SysMods::AltCmd)
    /// - 3 => Some(SysMods::CmdShift)
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

/// This enum is use for represent the different
/// actions linked to available shortcuts
///
/// Shortcuts available are:
/// - NewScreenshot
/// - Save
/// - SaveAs
#[derive(
    Debug, Data, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, Deserialize, Serialize,
)]
pub enum Action {
    NewScreenshot,
    Save,
    SaveAs,
}

impl Action {
    /// Translate `Action` type to `String`
    ///
    /// Table conversion is:
    /// - Action::NewScreenshot => "New screenshot"
    /// - Action::Save => "Save"
    /// - Action::SaveAs => "Save as"
    pub fn to_string(&self) -> String {
        match self {
            Action::NewScreenshot => String::from_str("New screenshot").unwrap(),
            Action::Save => String::from_str("Save").unwrap(),
            Action::SaveAs => String::from_str("Save as").unwrap(),
        }
    }

    /// Translate `String` type to `Action`
    ///
    /// Table conversion is:
    /// - "New screenshot" => Action::NewScreenshot
    /// - "Save" => Action::Save
    /// - "Save as" => Action::SaveAs
    pub fn from_string(action: String) -> Self {
        match action.as_str() {
            "New screenshot" => Action::NewScreenshot,
            "Save" => Action::Save,
            "Save as" => Action::SaveAs,
            _ => panic!("Could not translate string to enum Action!"),
        }
    }
}

/// Data structure for represent a shortcut
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

    pub fn get_value(&self) -> (usize, char) {
        return (self.code, self.character);
    }

    /// This function verify if a new combination of keys is valid
    /// for update an existing one
    pub fn validation_value(new_key_combination: Vector<Key>) -> Option<(usize, char)> {
        for k in new_key_combination.iter().enumerate() {
            match k.0 {
                0 => match k.1 {
                    Key::Control => continue,
                    Key::Shift => continue,
                    _ => {
                        return Option::None;
                    }
                },
                1 => match k.1 {
                    Key::Shift => continue,
                    Key::Alt => continue,
                    Key::Character(s) => {
                        let code = SysMods::to_code(
                            Key::to_sysmods(new_key_combination[0].clone()).unwrap(),
                        )
                        .unwrap();
                        let char = s.chars().collect::<Vec<char>>()[0];

                        return Some((code, char));
                    }
                    _ => {
                        return Option::None;
                    }
                },
                2 => match k.1 {
                    Key::Character(s) => {
                        let code = SysMods::to_code(
                            Key::to_sysmods_combination((
                                new_key_combination[0].to_owned(),
                                new_key_combination[1].to_owned(),
                            ))
                            .unwrap(),
                        )
                        .unwrap();
                        let char = s.chars().collect::<Vec<char>>()[0];

                        return Some((code, char));
                    }
                    _ => {
                        return Option::None;
                    }
                },
                _ => {}
            }
        }

        return Option::None;
    }

    /// Translate `Shortcut` type to `String`
    pub fn to_string(&self) -> String {
        let s = self.clone();

        return format!(
            "{} + {}",
            SysMods::to_string(SysMods::from_code(s.code).unwrap()).unwrap(),
            s.character
        );
    }
}

/// This data type is used to serialize and deserialize
/// data to/from file to save user preferences about shortcut key combination
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Shortcuts {
    shortcuts: BTreeMap<Action, Shortcut>,
}

impl Data for Shortcuts {
    fn same(&self, other: &Self) -> bool {
        return self.shortcuts == other.shortcuts;
    }
}

impl Shortcuts {
    /// This function create a config file with the default
    /// implementation of shortcuts. The location of config
    /// file is `./conf` in `project` folder
    ///
    /// Table of default keys combination is:
    /// - Action::NewScreenshot => SysMods::Cmd + 'n'
    /// - Action::Save => SysMods::Cmd + 's'
    /// - Action::SaveAs => SysMods::CmdShift + 's'
    fn create_toml() {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(CONF_SHORTCUT_FILE_PATH)
            .expect("Unable to open shortcut_conf file");

        let mut new_shortcuts = Shortcuts::default();

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

    /// This function is use to retrive shortcuts from
    /// config file
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

    /// This function reset the user preferencesù
    /// to default implementation
    pub fn reset(self) -> Self {
        Shortcuts::create_toml();
        return Shortcuts::new();
    }

    /// This function is used to extract a keys combination
    /// for an shortcut, for dynamic update of gui menu.
    /// The type returned (`HotKey`) is the type
    /// used by Druid library for handle and manage shortcuts.
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

    /// This function is used to extract a `Shortcut`, for
    /// gui label and other
    pub fn extract_value_for_gui(&self, key: &Action) -> Option<Shortcut> {
        let shortcut = match self.shortcuts.get(key) {
            Some(s) => s.clone(),
            None => panic!("Unable to extract HotKey, Action does not exist"),
        };

        return Some(shortcut);
    }

    /// This function allows to update a shortcut
    /// with new user preference
    pub fn update_value(
        &mut self,
        key: Action,
        new_key_combination: Vector<Key>,
    ) -> Result<(), String> {
        if !self.shortcuts.contains_key(&key) {
            panic!("Unable to find specified shortcut");
        }

        let new_value: (usize, char) = match Shortcut::validation_value(new_key_combination) {
            Some(v) => v,
            None => {
                return Err(format!("Combination of keys is not valid!"));
            }
        };

        let mut used = false;
        self.shortcuts.iter().for_each(|k| {
            if k.1.get_value() == new_value {
                used = true;
            }
        });

        if used {
            return Err(format!("Combination of keys is already used!"));
        }

        self.shortcuts.remove(&key);

        self.shortcuts.insert(
            key,
            Shortcut::new(SysMods::from_code(new_value.0).unwrap(), new_value.1),
        );

        let toml_string = toml::to_string(&self).expect("Unable to encode data to toml format");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(CONF_SHORTCUT_FILE_PATH)
            .expect("Unable to open shortcut_conf file");

        file.write(toml_string.as_bytes())
            .expect("Could not write to shortcut_conf file");

        file.flush().expect("Could not write to shortcut_conf file");

        return Ok(());
    }
}

/// This data type is used to serialize and deserialize
/// data to/from file to save user preference about dir destination
/// of screenshots when are saved
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SavePath {
    save_path: PathBuf,
}

impl Data for SavePath {
    fn same(&self, other: &Self) -> bool {
        return self.save_path == other.save_path;
    }
}

impl SavePath {
    /// This function create a config file with the default
    /// dir to save shortcuts. The location of config
    /// file is `./conf` in `project` folder
    ///
    /// Default dir is the image dir of current user logged on pc
    /// for example: `C:\Users\Student\Pictures`
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

        new_save_path.save_path = img_dir;

        let toml_string =
            toml::to_string(&new_save_path).expect("Unable to encode data to toml format");

        file.write(toml_string.as_bytes())
            .expect("Could not write to save_path_conf file");

        file.flush()
            .expect("Could not write to save_path_conf file");
    }

    /// This function is use to retrive path from
    /// config file
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

    /// This function return the favourite save dir path
    pub fn get_save_path(&self) -> PathBuf {
        return self.save_path.clone();
    }

    /// This function update the favourite save dir path
    pub fn update_save_path(&mut self, new_save_path: PathBuf) {
        read_dir(&new_save_path).expect("Unable to find dir");

        self.save_path = new_save_path;

        let toml_string = toml::to_string(&self).expect("Unable to encode data to toml format");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(CONF_SAVEPATH_FILE_PATH)
            .expect("Unable to open save_path_conf file");

        file.write(toml_string.as_bytes())
            .expect("Could not write to save_path_conf file");

        file.flush()
            .expect("Could not write to save_path_conf file");
    }

    /// This function translate `SavePath` type to `String`
    pub fn to_string(&self) -> String {
        return String::from_str(self.save_path.clone().to_str().unwrap()).unwrap();
    }
}
