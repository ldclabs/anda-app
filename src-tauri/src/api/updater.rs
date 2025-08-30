use parking_lot::RwLock;
use serde_json::{Value as Json, json};
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Manager, Url, async_runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

use super::Result;
use crate::{AppStateCell, service::assistant::AndaAssistantExt};

#[derive(Clone)]
pub struct Updater {
    info: Arc<RwLock<Option<Update>>>,
    package: Arc<RwLock<Option<Vec<u8>>>>,
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            info: Arc::new(RwLock::new(None)),
            package: Arc::new(RwLock::new(None)),
        }
    }
}

pub fn is_mas_build() -> bool {
    #[cfg(target_os = "macos")]
    {
        // 通过 App Store 收据判断：Your.app/Contents/_MASReceipt/receipt
        if let Ok(exe) = std::env::current_exe() {
            let app_bundle = exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent());
            if let Some(bundle) = app_bundle {
                let receipt: PathBuf = bundle.join("Contents/_MASReceipt/receipt");
                if receipt.exists() {
                    return true;
                }
            }
        }
    }
    false
}

#[tauri::command]
pub fn update_supported() -> bool {
    // 前端可据此隐藏“检查更新”入口
    #[cfg(desktop)]
    {
        !is_mas_build()
    }

    #[cfg(not(desktop))]
    false
}

#[tauri::command]
pub async fn quit(app: AppHandle) {
    app.assistant().close().await;
    app.exit(0)
}

#[tauri::command]
pub async fn restart(app: AppHandle) {
    app.assistant().close().await;

    if !is_mas_build()
        && let Some(app_updater) = app.try_state::<Updater>()
        && app_updater.info.read().is_some()
    {
        // 先尝试用缓存的字节
        let mut package = app_updater.package.write().take();

        if package.is_none() {
            log::warn!("No update package, downloading...");
            // 下载阶段使用一个克隆，且不持有锁跨越 await
            let update_for_download = app_updater.info.read().as_ref().cloned();
            if let Some(update_for_download) = update_for_download {
                package = update_for_download.download(|_, _| {}, || {}).await.ok();
            }
        }

        if let Some(package) = package {
            // await 之后再获取一个新的克隆用于安装
            // MacOS issue: "Failed to move the new app into place"
            // https://github.com/tauri-apps/plugins-workspace/issues/2455
            if let Some(updater) = app_updater.info.read().as_ref().cloned()
                && let Err(err) = updater.install(&package)
            {
                log::error!("Failed to install update: {}", err);
            }
        }
    };

    app.restart()
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<Json>> {
    if is_mas_build() {
        // MAS 版本直接禁用检查更新
        log::info!("Mac App Store build: updates managed by App Store");
        return Ok(None);
    }

    let app_updater = match app.try_state::<Updater>() {
        Some(updater) => updater,
        None => return Ok(None),
    };
    if let Some(update) = app_updater.info.read().as_ref() {
        return Ok(Some(json!({
            "current_version": update.current_version,
            "version": update.version,
            "ready": app_updater.package.read().is_some(),
            "notes": update.body.clone().unwrap_or_default(),
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
        let package = app_updater.package.clone();
        let res = json!({
            "current_version": &update.current_version,
            "version": &update.version,
            "ready": false,
            "notes": update.body.clone().unwrap_or_default(),
        });
        async_runtime::spawn(async move {
            if let Ok(data) = update.download(|_, _| {}, || {}).await {
                package.write().replace(data);
            }
        });

        return Ok(Some(res));
    }

    Ok(None)
}
