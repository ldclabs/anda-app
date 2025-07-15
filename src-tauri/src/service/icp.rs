use candid::{
    CandidType, Decode, Principal,
    utils::{ArgumentEncoder, encode_args},
};
use ic_agent::{Agent, Identity};
use ic_auth_verifier::{AtomicIdentity, get_expiration};
use ic_cose_types::{BoxError, CanisterCaller};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, Manager, Runtime, async_runtime,
    plugin::{Builder, TauriPlugin},
};

// #[cfg(debug_assertions)]
// pub const ICP_HOST: &str = "http://127.0.0.1:4943";

// #[cfg(not(debug_assertions))]
pub const ICP_HOST: &str = "https://icp-api.io";

pub const IDENTITY_EVENT: &str = "IdentityChanged";
#[derive(Clone, Deserialize, Serialize)]
pub struct IdentityInfo {
    pub id: Principal,
    pub expiration: Option<u64>, // in miniseconds
}

impl IdentityInfo {
    pub fn from(id: &impl Identity) -> Self {
        IdentityInfo {
            id: id.sender().unwrap(),
            expiration: get_expiration(id).map(|v| v / 1000000),
        }
    }
}

pub struct ICPClient<R: Runtime> {
    app: AppHandle<R>,
    identity: Arc<AtomicIdentity>,
    agent: Agent,
}

impl<R: Runtime> ICPClient<R> {
    const NAME: &'static str = "icp-client";

    pub fn init() -> TauriPlugin<R> {
        Builder::new(Self::NAME)
            .setup(move |app, _api| {
                let identity = Arc::new(AtomicIdentity::default());
                let agent = Agent::builder()
                    .with_url(ICP_HOST)
                    .with_arc_identity(identity.clone())
                    .with_verify_query_signatures(false);

                let agent = if ICP_HOST.starts_with("https://") {
                    agent.with_background_dynamic_routing().build()?
                } else {
                    agent.build()?
                };

                let _agent = agent.clone();
                if ICP_HOST.starts_with("http://") {
                    async_runtime::spawn(async move {
                        // ignore the error
                        let _ = _agent.fetch_root_key().await;
                    });
                }
                app.manage(ICPClient {
                    app: app.clone(),
                    identity,
                    agent,
                });
                log::info!("Initialized {} with ICP_HOST {:?}", Self::NAME, ICP_HOST);
                Ok(())
            })
            .build()
    }

    pub fn identity(&self) -> Arc<AtomicIdentity> {
        self.identity.clone()
    }

    pub fn set_identity(&self, identity: Box<dyn Identity>) {
        let payload = IdentityInfo::from(&identity);
        self.identity.set(identity);
        let _ = self.app.emit(IDENTITY_EVENT, payload);
    }
}

impl<R: Runtime> CanisterCaller for ICPClient<R> {
    async fn canister_query<
        In: ArgumentEncoder + Send,
        Out: CandidType + for<'a> candid::Deserialize<'a>,
    >(
        &self,
        canister: &Principal,
        method: &str,
        args: In,
    ) -> Result<Out, BoxError> {
        let input = encode_args(args)?;
        let res = self
            .agent
            .query(canister, method)
            .with_arg(input)
            .call()
            .await?;
        let output = Decode!(res.as_slice(), Out)?;
        Ok(output)
    }

    async fn canister_update<
        In: ArgumentEncoder + Send,
        Out: CandidType + for<'a> candid::Deserialize<'a>,
    >(
        &self,
        canister: &Principal,
        method: &str,
        args: In,
    ) -> Result<Out, BoxError> {
        let input = encode_args(args)?;
        let res = self
            .agent
            .update(canister, method)
            .with_arg(input)
            .call_and_wait()
            .await?;
        let output = Decode!(res.as_slice(), Out)?;
        Ok(output)
    }
}

pub trait ICPClientExt<R: Runtime> {
    fn icp(&self) -> &ICPClient<R>;
}

impl<R: Runtime, T: Manager<R>> ICPClientExt<R> for T {
    fn icp(&self) -> &ICPClient<R> {
        self.state::<ICPClient<R>>().inner()
    }
}
