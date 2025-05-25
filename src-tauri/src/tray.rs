// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use rust_i18n::t;
use tauri::{
    Manager, Runtime,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_opener::OpenerExt;

use crate::Result;

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(
                app,
                "open-main",
                t!("tray.menu.open_main"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "follow-us",
                t!("tray.menu.follow_us"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "quit", t!("menu.quit"), true, None::<&str>)?,
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
            "open-main" => {
                if let Some(window) = app.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(true) => {}
                        _ => {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
            "follow-us" => {
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
