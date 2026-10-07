//! Compile translated format strings so Rust checks both languages' placeholders.
//! AI編輯：擴充為同時編譯日文與繁體中文（zh-hant）的複數格式字串。
use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=locales/ja-formats.json");
    println!("cargo:rerun-if-changed=locales/zh-hant-formats.json");
    let ja: BTreeMap<String, String> = serde_json::from_str(&fs::read_to_string("locales/ja-formats.json")?)?;
    let zh: BTreeMap<String, String> = serde_json::from_str(&fs::read_to_string("locales/zh-hant-formats.json")?)?;
    let mut source = String::from("macro_rules! tr_format {\n");
    for (english, japanese) in &ja {
        let en = serde_json::to_string(english)?;
        let ja_str = serde_json::to_string(japanese)?;
        // AI編輯：繁中格式不存在時退回英文格式（不影響編譯，僅顯示英文）。
        let zh_str = serde_json::to_string(zh.get(english).map(String::as_str).unwrap_or(english))?;
        source.push_str(&format!(
            "({en} $(, $($args:tt)*)?) => {{ if $crate::i18n::is_japanese() {{ format!({ja_str} $(, $($args)*)?) }} else if $crate::i18n::is_zhhant() {{ format!({zh_str} $(, $($args)*)?) }} else {{ format!({en} $(, $($args)*)?) }} }};\n"
        ));
    }
    source.push_str("}\npub(crate) use tr_format;\n");
    fs::write(PathBuf::from(env::var("OUT_DIR")?).join("ja-formats.rs"), source)?;
    Ok(())
}
