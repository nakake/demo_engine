use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    AdapterRequest(String),
    DeviceRequest(String),
    SurfaceCreation(String),
    RenderError(String),
    ResourceNotFound(String),
    EventLoopCreation(String),
    EventLoopRun(String),
    SceneNotFound(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::AdapterRequest(msg) => write!(f, "Adapter request error: {}", msg),
            EngineError::DeviceRequest(msg) => write!(f, "Device request error: {}", msg),
            EngineError::SurfaceCreation(msg) => write!(f, "Surface creation error: {}", msg),
            EngineError::RenderError(msg) => write!(f, "Render error: {}", msg),
            EngineError::ResourceNotFound(msg) => write!(f, "Resource not found: {}", msg),
            EngineError::EventLoopCreation(msg) => write!(f, "Event loop creation error: {}", msg),
            EngineError::EventLoopRun(msg) => write!(f, "Event loop run error: {}", msg),
            EngineError::SceneNotFound(msg) => write!(f, "Scene not found: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;
