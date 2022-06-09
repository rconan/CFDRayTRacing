use std::{fs::File, path::Path};

fn main() -> anyhow::Result<()> {
    let path = Path::new("data/optvol_optvol_3.000000e+02.bin");
    let data: Vec<f64> = bincode::deserialize_from(File::open(path)?)?;
    serde_pickle::to_writer(
        &mut File::create(path.with_extension("pkl"))?,
        &data,
        Default::default(),
    )?;
    Ok(())
}
