//! Compile-time embedded translations.
//!
//! Translations live in `i18n/*.json`. We deliberately do not use `rust-i18n` macros so we can
//! ship new languages without touching code — adding a new `xx.json` is enough.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lang(pub &'static str);

impl Lang {
    pub const EN: Lang = Lang("en");
    pub const RU: Lang = Lang("ru");
    pub const UK: Lang = Lang("uk");
    pub const DE: Lang = Lang("de");
    pub const FR: Lang = Lang("fr");
    pub const ES: Lang = Lang("es");
    pub const ZH: Lang = Lang("zh");
    pub const JA: Lang = Lang("ja");

    pub const ALL: &'static [Lang] = &[
        Self::EN,
        Self::RU,
        Self::UK,
        Self::DE,
        Self::FR,
        Self::ES,
        Self::ZH,
        Self::JA,
    ];

    pub fn label(self) -> &'static str {
        match self.0 {
            "en" => "English",
            "ru" => "Русский",
            "uk" => "Українська",
            "de" => "Deutsch",
            "fr" => "Français",
            "es" => "Español",
            "zh" => "中文",
            "ja" => "日本語",
            _ => self.0,
        }
    }
}

macro_rules! embed_lang {
    ($code:literal) => {
        ($code, include_str!(concat!("../i18n/", $code, ".json")))
    };
}

const RAW: &[(&str, &str)] = &[
    embed_lang!("en"),
    embed_lang!("ru"),
    embed_lang!("uk"),
    embed_lang!("de"),
    embed_lang!("fr"),
    embed_lang!("es"),
    embed_lang!("zh"),
    embed_lang!("ja"),
];

static CATALOG: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

fn catalog() -> &'static HashMap<&'static str, HashMap<String, String>> {
    CATALOG.get_or_init(|| {
        let mut out = HashMap::new();
        for (code, raw) in RAW {
            let parsed: HashMap<String, String> =
                serde_json::from_str(raw).expect("embedded translation file is malformed");
            out.insert(*code, parsed);
        }
        out
    })
}

/// Look up a translation key for the given language, falling back to English then to the key
/// itself if it is missing.
pub fn t(lang: Lang, key: &str) -> String {
    let cat = catalog();
    if let Some(map) = cat.get(lang.0) {
        if let Some(v) = map.get(key) {
            return v.clone();
        }
    }
    if let Some(map) = cat.get(Lang::EN.0) {
        if let Some(v) = map.get(key) {
            return v.clone();
        }
    }
    key.to_string()
}

/// Best-effort detection of the user's preferred language from environment variables.
pub fn detect() -> Lang {
    let raw = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_lowercase();
    let short = raw.split(['_', '.', '-']).next().unwrap_or("");
    for lang in Lang::ALL {
        if lang.0 == short {
            return *lang;
        }
    }
    Lang::EN
}
