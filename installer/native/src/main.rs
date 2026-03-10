#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Globalization::GetUserDefaultUILanguage,
        Graphics::Gdi::{COLOR_BTNFACE, DEFAULT_GUI_FONT, GetStockObject, GetSysColorBrush},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx},
            WindowsAndMessaging::{
                BM_GETCHECK, BN_CLICKED, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_GROUPBOX,
                BS_PUSHBUTTON, CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
                DestroyWindow, DispatchMessageW, ES_AUTOHSCROLL, ES_READONLY, GWLP_USERDATA,
                GetMessageW, GetWindowLongPtrW, HMENU, IDC_ARROW, LoadCursorW, MB_ICONERROR,
                MB_ICONINFORMATION, MB_OK, MSG, MessageBoxW, PostMessageW, PostQuitMessage,
                RegisterClassW, SW_SHOW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW,
                SetWindowTextW, ShowWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CLOSE,
                WM_COMMAND, WM_CREATE, WM_DESTROY, WM_NCCREATE, WM_SETFONT, WNDCLASSW, WS_BORDER,
                WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::PCWSTR,
};

const APP_EXE_BYTES: &[u8] = include_bytes!("../payload/fix-x.exe");
const ICON_BYTES: &[u8] = include_bytes!("../payload/fix-x.ico");
const UNINSTALL_SCRIPT: &str = include_str!("../../uninstall.ps1");
const WINDOW_CLASS: &str = "FixXInstallerWindow";
const BST_CHECKED: usize = 1;
const WM_INSTALL_FINISHED: u32 = WM_APP + 1;

const ID_PATH_VALUE: usize = 101;
const ID_LAUNCH_CHECKBOX: usize = 102;
const ID_INSTALL_BUTTON: usize = 103;
const ID_CANCEL_BUTTON: usize = 104;
const ID_DETAILS_GROUP: usize = 107;

fn main() {
    let options = InstallOptions::from_env();
    if options.silent {
        if let Err(error) = install(&options, default_launch_after_install()) {
            show_message(
                &localized("fix-x インストーラー", "fix-x Installer"),
                &format!(
                    "{}\n\n{}",
                    localized("インストールに失敗しました。", "Installation failed."),
                    error
                ),
                true,
            );
        }
        return;
    }

    unsafe {
        let hinstance: HINSTANCE = match GetModuleHandleW(None) {
            Ok(module) => HINSTANCE(module.0),
            Err(_) => return,
        };

        let classes = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES,
        };
        let _ = InitCommonControlsEx(&classes);

        let state = Box::new(InstallerState::new(hinstance, options));
        let state_ptr = Box::into_raw(state);

        if register_window_class(hinstance).is_err() {
            let _ = Box::from_raw(state_ptr);
            return;
        }

        let title = wide(&localized("fix-x セットアップ", "fix-x Setup"));
        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name(),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPED | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            500,
            340,
            None,
            None,
            Some(hinstance),
            Some(state_ptr.cast()),
        ) {
            Ok(hwnd) => hwnd,
            Err(_) => {
                let _ = Box::from_raw(state_ptr);
                return;
            }
        };

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            DispatchMessageW(&message);
        }

        let _ = Box::from_raw(state_ptr);
    }
}

#[derive(Clone)]
struct InstallOptions {
    silent: bool,
    no_launch: bool,
}

impl InstallOptions {
    fn from_env() -> Self {
        let mut options = Self {
            silent: false,
            no_launch: false,
        };

        for arg in env::args().skip(1) {
            match arg.as_str() {
                "--silent" => options.silent = true,
                "--no-launch" => options.no_launch = true,
                _ => {}
            }
        }

        options
    }
}

#[derive(Default)]
struct InstallerControls {
    launch_checkbox: HWND,
    install_button: HWND,
}

struct InstallerState {
    hinstance: HINSTANCE,
    hwnd: HWND,
    controls: InstallerControls,
    options: InstallOptions,
    install_root: PathBuf,
    installing: bool,
}

impl InstallerState {
    fn new(hinstance: HINSTANCE, options: InstallOptions) -> Self {
        Self {
            hinstance,
            hwnd: HWND::default(),
            controls: InstallerControls::default(),
            options,
            install_root: install_root().unwrap_or_else(|| PathBuf::from(r"C:\fix-x")),
            installing: false,
        }
    }
}

