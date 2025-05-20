use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Runtime, WebviewWindow, Window, WindowEvent,
    path::BaseDirectory,
};
use tauri_plugin_deep_link::DeepLinkExt;

mod api;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ctx = tauri::generate_context!();

    let mut app_builder = tauri::Builder::default().plugin(
        tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Info)
            .target(tauri_plugin_log::Target::new(
                tauri_plugin_log::TargetKind::Stdout,
            ))
            .build(),
    );

    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|_app, argv, cwd| {
            log::info!(
                cwd = cwd,
                argv:serde = argv;
                "a new app instance was opened and the deep link event was already triggered");
            // when defining deep link schemes at runtime, you must also check `argv` here
        }));
    }

    let app = app_builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![api::greet])
        .setup(|app| {
            #[cfg(desktop)]
            {
                #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
                {
                    app.deep_link().register("anda")?;
                    app.deep_link().register_all()?;
                }
            }

            app.deep_link().on_open_url(|event| {
                log::info!(urls:serde = event.urls(); "deep link URLs");
            });

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing
                log::info!("Close requested event received");
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .build(ctx)
        .expect("error while running tauri application");

    app.run(|app_handle, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } => {
            log::info!(
                "Reopen event received: has_visible_windows = {}",
                has_visible_windows
            );
            if has_visible_windows {
                return;
            }
        }
        _ => {
            let _ = app_handle;
        }
    });
}
