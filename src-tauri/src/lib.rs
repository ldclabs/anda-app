use tauri::{Manager, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;

mod api;
mod deeplink;
mod logging;
mod menu;
mod model;
mod service;
mod utils;

use deeplink::{DeepLinkService, DeepLinkServiceExt};
use model::app::{AppState, AssistantConfig, SecretState};
use service::{
    assistant::{AndaAssistant, AndaAssistantExt},
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
        // the single instance plugin which should always be the first plugin
        app_builder = app_builder
            .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
                log::info!(
                    cwd = cwd,
                    argv:serde = argv;
                    "a new app instance was opened and the deep link event was already triggered");
                // when defining deep link schemes at runtime, you must also check `argv` here

                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    let app = app_builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .format(logging::formatter)
                .max_file_size(1024 * 1024 * 10)
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(ICPClient::init())
        .plugin(DeepLinkService::init())
        .plugin(AppStateCell::init("app_state.cbor".into()))
        .plugin(AndaAssistant::init("object_store".into()))
        .invoke_handler(tauri::generate_handler![
            api::auth::identity,
            api::auth::get_user,
            api::auth::sign_in,
            api::auth::sign_in_by_url,
            api::auth::logout,
            api::i18n::get_translation,
            api::assistant::assistant_info,
            api::assistant::assistant_name,
            api::assistant::caller_name,
            api::assistant::tool_call,
            api::assistant::agent_run,
            api::settings::get_settings,
            api::settings::set_setting,
            api::settings::get_secret_setting,
            api::settings::set_secret_setting,
            api::updater::quit,
            api::updater::restart,
            api::updater::check_update,
        ])
        .setup(|app| {
            if tauri::is_dev() {
                log::info!("Running in development mode: {}", app.config().identifier);

                if !app.config().identifier.ends_with("dev") {
                    panic!(
                        "The app identifier {} does not end with 'dev', this may cause issues in development.",
                        app.config().identifier
                    );
                }
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
                if state.session_secret.as_slice() == [0u8; 32] {
                    state.session_secret = SensitiveData(rand_bytes::<32>().into());
                }

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

                if state.assistant.is_none() {
                    state.assistant = Some(AssistantConfig {
                        root_secret: SensitiveData(rand_bytes::<48>().into()),
                        preferred_provider: "gemini".to_string(),
                        gemini: None,
                        deepseek: None,
                        xai: None,
                        openai: None,
                    });
                }

                Ok::<(), String>(())
            })?;
            secret_state.save()?;

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                #[cfg(target_os = "macos")]
                let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::META), Code::KeyP);
                #[cfg(not(target_os = "macos"))]
                let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyP);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |app, shortcut, event| {
                            if shortcut == &ctrl_n_shortcut && event.state() == ShortcutState::Pressed {
                                let _ = menu::reopen_window(app, "main", None, true);
                            }
                        })
                        .build(),
                )?;
                app.global_shortcut().register(ctrl_n_shortcut)?;

                app.manage(api::updater::Updater::default());
                menu::setup_app_menu(app.handle())?;
                menu::setup_app_tray(app.handle())?;
            }

            let dls = app.deep_link_service_owned();
            app.deep_link().on_open_url(move |event| {
                dls.on_open_url(event.urls());
            });

            app.connect_assistant();
            log::info!("Application initialized");

            Ok(())
        })
        .on_window_event(|window, event| {
            match window.label() {
                "settings" => {
                    if let WindowEvent::CloseRequested { .. } = event {
                        log::info!("Close settings window event received");
                        window.app_handle().save_assistant();
                        // 配置变更，建议重启 assistant
                        window.app_handle().try_reconnect_assistant();
                    }
                }
                "main" => {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing
                        log::info!("Close requested event received");
                        api.prevent_close();
                        let _ = window.hide();
                        window.app_handle().save_assistant();
                    }
                }
                _ => {}
            }

        })
        .build(ctx)
        .expect("error while running tauri application");

    app.run(|app, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } => {
            log::info!(
                "Reopen event received: has_visible_windows = {}",
                has_visible_windows
            );
            // 点击 Dock 图标时，显示并聚焦主窗口
            let _ = menu::reopen_window(app, "main", None, true);
        }
        tauri::RunEvent::ExitRequested { code, .. } => {
            log::info!("Exit requested event received: code = {code:?}");
        }
        tauri::RunEvent::Exit => {
            log::info!("Application is exiting");
        }
        _ => {}
    });
}
