use cfd_raytrace::{FromCompressedCsv, Shepard, TemperatureVelocityField};
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
        println!("{:?} ({})", data, data.refraction_index());
    }
    println!("nearest neighbor: {}mus", now.elapsed().as_micros());

    let now = Instant::now();
    if let Some(data) = tree.nearest_neighbor(&[10.46875, -2.02853888946131, 53.6739675783237]) {
        println!("{:?} ({})", data, data.refraction_index());
    }
    println!("nearest neighbor: {}mus", now.elapsed().as_micros());

    let now = Instant::now();
    if let Some(data) = tree.shepard(&[0f64, 0f64, 18f64], 2.25) {
        println!("{:?}", data);
    }
    println!("shepard: {}mus", now.elapsed().as_micros());

    let now = Instant::now();
    if let Some(data) = tree.shepard(&[10.46875, -2.02853888946131, 53.6739675783237], 2.25) {
        println!("{:?}", data);
    }
    println!("shepard: {}mus", now.elapsed().as_micros());

    /*     let x = 0f64;
       let y = 0f64;
       let z = 18f64;
       let h = 1.;
       let square = AABB::from_corners([x - h, y - h, z - h], [x + h, y + h, z + h]);
       let elements_in_square = tree.locate_in_envelope(&square);
       elements_in_square
           .enumerate()
           .for_each(|e| println!("{e:?}"));

       let elements_within_distance2 = tree.locate_within_distance([x, y, z], h * h);
       elements_within_distance2
           .enumerate()
           .for_each(|e| println!("{e:?}"));
    */
    Ok(())
}
