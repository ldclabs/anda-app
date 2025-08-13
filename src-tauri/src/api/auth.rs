use candid::Principal;
use ic_agent::Identity;
use ic_auth_verifier::AnonymousIdentity;
use ic_cose_types::CanisterCaller;
use tauri::{AppHandle, Manager};

use super::Result;
use crate::SecretStateCell;
use crate::deeplink::{DeepLinkResponse, DeepLinkServiceExt};
use crate::model::user::UserInfo;
use crate::service::icp::{ICPClientExt, IdentityInfo};

#[tauri::command]
pub async fn identity(app: AppHandle) -> Result<IdentityInfo> {
    let id = app.icp().identity();
    Ok(IdentityInfo::from(id.as_ref()))
}

// dMsg canister: "nscli-qiaaa-aaaaj-qa4pa-cai"
static IC_MESSAGE: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 48, 7, 30, 1, 1]);

#[tauri::command]
pub async fn get_user(app: AppHandle) -> Result<UserInfo> {
    let id = app.icp().identity();
    let user: std::result::Result<UserInfo, String> = app
        .icp()
        .canister_query(&IC_MESSAGE, "get_user", (&Some(id.sender().unwrap()),))
        .await
        .map_err(|err| err.to_string())?;
    let user = user?;
    Ok(user)
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
pub async fn sign_in_by_url(app: AppHandle, url: String) -> Result<bool> {
    if let Ok(dr) = DeepLinkResponse::from_str(&url) {
        app.deep_link_service().on_sign_in(dr)?;
    }

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
