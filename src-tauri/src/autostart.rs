//! Starting Shellman when Windows starts: a value in the user's own Run key (no admin rights needed),
//! which Task Manager also lists under Startup apps and can turn off. Started that way, Shellman opens
//! minimized.

use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Where Task Manager keeps its own on/off switch for each Run value.
pub const APPROVED_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
pub const VALUE_NAME: &str = "Shellman";
/// On the command line Windows runs at sign-in: open minimized.
pub const MINIMIZED_ARG: &str = "--minimized";

/// The command Windows runs at sign-in.
pub fn command(exe: &Path) -> String {
    format!("\"{}\" {MINIMIZED_ARG}", exe.display())
}

/// Whether Shellman starts with Windows: its Run value is there and Task Manager hasn't turned it off
/// (the first byte of the StartupApproved value is odd when it's off).
pub fn is_enabled(run_key: &str, approved_key: &str, name: &str) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let present = hkcu
        .open_subkey_with_flags(run_key, KEY_READ)
        .and_then(|k| k.get_value::<String, _>(name))
        .is_ok();
    let off_in_task_manager = hkcu
        .open_subkey_with_flags(approved_key, KEY_READ)
        .and_then(|k| k.get_raw_value(name))
        .map(|v| v.bytes.first().is_some_and(|b| b % 2 == 1))
        .unwrap_or(false);
    present && !off_in_task_manager
}

/// Turns starting with Windows on (for `exe`) or off. Turning it on also clears Task Manager's "off", so
/// the checkbox and Task Manager agree.
pub fn set(run_key: &str, approved_key: &str, name: &str, exe: &Path, on: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(approved) = hkcu.open_subkey_with_flags(approved_key, KEY_SET_VALUE) {
        let _ = approved.delete_value(name);
    }
    if on {
        let (key, _) = hkcu.create_subkey(run_key).map_err(|e| e.to_string())?;
        key.set_value(name, &command(exe)).map_err(|e| e.to_string())
    } else {
        match hkcu.open_subkey_with_flags(run_key, KEY_SET_VALUE) {
            Ok(key) => match key.delete_value(name) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
                _ => Ok(()),
            },
            Err(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway key per test, so the real Run key is never touched.
    struct TestKeys(String);
    impl TestKeys {
        fn new(name: &str) -> Self {
            Self(format!(r"Software\Shellman-test-autostart-{}-{name}", std::process::id()))
        }
        fn run(&self) -> String {
            format!(r"{}\Run", self.0)
        }
        fn approved(&self) -> String {
            format!(r"{}\Approved", self.0)
        }
    }
    impl Drop for TestKeys {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    #[test]
    fn command_quotes_the_path_and_starts_minimized() {
        assert_eq!(command(Path::new(r"C:\Users\me\AppData\Local\Shellman\shellman.exe")), r#""C:\Users\me\AppData\Local\Shellman\shellman.exe" --minimized"#);
    }

    #[test]
    fn turns_on_and_off() {
        let k = TestKeys::new("on-off");
        let exe = Path::new(r"C:\Program Files\Shellman\shellman.exe");
        assert!(!is_enabled(&k.run(), &k.approved(), VALUE_NAME), "off before anything is written");
        set(&k.run(), &k.approved(), VALUE_NAME, exe, true).unwrap();
        assert!(is_enabled(&k.run(), &k.approved(), VALUE_NAME));
        let written: String = RegKey::predef(HKEY_CURRENT_USER).open_subkey(k.run()).unwrap().get_value(VALUE_NAME).unwrap();
        assert_eq!(written, command(exe));
        set(&k.run(), &k.approved(), VALUE_NAME, exe, false).unwrap();
        assert!(!is_enabled(&k.run(), &k.approved(), VALUE_NAME));
        // Turning off what is already off is fine.
        set(&k.run(), &k.approved(), VALUE_NAME, exe, false).unwrap();
    }

    #[test]
    fn follows_task_manager_and_turning_on_clears_its_off() {
        let k = TestKeys::new("task-manager");
        let exe = Path::new(r"C:\Shellman\shellman.exe");
        set(&k.run(), &k.approved(), VALUE_NAME, exe, true).unwrap();
        // Task Manager writes 03 (off) or 02 (on) as the first byte, then a timestamp.
        let (approved, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(k.approved()).unwrap();
        let raw = |first: u8| winreg::RegValue { bytes: vec![first, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into(), vtype: winreg::enums::REG_BINARY };
        approved.set_raw_value(VALUE_NAME, &raw(3)).unwrap();
        assert!(!is_enabled(&k.run(), &k.approved(), VALUE_NAME), "turned off in Task Manager");
        approved.set_raw_value(VALUE_NAME, &raw(2)).unwrap();
        assert!(is_enabled(&k.run(), &k.approved(), VALUE_NAME), "turned back on in Task Manager");
        approved.set_raw_value(VALUE_NAME, &raw(3)).unwrap();
        set(&k.run(), &k.approved(), VALUE_NAME, exe, true).unwrap();
        assert!(is_enabled(&k.run(), &k.approved(), VALUE_NAME), "turning on clears Task Manager's off");
    }
}
