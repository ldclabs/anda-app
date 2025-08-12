use anda_assistant::Assistant;
use anda_core::{BoxError, Path as DBPath, derivation_path_with};
use anda_db::{
    database::{AndaDB, DBConfig},
    storage::StorageConfig,
};
use anda_engine::{
    context::{Web3ClientFeatures, Web3SDK},
    engine::{AgentInfo, Engine, EngineBuilder},
    management::{BaseManagement, SYSTEM_PATH, Visibility},
    memory::MemoryTool,
    model::{Model, gemini},
    store::{LocalFileSystem, Store},
};
use anda_object_store::EncryptedStoreBuilder;
use anda_web3_client::client::Client as Web3Client;
use arc_swap::ArcSwap;
use futures::try_join;
use ic_auth_verifier::AtomicIdentity;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::Arc,
};
use tauri::{
    AppHandle, Manager, Runtime, async_runtime,
    plugin::{Builder, TauriPlugin},
};

use crate::{SecretStateCell, model::app::AssistantConfig};

use super::icp::ICP_HOST;

pub struct AndaAssistant<R: Runtime> {
    app: AppHandle<R>,
    inner: Arc<InnerAssistant>,
}

struct InnerAssistant {
    dir: PathBuf,
    identity: Arc<AtomicIdentity>,
    db: ArcSwap<Option<Arc<AndaDB>>>,
    engine: ArcSwap<Engine>,
}

impl<R: Runtime> AndaAssistant<R> {
    const NAME: &'static str = "ai-assistant";

    pub fn init(object_store_dir: PathBuf) -> TauriPlugin<R> {
        Builder::new(Self::NAME)
            .setup(move |app, _api| {
                let app_data_dir = app
                    .path()
                    .app_local_data_dir()
                    .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
                let dir = app_data_dir.join(object_store_dir);
                fs::create_dir_all(&dir).map_err(|e| {
                    format!(
                        "Failed to create object store directory at {:?}: {}",
                        dir, e
                    )
                })?;

                app.manage(AndaAssistant {
                    app: app.clone(),
                    inner: Arc::new(InnerAssistant {
                        dir,
                        identity: Arc::new(AtomicIdentity::default()),
                        db: ArcSwap::new(Arc::new(None)),
                        engine: ArcSwap::new(Arc::new(InnerAssistant::builder().empty())),
                    }),
                });

                Ok(())
            })
            .on_event(|_app, event| {
                match event {
                    tauri::RunEvent::ExitRequested { .. } => {
                        // app.assistant().flush();
                        // TODO:: Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks.
                    }
                    tauri::RunEvent::Exit => {
                        // app.assistant().close();
                        // TODO: Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks.
                    }
                    _ => {}
                }
            })
            .build()
    }

    pub fn engine(&self) -> Arc<Engine> {
        self.inner.engine.load().clone()
    }

    pub fn flush(&self) {
        let db = self.inner.db.load().as_ref().clone();
        if let Some(db) = db {
            async_runtime::block_on(async {
                match db.flush().await {
                    Ok(_) => log::info!("Anda Assistant flushed successfully"),
                    Err(e) => log::error!("Failed to flush Anda Assistant: {}", e),
                }
            });
        }
    }

    pub fn close(&self) {
        let engine = self.inner.engine.load().clone();
        let db = self.inner.db.load().as_ref().clone();
        async_runtime::block_on(async {
            match try_join!(engine.close(), async {
                if let Some(db) = db {
                    db.close().await?;
                }
                Ok(())
            }) {
                Ok(_) => log::info!("Anda Assistant closed successfully"),
                Err(e) => log::error!("Failed to close Anda Assistant: {}", e),
            }
        });
    }
}

impl InnerAssistant {
    fn builder() -> EngineBuilder {
        EngineBuilder::new().with_info(AgentInfo {
            handle: "assistant".to_string(),
            handle_canister: None,
            name: "AI Assistant".to_string(),
            description: "AI Assistant".to_string(),
            endpoint: "https://localhost:8443/default".to_string(),
            protocols: BTreeMap::new(),
            payments: BTreeSet::new(),
        })
    }

