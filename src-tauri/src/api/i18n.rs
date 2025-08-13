#[tauri::command]
pub fn get_translation(key: &str, locale: &str) -> String {
    let res = rust_i18n::t!(key, locale = locale);
    res.to_string()
}
