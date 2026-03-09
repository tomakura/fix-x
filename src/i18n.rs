use serde::{Deserialize, Serialize};
use windows::Win32::Globalization::GetUserDefaultUILanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLanguage {
    Auto,
    Ja,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedLanguage {
    Ja,
    En,
}

pub struct Strings {
    pub settings_title: &'static str,
    pub tray_open_settings: &'static str,
    pub tray_enabled: &'static str,
    pub tray_exit: &'static str,
    pub enable_checkbox: &'static str,
    pub target_label: &'static str,
    pub target_fx: &'static str,
    pub target_vx: &'static str,
    pub startup_checkbox: &'static str,
    pub language_label: &'static str,
    pub language_auto: &'static str,
    pub language_ja: &'static str,
    pub language_en: &'static str,
    pub close_button: &'static str,
}

const JA_STRINGS: Strings = Strings {
    settings_title: "fix-x 設定",
    tray_open_settings: "設定を開く",
    tray_enabled: "有効",
    tray_exit: "終了",
    enable_checkbox: "自動変換を有効にする",
    target_label: "変換先",
    target_fx: "fxtwitter.com を使う",
    target_vx: "vxtwitter.com を使う",
    startup_checkbox: "Windows 起動時に自動起動する",
    language_label: "表示言語",
    language_auto: "自動",
    language_ja: "日本語",
    language_en: "English",
    close_button: "閉じる",
};

const EN_STRINGS: Strings = Strings {
    settings_title: "fix-x Settings",
    tray_open_settings: "Open Settings",
    tray_enabled: "Enabled",
    tray_exit: "Exit",
    enable_checkbox: "Enable automatic rewrite",
    target_label: "Rewrite target",
    target_fx: "Use fxtwitter.com",
    target_vx: "Use vxtwitter.com",
    startup_checkbox: "Launch on Windows startup",
    language_label: "Language",
    language_auto: "Auto",
    language_ja: "Japanese",
    language_en: "English",
    close_button: "Close",
};

pub fn resolve_language(language: UiLanguage) -> ResolvedLanguage {
    match language {
        UiLanguage::Auto => detect_system_language(),
        UiLanguage::Ja => ResolvedLanguage::Ja,
        UiLanguage::En => ResolvedLanguage::En,
    }
}

pub fn strings(language: UiLanguage) -> &'static Strings {
    match resolve_language(language) {
        ResolvedLanguage::Ja => &JA_STRINGS,
        ResolvedLanguage::En => &EN_STRINGS,
    }
}

fn detect_system_language() -> ResolvedLanguage {
    let language_id = unsafe { GetUserDefaultUILanguage() };
    let primary_language = language_id & 0x03ff;

    if primary_language == 0x0011 {
        ResolvedLanguage::Ja
    } else {
        ResolvedLanguage::En
    }
}
