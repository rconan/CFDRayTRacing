use super::Result;
use flate2::read::GzDecoder;
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use serde::Deserialize;
use std::path::Path;
use std::{
    fs::File,
    io::{Cursor, Read},
};

#[derive(Debug, Deserialize)]
pub struct TemperatureVelocityField {
    #[serde(rename = "Temperature (K)")]
    temperature: f64,
    #[serde(rename = "Velocity: Magnitude (m/s)")]
    #[allow(dead_code)]
    velocity: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}

impl TemperatureVelocityField {
    pub fn coordinates(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    pub fn refraction_index(&self) -> f64 {
        let pref = 75000.0;
        let wlm = 0.5;
        7.76e-7 * pref * (1. + 0.00752 / (wlm * wlm)) / self.temperature
    }
}

impl RTreeObject for TemperatureVelocityField {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.coordinates())
    }
}

impl PointDistance for TemperatureVelocityField {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        self.coordinates()
            .into_iter()
            .zip(point)
            .map(|(x, &y)| x - y)
            .map(|x| x * x)
            .sum()
    }
}

pub trait FromCompressedCsv {
    fn from_gz<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}
impl FromCompressedCsv for RTree<TemperatureVelocityField> {
    fn from_gz<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes).unwrap();

        let buff = Cursor::new(bytes);
        let mut rdr = csv::Reader::from_reader(buff);
        Ok(RTree::bulk_load(
            rdr.deserialize()
                .collect::<std::result::Result<Vec<TemperatureVelocityField>, csv::Error>>()?,
        ))
    }
}
