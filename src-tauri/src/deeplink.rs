use ciborium::from_reader;
use ic_agent::Identity;
use ic_auth_types::{ByteBufB64, SignedDelegationCompact};
use ic_cose::rand_bytes;
use ic_cose_types::to_cbor_bytes;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::str::FromStr;
use tauri::{
    AppHandle, Manager, Runtime, Url,
    plugin::{Builder, TauriPlugin},
};
use tauri_plugin_opener::OpenerExt;

use crate::{
    AppStateCell, Result, SecretStateCell, model::app::InternetIdentityAuth,
    service::icp::ICPClientExt, utils::SensitiveData,
};

#[derive(Debug)]
pub struct DeepLinkRequest<'a, T: Serialize> {
    pub os: &'a str,        // "linux" | "windows" | "macos" | "ios" | "android"
    pub action: &'a str,    // "signin"
    pub next_url: &'a str,  // "https://anda.ai/deeplink"
    pub payload: Option<T>, // encode as base64url
}

impl<T> DeepLinkRequest<'_, T>
where
    T: Serialize,
{
    pub fn to_url(&self, endpoint: &Url) -> Url {
        let mut url = endpoint.clone();
        url.query_pairs_mut()
            .append_pair("os", self.os)
            .append_pair("action", self.action)
            .append_pair("next_url", self.next_url);

        if let Some(payload) = &self.payload {
            let data: ByteBufB64 = to_cbor_bytes(payload).into();
            url.set_fragment(Some(data.to_string().as_str()));
        }

        url
    }
}

#[derive(Debug, Serialize)]
pub struct DeepLinkResponse {
    pub url: Url,
    pub os: String,
    pub action: String,              // "SignIn"
    pub payload: Option<ByteBufB64>, // decode from base64url
}

impl DeepLinkResponse {
    pub fn from_str(url: &str) -> Result<Self> {
        let url = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        Self::from_url(url)
    }
    pub fn from_url(url: Url) -> Result<Self> {
        let mut query_pairs = url.query_pairs();
        let payload = match url.fragment() {
            Some(f) => Some(ByteBufB64::from_str(f)?),
            None => None,
        };

        Ok(DeepLinkResponse {
            os: query_pairs
                .find(|(k, _)| k == "os")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default(),
            action: query_pairs
                .find(|(k, _)| k == "action")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default(),
            payload,
            url,
        })
    }

    pub fn get_payload<T: DeserializeOwned>(&self) -> Result<T> {
        if let Some(payload) = &self.payload {
            Ok(from_reader(payload.as_slice())?)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "payload is None",
            )))
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SignInRequest {
    #[serde(rename = "s")]
    pub session_pubkey: ByteBufB64,
    #[serde(rename = "m")]
    pub max_time_to_live: u64, // in miniseconds
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SignInResponse {
    #[serde(rename = "u")]
    pub user_pubkey: ByteBufB64,
    #[serde(rename = "d")]
    pub delegations: Vec<SignedDelegationCompact>,
    #[serde(rename = "a")]
    pub authn_method: String,
    #[serde(rename = "o")]
    pub origin: String,
}

impl From<SignInResponse> for InternetIdentityAuth {
    fn from(value: SignInResponse) -> Self {
        Self {
            user_pubkey: value.user_pubkey,
            delegations: value.delegations,
            authn_method: value.authn_method,
            origin: value.origin,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// https://dmsg.net/oauth/deeplink
    pub sign_in_endpoint: String,
}

pub struct DeepLinkService<R: Runtime> {
    app: AppHandle<R>,
    sign_in_endpoint: Url,
}

const MAX_TIME_TO_LIVE: u64 = 1000 * 60 * 60 * 24 * 30; // 30 days

impl<R: Runtime> DeepLinkService<R> {
    const NAME: &'static str = "deep-link-service";

    pub fn init() -> TauriPlugin<R, Config> {
        Builder::new(Self::NAME)
            .setup(move |app, api| {
                let cfg: &Config = api.config();
                let sign_in_endpoint = Url::parse(&cfg.sign_in_endpoint).map_err(|err| {
                    format!(
                        "failed to parse sign_in_endpoint {:?}: {err:?}",
                        cfg.sign_in_endpoint
                    )
                })?;

                app.manage(DeepLinkService {
                    app: app.clone(),
                    sign_in_endpoint,
                });
                log::info!("Initialized {}", Self::NAME);
                Ok(())
            })
            .build()
    }

    pub fn start_sign_in(&self) -> Result<()> {
        let os = self
            .app
            .state::<AppStateCell>()
            .with(|state| state.os_platform.clone());
        let session_pubkey = self.app.state::<SecretStateCell>().with_mut(|state| {
            state.session_secret = SensitiveData(rand_bytes().into());
            state.auth = None;
            state.session_pubkey()
        });

        // don't save here, save in on_sign_in
        // secret_state.save()?;
        let request = DeepLinkRequest::<SignInRequest> {
            os: os.as_str(),
            next_url: "https://anda.ai/deeplink/",
            action: "SignIn",
            payload: Some(SignInRequest {
                session_pubkey,
                max_time_to_live: MAX_TIME_TO_LIVE,
            }),
        };
        let url = request.to_url(&self.sign_in_endpoint);
        self.app.opener().open_url(url.to_string(), None::<&str>)?;
        Ok(())
    }

    pub fn on_open_url(&self, urls: Vec<Url>) {
        for url in urls {
            let s = url.to_string();
            match DeepLinkResponse::from_url(url) {
                Ok(res) => {
                    log::debug!(deep_link:serde = res; "deep link response");
                    match res.action.as_str() {
                        "SignIn" => {
                            if let Err(err) = self.on_sign_in(res) {
                                log::error!(
                                    action = "SignIn",
                                    error = format!("{err:?}");
                                    "failed to sign in with deep link");
                            }
                        }
                        _ => {
                            log::warn!(
                                action = res.action,
                                response:serde = res;
                                "unknown deep link action");
                        }
                    }
                }
                Err(err) => {
                    log::error!(
                        url = s,
                        error = format!("{err:?}");
                        "failed to parse deep link URL");
                }
            }
        }
    }

    pub fn on_sign_in(&self, res: DeepLinkResponse) -> Result<()> {
        let res: SignInResponse = res.get_payload()?;
        let auth = InternetIdentityAuth::from(res);
        let secret_state = self.app.state::<SecretStateCell>();
        let principal = secret_state.with_mut(|state| {
            let id = auth.to_identity(**state.session_secret)?;
            let principal = id.sender().unwrap();
            self.app.icp().set_identity(Box::new(id));

            state.auth = Some(auth);
            Ok::<_, Box<dyn std::error::Error>>(principal)
        })?;
        secret_state.save()?;
        log::info!(
            service = "DeepLink",
            action = "SignIn",
            principal = principal.to_text();
            "success");

        Ok(())
    }
}

pub trait DeepLinkServiceExt<R: Runtime> {
    fn deep_link_service(&self) -> &DeepLinkService<R>;
    fn deep_link_service_owned(&self) -> DeepLinkService<R>;
}

impl<R: Runtime, T: Manager<R>> DeepLinkServiceExt<R> for T {
    fn deep_link_service(&self) -> &DeepLinkService<R> {
        self.state::<DeepLinkService<R>>().inner()
    }

    fn deep_link_service_owned(&self) -> DeepLinkService<R> {
        let dls = self.state::<DeepLinkService<R>>().inner();
        DeepLinkService {
            app: dls.app.clone(),
            sign_in_endpoint: dls.sign_in_endpoint.clone(),
        }
    }
}
