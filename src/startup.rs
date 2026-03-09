use std::{io, path::Path};

use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "fix-x";

pub fn sync_launch_on_startup(enabled: bool, exe_path: &Path) -> io::Result<()> {
    let mut store = WindowsRunKeyStore;
    sync_launch_on_startup_with_store(&mut store, enabled, exe_path)
}

fn sync_launch_on_startup_with_store<T: RunKeyStore>(
    store: &mut T,
    enabled: bool,
    exe_path: &Path,
) -> io::Result<()> {
    if enabled {
        store.set(APP_NAME, &quoted_command(exe_path))
    } else {
        store.delete(APP_NAME)
    }
}

fn quoted_command(exe_path: &Path) -> String {
    format!("\"{}\"", exe_path.display())
}

trait RunKeyStore {
    fn set(&mut self, name: &str, value: &str) -> io::Result<()>;
    fn delete(&mut self, name: &str) -> io::Result<()>;
}

struct WindowsRunKeyStore;

impl RunKeyStore for WindowsRunKeyStore {
    fn set(&mut self, name: &str, value: &str) -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(RUN_KEY_PATH)?;
        key.set_value(name, &value)
    }

    fn delete(&mut self, name: &str) -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        match hkcu.open_subkey_with_flags(RUN_KEY_PATH, winreg::enums::KEY_SET_VALUE) {
            Ok(key) => match key.delete_value(name) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use super::sync_launch_on_startup_with_store;

    #[derive(Default)]
    struct MockRunKeyStore {
        values: HashMap<String, String>,
    }

    impl super::RunKeyStore for MockRunKeyStore {
        fn set(&mut self, name: &str, value: &str) -> std::io::Result<()> {
            self.values.insert(name.to_string(), value.to_string());
            Ok(())
        }

        fn delete(&mut self, name: &str) -> std::io::Result<()> {
            self.values.remove(name);
            Ok(())
        }
    }

    #[test]
    fn enabling_startup_writes_run_key_value() {
        let mut store = MockRunKeyStore::default();
        let exe = Path::new("C:\\Program Files\\fix-x\\fix-x.exe");

        sync_launch_on_startup_with_store(&mut store, true, exe).unwrap();

        assert_eq!(
            store.values.get("fix-x").unwrap(),
            "\"C:\\Program Files\\fix-x\\fix-x.exe\""
        );
    }

    #[test]
    fn disabling_startup_deletes_run_key_value() {
        let mut store = MockRunKeyStore::default();
        store.values.insert("fix-x".into(), "value".into());

        sync_launch_on_startup_with_store(&mut store, false, Path::new("C:\\fix-x.exe")).unwrap();

        assert!(!store.values.contains_key("fix-x"));
    }
}
