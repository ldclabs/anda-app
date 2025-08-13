// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use rust_i18n::t;
use tauri::{
    Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
    image::Image,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_opener::OpenerExt;

use crate::Result;

static ICON_BYTES: &[u8] = include_bytes!("../icons/icon.png");

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "open_main", t!("menu.open_main"), true, None::<&str>)?,
            &MenuItem::with_id(app, "follow_us", t!("menu.follow_us"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::about(
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
            )?,
            &MenuItem::with_id(
                app,
                "version",
                t!("menu.version", version = app.package_info().version),
                false,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "check_update",
                t!("menu.check_update"),
                false,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "settings", t!("menu.settings"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(&t!("menu.quit")))?,
        ],
    )?;

    let _ = TrayIconBuilder::with_id("tray-1")
        .tooltip("Anda AI")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_tray_icon_event(|_tray, _event| {})
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
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
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    Ok(())
}

fn reopen_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
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
                && let WebviewUrl::App(url) = &mut cfg.url {
                    url.push(params);
                };
            WebviewWindowBuilder::from_config(app, &cfg)?.build()?
        }
    };

    match window.is_visible() {
        Ok(true) => {}
        _ => {
            let _ = window.show();
        }
    };
    let _ = window.set_focus();
    Ok(())
}
