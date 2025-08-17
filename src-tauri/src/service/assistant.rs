use anda_assistant::Assistant;
use anda_core::{BoxError, BoxPinFut, Path as DBPath, derivation_path_with};
use anda_db::{
    database::{AndaDB, DBConfig},
    storage::StorageConfig,
};
use anda_engine::{
    context::{Web3ClientFeatures, Web3SDK},
    engine::{AgentInfo, Engine, EngineBuilder},
    management::{BaseManagement, SYSTEM_PATH, Visibility},
    memory::MemoryTool,
    model::{Model, Proxy, deepseek, gemini, openai, request_client_builder, xai},
    store::{LocalFileSystem, Store},
};
use anda_object_store::EncryptedStoreBuilder;
use anda_web3_client::client::Client as Web3Client;
use arc_swap::ArcSwap;
use futures::try_join;
use ic_agent::Agent;
use ic_auth_types::ByteBufB64;
use ic_auth_verifier::{AtomicIdentity, sha3_256};
use parking_lot::RwLock;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tauri::{
    AppHandle, Emitter, Manager, Runtime, async_runtime,
    plugin::{Builder, TauriPlugin},
};
use tokio_util::sync::CancellationToken;

use crate::{AppStateCell, SecretStateCell, model::app::AssistantConfig};

use super::icp::{ICP_HOST, ICPClientExt};

pub const ASSISTANT_EVENT: &str = "AssistantReady";

pub struct AndaAssistant<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    inner: Arc<InnerAssistant>,
}

struct InnerAssistant {
    dir: PathBuf,
    db: RwLock<Option<Arc<AndaDB>>>,
    assistant: RwLock<Option<Arc<Assistant>>>,
    engine: ArcSwap<Engine>,
    should_restart: Arc<AtomicU64>,
    cancel_token: CancellationToken,
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
                        db: RwLock::new(None),
                        assistant: RwLock::new(None),
                        engine: ArcSwap::new(Arc::new(InnerAssistant::builder().empty())),
                        should_restart: Arc::new(AtomicU64::new(0)),
                        cancel_token: CancellationToken::new(),
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

    pub fn self_name(&self) -> BoxPinFut<Option<String>> {
        let assistant = self.inner.assistant.read().clone();

        Box::pin(async move {
            if let Some(assistant) = assistant {
                assistant.self_name().await
            } else {
                None
            }
        })
    }

    pub fn engine(&self) -> Arc<Engine> {
        self.inner.engine.load().clone()
    }

    pub fn flush(&self) {
        let db = self.inner.db.read().clone();
        if let Some(db) = db {
            async_runtime::spawn(async move {
                match db.flush().await {
                    Ok(_) => log::info!("Anda Assistant flushed successfully"),
                    Err(e) => log::error!("Failed to flush Anda Assistant: {}", e),
                }
            });
        }
    }

