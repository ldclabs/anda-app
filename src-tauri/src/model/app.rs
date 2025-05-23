use candid::Principal;
use ic_agent::Identity;
use ic_auth_types::{ByteArrayB64, ByteBufB64, SignedDelegation};
use ic_cose_types::cose::kdf::{derive_a256gcm_key, hkdf256};
use ic_tee_agent::identity::{DelegatedIdentity, basic_identity, signed_delegation_from};
use serde::{Deserialize, Serialize};
use tauri::Theme;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct AppState {
    pub os_arch: String,
    pub os_platform: String,
    pub settings: Settings,
    pub seed: ByteArrayB64<32>,
}

impl AppState {
    pub fn to_client_state(&self, salt: Option<&[u8]>) -> AppState {
        let seed = hkdf256(self.seed.as_slice(), salt, &[]);
        AppState {
            os_arch: self.os_arch.clone(),
            os_platform: self.os_platform.clone(),
            settings: self.settings.clone(),
            seed: seed.into(),
        }
    }

    pub fn derive_a256gcm_key(&self, salt: &[u8]) -> [u8; 32] {
        derive_a256gcm_key(self.seed.as_slice(), Some(salt))
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Settings {
    pub locale: String,
    pub theme: Option<Theme>, // "light" | "dark"
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SecretState {
    pub session_secret: ByteArrayB64<32>, // ed25519 private key
    pub auth: Option<InternetIdentityAuth>,
}

impl SecretState {
    pub fn session_pubkey(&self) -> ByteBufB64 {
        let session = basic_identity(*self.session_secret);
        session.public_key().unwrap().into()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct InternetIdentityAuth {
    pub user_pubkey: ByteBufB64,
    pub delegations: Vec<SignedDelegation>,
    pub authn_method: String, // "pin" | "passkey" | "recovery"
    pub origin: String,
}

impl InternetIdentityAuth {
    pub fn principal(&self) -> Principal {
        Principal::self_authenticating(self.user_pubkey.as_slice())
    }

    pub fn to_identity(&self, session_secret: [u8; 32]) -> Result<DelegatedIdentity, String> {
        let session = basic_identity(session_secret);
        let id = DelegatedIdentity::new(
            self.user_pubkey.to_vec(),
            Box::new(session),
            self.delegations
                .clone()
                .into_iter()
                .map(signed_delegation_from)
                .collect(),
        )
        .map_err(|err| format!("{:?}", err))?;

        Ok(id)
    }
}
