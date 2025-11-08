#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("state: {0}")] State(#[from] knhk_state::StateError),
    #[error("admission: {0}")] Admission(String),
    #[error("pattern: {0}")] Pattern(String),
    #[error("not found: {0}")] NotFound(String),
}
src/legacy.rs (façade + promotion hooks)
/// Legacy-mode façade lives here (YAWL-native behaviors).
/// The reflex bridge consults promotion policies to swap safe segments to hot-path.
pub struct LegacyFacade;
impl LegacyFacade {
    pub fn promoteable(_spec: &serde_json::Value) -> bool { true } // stub
}