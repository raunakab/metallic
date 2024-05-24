pub mod primitives;
pub mod rendering_engine;

use thiserror::Error;
use wgpu::{CreateSurfaceError, RequestDeviceError, SurfaceError};
use winit::error::OsError;

pub type MetallicResult<T> = Result<T, MetallicError>;

#[derive(Error, Debug)]
pub enum MetallicError {
    #[error("Surface error: {0:?}")]
    SurfaceError(#[from] SurfaceError),

    #[error("Os error: {0:?}")]
    OsError(#[from] OsError),

    #[error("Create surface error: {0:?}")]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error("Request device error: {0:?}")]
    RequestDeviceError(#[from] RequestDeviceError),

    #[error("No adapter found error")]
    NoAdapterFoundError,

    #[error("Invalid configuration error: {0:?}")]
    InvalidConfigurationError(#[from] InvalidConfigurationError),
}

#[derive(Error, Debug)]
pub enum InvalidConfigurationError {
    #[error("No 'srgb' texture-formats were found; at least one was expected")]
    NoTextureFormatFoundError,

    #[error("No 'fifo' present-mode was found; this present mode is required")]
    NoFifoPresentModeFoundError,

    #[error("No alpha-modes were found; at least one was expected")]
    NoAlphaModeFoundError,
}
