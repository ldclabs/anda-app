use tauri::{AppHandle, Emitter, Manager, Theme};

use super::Result;
use crate::{AppStateCell, BoxError, SecretStateCell, model::app::Settings};

pub const SETTINGS_EVENT: &str = "SettingsChanged";
pub const SECRET_SETTINGS_EVENT: &str = "SecretSettingsChanged";

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings> {
    let app_state = app.state::<AppStateCell>();
    let settings = app_state.with(|state| state.settings.clone());
    Ok(settings)
}

#[tauri::command]
pub async fn set_setting(app: AppHandle, key: String, value: String) -> Result<Settings> {
    let app_state = app.state::<AppStateCell>();
    // TODO: validate value
    match key.as_str() {
        "locale" => {
            app_state.with_mut(|state| {
                state.settings.locale = value;
                Ok::<(), BoxError>(())
            })?;
        }
        "theme" => {
            let theme = match value.as_str() {
                "light" => Some(Theme::Light),
                "dark" => Some(Theme::Dark),
                _ => None,
            };
            app_state.with_mut(|state| {
                state.settings.theme = theme;
                Ok::<(), BoxError>(())
            })?;
        }
        "https_proxy" => {
            app_state.with_mut(|state| {
                state.settings.https_proxy = if value.is_empty() { None } else { Some(value) };
                Ok::<(), BoxError>(())
            })?;
        }
        _ => return Err(format!("Unknown setting key: {:?}", key).into()),
    }

    app_state.save()?;
    let _ = app.emit(SETTINGS_EVENT, key);
    Ok(app_state.with(|state| state.settings.clone()))
}

#[tauri::command]
pub async fn get_secret_setting(app: AppHandle, key: String) -> Result<String> {
    let secret_state = app.state::<SecretStateCell>();
    secret_state.with(|state| match key.as_str() {
        "gemini_api_key" => match state.assistant.as_ref() {
            Some(cfg) => Ok(cfg.gemini_api_key.clone().unwrap_or_default()),
            None => Ok(String::new()),
        },
        "openai_api_key" => match state.assistant.as_ref() {
            Some(cfg) => Ok(cfg.openai_api_key.clone().unwrap_or_default()),
            None => Ok(String::new()),
        },
        _ => Err(format!("Unknown secret setting key: {:?}", key).into()),
    })
}

#[tauri::command]
pub async fn set_secret_setting(app: AppHandle, key: String, value: String) -> Result<()> {
    let secret_state = app.state::<SecretStateCell>();
    let updated = secret_state.with_mut(|state| match key.as_str() {
        "gemini_api_key" => {
            if let Some(cfg) = state.assistant.as_mut() {
                cfg.gemini_api_key = Some(value);
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        "openai_api_key" => {
            if let Some(cfg) = state.assistant.as_mut() {
                cfg.openai_api_key = Some(value);
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        _ => Err(format!("Unknown secret setting key: {:?}", key).into()),
    })?;

    if updated {
        secret_state.save()?;
        let _ = app.emit(SECRET_SETTINGS_EVENT, key);
    }
    Ok(())
}