struct InstallFinished {
    error: Option<String>,
}

unsafe fn register_window_class(hinstance: HINSTANCE) -> windows::core::Result<()> {
    let class = WNDCLASSW {
        hInstance: hinstance,
        lpszClassName: class_name(),
        lpfnWndProc: Some(wnd_proc),
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
        ..Default::default()
    };

    if RegisterClassW(&class) == 0 {
        return Err(windows::core::Error::from_thread());
    }

    Ok(())
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            let state_ptr = create.lpCreateParams as *mut InstallerState;
            if let Some(state) = state_ptr.as_mut() {
                state.hwnd = hwnd;
            }
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            LRESULT(1)
        }
        WM_CREATE => {
            if let Some(state) = state_from_hwnd(hwnd) {
                state.controls = create_controls(state);
                return LRESULT(0);
            }
            LRESULT(-1)
        }
        WM_COMMAND => {
            if let Some(state) = state_from_hwnd(hwnd) {
                match loword(wparam.0) as usize {
                    ID_INSTALL_BUTTON if hiword(wparam.0) == BN_CLICKED as u16 => {
                        if state.installing {
                            return LRESULT(0);
                        }

                        state.installing = true;
                        set_window_text(
                            state.controls.install_button,
                            &localized("インストール中...", "Installing..."),
                        );
                        let launch =
                            !state.options.no_launch && is_checked(state.controls.launch_checkbox);
                        let options = state.options.clone();
                        let target_hwnd = hwnd.0 as isize;
                        thread::spawn(move || {
                            let result = InstallFinished {
                                error: install(&options, launch).err(),
                            };
                            let result_ptr = Box::into_raw(Box::new(result));
                            unsafe {
                                let _ = PostMessageW(
                                    Some(HWND(target_hwnd as *mut core::ffi::c_void)),
                                    WM_INSTALL_FINISHED,
                                    WPARAM(0),
                                    LPARAM(result_ptr as isize),
                                );
                            }
                        });
                    }
                    ID_CANCEL_BUTTON if hiword(wparam.0) == BN_CLICKED as u16 => {
                        if !state.installing {
                            let _ = DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_INSTALL_FINISHED => {
            if let Some(state) = state_from_hwnd(hwnd) {
                state.installing = false;
                set_window_text(
                    state.controls.install_button,
                    &localized("インストール", "Install"),
                );

                let result_ptr = lparam.0 as *mut InstallFinished;
                let result = Box::from_raw(result_ptr);

                if let Some(error) = &result.error {
                    show_message(
                        &localized("fix-x インストーラー", "fix-x Installer"),
                        &format!(
                            "{}\n\n{}",
                            localized("インストールに失敗しました。", "Installation failed."),
                            error
                        ),
                        true,
                    );
                } else {
                    show_message(
                        &localized("fix-x インストーラー", "fix-x Installer"),
                        &format!(
                            "{}\n{}",
                            localized("インストールが完了しました。", "Installation completed."),
                            state.install_root.display()
                        ),
                        false,
                    );
                    let _ = DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn create_controls(state: &InstallerState) -> InstallerControls {
    let title = create_static(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized("fix-x セットアップ", "fix-x Setup"),
            id: 0,
            x: 24,
            y: 18,
            width: 420,
            height: 28,
            style: WINDOW_STYLE(0),
        },
    );
    let subtitle = create_static(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized(
                "fix-x をこのPCにインストールします。",
                "Install fix-x on this PC.",
            ),
            id: 0,
            x: 24,
            y: 50,
            width: 430,
            height: 22,
            style: WINDOW_STYLE(0),
        },
    );
    let details_group = create_button(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized("インストール設定", "Installation options"),
            id: ID_DETAILS_GROUP,
            x: 18,
            y: 88,
            width: 446,
            height: 150,
            style: WINDOW_STYLE(BS_GROUPBOX as u32),
        },
    );
    let path_label = create_static(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized("インストール先", "Install location"),
            id: 0,
            x: 34,
            y: 118,
            width: 140,
            height: 20,
            style: WINDOW_STYLE(0),
        },
    );
    let path_value = create_edit(
        state.hinstance,
        state.hwnd,
        &state.install_root.display().to_string(),
        34,
        142,
        414,
        24,
    );
    let launch_checkbox = create_button(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized(
                "インストール完了後に fix-x を起動する",
                "Launch fix-x after installation",
            ),
            id: ID_LAUNCH_CHECKBOX,
            x: 34,
            y: 180,
            width: 290,
            height: 24,
            style: WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP,
        },
    );
    set_checked(launch_checkbox, default_launch_after_install());
    let cancel_button = create_button(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized("キャンセル", "Cancel"),
            id: ID_CANCEL_BUTTON,
            x: 260,
            y: 256,
            width: 90,
            height: 30,
            style: WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP,
        },
    );
    let install_button = create_button(
        state.hinstance,
        state.hwnd,
        &ControlSpec {
            text: &localized("インストール", "Install"),
            id: ID_INSTALL_BUTTON,
            x: 360,
            y: 256,
            width: 90,
            height: 30,
            style: WINDOW_STYLE(BS_DEFPUSHBUTTON as u32) | WS_TABSTOP,
        },
    );

    let _ = title;
    let _ = subtitle;
    let _ = details_group;
    let _ = path_label;
    let _ = path_value;
    let _ = cancel_button;
    InstallerControls {
        launch_checkbox,
        install_button,
    }
}

struct ControlSpec<'a> {
    text: &'a str,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: WINDOW_STYLE,
}

unsafe fn create_button(hinstance: HINSTANCE, parent: HWND, spec: &ControlSpec<'_>) -> HWND {
    let text_wide = wide(spec.text);
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

unsafe fn create_static(hinstance: HINSTANCE, parent: HWND, spec: &ControlSpec<'_>) -> HWND {
    let text_wide = wide(spec.text);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        static_class_name(),
        PCWSTR(text_wide.as_ptr()),
        WS_CHILD | WS_VISIBLE | spec.style,
        spec.x,
        spec.y,
        spec.width,
        spec.height,
        Some(parent),
        None,
        Some(hinstance),
        None,
    )
    .unwrap_or_default();
    apply_default_font(hwnd);
    hwnd
}

unsafe fn create_edit(
    hinstance: HINSTANCE,
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    let text_wide = wide(text);
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        edit_class_name(),
        PCWSTR(text_wide.as_ptr()),
        WS_CHILD
            | WS_VISIBLE
            | WS_BORDER
            | WINDOW_STYLE(ES_READONLY as u32)
            | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
        x,
        y,
        width,
        height,
        Some(parent),
        Some(HMENU(ID_PATH_VALUE as *mut core::ffi::c_void)),
        Some(hinstance),
        None,
    )
    .unwrap_or_default();
    apply_default_font(hwnd);
    hwnd
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

unsafe fn set_checked(hwnd: HWND, checked: bool) {
    let value = if checked { 1 } else { 0 };
    let _ = SendMessageW(hwnd, 0x00F1, Some(WPARAM(value)), Some(LPARAM(0)));
}

unsafe fn is_checked(hwnd: HWND) -> bool {
    SendMessageW(hwnd, BM_GETCHECK, Some(WPARAM(0)), Some(LPARAM(0))).0 == BST_CHECKED as isize
}

unsafe fn state_from_hwnd(hwnd: HWND) -> Option<&'static mut InstallerState> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InstallerState;
    ptr.as_mut()
}

