use tauri::{Manager, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;

mod api;
mod deeplink;
mod logging;
mod model;
mod service;
mod tray;
mod utils;

use deeplink::{DeepLinkService, DeepLinkServiceExt};
use model::app::{AppState, SecretState};
use service::{
    icp::{ICPClient, ICPClientExt},
    stablecell::{CipherCell, PlainCell},
};
use utils::{SensitiveData, rand_bytes};

const APP_SALT: &[u8] = b"Anda.AI";

pub type BoxError = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, BoxError>;
pub type AppStateCell = PlainCell<AppState>;
pub type SecretStateCell = CipherCell<SecretState>;

rust_i18n::i18n!("locales");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ctx = tauri::generate_context!();

    let mut app_builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            log::info!(
                cwd = cwd,
                argv:serde = argv;
                "a new app instance was opened and the deep link event was already triggered, {argv:?}, cwd: {cwd:?}");
            // when defining deep link schemes at runtime, you must also check `argv` here

            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    let app = app_builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .format(logging::formatter)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .max_file_size(1024 * 1024 * 10)
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(ICPClient::init())
        .plugin(DeepLinkService::init())
        .plugin(AppStateCell::init("app_state.cbor".into()))
        .invoke_handler(tauri::generate_handler![
            api::auth::identity,
            api::auth::get_user,
            api::auth::sign_in,
            api::auth::logout,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
                {
                    app.deep_link().register("anda")?;
                    app.deep_link().register_all()?;
                }

                tray::create_tray(app.handle())?;
            }

            let app_state = app.state::<AppStateCell>();
            let aes_secret = app_state.with_mut(|state| {
                state.os_arch = tauri_plugin_os::arch().to_string();
                state.os_platform = tauri_plugin_os::platform().to_string();

                if state.seed.as_slice() == [0u8; 32] {
                    state.seed = SensitiveData(rand_bytes::<32>().into());
                }

                if state.settings.locale.is_empty() {
                    let locale = match tauri_plugin_os::locale() {
                        Some(locale) => match locale.as_str() {
                            lo if lo.starts_with("zh") => "zh".to_string(),
                            _ => "en".to_string(),
                        },
                        None => "en".to_string(),
                    };
                    state.settings.locale = locale;
                }

                rust_i18n::set_locale(&state.settings.locale);

                if let Some(theme) = state.settings.theme {
                    app.set_theme(Some(theme));
                }

                state.derive_a256gcm_key(APP_SALT)
            });
            app_state.save()?;

            app.handle().plugin(SecretStateCell::init(
                "secret_state.cbor".into(),
                aes_secret,
            ))?;

            let secret_state = app.state::<SecretStateCell>();
            secret_state.with_mut(|state| {
                if let Some(auth) = &state.auth {
                    let principal = auth.principal();
                    match auth.to_identity(**state.session_secret) {
                        Ok(id) => {
                            app.icp().set_identity(Box::new(id));
                        }
                        Err(err) => {
                            log::error!("Failed to create identity from {principal}: {err:?}");
                        }
                    }
                }

                Ok::<(), String>(())
            })?;
            secret_state.save()?;

            let dls = app.deep_link_service_owned();
            app.deep_link().on_open_url(move |event| {
                dls.on_open_url(event.urls());
            });

            log::info!("Initialized application");

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing
                log::info!("Close requested event received");
                window.hide().unwrap();
                api.prevent_close();
            }
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
            if has_visible_windows {}
        }
        _ => {
            let _ = app_handle;
        }
    });
}
