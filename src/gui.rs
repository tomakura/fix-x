use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{COLOR_BTNFACE, DEFAULT_GUI_FONT, GetStockObject, GetSysColorBrush},
        UI::{
            Controls::{ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx},
            WindowsAndMessaging::{
                BM_GETCHECK, BN_CLICKED, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_GROUPBOX,
                BS_PUSHBUTTON, CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
                GWLP_USERDATA, GetWindowLongPtrW, HMENU, IDC_ARROW, LoadCursorW, RegisterClassW,
                SW_HIDE, SW_SHOW, SW_SHOWNORMAL, SendMessageW, SetForegroundWindow,
                SetWindowLongPtrW, ShowWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND,
                WM_CREATE, WM_NCCREATE, WM_SETFONT, WNDCLASSW, WS_CHILD, WS_GROUP, WS_OVERLAPPED,
                WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::{Error, PCWSTR, Result},
};

use crate::{AppState, SettingsControls, i18n::UiLanguage, to_wide};

const BST_CHECKED: usize = 1;
const SETTINGS_CLASS_NAME_TEXT: &str = "FixXSettingsWindow";

const ID_ENABLE_CHECKBOX: usize = 200;
const ID_TARGET_FX: usize = 201;
const ID_TARGET_VX: usize = 202;
const ID_STARTUP_CHECKBOX: usize = 203;
const ID_CLOSE_BUTTON: usize = 204;
const ID_LANGUAGE_AUTO: usize = 205;
const ID_LANGUAGE_JA: usize = 206;
const ID_LANGUAGE_EN: usize = 207;

struct ControlSpec<'a> {
    text: &'a str,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: WINDOW_STYLE,
}

pub(crate) unsafe fn register_settings_class(hinstance: HINSTANCE) -> Result<()> {
    let classes = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES,
    };
    let _ = InitCommonControlsEx(&classes);

    let class = WNDCLASSW {
        hInstance: hinstance,
        lpszClassName: settings_class_name(),
        lpfnWndProc: Some(settings_wnd_proc),
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hIcon: crate::load_app_icon().unwrap_or_default(),
        hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
        ..Default::default()
    };

    if RegisterClassW(&class) == 0 {
        return Err(Error::from_thread());
    }

    Ok(())
}

pub(crate) unsafe fn open_settings_window(app: &mut AppState) -> Result<()> {
    if !app.settings.hwnd.0.is_null() {
        app.sync_settings_controls();
        let _ = ShowWindow(app.settings.hwnd, SW_SHOW);
        let _ = SetForegroundWindow(app.settings.hwnd);
        return Ok(());
    }

    let title = to_wide(app.strings().settings_title);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        settings_class_name(),
        PCWSTR(title.as_ptr()),
        WS_OVERLAPPED | WS_SYSMENU,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        376,
        390,
        Some(app.hwnd),
        None,
        Some(app.hinstance),
        Some((app as *mut AppState).cast()),
    )?;

    app.settings.hwnd = hwnd;
    app.sync_settings_controls();
    let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
    let _ = SetForegroundWindow(hwnd);
    Ok(())
}