fn install(options: &InstallOptions, launch_after_install: bool) -> Result<(), String> {
    let install_root = install_root().ok_or_else(|| {
        localized(
            "LOCALAPPDATA を取得できませんでした。",
            "Failed to resolve LOCALAPPDATA.",
        )
    })?;
    let start_menu_dir = start_menu_dir().ok_or_else(|| {
        localized(
            "スタートメニューの場所を取得できませんでした。",
            "Failed to resolve Start Menu path.",
        )
    })?;

    let installed_exe = install_root.join("fix-x.exe");
    let installed_icon = install_root.join("fix-x.ico");
    let uninstall_script = install_root.join("uninstall.ps1");

    let _ = Command::new("taskkill")
        .args(["/IM", "fix-x.exe", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    fs::create_dir_all(&install_root)
        .map_err(path_error("create install directory", &install_root))?;
    fs::create_dir_all(&start_menu_dir)
        .map_err(path_error("create start menu directory", &start_menu_dir))?;
    fs::write(&installed_exe, APP_EXE_BYTES)
        .map_err(path_error("write app executable", &installed_exe))?;
    fs::write(&installed_icon, ICON_BYTES).map_err(path_error("write icon", &installed_icon))?;
    fs::write(&uninstall_script, UNINSTALL_SCRIPT)
        .map_err(path_error("write uninstaller", &uninstall_script))?;

    create_shortcuts(
        &start_menu_dir.join("fix-x.lnk"),
        &start_menu_dir.join("Uninstall fix-x.lnk"),
        &installed_exe,
        &install_root,
        &installed_icon,
        &uninstall_script,
    )?;

    if launch_after_install && !options.no_launch {
        Command::new(&installed_exe)
            .current_dir(&install_root)
            .spawn()
            .map_err(path_error("launch installed app", &installed_exe))?;
    }

    Ok(())
}

fn default_launch_after_install() -> bool {
    true
}

fn install_root() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join("Programs").join("fix-x"))
}

