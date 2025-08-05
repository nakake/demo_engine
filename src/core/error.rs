use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    WindowCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
    SurfaceCreation(String),
    SurfaceConfiguration(String),
    RenderError(String),
    ResourceNotFound(String),
    ShaderCompilation(String),
    BufferCreation(String),
    PipelineCreation(String),
    EventLoopCreation(String),
    EventLoopRun(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            EngineError::AdapterRequest(msg) => write!(f, "Adapter request error: {}", msg),
            EngineError::DeviceRequest(msg) => write!(f, "Device request error: {}", msg),
            EngineError::SurfaceCreation(msg) => write!(f, "Surface creation error: {}", msg),
            EngineError::SurfaceConfiguration(msg) => write!(f, "Surface configuration error: {}", msg),
            EngineError::RenderError(msg) => write!(f, "Render error: {}", msg),
            EngineError::ResourceNotFound(msg) => write!(f, "Resource not found: {}", msg),
            EngineError::ShaderCompilation(msg) => write!(f, "Shader compilation error: {}", msg),
            EngineError::BufferCreation(msg) => write!(f, "Buffer creation error: {}", msg),
            EngineError::PipelineCreation(msg) => write!(f, "Pipeline creation error: {}", msg),
            EngineError::EventLoopCreation(msg) => write!(f, "Event loop creation error: {}", msg),
            EngineError::EventLoopRun(msg) => write!(f, "Event loop run error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;