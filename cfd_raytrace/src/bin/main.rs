use cfd_raytrace::{FromCompressedCsv, RayTracer, TemperatureVelocityField};
use rstar::RTree;
use std::{fs::File, time::Instant};

#[cfg(not(feature = "s3"))]
fn main() -> anyhow::Result<()> {
    let gs_onaxis_params = RayTracer::from_npz("data/gs_onaxis_params_512.u8.npz")?;
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz("data/optvol_optvol_3.000000e+02.csv.gz")?;
    let now = Instant::now();
    let opd = gs_onaxis_params.ray_trace(&tree);
    println!("OPD in {}s", now.elapsed().as_secs());

    serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;

    Ok(())
}

#[cfg(feature = "s3")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Downloading ray tracer ...");
    let now = Instant::now();

    let gs_onaxis_params = RayTracer::from_npz("gs_onaxis_params_512.u8.npz").await?;
    println!(" -> done in {}s", now.elapsed().as_secs());
    println!("Downloading CFD data ...");
    let now = Instant::now();
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz("CASES/zen60az180_OS7/optvol/optvol_optvol_3.000000e+02.csv.gz").await?;
    println!(" -> done in {}s", now.elapsed().as_secs());

    /*let (gs_onaxis_params, tree) = tokio::join!(
        do_stuff_async(),
        more_async_work());
    }*/

    let now = Instant::now();
    let opd = gs_onaxis_params.ray_trace(&tree);
    println!("OPD in {}s", now.elapsed().as_secs());

    serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;

    Ok(())
}
