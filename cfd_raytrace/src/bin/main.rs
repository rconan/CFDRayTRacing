use cfd_raytrace::{FromCompressedCsv, GsOnAxisParams, TemperatureVelocityField};
use rstar::RTree;
use std::{fs::File, time::Instant};

fn main() -> anyhow::Result<()> {
    let gs_onaxis_params = GsOnAxisParams::from_npz("data/gs_onaxis_params_512.u8.npz")?;
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz("data/OPDData_OPD_Data_1.400028e+03.csv.gz")?;
    let now = Instant::now();
    let opd = gs_onaxis_params.ray_trace(&tree);
    println!("OPD in {}s", now.elapsed().as_secs());

    serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;

    Ok(())
}
