//! Presentation-only localization. Command ids, user names and catalog data remain unchanged.
use std::{cell::Cell, collections::BTreeMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/ja-formats.rs"));

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
    // AI編輯：新增繁體中文（zh-Hant）介面語言。
    #[serde(rename = "zh-hant")]
    ZhHant,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::En, Self::Ja, Self::ZhHant];
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ja => "日本語",
            // AI編輯：新增語言名稱。
            Self::ZhHant => "繁體中文",
        }
    }
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
            // AI編輯：接受「zh-hant / zh-tw / zh」三種代碼。
            "zh-hant" | "zh-tw" | "zh" => Some(Self::ZhHant),
            _ => None,
        }
    }
    pub fn tr(self, source: &str) -> &str {
        match self {
            Self::Ja => japanese().get(source).map(String::as_str).unwrap_or(source),
            // AI編輯：新增繁體中文翻譯查詢。
            Self::ZhHant => zh_hant().get(source).map(String::as_str).unwrap_or(source),
            Self::En => source,
        }
    }
}

thread_local! {
    static LANGUAGE: Cell<Language> = const { Cell::new(Language::En) };
}

pub fn default_language() -> Language {
    // AI編輯：支援 LIGHTCRAFT_LANGUAGE=zh-hant / zh-tw / zh。
    match std::env::var("LIGHTCRAFT_LANGUAGE").as_deref() {
        Ok("ja") => Language::Ja,
        Ok("zh-hant") | Ok("zh-tw") | Ok("zh") => Language::ZhHant,
        _ => Language::En,
    }
}

pub fn set_language(language: Language) {
    LANGUAGE.with(|value| value.set(language));
}

pub fn is_japanese() -> bool {
    LANGUAGE.with(|value| value.get() == Language::Ja)
}

// AI編輯：新增「目前是否為繁中介面」查詢。
pub fn is_zhhant() -> bool {
    LANGUAGE.with(|value| value.get() == Language::ZhHant)
}

fn japanese() -> &'static BTreeMap<String, String> {
    static MESSAGES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    MESSAGES.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/ja.json")).unwrap_or_else(|error| {
            log::error!("Invalid Japanese message catalog: {error}");
            BTreeMap::new()
        })
    })
}

// AI編輯：繁體中文（zh-hant）訊息目錄，由 AI 完整翻譯（1019 條）。
fn zh_hant() -> &'static BTreeMap<String, String> {
    static MESSAGES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    MESSAGES.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/zh-hant.json")).unwrap_or_else(|error| {
            log::error!("Invalid Traditional Chinese message catalog: {error}");
            BTreeMap::new()
        })
    })
}

