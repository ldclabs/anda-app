use parking_lot::RwLock;
use serde_json::{Value as Json, json};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Url, async_runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

use super::Result;
use crate::{AppStateCell, service::assistant::AndaAssistantExt};

#[derive(Clone)]
pub struct Updater {
    info: Arc<RwLock<Option<Update>>>,
    bytes: Arc<RwLock<Option<Vec<u8>>>>,
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            info: Arc::new(RwLock::new(None)),
            bytes: Arc::new(RwLock::new(None)),
        }
    }
}

#[tauri::command]
pub async fn quit(app: AppHandle) {
    app.assistant().close().await;
    app.exit(0)
}

#[tauri::command]
pub async fn restart(app: AppHandle) {
    app.assistant().close().await;

    if let Some(app_updater) = app.try_state::<Updater>() {
        if app_updater.info.read().is_some() {
            // 先尝试用缓存的字节
            let mut bytes = app_updater.bytes.write().take();

            if bytes.is_none() {
                log::warn!("No update bytes, downloading...");
                // 下载阶段使用一个克隆，且不持有锁跨越 await
                let update_for_download = app_updater.info.read().as_ref().cloned();
                if let Some(update_for_download) = update_for_download {
                    bytes = update_for_download.download(|_, _| {}, || {}).await.ok();
                }
            }

            if let Some(bytes) = bytes {
                // await 之后再获取一个新的克隆用于安装
                if let Some(update_for_install) = app_updater.info.read().as_ref().cloned() {
                    if let Err(e) = update_for_install.install(&bytes) {
                        log::error!("Failed to install update: {}", e);
                    }
                }
            }
        }
    };

    app.restart()
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<Json>> {
    let app_updater = match app.try_state::<Updater>() {
        Some(updater) => updater,
        None => return Ok(None),
    };
    if let Some(update) = app_updater.info.read().as_ref() {
        return Ok(Some(json!({
            "current_version": update.current_version,
            "version": update.version,
            "ready": app_updater.bytes.read().is_some(),
        })));
    }

    let proxy = app
        .state::<AppStateCell>()
        .with(|state| state.settings.https_proxy.clone())
        .and_then(|s| Url::parse(&s).ok());
    let mut updater = app.updater_builder();
    if let Some(proxy) = proxy {
        updater = updater.proxy(proxy);
    }
    let updater = updater.build()?;
    if let Some(update) = updater.check().await? {
        app_updater.info.write().replace(update.clone());
        let bytes_ = app_updater.bytes.clone();
        let res = json!({
            "current_version": &update.current_version,
            "version": &update.version,
            "ready": false,
        });
        async_runtime::spawn(async move {
            if let Ok(bytes) = update.download(|_, _| {}, || {}).await {
                bytes_.write().replace(bytes);
            }
        });

        return Ok(Some(res));
    }

    Ok(None)
}
