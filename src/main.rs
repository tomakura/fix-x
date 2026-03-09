#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]

mod clipboard;
mod config;
mod gui;
mod startup;

use std::mem::size_of;

use config::{AppConfig, RewriteTarget};
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
                LoadCursorW, LoadIconW, MF_BYCOMMAND, MF_CHECKED, MF_STRING, MSG, PostMessageW,
                PostQuitMessage, RegisterClassW, SW_HIDE, SendMessageW, SetForegroundWindow,
                SetWindowLongPtrW, ShowWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu,
                TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_CREATE,
                WM_DESTROY, WM_LBUTTONUP, WM_NCCREATE, WM_NULL, WM_RBUTTONUP, WNDCLASSW,
                WS_OVERLAPPEDWINDOW,
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
    fx_radio: HWND,
    vx_radio: HWND,
    startup_checkbox: HWND,
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

    pub(crate) unsafe fn sync_settings_controls(&self) {
        if self.settings.hwnd.0.is_null() {
            return;
        }

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
    let mut data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: LoadIconW(None, IDI_APPLICATION)?,
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

    let open_settings = to_wide("Open Settings");
    let enabled = to_wide("Enabled");
    let exit = to_wide("Exit");

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