/// Translate a built-in display label, preserving unknown labels verbatim.
/// Never call this on editable user text, filenames or command identifiers.
pub fn tr(source: &str) -> &str {
    if is_japanese() {
        japanese().get(source).map(String::as_str).unwrap_or(source)
    } else if is_zhhant() {
        // AI編輯：繁中查詢路徑。
        zh_hant().get(source).map(String::as_str).unwrap_or(source)
    } else {
        source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_valid_and_contains_core_workflows() {
        let messages: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/ja.json")).unwrap();
        for key in ["Import Photos…", "Export…", "Exposure", "White Balance", "Settings", "Language"] {
            assert!(messages.get(key).is_some_and(|value| !value.is_empty() && value != key), "{key}");
        }
    }

    #[test]
    fn language_switches_and_unknown_text_survives() {
        set_language(Language::Ja);
        assert_eq!(tr("Exposure"), "露出");
        assert_eq!(tr("my-photo.jpg"), "my-photo.jpg");
        assert_eq!(tr("develop.set"), "develop.set");
        assert_eq!(crate::menubar::display_item_label("album.addPhotos", &serde_json::json!({"id": 1}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("app.export", &serde_json::json!({"preset": "Color"}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("view.photoGrid", &serde_json::Value::Null, "Color"), "カラー");
        set_language(Language::En);
        assert_eq!(tr("Exposure"), "Exposure");
    }

    // AI編輯：繁體中文目錄完整且涵蓋核心工作流程。
    #[test]
    fn traditional_chinese_catalog_is_valid_and_contains_core_workflows() {
        let messages: BTreeMap<String, String> = serde_json::from_str(include_str!("../locales/zh-hant.json")).unwrap();
        for key in ["Import Photos…", "Export…", "Exposure", "White Balance", "Settings", "Language"] {
            assert!(messages.get(key).is_some_and(|value| !value.is_empty() && value != key), "{key}");
        }
    }

    // AI編輯：繁中切換、未知文字保留與選單標籤翻譯。
    #[test]
    fn language_switches_to_traditional_chinese_and_unknown_text_survives() {
        set_language(Language::ZhHant);
        assert_eq!(tr("Exposure"), "曝光");
        assert_eq!(tr("my-photo.jpg"), "my-photo.jpg");
        assert_eq!(tr("develop.set"), "develop.set");
        assert_eq!(crate::menubar::display_item_label("album.addPhotos", &serde_json::json!({"id": 1}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("app.export", &serde_json::json!({"preset": "Color"}), "Color"), "Color");
        assert_eq!(crate::menubar::display_item_label("view.photoGrid", &serde_json::Value::Null, "Color"), "顏色");
        set_language(Language::En);
        assert_eq!(tr("Exposure"), "Exposure");
    }

    // AI編輯：繁中複數格式保留計數。
    #[test]
    fn traditional_chinese_formats_preserve_counts_and_remove_english_plural_suffixes() {
        set_language(Language::ZhHant);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12 張相片");
        assert_eq!(tr_format!("Exported {ok} of {total} photo{}", "s", ok = 4, total = 12), "已匯出 12 張中的 4 張相片");
        set_language(Language::En);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12 photos");
    }

    // AI編輯：繁中語言設定可序列化往返（zh-hant）。
    #[test]
    fn traditional_chinese_preferences_round_trip() {
        let old: crate::state::UiState = serde_json::from_str("{}").unwrap();
        let settings = crate::state::UiState { language: Language::ZhHant, ..old };
        let saved = serde_json::to_string(&settings).unwrap();
        let restored: crate::state::UiState = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.language, Language::ZhHant);
    }

    // AI編輯：繁中介面實際繪製出中文（需系統已安裝中文字型；未安裝時跳過字形檢查）。
    #[test]
    fn traditional_chinese_is_painted() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut app = crate::LightcraftApp::new(lightcraft_engine::Session::with_demo(), Default::default());
        app.ui.language = Language::ZhHant;
        app.ui.left_panel = true;
        let mut text = String::new();
        fn collect(shape: &egui::epaint::Shape, text: &mut String) {
            match shape {
                egui::epaint::Shape::Text(shape) => {
                    text.push_str(&shape.galley.job.text);
                    text.push('\n');
                }
                egui::epaint::Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, text)),
                _ => {}
            }
        }
        for frame in 0..4 {
            let input = crate::headless::HeadlessView::raw_input(egui::vec2(1600.0, 1000.0), 1.0, frame as f64 / 60.0, vec![]);
            let mut out = ctx.run_ui(input, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            out.textures_delta.clear();
            text.clear();
            for shape in out.shapes {
                collect(&shape.shape, &mut text);
            }
        }
        assert!(text.contains("我的相片"), "{text}");
        assert!(text.contains("所有相片"), "{text}");
        if crate::theme::system_cjk_fonts().is_empty() {
            eprintln!("skipped glyph coverage: no system CJK font installed");
        } else {
            ctx.fonts_mut(|fonts| {
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into())] {
                    let font = egui::FontId::new(13.0, family);
                    for message in zh_hant().values() {
                        for ch in message.chars().filter(|ch| !ch.is_whitespace()) {
                            assert!(fonts.has_glyph(&font, ch), "Missing glyph {ch} in {message}");
                        }
                    }
                }
            });
        }
        // Locale affects presentation only: command ids remain the same.
        let ids = |app: &crate::LightcraftApp| crate::menus::menu_entries(app).into_iter().map(|entry| entry.id).collect::<Vec<_>>();
        let chinese_ids = ids(&app);
        set_language(Language::En);
        assert_eq!(chinese_ids, ids(&app));
    }

    #[test]
    fn translated_formats_preserve_counts_and_remove_english_plural_suffixes() {
        set_language(Language::Ja);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12枚");
        assert_eq!(tr_format!("Exported {ok} of {total} photo{}", "s", ok = 4, total = 12), "12枚中4枚を書き出しました");
        set_language(Language::En);
        assert_eq!(tr_format!("{n} photo{}", "s", n = 12), "12 photos");
    }

    #[test]
    fn preferences_round_trip_and_old_settings_remain_readable() {
        let old: crate::state::UiState = serde_json::from_str("{}").unwrap();
        assert_eq!(old.language, Language::En);
        let settings = crate::state::UiState { language: Language::Ja, ..old };
        let saved = serde_json::to_string(&settings).unwrap();
        let restored: crate::state::UiState = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.language, Language::Ja);
    }

    #[test]
    fn japanese_is_painted_and_both_font_weights_cover_the_catalog() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut app = crate::LightcraftApp::new(lightcraft_engine::Session::with_demo(), Default::default());
        app.ui.language = Language::Ja;
        app.ui.left_panel = true;
        let mut text = String::new();
        fn collect(shape: &egui::epaint::Shape, text: &mut String) {
            match shape {
                egui::epaint::Shape::Text(shape) => {
                    text.push_str(&shape.galley.job.text);
                    text.push('\n');
                }
                egui::epaint::Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, text)),
                _ => {}
            }
        }
        for frame in 0..4 {
            let input = crate::headless::HeadlessView::raw_input(egui::vec2(1600.0, 1000.0), 1.0, frame as f64 / 60.0, vec![]);
            let mut out = ctx.run_ui(input, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            // This assertion inspects shapes without a renderer; discard texture uploads explicitly.
            out.textures_delta.clear();
            text.clear();
            for shape in out.shapes {
                collect(&shape.shape, &mut text);
            }
        }
        assert!(text.contains("マイフォト"), "{text}");
        assert!(text.contains("すべての写真"), "{text}");
        if lightcraft_engine::fonts::japanese(lightcraft_engine::CRAFT_FONTS).next().is_none() {
            eprintln!("skipped glyph coverage: built without CRAFT_FONTS_DIR, so there is no Japanese UI font");
        } else {
            ctx.fonts_mut(|fonts| {
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into())] {
                    let font = egui::FontId::new(13.0, family);
                    for message in japanese().values() {
                        for ch in message.chars().filter(|ch| !ch.is_whitespace()) {
                            assert!(fonts.has_glyph(&font, ch), "Missing glyph {ch} in {message}");
                        }
                    }
                }
            });
        }
        // Locale affects presentation only: command ids remain the same.
        let ids = |app: &crate::LightcraftApp| crate::menus::menu_entries(app).into_iter().map(|entry| entry.id).collect::<Vec<_>>();
        let japanese_ids = ids(&app);
        set_language(Language::En);
        assert_eq!(japanese_ids, ids(&app));
    }
    /// The UI families, after one frame so the font definitions are loaded.
    fn fonts_ctx(craft: &'static [lightcraft_engine::CraftFont]) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_fonts(crate::theme::font_definitions(craft));
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        ctx
    }

    fn ui_families() -> [egui::FontFamily; 3] {
        [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into()), egui::FontFamily::Monospace]
    }

    /// Built with craft-fonts, Japanese renders with real glyphs (no tofu) in every UI family.
    #[test]
    fn craft_fonts_render_japanese_in_the_ui() {
        if lightcraft_engine::fonts::japanese(lightcraft_engine::CRAFT_FONTS).next().is_none() {
            eprintln!("skipped: built without CRAFT_FONTS_DIR, so there is no Japanese UI font");
            return;
        }
        let ctx = fonts_ctx(lightcraft_engine::CRAFT_FONTS);
        ctx.fonts_mut(|fonts| {
            for family in ui_families() {
                let font = egui::FontId::new(13.0, family);
                for ch in "日本語の文字".chars() {
                    assert!(fonts.has_glyph(&font, ch), "{ch} in {font:?}");
                }
                let galley = fonts.layout_no_wrap("日本語の文字".into(), font.clone(), egui::Color32::WHITE);
                assert!(galley.size().x > 13.0 * 5.0, "{font:?}: six full-width glyphs, {:?}", galley.size());
            }
        });
    }

    /// Built without craft-fonts, the UI (in Japanese, too) still installs its fonts and runs;
    /// Latin text keeps Inter.
    #[test]
    fn the_ui_works_without_craft_fonts() {
        let ctx = fonts_ctx(&[]);
        ctx.fonts_mut(|fonts| {
            // (Not Monospace: egui's `has_glyph` reports false for glyphs of the family's
            // replacement-glyph face, which there is Hack, the face that draws Latin.)
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Name(crate::theme::FONT_SEMIBOLD.into())] {
                let font = egui::FontId::new(13.0, family);
                assert!("LightCraft".chars().all(|ch| fonts.has_glyph(&font, ch)), "{font:?}");
            }
        });
        let mut app = crate::LightcraftApp::new(lightcraft_engine::Session::with_demo(), Default::default());
        app.ui.language = Language::Ja;
        for frame in 0..3 {
            let input = crate::headless::HeadlessView::raw_input(egui::vec2(1200.0, 800.0), 1.0, frame as f64 / 60.0, vec![]);
            let mut out = ctx.run_ui(input, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            out.textures_delta.clear();
        }
        set_language(Language::En);
    }
}
