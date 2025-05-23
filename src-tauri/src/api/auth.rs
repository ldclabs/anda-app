use candid::Principal;
use ic_agent::Identity;
use ic_tee_agent::AnonymousIdentity;
use tauri::{AppHandle, Manager};

use super::Result;
use crate::SecretStateCell;
use crate::deeplink::DeepLinkServiceExt;
use crate::service::icp::{ICPClientExt, IdentityInfo};

#[tauri::command]
pub async fn identity(app: AppHandle) -> Result<IdentityInfo> {
    let id = app.icp().identity();
    Ok(IdentityInfo::from(id.as_ref()))
}

#[tauri::command]
pub async fn sign_in(app: AppHandle) -> Result<bool> {
    if app.icp().identity().is_authenticated() {
        return Ok(false);
    }

    app.deep_link_service().start_sign_in()?;
    Ok(true)
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<bool> {
    if app.icp().identity().sender().unwrap() == Principal::anonymous() {
        return Ok(false);
    }

    let secret_state = app.state::<SecretStateCell>();
    secret_state.with_mut(|state| {
        state.auth = None;
        app.icp().set_identity(Box::new(AnonymousIdentity));

        Ok::<(), String>(())
    })?;
    secret_state.save()?;

    Ok(true)
}
