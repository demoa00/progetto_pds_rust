use bincode;
use druid::{im::HashMap, keyboard_types::Key, Data, HotKey, SysMods};
use serde::{Deserialize, Serialize};
use std::fs::{copy, read_dir, remove_file};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    str::FromStr,
};
use strum_macros::EnumIter;

#[derive(Debug, Data, Clone, Hash, PartialEq, Eq, EnumIter, Deserialize, Serialize)]
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

#[derive(Debug, Data, Clone)]
pub struct Shortcuts {
    map: HashMap<Action, (String, String)>,
}

impl Shortcuts {
    fn create_default_shortcut_conf() {
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .open("./conf/default_shortcut")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        // NEW SCREENSHOT
        file.write(
            &bincode::serialize(&(
                Action::NewScreenshot,
                SysMods::to_code(SysMods::Cmd).unwrap(),
                Key::Character("n".to_string()).to_string(),
            ))
            .unwrap(),
        )
        .expect("Error in writing config file!");
        file.flush().expect("Error in flush config file!");

        // SAVE
        file.write(
            &bincode::serialize(&(
                Action::Save,
                SysMods::to_code(SysMods::Cmd).unwrap(),
                Key::Character("s".to_string()).to_string(),
            ))
            .unwrap(),
        )
        .expect("Error in writing config file!");
        file.flush().expect("Error in flush config file!");

        // SAVE AS
        file.write(
            &bincode::serialize(&(
                Action::SaveAs,
                SysMods::to_code(SysMods::CmdShift).unwrap(),
                Key::Character("s".to_string()).to_string(),
            ))
            .unwrap(),
        )
        .expect("Error in writing config file!");
        file.flush().expect("Error in flush config file!");
    }

    pub fn reset_shortcut_conf() {
        let read_dir = match read_dir("./conf") {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };

        let mut found = false;
        for e in read_dir {
            let name = e.unwrap().file_name();
            if name == "shortcut" {
                remove_file("./conf/shortcut").expect("Error in removing file!");
                break;
            }
            if name == "default_shortcut" {
                found = true;
            }
        }

        if !found {
            Shortcuts::create_default_shortcut_conf();
        }

        match copy("./conf/default_shortcut", "./conf/shortcut") {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        };
    }

    pub fn new() -> Self {
        let read_dir = match read_dir("./conf") {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };

        let mut found = false;
        for e in read_dir {
            if e.unwrap().file_name() == "shortcut" {
                found = true;
                break;
            }
        }

        if !found {
            Shortcuts::reset_shortcut_conf();
        }

        let mut file = match OpenOptions::new().read(true).open("./conf/shortcut") {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        let mut map: HashMap<Action, (String, String)> = HashMap::new();

        /*
            let size = bincode::serialized_size(&(Action::NewScreenshot, 0 as usize, Key::Character("n".to_string()).to_string())).unwrap();
            println!("{}", size); // size = 21
        */

        loop {
            let mut buf: [u8; 21] = [0; 21];

            match file.read_exact(&mut buf) {
                Ok(_) => {
                    let shortcut: (Action, usize, String) = bincode::deserialize(&buf).unwrap();

                    //println!("{:?}", shortcut);

                    let v = match SysMods::from_code(shortcut.1) {
                        Some(s) => match SysMods::to_string(s) {
                            Some(k) => k,
                            None => panic!("Error in parsing shortcut!"),
                        },
                        None => panic!("Error in parsing shortcut!"),
                    };

                    map.insert(shortcut.0, (v, shortcut.2));
                }
                Err(_) => {
                    break;
                }
            }
        }

        //println!("{:?}", map);

        return Self { map: map };
    }

    pub fn reset(self) -> Self{
        Shortcuts::reset_shortcut_conf();
        return Shortcuts::new();
    }

    pub fn extract_value(&self, k: Action) -> Option<HotKey> {
        let v = match self.map.get(&k) {
            Some(v) => v.clone(),
            None => panic!("Error in extracting action!"),
        };

        let s = match SysMods::from_string(v.0) {
            Some(s) => s,
            None => panic!("Error in parsing shortcut!"),
        };

        return Some(HotKey::new(s, v.1.as_str()));
    }

    pub fn update_value(&mut self, key: Action, new_value: (String, String)) {
        if !self.map.contains_key(&key) {
            panic!("Action does not exist!")
        }

        self.map.remove(&key);

        let v = match SysMods::from_string(new_value.0) {
            Some(s) => match SysMods::to_string(s) {
                Some(k) => k,
                None => panic!("Error in parsing shortcut!"),
            },
            None => panic!("Error in parsing shortcut!"),
        };

        self.map.insert(key, (v, new_value.1));

        let read_dir = match read_dir("./conf") {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };

        for e in read_dir {
            if e.unwrap().file_name() == "shortcut" {
                remove_file("./conf/shortcut").expect("Error in removing file!");
                break;
            }
        }

        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .open("./conf/shortcut")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        for e in self.map.clone() {
            file.write(&bincode::serialize(&(e.0, SysMods::to_code(SysMods::from_string(e.1.0).unwrap()).unwrap(), e.1 .1)).unwrap())
                .expect("Error in writing config file!");
            file.flush().expect("Error in flush config file!");
        }
    }
}
