use super::cfd::Shepard;
use super::{Result, TemperatureVelocityField};
use nalgebra::DMatrix;
use rstar::RTree;
#[cfg(feature = "linya")]
use std::fmt::Write;
use std::path::Path;

/// Ray tracing parameters
pub struct GsOnAxisParams {
    m: Vec<bool>,
    pub xyz: Vec<DMatrix<f64>>,
    pub klm: Vec<DMatrix<f64>>,
    shepard_radius2: f64,
    step_length: f64,
}
impl Default for GsOnAxisParams {
    fn default() -> Self {
        Self {
            m: Default::default(),
            xyz: Default::default(),
            klm: Default::default(),
            shepard_radius2: 0.25,
            step_length: 0.125,
        }
    }
}
impl GsOnAxisParams {
    /// Loads the parameters from a Numpy npz data file
    pub fn from_npz<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut archive = npyz::npz::NpzArchive::open(path)?;
        let mut gs_onaxis_params: GsOnAxisParams = Default::default();
        if let Ok(Some(data)) = archive.by_name("m") {
            let val: Vec<u8> = data.into_vec()?;
            gs_onaxis_params.m = val.into_iter().map(|x| x != 0).collect();
        }
        for k in 0..4 {
            if let Ok(Some(data)) = archive.by_name(&format!("xyz{k}")) {
                let mat = DMatrix::from_row_slice(
                    data.len() as usize / 3,
                    3_,
                    data.into_vec()?.as_slice(),
                );
                let rows: Vec<_> = mat
                    .row_iter()
                    .zip(&gs_onaxis_params.m)
                    .filter_map(|(row, &m)| m.then(|| row))
                    .collect();
                gs_onaxis_params.xyz.push(DMatrix::from_rows(&rows));
            }
            if let Ok(Some(data)) = archive.by_name(&format!("klm{k}")) {
                let mat = DMatrix::from_row_slice(
                    data.len() as usize / 3,
                    3_,
                    data.into_vec()?.as_slice(),
                );
                let rows: Vec<_> = mat
                    .row_iter()
                    .zip(&gs_onaxis_params.m)
                    .filter_map(|(row, &m)| m.then(|| row))
                    .collect();
                gs_onaxis_params.klm.push(DMatrix::from_rows(&rows));
            }
        }
        Ok(gs_onaxis_params)
    }
    pub fn shepard_radius(mut self, radius: f64) -> Self {
        self.shepard_radius2 = radius * radius;
        self
    }
    pub fn ray_tracing_step(mut self, step: f64) -> Self {
        self.step_length = step;
        self
    }
    /// Returns the number of OPD sample within the exit pupil
    pub fn n_sample(&self) -> usize {
        self.m.iter().filter(|x| **x).map(|_| 1).sum()
    }
    /// Ray traces through the GMT , returning the OPD
    ///
    /// Ray tracing step is set to 0.125m.
    /// CFD data is interpolated using Shepard interpolation within a 1m<sup><\sup> radius sphere
    pub fn ray_trace(&self, cfd_data: &RTree<TemperatureVelocityField>) -> Vec<f64> {
        #[cfg(feature = "linya")]
        let mut progress = linya::Progress::new();
        #[cfg(feature = "linya")]
        let bar: linya::Bar = progress.bar(1000, "Ray tracing");
        // Optical path length
        let mut opl = vec![0f64; self.n_sample()];
        for k in 0..3 {
            #[cfg(feature = "linya")]
            writeln!(progress.stderr(), "Ray trace #{}/3", k + 1)
                .expect("failed to wite to stderr");
            #[cfg(feature = "linya")]
            progress.set_and_draw(&bar, 0);
            //dbg!(k);
            // Getting the range to the next surface
            let mut delta_s = self.xyz[k + 1].column(2) - self.xyz[k].column(2);
            delta_s
                .iter_mut()
                .zip(self.klm[k].column(2).iter())
                .for_each(|(ds, &m)| *ds /= m);
            let max = delta_s.max();
            let n_h = (max / self.step_length).ceil() as usize;
            delta_s /= (n_h - 1) as f64; // Upsampling the range

            let mut xyz = self.xyz[k].clone();
            for _ in 0..n_h {
                #[cfg(feature = "linya")]
                progress.inc_and_draw(&bar, 1000 / n_h);
                // Ray tracing to the next layer: v = u + s_i k ()
                xyz.column_iter_mut()
                    .zip(self.klm[k].column_iter())
                    .for_each(|(mut u, k)| {
                        //u += k * &delta_s
                        u.iter_mut()
                            .zip(k.iter())
                            .zip(delta_s.iter())
                            .for_each(|((u, &k), &ds)| *u += k * ds);
                    });
                //let z = xyz.row(0)[2];
                //dbg!(z);

                // interpolating through CFD temperature field
                let delta_opl = xyz.row_iter().zip(delta_s.iter()).filter_map(|(row, &ds)| {
                    let xyz: Vec<f64> = row.iter().cloned().collect();
                    cfd_data
                        //.nearest_neighbor(&[xyz[0], xyz[1], xyz[2]])
                        //.map(|nn| nn.refraction_index() * ds)
                        .shepard(&[xyz[0], xyz[1], xyz[2]], self.shepard_radius2)
                        .map(|x| x * ds)
                });
                opl.iter_mut()
                    .zip(delta_opl)
                    .for_each(|(opl, d_opl)| *opl += d_opl);
            }
        }

        let mean_opl = opl.iter().cloned().sum::<f64>() / opl.len() as f64;
        //dbg!(mean_opl);
        let zeroed_opl = opl.into_iter().map(|x| x - mean_opl);
        let mut opd = vec![f64::NAN; self.m.len()];
        opd.iter_mut()
            .zip(&self.m)
            .filter_map(|(opd, &m)| m.then(|| opd))
            .zip(zeroed_opl)
            .for_each(|(opd, zopl)| *opd = zopl);
        opd
    }
}
