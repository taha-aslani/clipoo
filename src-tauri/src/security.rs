const EXCLUDED_PROCESS_MARKERS: &[&str] = &["keepass", "bitwarden", "1password"];

pub fn is_excluded_process_name(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    EXCLUDED_PROCESS_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

#[cfg(windows)]
pub fn is_excluded_foreground_app() -> bool {
    foreground_process_file_name().is_some_and(|name| is_excluded_process_name(&name))
}

#[cfg(not(windows))]
pub fn is_excluded_foreground_app() -> bool {
    false
}

#[cfg(windows)]
fn foreground_process_file_name() -> Option<String> {
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut process_id = 0_u32;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        if process_id == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if handle.is_null() {
            return None;
        }

        let mut buffer = [0_u16; 512];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(handle);

        if ok == 0 || length == 0 {
            return None;
        }

        let os_string = std::ffi::OsString::from_wide(&buffer[..length as usize]);
        Path::new(&os_string)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::is_excluded_process_name;

    #[test]
    fn blocks_password_managers() {
        assert!(is_excluded_process_name("KeePass.exe"));
        assert!(is_excluded_process_name("KeePassXC.exe"));
        assert!(is_excluded_process_name("Bitwarden.exe"));
        assert!(is_excluded_process_name("1Password.exe"));
        assert!(is_excluded_process_name("1password.exe"));
    }

    #[test]
    fn allows_ordinary_apps() {
        assert!(!is_excluded_process_name("chrome.exe"));
        assert!(!is_excluded_process_name("Code.exe"));
        assert!(!is_excluded_process_name("clipoo.exe"));
    }
}
