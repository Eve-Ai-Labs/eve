use poem::{handler, web::Json};
use types::ai::models::{AiDownloadModel, AiWebModel};

/// AI models for download
#[handler]
pub fn handler_ai_model() -> poem::Result<Json<AiDownloadModel>> {
    Ok(Json(AiWebModel::DeepseekR1_1_5b.into()))
}
