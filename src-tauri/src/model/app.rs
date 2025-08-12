use candid::Principal;
use ic_agent::Identity;
use ic_auth_types::{ByteArrayB64, ByteBufB64, SignedDelegationCompact};
use ic_auth_verifier::{
    envelope::verify_delegation_chain,
    identity::{
        BasicIdentity, DelegatedIdentity, delegated_basic_identity, signed_delegation_from,
    },
    unix_timestamp,
};
use ic_cose_types::cose::kdf::{derive_a256gcm_key, hkdf256};
use serde::{Deserialize, Serialize};
use tauri::Theme;

use crate::utils::SensitiveData;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppState {
    pub os_arch: String,
    pub os_platform: String,
    pub settings: Settings,
    pub seed: SensitiveData<ByteArrayB64<32>>,
}

impl AppState {
    pub fn to_client_state(&self, salt: Option<&[u8]>) -> AppState {
        let seed = hkdf256(self.seed.as_slice(), salt, &[]);
        AppState {
            os_arch: self.os_arch.clone(),
            os_platform: self.os_platform.clone(),
            settings: self.settings.clone(),
            seed: SensitiveData(seed.into()),
        }
    }

    pub fn derive_a256gcm_key(&self, salt: &[u8]) -> [u8; 32] {
        derive_a256gcm_key(self.seed.as_slice(), Some(salt))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub locale: String,
    pub theme: Option<Theme>, // "light" | "dark"
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SecretState {
    pub session_secret: SensitiveData<ByteArrayB64<32>>, // ed25519 private key
    pub auth: Option<InternetIdentityAuth>,
    pub assistant: Option<AssistantConfig>,
}

impl SecretState {
    pub fn session_pubkey(&self) -> ByteBufB64 {
        let session = BasicIdentity::from_raw_key(&self.session_secret);
        session.public_key().unwrap().into()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InternetIdentityAuth {
    pub user_pubkey: ByteBufB64,
    pub delegations: Vec<SignedDelegationCompact>,
    pub authn_method: String, // "pin" | "passkey" | "recovery" | "dMsg"
    pub origin: String,       // "https://dmsg.net" | "https://panda.fans"
}

impl InternetIdentityAuth {
    pub fn principal(&self) -> Principal {
        Principal::self_authenticating(self.user_pubkey.as_slice())
    }

    pub fn to_identity(&self, session_secret: [u8; 32]) -> Result<DelegatedIdentity, String> {
        let session = BasicIdentity::from_raw_key(&session_secret);
        let session_pubkey = session.public_key().unwrap();
        verify_delegation_chain(
            &self.user_pubkey,
            session_pubkey.as_slice(),
            &self.delegations,
            unix_timestamp().as_millis() as u64,
            None,
        )?;
        let id = DelegatedIdentity::new_unchecked(
            self.user_pubkey.to_vec(),
            Box::new(session),
            self.delegations
                .clone()
                .into_iter()
                .map(|d| signed_delegation_from(d.into()))
                .collect(),
        );

        Ok(id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssistantConfig {
    pub ed25519_secret: SensitiveData<ByteArrayB64<32>>, // ed25519 private key
    pub root_secret: SensitiveData<ByteArrayB64<48>>,    // root private key
    pub user_pubkey: ByteBufB64,
    pub gemini_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub grok_api_key: Option<String>,
    pub openai_api_key: Option<String>,
}

impl AssistantConfig {
    pub fn principal(&self) -> Principal {
        Principal::self_authenticating(self.user_pubkey.as_slice())
    }

    pub fn to_identity(&self) -> DelegatedIdentity {
        let expires_in_ms = 1000 * 3600 * 24 * 365;
        let identity = BasicIdentity::from_raw_key(&self.ed25519_secret);
        delegated_basic_identity(&identity, expires_in_ms)
    }
}
