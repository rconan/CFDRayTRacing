use cfd_raytrace::RayTracer;

fn main() -> anyhow::Result<()> {
    let mut archive = npyz::npz::NpzArchive::open("data/gs_onaxis_params_512.u8.npz")?;
    let names: Vec<_> = archive.array_names().map(|x| x.to_owned()).collect();
    for name in names.into_iter() {
        print!("{} :", &name);
        if let Ok(Some(data)) = archive.by_name(&name) {
            println!("{:?} {:?}", data.shape(), data.dtype());
        }
    }

    let gs_onaxis_params = RayTracer::from_npz("data/gs_onaxis_params_512.u8.npz")?;
    gs_onaxis_params.xyz[0]
        .row_iter()
        .take(3)
        .for_each(|row| println!("{}", row));
    gs_onaxis_params.klm[2]
        .row_iter()
        .take(3)
        .for_each(|row| println!("{}", row));
    Ok(())
}
