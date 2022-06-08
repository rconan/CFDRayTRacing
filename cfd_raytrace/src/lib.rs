mod ray_tracing;
pub use ray_tracing::GsOnAxisParams;
mod cfd;
pub use cfd::{FromCompressedCsv, TemperatureVelocityField};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open npz data file")]
    NPZ,
    #[error("failed to read variable")]
    Read(#[from] std::io::Error),
    #[error("failed to read csv data")]
    CSV(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
