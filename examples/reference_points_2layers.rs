use std::env;
use std::error::Error;
use std::path::PathBuf;

use gnuplot::PlotOption::{Color, PointSymbol};
use gnuplot::{AxesCommon, Figure};
use nsga_rs::utils::{DasDarren1998, NumberOfPartitions, TwoLayerPartitions};

fn main() -> Result<(), Box<dyn Error>> {
    // Consider the case of a 3D hyperplane with 3 objectives
    let number_of_objectives = 3;
    let layers = TwoLayerPartitions {
        // In the first layer points have a gap of 5
        boundary_layer: 5,
        // In the second layer points have a gap of 3
        inner_layer: 4,
        scaling: None,
    };
    let partitions = NumberOfPartitions::TwoLayers(layers);
    let m = DasDarren1998::new(number_of_objectives, &partitions)?;
    // This returns the coordinates of the reference points between 0 and 1
    println!("Total points = {:?}", m.number_of_points());

    let weights = m.get_weights();
    println!("Weights = {:?}", weights);

    // Save the serialise data to inspect them
    let out_path = PathBuf::from(&env::current_dir().unwrap())
        .join("examples")
        .join("results")
        .join("ref_points_2layers_3obj_5gaps.json");
    DasDarren1998::serialise(&weights, &out_path)?;

    // Plot the points
    // group by objective
    let mut weights_by_objectives = vec![vec![]; number_of_objectives];
    for weight_point in weights.into_iter() {
        for obj_num in 0..number_of_objectives {
            weights_by_objectives[obj_num].push(weight_point[obj_num]);
        }
    }

    let mut fg = Figure::new();
    fg.axes3d()
        .points(
            &weights_by_objectives[0],
            &weights_by_objectives[1],
            &weights_by_objectives[2],
            &[PointSymbol('O'), Color("black".into())],
        )
        .set_x_label("Objective 1", &[])
        .set_y_label("Objective 2", &[])
        .set_z_label("Objective 3", &[])
        .set_x_grid(true)
        .set_y_grid(true)
        .set_z_grid(true)
        .set_x_range(gnuplot::AutoOption::Fix(0.0), gnuplot::AutoOption::Fix(1.0))
        .set_y_range(gnuplot::AutoOption::Fix(0.0), gnuplot::AutoOption::Fix(1.0))
        .set_z_range(gnuplot::AutoOption::Fix(0.0), gnuplot::AutoOption::Fix(1.0))
        .set_view(60.0, 110.0)
        .set_title(
            &format!(
                "Reference points - Das & Darren (2019)\n{} objectives",
                number_of_objectives
            ),
            &[],
        );

    fg.save_to_png(
        &out_path
            .parent()
            .unwrap()
            .join("ref_points_2layers_3obj_5gaps.png"),
        800,
        600,
    )?;

    Ok(())
}