    async fn connect(&self, cfg: AssistantConfig) -> Result<(), BoxError> {
        self.identity.set(Box::new(cfg.to_identity()));
        let web3 = Web3Client::builder()
            .with_ic_host(ICP_HOST)
            .with_identity(self.identity.clone())
            .with_root_secret(**cfg.root_secret)
            .build()
            .await?;

        let my_principal = web3.get_principal();
        log::info!(
            "start AI assistant, principal: {:?}",
            my_principal.to_text()
        );

        log::info!("start to connect object_store");
        let os_secret = web3
            .a256gcm_key(derivation_path_with(
                &DBPath::from(SYSTEM_PATH),
                vec![b"object_store".to_vec(), b"A256GCM".to_vec()],
            ))
            .await?;

        let object_store = LocalFileSystem::new_with_prefix(&self.dir)?;
        let object_store = EncryptedStoreBuilder::with_secret(object_store, 10000, os_secret)
            .with_chunk_size(1024 * 1024)
            .with_conditional_put()
            .build();
        let object_store = Arc::new(object_store);

        let db_config = DBConfig {
            name: "anda_db".to_string(),
            description: "Anda DB".to_string(),
            storage: StorageConfig {
                cache_max_capacity: 10000,
                compress_level: 3,
                object_chunk_size: 256 * 1024,
                bucket_overload_size: 1024 * 1024,
                max_small_object_size: 1024 * 1024 * 10,
            },
            lock: Some(my_principal.as_slice().to_vec().into()),
        };

        let db = AndaDB::connect(object_store.clone(), db_config).await?;
        let db = Arc::new(db);
        let web3 = Arc::new(Web3SDK::from_web3(Arc::new(web3.clone())));
        let agent = Assistant::connect(db.clone(), web3.clone()).await?;
        let memory_tool = MemoryTool::new(agent.memory());

        self.db.store(Arc::new(Some(db)));

        // Build agent engine with all configured components
        let engine = Self::builder()
            .with_web3_client(web3.clone())
            .with_store(Store::new(object_store))
            .with_management(Arc::new(BaseManagement {
                controller: my_principal,
                managers: BTreeSet::new(),
                visibility: Visibility::Private,
            }))
            .register_tools(agent.tools()?)?
            .register_tool(memory_tool)?
            .register_agent(agent)?
            .export_tools(vec![MemoryTool::NAME.to_string()]);

        if let Some(api_key) = &cfg.gemini_api_key {
            let model = Model::with_completer(Arc::new(
                gemini::Client::new(api_key, None).completion_model(gemini::GEMINI_2_5_PRO),
            ));

            let engine = engine
                .with_model(model)
                .build(Assistant::NAME.to_string())
                .await?;
            self.engine.store(Arc::new(engine));
            Ok(())
        } else {
            self.engine.store(Arc::new(engine.empty()));
            log::error!("Gemini API key is missing");
            Ok(())
        }
    }
}

pub trait AndaAssistantExt<R: Runtime> {
    fn assistant(&self) -> &AndaAssistant<R>;
    fn connect_assistant(&self);
}

impl<R: Runtime, T: Manager<R>> AndaAssistantExt<R> for T {
    fn assistant(&self) -> &AndaAssistant<R> {
        self.state::<AndaAssistant<R>>().inner()
    }

    fn connect_assistant(&self) {
        let secret_state = self.state::<SecretStateCell>();
        let cfg = secret_state.with(|state| state.assistant.clone().unwrap());
        let assistant = self.assistant().inner.clone();

        async_runtime::spawn(async move {
            assistant.connect(cfg).await.unwrap_or_else(|err| {
                log::error!("Failed to connect assistant: {err}");
            });
        });
    }
}

// Allows blocking on async code without creating a nested runtime.
// fn run_async_command<F: std::future::Future>(cmd: F) -> F::Output {
//     if tokio::runtime::Handle::try_current().is_ok() {
//         tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
//     } else {
//         tauri::async_runtime::block_on(cmd)
//     }
// }
