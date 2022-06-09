mod ray_tracing;
pub use ray_tracing::RayTracer;
mod cfd;
pub use cfd::{FromCompressedCsv, Shepard, TemperatureVelocityField};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open npz data file")]
    NPZ,
    #[error("failed to read variable")]
    Read(#[from] std::io::Error),
    #[error("failed to read csv data")]
    CSV(#[from] csv::Error),
    #[cfg(feature = "s3")]
    #[error("failed to get S3 object")]
    S3(#[from] s3::error::S3Error),
    #[error("failed to parse UTF8")]
    UTF8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
