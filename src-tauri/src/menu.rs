// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use rust_i18n::t;
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, async_runtime,
    image::Image,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_opener::OpenerExt;

use crate::{Result, service::assistant::AndaAssistantExt};

static ICON_BYTES: &[u8] = include_bytes!("../icons/icon.png");

pub fn setup_app_menu(app: &tauri::AppHandle) -> Result<()> {
    // 应用名 / 版本取自 tauri.conf.json
    let app_name = app.package_info().name.clone();

    // 第一个 Submenu 会成为 macOS 左上角的“应用菜单”，标题会自动显示为应用名
    let app_menu = Submenu::with_items(
        app,
        &app_name,
        true,
        &[
            &menu_item_about(app)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, Some(&t!("menu.services")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some(&t!("menu.hide_app")))?,
            &PredefinedMenuItem::hide_others(app, Some(&t!("menu.hide_others")))?,
            &PredefinedMenuItem::show_all(app, Some(&t!("menu.show_all")))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", t!("menu.quit"), true, None::<&str>)?,
        ],
    )?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None::<&str>)?,
            &PredefinedMenuItem::redo(app, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None::<&str>)?,
            &PredefinedMenuItem::copy(app, None::<&str>)?,
            &PredefinedMenuItem::paste(app, None::<&str>)?,
            &PredefinedMenuItem::select_all(app, None::<&str>)?,
        ],
    )?;
    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None::<&str>)?,
            &PredefinedMenuItem::maximize(app, None::<&str>)?,
            &PredefinedMenuItem::fullscreen(app, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None::<&str>)?,
        ],
    )?;
    let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &window_menu])?;
    app.on_menu_event(move |app, event| {
        if event.id.as_ref() == "quit" {
            quit(app);
        }
    });
    app.set_menu(menu)?;
    Ok(())
}

pub fn setup_app_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "open_main", t!("menu.open_main"), true, None::<&str>)?,
            &MenuItem::with_id(app, "follow_us", t!("menu.follow_us"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &menu_item_about(app)?,
            &MenuItem::with_id(
                app,
                "version",
                t!("menu.version", version = app.package_info().version),
                false,
                None::<&str>,
            )?,
            // &MenuItem::with_id(
            //     app,
            //     "check_update",
            //     t!("menu.check_update"),
            //     false,
            //     None::<&str>,
            // )?,
            &MenuItem::with_id(app, "settings", t!("menu.settings"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", t!("menu.quit"), true, None::<&str>)?,
        ],
    )?;

    let _ = TrayIconBuilder::with_id("tray-1")
        .tooltip("Anda AI")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                quit(app);
            }
            "open_main" => {
                reopen_window(app, "main", None).unwrap();
            }
            "settings" => {
                reopen_window(app, "settings", None).unwrap();
            }
            "follow_us" => {
                let _ = app
                    .opener()
                    .open_url("https://x.com/ICPandaDAO", None::<&str>);
            }

            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button_state: MouseButtonState::Down,
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    let _ = reopen_window(app, "main", None);
                }
            }
        })
        .build(app);

    Ok(())
}

fn menu_item_about<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PredefinedMenuItem<R>> {
    let item = PredefinedMenuItem::about(
        app,
        Some(&t!("menu.about")),
        Some(AboutMetadata {
            name: Some("Anda AI".to_string()),
            version: Some(app.package_info().version.to_string()),
            short_version: Some(app.package_info().version.to_string()),
            authors: Some(vec!["ICPanda".to_string()]),
            comments: Some("Anda AI Assistant".to_string()),
            copyright: Some("Copyright 2025 LDC Labs".to_string()),
            license: Some("MIT".to_string()),
            website: Some("https://anda.ai".to_string()),
            website_label: Some("Anda AI".to_string()),
            credits: Some("ICPanda".to_string()),
            icon: Image::from_bytes(ICON_BYTES).ok(),
        }),
    )?;
    Ok(item)
}

fn quit<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    async_runtime::spawn(async move {
        app.assistant().close().await;
        app.exit(0)
    });
}

pub fn reopen_window<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    search_params: Option<&str>,
) -> Result<()> {
    let window = match app.get_webview_window(label) {
        Some(window) => window,
        None => {
            let mut cfg = app
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == label)
                .ok_or_else(|| format!("window {} not found", label))?
                .clone();
            if let Some(params) = search_params
                && let WebviewUrl::App(url) = &mut cfg.url
            {
                url.push(params);
            };
            WebviewWindowBuilder::from_config(app, &cfg)?.build()?
        }
    };

    // 若为最小化状态，先尝试还原
    let _ = window.unminimize();
    match window.is_visible() {
        Ok(true) => {}
        _ => {
            let _ = window.show();
        }
    };
    let _ = window.set_focus();

    // #[cfg(debug_assertions)]
    // window.open_devtools();

    Ok(())
}
