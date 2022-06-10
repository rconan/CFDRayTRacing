use cfd_raytrace::{FromCompressedCsv, RayTracer, TemperatureVelocityField};
use rstar::RTree;
use std::time::Instant;

#[cfg(not(feature = "s3"))]
fn main() -> anyhow::Result<()> {
    let gs_onaxis_params =
        RayTracer::from_npz("data/gs_onaxis_params_512.u8.npz")?.ray_tracing_step(0.5);
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz("data/optvol_optvol_3.000000e+02.csv.gz")?;
    let now = Instant::now();
    let opd = gs_onaxis_params.ray_trace(&tree);
    println!("OPD in {}s", now.elapsed().as_secs());

    // serde_pickle::to_writer(&mut File::create("data/opd.pkl")?, &opd, Default::default())?;
    bincode::serialize_into(&mut std::fs::File::create("data/opd.bin")?, &opd)?;

    Ok(())
}

#[cfg(feature = "s3")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use s3::bucket::Bucket;
    use s3::creds::Credentials;
    use std::{env, path::Path};

    let cfd_case = env::var("CFD_CASE").expect("`CFD_CASE` environment variable is not set");
    let file_id = env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .expect("`AWS_BATCH_JOB_ARRAY_INDEX` environment variable is not set")
        .parse::<usize>()
        .expect("failed to convert `AWS_BATCH_JOB_ARRAY_INDEX` into usize");

    let bucket_name = "gmto.cfd.2022";
    let region = "us-east-2".parse()?;
    let credentials = Credentials::default()?;
    let bucket = Bucket::new(bucket_name, region, credentials)?;
    let results = bucket
        .list(format!("CASES/{}/optvol/optvol_optvol", cfd_case), None)
        .await?;
    let key = results
        .into_iter()
        .flat_map(|res| {
            res.contents
                .into_iter()
                .map(|object| object.key)
                .filter(|key| key.ends_with(".csv.gz"))
        })
        .nth(file_id)
        .expect(&format!("failed to get key #{}", file_id));
    println!("key: {}", key);

    println!("Downloading ray tracer ...");
    let now = Instant::now();
    let gs_onaxis_params = RayTracer::from_npz("gs_onaxis_params_769.u8.npz").await?;
    println!(" -> done in {}s", now.elapsed().as_secs());
    println!("Downloading CFD data ...");
    let now = Instant::now();
    let key = Path::new(&key);
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
        let bucket_name = "gmto.im.grim";
        let region = "us-west-2".parse()?;
        let credentials = Credentials::default()?;
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
