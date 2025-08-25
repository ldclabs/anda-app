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
            // Issue: "Failed to move the new app into place" on MacOS
            // https://github.com/tauri-apps/plugins-workspace/issues/2455
            if let Some(_update_for_install) = app_updater.info.read().as_ref().cloned() {
                #[cfg(target_os = "macos")]
                {
                    use std::{env, fs, path::PathBuf, process::Command};

                    // 将下载的更新包写入临时文件（不在 /Applications 上写，避免权限问题）
                    let bundle_id = app.config().identifier.clone();
                    let pkg_path: PathBuf =
                        env::temp_dir().join(format!("{}.update.tar.gz", bundle_id));
                    if let Err(e) = fs::write(&pkg_path, &package) {
                        log::error!("Failed to write update package: {}", e);
                    } else {
                        // 推导当前 .app bundle 路径和可执行名
                        let exe = env::current_exe().unwrap_or_default();
                        let proc_name = exe
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("anda_ai")
                            .to_string();
                        // 从 .../Your.app/Contents/MacOS/your_exec 回退到 Your.app
                        let app_bundle = exe
                            .parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| p.parent())
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| PathBuf::from("/Applications/Anda AI.app"));

                        // AppleScript：等待当前进程退出 -> 解压 -> ditto 覆盖 -> 重启 -> 清理
                        let script = format!(
                            r#"
do shell script "
tmpdir=$(mktemp -d) || exit 1
cd \"$tmpdir\" || exit 1
/usr/bin/tar -xzf \"{pkg}\" || exit 1
name=$(ls -1 | head -n1)
while pgrep -x \"{pname}\" >/dev/null; do sleep 0.2; done
rm -rf \"{app}\"
/usr/bin/ditto \"$name\" \"{app}\" || /bin/mv -f \"$name\" \"{app}\"
/usr/bin/open -n -b \"{bid}\"
rm -rf \"$tmpdir\"
" with administrator privileges
"#,
                            pkg = pkg_path.display(),
                            app = app_bundle.display(),
                            bid = bundle_id,
                            pname = proc_name
                        );

                        // 以管理员权限执行脚本（异步），随后本进程退出
                        if let Err(e) = Command::new("osascript").arg("-e").arg(script).spawn() {
                            log::error!("Failed to spawn osascript for update: {}", e);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                if let Err(e) = _update_for_install.install(&package) {
                    log::error!("Failed to install update: {}", e);
                }
            }
        }
    };

    // macOS: 由脚本完成重启，这里直接退出
    #[cfg(target_os = "macos")]
    {
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
