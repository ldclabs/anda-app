use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: Principal,
    pub name: String,
    pub image: String,
    pub profile_canister: Principal,
    pub cose_canister: Option<Principal>,
    pub username: Option<String>,
}
