#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]

mod clipboard;
mod config;
mod gui;
mod i18n;
mod startup;

use std::{mem::size_of, path::PathBuf};

use config::{AppConfig, RewriteTarget};
use i18n::{Strings, UiLanguage};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, BM_SETCHECK, CREATESTRUCTW, CreatePopupMenu, CreateWindowExW,
                DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
                GetCursorPos, GetMessageW, GetWindowLongPtrW, IDC_ARROW, IDI_APPLICATION,
                IMAGE_ICON, LR_LOADFROMFILE, LoadCursorW, LoadIconW, LoadImageW, MF_BYCOMMAND,
                MF_CHECKED, MF_STRING, MSG, PostMessageW, PostQuitMessage, RegisterClassW, SW_HIDE,
                SendMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowTextW, ShowWindow,
                TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE,
                WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_CREATE, WM_DESTROY, WM_LBUTTONUP,
                WM_NCCREATE, WM_NULL, WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    core::{Error, PCWSTR, Result},
};

const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;
const MAIN_CLASS_NAME_TEXT: &str = "FixXMainWindow";
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_ICON_ID: u32 = 1;

const ID_TRAY_OPEN_SETTINGS: usize = 100;
const ID_TRAY_TOGGLE_ENABLED: usize = 101;
const ID_TRAY_EXIT: usize = 102;

#[derive(Default)]
pub(crate) struct SettingsControls {
    hwnd: HWND,
    enable_checkbox: HWND,
    target_label: HWND,
    fx_radio: HWND,
    vx_radio: HWND,
    startup_checkbox: HWND,
    language_label: HWND,
    language_auto_radio: HWND,
    language_ja_radio: HWND,
    language_en_radio: HWND,
    close_button: HWND,
}

pub(crate) struct AppState {
    hinstance: HINSTANCE,
    hwnd: HWND,
    settings: SettingsControls,
    config: AppConfig,
    last_clipboard_write: Option<String>,
}

impl AppState {
    fn new(hinstance: HINSTANCE, config: AppConfig) -> Self {
        Self {
            hinstance,
            hwnd: HWND::default(),
            settings: SettingsControls::default(),
            config,
            last_clipboard_write: None,
        }
    }

