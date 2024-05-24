pub mod primitives;
pub mod rendering_engine;

use thiserror::Error;
use wgpu::{CreateSurfaceError, RequestDeviceError, SurfaceError};
use winit::error::OsError;

pub type MetallicResult<T> = Result<T, MetallicError>;

#[derive(Error, Debug)]
pub enum MetallicError {
    #[error("...")]
    SurfaceError(#[from] SurfaceError),

    #[error("...")]
    OsError(#[from] OsError),

    #[error("...")]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error("...")]
    RequestDeviceError(#[from] RequestDeviceError),

    #[error("...")]
    NoAdapterFoundError,

    #[error("...")]
    InvalidConfigurationError(#[from] InvalidConfigurationError),
}

#[derive(Error, Debug)]
pub enum InvalidConfigurationError {
    #[error("...")]
    NoTextureFormatFoundError,

    #[error("...")]
    NoPresentModeFoundError,

    #[error("...")]
    NoAlphaModeFoundError,
}
