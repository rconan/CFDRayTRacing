use cfd_raytrace::RayTracer;
use flate2::read::GzDecoder;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::io::Cursor;
use std::io::Read;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bucket_name = "gmto.cfd.2022";
    let region = "us-east-2".parse()?;
    let credentials = Credentials::default()?;
    let bucket = Bucket::new(bucket_name, region, credentials)?;

    let (data, _) = bucket
        .get_object("CASES/zen60az180_OS7/optvol/optvol_optvol_3.000000e+02.csv.gz")
        .await?;
    let stream = std::io::Cursor::new(data);
    let mut decoder = GzDecoder::new(stream);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();

    /*     let stream = Cursor::new(data);

       let mut archive = npyz::npz::NpzArchive::new(stream)?;
       let names: Vec<_> = archive.array_names().map(|x| x.to_owned()).collect();
       for name in names.into_iter() {
           print!("{} :", &name);
           if let Ok(Some(data)) = archive.by_name(&name) {
               println!("{:?} {:?}", data.shape(), data.dtype());
           }
       }

       let gs_onaxis_params = RayTracer::from_npz("gs_onaxis_params_512.u8.npz").await?;
       gs_onaxis_params.xyz[0]
           .row_iter()
           .take(3)
           .for_each(|row| println!("{}", row));
       gs_onaxis_params.klm[2]
           .row_iter()
           .take(3)
           .for_each(|row| println!("{}", row));
    */
    Ok(())
}
