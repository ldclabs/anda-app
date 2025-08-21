use anda_core::Json;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, Theme};

use super::Result;
use crate::{
    AppStateCell, BoxError, SecretStateCell, model::app::Settings,
    service::assistant::AndaAssistantExt,
};

pub const SETTINGS_EVENT: &str = "SettingsChanged";
pub const SECRET_SETTINGS_EVENT: &str = "SecretSettingsChanged";

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings> {
    let app_state = app.state::<AppStateCell>();
    let settings = app_state.with(|state| state.settings.clone());
    Ok(settings)
}

#[tauri::command]
pub async fn set_setting(app: AppHandle, key: String, value: Json) -> Result<bool> {
    let app_state = app.state::<AppStateCell>();
    let updated = app_state.with_mut(|state| {
        // TODO: validate value
        match key.as_str() {
            "locale" => {
                match value.as_str() {
                    Some(v) => state.settings.locale = v.to_string(),
                    None => return Err("Invalid locale value".into()),
                }
                rust_i18n::set_locale(&state.settings.locale);
                Ok::<bool, BoxError>(true)
            }
            "theme" => {
                let theme = match value.as_str() {
                    Some("light") => Some(Theme::Light),
                    Some("dark") => Some(Theme::Dark),
                    _ => None,
                };
                state.settings.theme = theme;
                Ok::<bool, BoxError>(true)
            }
            "https_proxy" => {
                match value.as_str() {
                    Some(v) => {
                        app.propose_reconnect_assistant();
                        state.settings.https_proxy = if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    }
                    None => return Err("Invalid https_proxy value".into()),
                }
                Ok::<bool, BoxError>(true)
            }
            _ => Err(format!("Unknown setting key: {:?}", key).into()),
        }
    })?;

    if updated {
        app_state.save()?;
        let _ = app.emit(SETTINGS_EVENT, key);
    }
    Ok(updated)
}

#[tauri::command]
pub async fn get_secret_setting(app: AppHandle, key: String) -> Result<Json> {
    let secret_state = app.state::<SecretStateCell>();
    secret_state.with(|state| match key.as_str() {
        "preferred_provider" => match state.assistant.as_ref() {
            Some(cfg) => Ok(json!(&cfg.preferred_provider)),
            None => Ok(Json::Null),
        },
        "gemini" => match state.assistant.as_ref() {
            Some(cfg) => Ok(cfg.gemini.as_ref().map(|v| json!(v)).unwrap_or(Json::Null)),
            None => Ok(Json::Null),
        },
        "openai" => match state.assistant.as_ref() {
            Some(cfg) => Ok(cfg.openai.as_ref().map(|v| json!(v)).unwrap_or(Json::Null)),
            None => Ok(Json::Null),
        },
        "deepseek" => match state.assistant.as_ref() {
            Some(cfg) => Ok(cfg
                .deepseek
                .as_ref()
                .map(|v| json!(v))
                .unwrap_or(Json::Null)),
            None => Ok(Json::Null),
        },
        _ => Err(format!("Unknown secret setting key: {:?}", key).into()),
    })
}

#[tauri::command]
pub async fn set_secret_setting(app: AppHandle, key: String, value: Json) -> Result<bool> {
    let secret_state = app.state::<SecretStateCell>();
    let updated = secret_state.with_mut(|state| match key.as_str() {
        "preferred_provider" => {
            if let Some(cfg) = state.assistant.as_mut()
                && let Some(v) = value.as_str()
            {
                cfg.preferred_provider = v.to_string();
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        "gemini" => {
            if let Some(cfg) = state.assistant.as_mut() {
                cfg.gemini = Some(serde_json::from_value(value)?);
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        "openai" => {
            if let Some(cfg) = state.assistant.as_mut() {
                cfg.openai = Some(serde_json::from_value(value)?);
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        "deepseek" => {
            if let Some(cfg) = state.assistant.as_mut() {
                cfg.deepseek = Some(serde_json::from_value(value)?);
                return Ok::<bool, BoxError>(true);
            }
            Ok(false)
        }
        _ => Err(format!("Unknown secret setting key: {:?}", key).into()),
    })?;

    if updated {
        secret_state.save()?;
        app.propose_reconnect_assistant();
        let _ = app.emit(SECRET_SETTINGS_EVENT, key);
    }
    Ok(updated)
}
