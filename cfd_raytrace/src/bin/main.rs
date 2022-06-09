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

    // serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;
    bincode::serialize_into(&mut File::create("data/opd.bin")?, &opd)?;

    Ok(())
}

#[cfg(feature = "s3")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::path::Path;

    println!("Downloading ray tracer ...");
    let now = Instant::now();

    let gs_onaxis_params = RayTracer::from_npz("gs_onaxis_params_512.u8.npz").await?;
    println!(" -> done in {}s", now.elapsed().as_secs());
    println!("Downloading CFD data ...");
    let now = Instant::now();
    let key = Path::new("CASES/zen60az180_OS7/optvol/optvol_optvol_3.000000e+02.csv.gz");
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz(key.to_str().expect("failed to convert path to str")).await?;
    println!(" -> done in {}s", now.elapsed().as_secs());

    /*let (gs_onaxis_params, tree) = tokio::join!(
        do_stuff_async(),
        more_async_work());
    }*/
    let now = Instant::now();
    let opd = gs_onaxis_params.ray_trace(&tree);
    println!("OPD in {}s", now.elapsed().as_secs());

    println!("Uploading OPD ...");
    let now = Instant::now();

    let bucket = {
        use s3::bucket::Bucket;
        use s3::creds::Credentials;
        let bucket_name = "gmto.im.grim";
        let region = "us-west-2".parse()?;
        let credentials = Credentials::default().map_err(|e| s3::error::S3Error::Credentials(e))?;
        Bucket::new(bucket_name, region, credentials)?
    };

    let stream = bincode::serialize(&opd)?;
    bucket
        .put_object(
            key.with_extension("")
                .with_extension("bin")
                .to_str()
                .expect("failed to convert path to str"),
            &stream,
        )
        .await?;
    println!(" -> done in {}s", now.elapsed().as_secs());

    //serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;

    Ok(())
}