    #[allow(dead_code)]
    pub fn close(&self) {
        self.inner.cancel_token.cancel();
        let engine = self.inner.engine.load().clone();
        let db = self.inner.db.read().clone();
        async_runtime::block_on(async move {
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

    async fn connect(
        &self,
        identity: Arc<AtomicIdentity>,
        agent: Agent,
        cfg: AssistantConfig,
        https_proxy: Option<String>,
    ) -> Result<bool, BoxError> {
        let mut http_client = request_client_builder();
        if let Some(proxy) = https_proxy {
            http_client = http_client.proxy(Proxy::all(proxy)?);
        }
        let http_client = http_client.build()?;

        let web3 = Web3Client::builder()
            .with_ic_host(ICP_HOST)
            .with_identity(identity)
            .with_agent(agent)
            .with_http_client(http_client.clone())
            .with_root_secret(**cfg.root_secret)
            .build()
            .await?;

        let web3 = Arc::new(web3);

        let my_principal = web3.get_principal();
        log::info!(
            "start AI assistant with principal ID: {:?}",
            my_principal.to_text()
        );

        let db = self.db.read().clone();
        let db = if let Some(db) = db {
            db
        } else {
            let os_secret = web3
                .a256gcm_key(derivation_path_with(
                    &DBPath::from(SYSTEM_PATH),
                    vec![b"object_store".to_vec(), b"A256GCM".to_vec()],
                ))
                .await?;

            let lock = sha3_256(&os_secret);
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
                lock: Some(ByteBufB64(lock.into())),
            };

            let db = AndaDB::connect(object_store.clone(), db_config).await?;

            let db = Arc::new(db);
            *self.db.write() = Some(db.clone());

            let db_ = db.clone();
            let cancel_token = self.cancel_token.child_token();
            tokio::spawn(async move {
                db_.auto_flush(cancel_token, Duration::from_millis(60 * 1000))
                    .await;
            });

            db
        };

        let web3 = Arc::new(Web3SDK::from_web3(web3));
        let object_store = db.object_store().clone();
        let assistant = Assistant::connect(db.clone(), None)
            .await?
            .with_max_input_tokens(256 * 1024);
        let memory_tool = MemoryTool::new(assistant.memory());

        {
            *self.assistant.write() = Some(Arc::new(assistant.clone()));
        }

        // Build agent engine with all configured components
        let engine = Self::builder()
            .with_web3_client(web3)
            .with_store(Store::new(object_store))
            .with_management(Arc::new(BaseManagement {
                controller: my_principal,
                managers: BTreeSet::new(),
                visibility: Visibility::Private,
            }))
            .register_tools(assistant.tools()?)?
            .register_tool(memory_tool)?
            .register_agent(assistant)?
            .export_tools(vec![MemoryTool::NAME.to_string()]);

        if let Some((name, provider)) = cfg.get_provider() {
            let model = match name {
                "gemini" => Model::with_completer(Arc::new(
                    gemini::Client::new(&provider.api_key, provider.api_base.clone())
                        .with_client(http_client)
                        .completion_model(&provider.model),
                )),
                "deepseek" => Model::with_completer(Arc::new(
                    deepseek::Client::new(&provider.api_key, provider.api_base.clone())
                        .with_client(http_client)
                        .completion_model(&provider.model),
                )),
                "xai" => Model::with_completer(Arc::new(
                    xai::Client::new(&provider.api_key, provider.api_base.clone())
                        .with_client(http_client)
                        .completion_model(&provider.model),
                )),
                "openai" => Model::with_completer(Arc::new(
                    openai::Client::new(&provider.api_key, provider.api_base.clone())
                        .with_client(http_client)
                        .completion_model(&provider.model),
                )),
                _ => return Err(format!("Unknown model provider: {}", name).into()),
            };

            let engine = engine
                .with_model(model)
                .build(Assistant::NAME.to_string())
                .await?;
            self.engine.store(Arc::new(engine));
            log::info!(
                "Connected to {} model provider with model: {}",
                name,
                provider.model
            );
            Ok(true)
        } else {
            self.engine.store(Arc::new(engine.empty()));
            log::error!("LLM API key is missing");
            Ok(false)
        }
    }
}

pub trait AndaAssistantExt<R: Runtime> {
    fn assistant(&self) -> &AndaAssistant<R>;
    fn connect_assistant(&self);
    fn propose_reconnect_assistant(&self);
    fn try_reconnect_assistant(&self);
    fn save_assistant(&self);
}

impl<R: Runtime, T: Manager<R>> AndaAssistantExt<R> for T {
    fn assistant(&self) -> &AndaAssistant<R> {
        self.state::<AndaAssistant<R>>().inner()
    }

    fn connect_assistant(&self) {
        let cfg = self
            .state::<SecretStateCell>()
            .with(|state| state.assistant.clone().unwrap());
        let proxy = self
            .state::<AppStateCell>()
            .with(|state| state.settings.https_proxy.clone());
        let assistant = self.assistant().inner.clone();
        let identity = self.icp().identity();
        let agent = self.icp().agent().clone();

        let app = self.app_handle().clone();
        async_runtime::spawn(async move {
            match assistant.connect(identity, agent, cfg, proxy).await {
                Ok(is_ready) => {
                    let _ = app.emit(ASSISTANT_EVENT, is_ready);
                }
                Err(err) => {
                    log::error!("Failed to connect assistant: {err}");
                }
            };
        });
    }

    fn propose_reconnect_assistant(&self) {
        let assistant = self.assistant().inner.clone();
        assistant.should_restart.fetch_add(1, Ordering::Relaxed);
    }

    fn try_reconnect_assistant(&self) {
        let assistant = self.assistant().inner.clone();
        let should_restart = assistant.should_restart.swap(0, Ordering::Relaxed);
        if should_restart > 0 {
            self.connect_assistant();
        }
    }

    fn save_assistant(&self) {
        self.state::<AndaAssistant<R>>().inner().flush();
    }
}
