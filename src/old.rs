use magick_rust::{magick_wand_genesis, MagickWand};
use regex::Regex;
use std::env;
use std::fmt::Error;
use std::process::{Command, Stdio};
use std::str;
use std::sync::Once;

static START: Once = Once::new();
static TMP_IMG: &str = "tmp";

#[derive(Clone)]
struct Vector {
    x: u16,
    y: u16,
}

#[derive(Clone)]
struct MonitorInfo {
    name: String,
    position: Vector,
    dimensions: Vector,
    rotation: u16,
}

#[derive(Clone)]
struct CanvasInfo {
    width: u16,
    height: u16,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let img_path = args.get(1);
    if img_path.is_none() {
        print!("No image path provided. exiting...");
        return;
    }

    let img_path_value = img_path.unwrap();
    let exists = std::path::Path::new(img_path_value).exists();

    if !exists {
        print!("Image does not exist!");
        return;
    }

    START.call_once(|| {
        magick_wand_genesis();
    });

    let (monitors, canvas, cut_resolution) = get_monitors();

    // set resampled image
    let wand = MagickWand::new();
    wand.read_image(img_path_value)
        .expect("Could not read image!");
    wand.resize_image(canvas.width.into(), canvas.height.into(), 8);

    let extension = &img_path_value
        .split('.')
        .last()
        .expect("Could not determine extension");

    wand.write_image(format!("{}.{}", TMP_IMG, extension).as_str())
        .expect("error while saving temp errror");

    for monitor in monitors {
        get_monitor_from_image(extension, &canvas, &monitor, &cut_resolution)
            .expect("could not get monitor");
    }
}

// returns monitors, canvas size and resolution used for cuts (largest resolution of image)
fn get_monitors() -> (Vec<MonitorInfo>, CanvasInfo, Vector) {
    let monitors = Command::new("hyprctl")
        .arg("monitors")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let grep = Command::new("grep")
        .arg("Monitor.*\n.*")
        .stdin(Stdio::from(monitors.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = grep.wait_with_output().unwrap();
    let res = str::from_utf8(&output.stdout).unwrap();

    let split = res.split("\n\n");

    let mut monitors: Vec<MonitorInfo> = Vec::new();

    for monitor in split {
        let lines: Vec<&str> = monitor.split('\n').collect();
        // print!("monitor: {}\n{}\n\n", lines[0], lines[1]);

        if lines[0].is_empty() {
            break;
        }

        let split_on_space: Vec<&str> = lines[0].split(' ').collect();
        let name = split_on_space[1];

        let monitor_info = get_monitor_info(name, lines[1], lines[9]);

        monitors.push(monitor_info);
    }

    let mut canvas_info = CanvasInfo {
        width: 0,
        height: 0,
    };

    let mut largest_monitor = Vector { x: 0, y: 0 };

    for monitor in monitors.clone() {
        if monitor.dimensions.x > largest_monitor.x && monitor.dimensions.y > largest_monitor.y {
            largest_monitor = monitor.dimensions.clone();
        }
    }

    for monitor in monitors.clone() {
        if monitor.rotation == 270 || monitor.rotation == 90 {
            if monitor.position.x + largest_monitor.y > canvas_info.width {
                canvas_info.width = monitor.position.x + largest_monitor.y;
            }

            if monitor.position.y + largest_monitor.x > canvas_info.height {
                canvas_info.height = monitor.position.y + largest_monitor.x;
            }
        } else {
            if monitor.position.x + largest_monitor.x > canvas_info.width {
                canvas_info.width = monitor.position.x + largest_monitor.x;
            }

            if monitor.position.y + largest_monitor.y > canvas_info.height {
                canvas_info.height = monitor.position.y + largest_monitor.y;
            }
        }
    }

    (monitors, canvas_info, largest_monitor)
}

fn get_monitor_info(name: &str, line: &str, transform: &str) -> MonitorInfo {
    let split: Vec<&str> = line.split(" at ").collect();

    let dimensions_string_raw = split[0].trim();
    let re = Regex::new(r"@.*").unwrap();
    let dimensions_string = re.replace_all(dimensions_string_raw, "");
    let split_dimensions_string: Vec<&str> = dimensions_string.split('x').collect();

    let width = split_dimensions_string[0].parse::<u16>().unwrap();
    let height = split_dimensions_string[1].parse::<u16>().unwrap();

    let dimensions = Vector {
        x: width,
        y: height,
    };

    let position_string: Vec<&str> = split[1].trim().split('x').collect();
    let x = position_string[0].parse::<u16>().unwrap();
    let y = position_string[1].parse::<u16>().unwrap();

    let position = Vector { x, y };

    let transform_value = transform.trim().split(':').collect::<Vec<&str>>()[1].trim();

    let rotation = match transform_value {
        "0" => 0,
        "1" => 90,
        "2" => 180,
        "3" => 270,
        "4" => 0,
        "5" => 90,
        "6" => 180,
        "7" => 270,
        _ => 0,
    };

    MonitorInfo {
        name: name.to_owned(),
        dimensions,
        position,
        rotation,
    }
}

fn get_monitor_from_image(
    extension: &str,
    canvas: &CanvasInfo,
    monitor: &MonitorInfo,
    cut_res: &Vector,
) -> Result<(), Error> {
    println!("canvas: {}x{}", canvas.width, canvas.height);
    let wand = MagickWand::new();
    // read temp value
    wand.read_image(format!("{}.{}", TMP_IMG, extension).as_str())
        .expect("Could not read image!");
    wand.resample_image(canvas.width.into(), canvas.height.into(), 1);

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
        monitor.position.x.try_into().ok().unwrap(),
        monitor.position.y.try_into().ok().unwrap(),
    ) {
        Ok(_) => match wand.write_image(format!("{}.{}", &monitor.name, extension).as_str()) {
            Ok(_) => Ok(()),
            Err(err) => panic!("panicked while saving: {}", err),
        },
        Err(err) => {
            panic!("{}", err)
        }
    }
}