    pub(crate) unsafe fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        self.persist();
        self.sync_settings_controls();
    }

    pub(crate) unsafe fn set_target(&mut self, target: RewriteTarget) {
        self.config.target = target;
        self.persist();
        self.sync_settings_controls();
    }

    pub(crate) unsafe fn set_launch_on_startup(&mut self, enabled: bool) {
        self.config.launch_on_startup = enabled;
        self.persist();
        self.sync_settings_controls();
    }

    pub(crate) unsafe fn set_language(&mut self, language: UiLanguage) {
        self.config.language = language;
        self.persist();
        self.sync_settings_controls();
    }

    pub(crate) unsafe fn sync_settings_controls(&self) {
        if self.settings.hwnd.0.is_null() {
            return;
        }

        self.refresh_localized_text();

        let enabled = if self.config.enabled {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let startup = if self.config.launch_on_startup {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let fx_checked = if self.config.target == RewriteTarget::Fx {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let vx_checked = if self.config.target == RewriteTarget::Vx {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let language_auto = if self.config.language == UiLanguage::Auto {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let language_ja = if self.config.language == UiLanguage::Ja {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };
        let language_en = if self.config.language == UiLanguage::En {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        };

        let _ = SendMessageW(
            self.settings.enable_checkbox,
            BM_SETCHECK,
            Some(WPARAM(enabled)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.startup_checkbox,
            BM_SETCHECK,
            Some(WPARAM(startup)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.fx_radio,
            BM_SETCHECK,
            Some(WPARAM(fx_checked)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.vx_radio,
            BM_SETCHECK,
            Some(WPARAM(vx_checked)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.language_auto_radio,
            BM_SETCHECK,
            Some(WPARAM(language_auto)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.language_ja_radio,
            BM_SETCHECK,
            Some(WPARAM(language_ja)),
            Some(LPARAM(0)),
        );
        let _ = SendMessageW(
            self.settings.language_en_radio,
            BM_SETCHECK,
            Some(WPARAM(language_en)),
            Some(LPARAM(0)),
        );
    }

    pub(crate) fn strings(&self) -> &'static Strings {
        i18n::strings(self.config.language)
    }

    unsafe fn refresh_localized_text(&self) {
        let strings = self.strings();
        set_window_text(self.settings.hwnd, strings.settings_title);
        set_window_text(self.settings.enable_checkbox, strings.enable_checkbox);
        set_window_text(self.settings.target_label, strings.target_label);
        set_window_text(self.settings.fx_radio, strings.target_fx);
        set_window_text(self.settings.vx_radio, strings.target_vx);
        set_window_text(self.settings.startup_checkbox, strings.startup_checkbox);
        set_window_text(self.settings.language_label, strings.language_label);
        set_window_text(self.settings.language_auto_radio, strings.language_auto);
        set_window_text(self.settings.language_ja_radio, strings.language_ja);
        set_window_text(self.settings.language_en_radio, strings.language_en);
        set_window_text(self.settings.close_button, strings.close_button);
    }

    unsafe fn persist(&self) {
        let _ = self.config.save();
        if let Ok(exe_path) = std::env::current_exe() {
            let _ = startup::sync_launch_on_startup(self.config.launch_on_startup, &exe_path);
        }
    }

    unsafe fn handle_clipboard_update(&mut self) {
        if !self.config.enabled {
            return;
        }

        let Some(text) = clipboard::read_clipboard_text(self.hwnd).ok().flatten() else {
            return;
        };

        if self.last_clipboard_write.as_deref() == Some(text.as_str()) {
            self.last_clipboard_write = None;
            return;
        }

        let Some(rewritten) = clipboard::rewrite_url(&text, self.config.target) else {
            return;
        };

        if rewritten == text {
            return;
        }

        if clipboard::write_clipboard_text(self.hwnd, &rewritten).is_ok() {
            self.last_clipboard_write = Some(rewritten);
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
        let config = AppConfig::load();

        if let Ok(exe_path) = std::env::current_exe() {
            let _ = startup::sync_launch_on_startup(config.launch_on_startup, &exe_path);
        }

        register_main_window_class(hinstance)?;
        gui::register_settings_class(hinstance)?;

        let app = Box::new(AppState::new(hinstance, config));
        let app_ptr = Box::into_raw(app);
        let title = to_wide("fix-x");

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            main_class_name(),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance),
            Some(app_ptr.cast()),
        )?;

        (*app_ptr).hwnd = hwnd;

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let _ = Box::from_raw(app_ptr);
        Ok(())
    }
}

unsafe fn register_main_window_class(hinstance: HINSTANCE) -> Result<()> {
    let class = WNDCLASSW {
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hInstance: hinstance,
        hIcon: load_app_icon().unwrap_or(LoadIconW(None, IDI_APPLICATION)?),
        lpszClassName: main_class_name(),
        lpfnWndProc: Some(main_wnd_proc),
        ..Default::default()
    };

    if RegisterClassW(&class) == 0 {
        return Err(Error::from_thread());
    }

    Ok(())
}

unsafe fn add_tray_icon(hwnd: HWND) -> Result<()> {
    let icon = load_app_icon().unwrap_or(LoadIconW(None, IDI_APPLICATION)?);
    let mut data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: icon,
        ..Default::default()
    };
    copy_wide("fix-x", &mut data.szTip);

    Shell_NotifyIconW(NIM_ADD, &data).ok()
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        ..Default::default()
    };
    let _ = Shell_NotifyIconW(NIM_DELETE, &data);
}

unsafe fn show_tray_menu(hwnd: HWND, app: &AppState) -> Result<()> {
    let menu = CreatePopupMenu()?;
    let strings = app.strings();

    let open_settings = to_wide(strings.tray_open_settings);
    let enabled = to_wide(strings.tray_enabled);
    let exit = to_wide(strings.tray_exit);

    AppendMenuW(
        menu,
        MF_STRING,
        ID_TRAY_OPEN_SETTINGS,
        PCWSTR(open_settings.as_ptr()),
    )?;

    let enabled_flags = if app.config.enabled {
        MF_STRING | MF_BYCOMMAND | MF_CHECKED
    } else {
        MF_STRING | MF_BYCOMMAND
    };
    AppendMenuW(
        menu,
        enabled_flags,
        ID_TRAY_TOGGLE_ENABLED,
        PCWSTR(enabled.as_ptr()),
    )?;
    AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, PCWSTR(exit.as_ptr()))?;

    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN,
        point.x,
        point.y,
        Some(0),
        hwnd,
        None,
    );
    let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    DestroyMenu(menu)?;
    Ok(())
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
            LRESULT(1)
        }
        WM_CREATE => {
            if add_tray_icon(hwnd).is_err() {
                return LRESULT(-1);
            }

            if windows::Win32::System::DataExchange::AddClipboardFormatListener(hwnd).is_err() {
                remove_tray_icon(hwnd);
                return LRESULT(-1);
            }

            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(app) = app_from_hwnd(hwnd) {
                match loword(wparam.0) as usize {
                    ID_TRAY_OPEN_SETTINGS => {
                        let _ = gui::open_settings_window(app);
                    }
                    ID_TRAY_TOGGLE_ENABLED => app.set_enabled(!app.config.enabled),
                    ID_TRAY_EXIT => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_CLIPBOARDUPDATE => {
            if let Some(app) = app_from_hwnd(hwnd) {
                app.handle_clipboard_update();
            }
            LRESULT(0)
        }
        WM_TRAYICON => match lparam.0 as u32 {
            WM_LBUTTONUP => {
                if let Some(app) = app_from_hwnd(hwnd) {
                    let _ = gui::open_settings_window(app);
                }
                LRESULT(0)
            }
            WM_RBUTTONUP | WM_CONTEXTMENU => {
                if let Some(app) = app_from_hwnd(hwnd) {
                    let _ = show_tray_menu(hwnd, app);
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        },
        WM_DESTROY => {
            if let Some(app) = app_from_hwnd(hwnd)
                && !app.settings.hwnd.0.is_null()
            {
                let _ = DestroyWindow(app.settings.hwnd);
            }

            let _ = windows::Win32::System::DataExchange::RemoveClipboardFormatListener(hwnd);
            remove_tray_icon(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn app_from_hwnd(hwnd: HWND) -> Option<&'static mut AppState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    ptr.as_mut()
}

pub(crate) fn copy_wide(value: &str, target: &mut [u16]) {
    let mut encoded = value.encode_utf16();
    for slot in target.iter_mut() {
        *slot = encoded.next().unwrap_or(0);
    }
}

pub(crate) fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) unsafe fn set_window_text(hwnd: HWND, text: &str) {
    if hwnd.0.is_null() {
        return;
    }

    let text = to_wide(text);
    let _ = SetWindowTextW(hwnd, PCWSTR(text.as_ptr()));
}

fn loword(value: usize) -> u16 {
    (value & 0xffff) as u16
}

fn main_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(
        CLASS_NAME
            .get_or_init(|| to_wide(MAIN_CLASS_NAME_TEXT))
            .as_ptr(),
    )
}

pub(crate) unsafe fn load_app_icon() -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
    for path in app_icon_candidates() {
        if !path.exists() {
            continue;
        }

        let wide = to_wide(&path.display().to_string());
        if let Ok(handle) = LoadImageW(
            None,
            PCWSTR(wide.as_ptr()),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE,
        ) {
            return Some(windows::Win32::UI::WindowsAndMessaging::HICON(handle.0));
        }
    }

    None
}

fn app_icon_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.push(dir.join("fix-x.ico"));
        candidates.push(dir.join("logo.ico"));
        candidates.push(dir.join("assets").join("logo.ico"));

        if let Some(parent) = dir.parent() {
            candidates.push(parent.join("assets").join("logo.ico"));
            if let Some(grand) = parent.parent() {
                candidates.push(grand.join("assets").join("logo.ico"));
            }
        }
    }
    candidates
}