unsafe extern "system" fn settings_wnd_proc(
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
            if let Some(app) = get_app(hwnd) {
                let strings = app.strings();

                app.settings = SettingsControls {
                    hwnd,
                    enable_checkbox: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.enable_checkbox,
                            id: ID_ENABLE_CHECKBOX,
                            x: 22,
                            y: 20,
                            width: 300,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP,
                        },
                    ),
                    target_label: create_groupbox(
                        app.hinstance,
                        hwnd,
                        strings.target_label,
                        18,
                        58,
                        320,
                        86,
                    ),
                    fx_radio: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.target_fx,
                            id: ID_TARGET_FX,
                            x: 34,
                            y: 86,
                            width: 270,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP | WS_GROUP,
                        },
                    ),
                    vx_radio: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.target_vx,
                            id: ID_TARGET_VX,
                            x: 34,
                            y: 112,
                            width: 270,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP,
                        },
                    ),
                    startup_checkbox: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.startup_checkbox,
                            id: ID_STARTUP_CHECKBOX,
                            x: 22,
                            y: 156,
                            width: 314,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP,
                        },
                    ),
                    language_label: create_groupbox(
                        app.hinstance,
                        hwnd,
                        strings.language_label,
                        18,
                        196,
                        320,
                        110,
                    ),
                    language_auto_radio: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.language_auto,
                            id: ID_LANGUAGE_AUTO,
                            x: 34,
                            y: 224,
                            width: 270,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP | WS_GROUP,
                        },
                    ),
                    language_ja_radio: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.language_ja,
                            id: ID_LANGUAGE_JA,
                            x: 34,
                            y: 250,
                            width: 270,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP,
                        },
                    ),
                    language_en_radio: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.language_en,
                            id: ID_LANGUAGE_EN,
                            x: 34,
                            y: 276,
                            width: 270,
                            height: 24,
                            style: WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP,
                        },
                    ),
                    close_button: create_button(
                        app.hinstance,
                        hwnd,
                        &ControlSpec {
                            text: strings.close_button,
                            id: ID_CLOSE_BUTTON,
                            x: 246,
                            y: 324,
                            width: 92,
                            height: 30,
                            style: WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP,
                        },
                    ),
                };

                app.sync_settings_controls();
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(app) = get_app(hwnd) {
                let command_id = loword(wparam.0) as usize;
                let notify_code = hiword(wparam.0);

                match command_id {
                    ID_ENABLE_CHECKBOX if notify_code == BN_CLICKED as u16 => {
                        app.set_enabled(is_checked(app.settings.enable_checkbox));
                    }
                    ID_TARGET_FX if notify_code == BN_CLICKED as u16 => {
                        app.set_target(crate::config::RewriteTarget::Fx);
                    }
                    ID_TARGET_VX if notify_code == BN_CLICKED as u16 => {
                        app.set_target(crate::config::RewriteTarget::Vx);
                    }
                    ID_STARTUP_CHECKBOX if notify_code == BN_CLICKED as u16 => {
                        app.set_launch_on_startup(is_checked(app.settings.startup_checkbox));
                    }
                    ID_LANGUAGE_AUTO if notify_code == BN_CLICKED as u16 => {
                        app.set_language(UiLanguage::Auto);
                    }
                    ID_LANGUAGE_JA if notify_code == BN_CLICKED as u16 => {
                        app.set_language(UiLanguage::Ja);
                    }
                    ID_LANGUAGE_EN if notify_code == BN_CLICKED as u16 => {
                        app.set_language(UiLanguage::En);
                    }
                    ID_CLOSE_BUTTON if notify_code == BN_CLICKED as u16 => {
                        let _ = ShowWindow(hwnd, SW_HIDE);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn get_app(hwnd: HWND) -> Option<&'static mut AppState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
    ptr.as_mut()
}

unsafe fn create_button(hinstance: HINSTANCE, parent: HWND, spec: &ControlSpec<'_>) -> HWND {
    let text_wide = to_wide(spec.text);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        button_class_name(),
        PCWSTR(text_wide.as_ptr()),
        WS_CHILD | WS_VISIBLE | spec.style,
        spec.x,
        spec.y,
        spec.width,
        spec.height,
        Some(parent),
        Some(HMENU(spec.id as *mut core::ffi::c_void)),
        Some(hinstance),
        None,
    )
    .unwrap_or_default();
    apply_default_font(hwnd);
    hwnd
}

unsafe fn create_groupbox(
    hinstance: HINSTANCE,
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    create_button(
        hinstance,
        parent,
        &ControlSpec {
            text,
            id: 0,
            x,
            y,
            width,
            height,
            style: WINDOW_STYLE(BS_GROUPBOX as u32),
        },
    )
}

unsafe fn apply_default_font(hwnd: HWND) {
    let font = GetStockObject(DEFAULT_GUI_FONT);
    let _ = SendMessageW(
        hwnd,
        WM_SETFONT,
        Some(WPARAM(font.0 as usize)),
        Some(LPARAM(1)),
    );
}

unsafe fn is_checked(hwnd: HWND) -> bool {
    SendMessageW(hwnd, BM_GETCHECK, Some(WPARAM(0)), Some(LPARAM(0))).0 == BST_CHECKED as isize
}

fn loword(value: usize) -> u16 {
    (value & 0xffff) as u16
}

fn hiword(value: usize) -> u16 {
    ((value >> 16) & 0xffff) as u16
}

fn settings_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(
        CLASS_NAME
            .get_or_init(|| to_wide(SETTINGS_CLASS_NAME_TEXT))
            .as_ptr(),
    )
}

fn button_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(CLASS_NAME.get_or_init(|| to_wide("BUTTON")).as_ptr())
}
