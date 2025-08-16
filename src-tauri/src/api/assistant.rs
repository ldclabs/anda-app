use anda_core::{AgentInput, AgentOutput, Json, ToolInput, ToolOutput};
use anda_engine::context::EngineCard;
use ic_agent::Identity;
use tauri::AppHandle;

use super::Result;
use crate::service::{assistant::AndaAssistantExt, icp::ICPClientExt};

#[tauri::command]
pub async fn assistant_info(app: AppHandle) -> Result<EngineCard> {
    let engine = app.assistant().engine();
    Ok(engine.information())
}

#[tauri::command]
pub async fn assistant_name(app: AppHandle) -> Option<String> {
    app.assistant().self_name().await
}

#[tauri::command]
pub async fn tool_call(app: AppHandle, input: ToolInput<Json>) -> Result<ToolOutput<Json>> {
    let id = app.icp().identity();
    let caller = id.sender().unwrap();
    let engine = app.assistant().engine();
    let res = engine.tool_call(caller, input).await?;
    Ok(res)
}

#[tauri::command]
pub async fn agent_run(app: AppHandle, input: AgentInput) -> Result<AgentOutput> {
    let id = app.icp().identity();
    let caller = id.sender().unwrap();
    let engine = app.assistant().engine();
    let res = engine.agent_run(caller, input).await?;
    Ok(res)
}
