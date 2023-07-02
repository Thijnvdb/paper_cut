use std::{sync::Once, env};
use domain::{MonitorInfo, Vector};
use magick_rust::{magick_wand_genesis, MagickWand};

mod hyprland_info;
mod domain;

static START: Once = Once::new();

fn main() {
    let image_path = parse_args();
    println!("using image: {}", image_path);
    let image_extension = get_extension(&image_path);

    let monitors = hyprland_info::get_monitors().expect("monitors could not be retrieved");
    let (largest_monitor_dimensions, _largest_monitor_ratio) = get_largest_monitor_dimensions(&monitors);
    let canvas = get_bounding_box(&monitors);

    START.call_once(|| {
        magick_wand_genesis();
    });

    // let wand = MagickWand::new();
    // wand.read_image(&image_path)
    //     .expect(format!("Could not read image! {}", image_path).as_str());

    for monitor in monitors {
        cut_from_image(monitor, canvas.clone(), largest_monitor_dimensions.clone(), &image_path, &image_extension).expect("could not cut image!");
    }
}

fn get_extension(image_path: &str) -> String {
    image_path
        .split('.')
        .last()
        .expect("Could not determine extension")
        .into()
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();

    let img_path = args.get(1);
    if img_path.is_none() {
        panic!("image path was empty!");
    }

    let img_path_value = img_path.unwrap();
    let exists = std::path::Path::new(img_path_value).exists();

    if !exists {
        panic!("image path was not a valid path!");
    }

    img_path_value.into()
}

// get bounding box as ratio values
fn get_bounding_box(monitors: &Vec<MonitorInfo>) -> Vector {
    let mut info = Vector { x: 0, y: 0 };

    for monitor in monitors {
        let mut x_extent = monitor.position.x + monitor.dimensions.x;
        let mut y_extent = monitor.position.y + monitor.dimensions.y;

        if monitor.rotation == 270 || monitor.rotation == 90 {
            x_extent = monitor.position.x + monitor.dimensions.y;
            y_extent = monitor.position.y + monitor.dimensions.x;
        }

        if x_extent > info.x {
            info.x = x_extent;
        }

        if y_extent> info.y {
            info.y = y_extent;
        }
    }

    info
}

// get size of largest monitor for converting from ratios to screen space
// based on x value
fn get_largest_monitor_dimensions(monitors: &Vec<MonitorInfo>) -> (Vector, f32) {
    let mut largest_dimensions: Vector = Vector { x: 0, y: 0 };
    let mut largest_ratio: f32 = 0.0;

    for monitor in monitors {
        let x_dimension: u16;
        if monitor.rotation == 270 || monitor.rotation == 90 {
            x_dimension = monitor.dimensions.y;
        } else {
            x_dimension = monitor.dimensions.x;
        }

        if x_dimension > largest_dimensions.x {
            largest_dimensions = monitor.dimensions.clone();
            largest_ratio = monitor.screen_ratio.clone();
        }
    }

    (largest_dimensions, largest_ratio)
}

fn cut_from_image(monitor: MonitorInfo, canvas: Vector, cut_res: Vector, image_path: &str, extension: &str) -> Result<(),()> {
    let wand = MagickWand::new();
    wand.read_image(image_path).expect("Could not read image!");
    wand.adaptive_resize_image(canvas.x.into(), canvas.y.into()).expect("resize failed");

    let mut width = cut_res.x;
    let mut height = cut_res.y;
    if monitor.rotation == 90 || monitor.rotation == 270 {
        height = width;
        width = cut_res.y;
    }

    println!(
        "using cut res {}x{} for {} (rotation {}, position {}, {})",
        width, height, monitor.name, monitor.rotation, monitor.position.x, monitor.position.y
    );

    match wand.crop_image(
        width.into(),
        height.into(),
        monitor.position.x.try_into().ok().expect("monitor x not ok"),
        monitor.position.y.try_into().ok().expect("monitor y not ok"),
    ) {
        Ok(_) => match wand.write_image(format!("{}.{}", &monitor.name, extension).as_str()) {
            Ok(_) => Ok(()),
            Err(err) => panic!("panicked while saving: {}", err),
        },
        Err(err) => panic!("{}", err)
    }
}
