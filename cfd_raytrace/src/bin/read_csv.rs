use cfd_raytrace::{FromCompressedCsv, TemperatureVelocityField};
use rstar::RTree;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let now = Instant::now();
    let tree: RTree<TemperatureVelocityField> =
        RTree::from_gz("data/OPDData_OPD_Data_1.400028e+03.csv.gz")?;
    println!("RTree: {}ms", now.elapsed().as_millis());

    let now = Instant::now();
    if let Some(data) = tree.nearest_neighbor(&[0f64; 3]) {
        println!("{:?}", data);
    }
    println!("nearest neighbor: {}mus", now.elapsed().as_micros());

    let now = Instant::now();
    if let Some(data) = tree.nearest_neighbor(&[0f64, 0f64, 18f64]) {
        println!("{:?}", data);
    }
    println!("nearest neighbor: {}mus", now.elapsed().as_micros());

    let now = Instant::now();
    if let Some(data) = tree.nearest_neighbor(&[10.46875, -2.02853888946131, 53.6739675783237]) {
        println!("{:?}", data);
    }
    println!("nearest neighbor: {}mus", now.elapsed().as_micros());

    Ok(())
}
