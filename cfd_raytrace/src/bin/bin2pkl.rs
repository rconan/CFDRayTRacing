use cfd_raytrace::Opd;
use std::{env, fs::File, path::Path};

fn main() -> anyhow::Result<()> {
    for arg in env::args().skip(1) {
        let path = Path::new(&arg);
        println!("{:?}", path);
        let opd: Opd = bincode::deserialize_from(File::open(path)?)?;
        let mut data = vec![f64::NAN; opd.mask.len()];
        let mut opd_iter = opd.values.iter();
        data.iter_mut()
            .zip(&opd.mask)
            .filter(|(_, m)| **m)
            .for_each(|(d, _)| *d = *opd_iter.next().unwrap());
        serde_pickle::to_writer(
            &mut File::create(path.with_extension("pkl"))?,
            &data,
            Default::default(),
        )?;
    }
    Ok(())
}
