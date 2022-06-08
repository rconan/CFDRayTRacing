use super::{Result, TemperatureVelocityField};
use nalgebra::DMatrix;
use rstar::RTree;
use std::path::Path;

/// Ray tracing parameters
#[derive(Default)]
pub struct GsOnAxisParams {
    m: Vec<bool>,
    pub xyz: Vec<DMatrix<f64>>,
    pub klm: Vec<DMatrix<f64>>,
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
    /// Returns the number of OPD sample within the exit pupil
    pub fn n_sample(&self) -> usize {
        self.m.iter().filter(|x| **x).map(|_| 1).sum()
    }
    /// Ray traces through the GMT , returning the OPD
    pub fn ray_trace(&self, n_h: usize, cfd_data: &RTree<TemperatureVelocityField>) -> Vec<f64> {
        // Optical path length
        let mut opl = vec![0f64; self.n_sample()];
        for k in 0..3 {
            dbg!(k);
            // Getting the range to the next surface
            let mut delta_s = self.xyz[k + 1].column(2) - self.xyz[k].column(2);
            delta_s
                .iter_mut()
                .zip(self.klm[k].column(2).iter())
                .for_each(|(ds, &m)| *ds /= m);
            delta_s /= (n_h - 1) as f64; // Upsampling the range

            let mut xyz = self.xyz[k].clone();

            for i in 0..n_h {
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
                // interpolating through CFD temperature field
                let delta_opl = xyz.row_iter().zip(delta_s.iter()).filter_map(|(row, &ds)| {
                    let xyz: Vec<f64> = row.iter().cloned().collect();
                    cfd_data
                        .nearest_neighbor(&[xyz[0], xyz[1], xyz[2]])
                        .map(|nn| nn.refraction_index() * ds)
                });
                opl.iter_mut()
                    .zip(delta_opl)
                    .for_each(|(opl, d_opl)| *opl += d_opl);
            }
        }

        let mean_opl = opl.iter().cloned().sum::<f64>() / opl.len() as f64;
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
