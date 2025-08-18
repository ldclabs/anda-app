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

#[tauri::command]
pub async fn quit(app: AppHandle) {
    app.assistant().close().await;
    app.exit(0)
}

#[tauri::command]
pub async fn restart(app: AppHandle) {
    app.assistant().close().await;

    if let Some(app_updater) = app.try_state::<Updater>()
        && app_updater.info.read().is_some() {
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
                if let Some(update_for_install) = app_updater.info.read().as_ref().cloned()
                    && let Err(e) = update_for_install.install(&package) {
                        log::error!("Failed to install update: {}", e);
                    }
            }
        };

    // macOS: 用 `open -n -b <bundle id>` 重新启动，避免崩溃提示
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // 读取 tauri.conf.json 中的 identifier
        let bundle_id = app.config().identifier.clone();

        // 启动新的实例（忽略错误，保持尽量健壮）
        let _ = Command::new("open").args(["-n", "-b", &bundle_id]).spawn();

        // 立即优雅退出当前进程，避免被系统标记为崩溃
        app.exit(0);
    }

    // 其它平台：仍用内置的重启
    #[cfg(not(target_os = "macos"))]
    {
        app.restart()
    }
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
            "ready": app_updater.package.read().is_some(),
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