fn start_menu_dir() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|base| {
        PathBuf::from(base)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
    })
}

fn create_shortcuts(
    app_shortcut_path: &Path,
    uninstall_shortcut_path: &Path,
    target_path: &Path,
    working_directory: &Path,
    icon_path: &Path,
    uninstall_script: &Path,
) -> Result<(), String> {
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
$app = $w.CreateShortcut('{app_shortcut}'); \
$app.TargetPath = '{target}'; \
$app.WorkingDirectory = '{working}'; \
$app.IconLocation = '{icon}'; \
$app.Save(); \
$uninstall = $w.CreateShortcut('{uninstall_shortcut}'); \
$uninstall.TargetPath = 'powershell.exe'; \
$uninstall.WorkingDirectory = '{working}'; \
$uninstall.IconLocation = '{icon}'; \
$uninstall.Arguments = '-NoProfile -ExecutionPolicy Bypass -File \"{uninstall_script}\"'; \
$uninstall.Save()",
        app_shortcut = ps_single_quote(&app_shortcut_path.display().to_string()),
        uninstall_shortcut = ps_single_quote(&uninstall_shortcut_path.display().to_string()),
        target = ps_single_quote(&target_path.display().to_string()),
        working = ps_single_quote(&working_directory.display().to_string()),
        icon = ps_single_quote(&icon_path.display().to_string()),
        uninstall_script = ps_single_quote(&uninstall_script.display().to_string()),
    );

    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(path_error(
            "run PowerShell for shortcut creation",
            app_shortcut_path,
        ))?;

    if status.success() {
        Ok(())
    } else {
        Err(localized(
            "ショートカット作成に失敗しました",
            "Failed to create shortcuts",
        ))
    }
}

fn localized(ja: &str, en: &str) -> String {
    if is_japanese_ui() {
        ja.to_string()
    } else {
        en.to_string()
    }
}

fn is_japanese_ui() -> bool {
    let language_id = unsafe { GetUserDefaultUILanguage() };
    (language_id & 0x03ff) == 0x0011
}

fn path_error(action: &'static str, path: &Path) -> impl FnOnce(std::io::Error) -> String {
    let path = path.display().to_string();
    move |error| format!("{action}: {path}\n{error}")
}

fn ps_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn show_message(title: &str, body: &str, is_error: bool) {
    let title_w = wide(title);
    let body_w = wide(body);
    let flags = if is_error {
        MB_OK | MB_ICONERROR
    } else {
        MB_OK | MB_ICONINFORMATION
    };

    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(body_w.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            flags,
        );
    }
}

fn set_window_text(hwnd: HWND, text: &str) {
    let text_w = wide(text);
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(text_w.as_ptr()));
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(CLASS_NAME.get_or_init(|| wide(WINDOW_CLASS)).as_ptr())
}

fn button_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(CLASS_NAME.get_or_init(|| wide("BUTTON")).as_ptr())
}

fn static_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(CLASS_NAME.get_or_init(|| wide("STATIC")).as_ptr())
}

fn edit_class_name() -> PCWSTR {
    static CLASS_NAME: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    PCWSTR(CLASS_NAME.get_or_init(|| wide("EDIT")).as_ptr())
}

fn loword(value: usize) -> u16 {
    (value & 0xffff) as u16
}

fn hiword(value: usize) -> u16 {
    ((value >> 16) & 0xffff) as u16
}
