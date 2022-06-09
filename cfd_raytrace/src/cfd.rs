use super::Result;
use async_trait::async_trait;
use flate2::read::GzDecoder;
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use serde::Deserialize;
use std::io::{Cursor, Read};
use std::path::Path;

// M1 vertez z coordinate in OSS reference frame
const OSS_M1_VERTEX: f64 = 3.9;

/// A CFD tempature and velocity sample
#[derive(Debug, Deserialize)]
pub struct TemperatureVelocityField {
    #[serde(rename = "Temperature (K)")]
    temperature: f64,
    #[serde(rename = "Velocity: Magnitude (m/s)")]
    #[allow(dead_code)]
    velocity: Option<f64>,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}

impl TemperatureVelocityField {
    /// Returns the (x,y,z) coordinates
    ///
    /// The coordinates are given with respect to M1 vertex
    pub fn coordinates(&self) -> [f64; 3] {
        [self.x, self.y, self.z - OSS_M1_VERTEX]
    }
    /// Returns the index of refraction
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

    fn distance_2_if_less_or_equal(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
        max_distance_2: <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar,
    ) -> Option<<<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar> {
        let distance_2 = self.distance_2(point);
        if distance_2 <= max_distance_2 {
            return Some(distance_2);
        }
        None
    }
}

/// Interface to compressed CFD optical turbulence csv file
#[async_trait]
pub trait FromCompressedCsv {
    #[cfg(not(feature = "s3"))]
    fn from_gz<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::convert::AsRef<str> + Send,
        Self: Sized;
    #[cfg(feature = "s3")]
    async fn from_gz<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::convert::AsRef<str> + Send,
        Self: Sized;
}
#[async_trait]
impl FromCompressedCsv for RTree<TemperatureVelocityField> {
    #[cfg(not(feature = "s3"))]
    /// Loads a csv file into a R-Tree
    fn from_gz<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::convert::AsRef<str> + Send,
    {
        let file = std::fs::File::open(path)?;
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
    #[cfg(feature = "s3")]
    /// Loads a csv file into a R-Tree
    async fn from_gz<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::convert::AsRef<str> + Send,
    {
        use s3::bucket::Bucket;
        use s3::creds::Credentials;
        let bucket_name = "gmto.cfd.2022";
        let region = "us-east-2".parse()?;
        let credentials = Credentials::default().map_err(|e| s3::error::S3Error::Credentials(e))?;
        let bucket = Bucket::new(bucket_name, region, credentials)?;
        let (data, _) = bucket.get_object(path).await?;
        let stream = std::io::Cursor::new(data);

        let mut decoder = GzDecoder::new(stream);
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

/// Shepard radial basis function interpolation
pub trait Shepard {
    /// Interpolates the refraction index
    ///
    /// Interpolates the refraction index at `query_point` using all the samples within the
    /// squared radius `max_squared_radius` from `query_point`.
    /// The radial basis function is r<sup>-2</sup>.
    fn shepard(&self, query_point: &[f64; 3], max_squared_radius: f64) -> Option<f64>;
}
impl Shepard for rstar::RTree<TemperatureVelocityField> {
    fn shepard(&self, query_point: &[f64; 3], max_squared_radius: f64) -> Option<f64> {
        let samples = self.locate_within_distance(query_point.clone(), max_squared_radius);
        let mut num = None;
        let mut denom = None;
        for sample in samples {
            let d2 = sample.distance_2(&query_point);
            if d2 > 0f64 {
                let rbf = d2.recip();
                *num.get_or_insert(0f64) += rbf * sample.refraction_index();
                *denom.get_or_insert(0f64) += rbf;
            } else {
                return Some(sample.refraction_index());
            }
        }
        match (num, denom) {
            (Some(num), Some(denom)) => Some(num / denom),
            _ => self
                .nearest_neighbor(query_point)
                .map(|nn| nn.refraction_index()),
        }
    }
}
