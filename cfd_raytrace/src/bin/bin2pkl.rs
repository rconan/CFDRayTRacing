use std::{env, fs::File, path::Path};

fn main() -> anyhow::Result<()> {
    for arg in env::args().skip(1) {
        let path = Path::new(&arg);
        println!("{:?}", path);
        let data: Vec<f64> = bincode::deserialize_from(File::open(path)?)?;
        serde_pickle::to_writer(
            &mut File::create(path.with_extension("pkl"))?,
            &data,
            Default::default(),
        )?;
    }
    Ok(())
}
