use std::{process::{Command, Stdio}, fmt::Error, str};
use regex::Regex;

use crate::domain::{MonitorInfo, Vector};


pub fn get_monitors() -> Result<Vec<MonitorInfo>, Error> {
    let monitors = Command::new("hyprctl")
        .arg("monitors")
        .stdout(Stdio::piped())
        .spawn()
        .expect("'hyprctl monitors' could not be executed");

    let grep = Command::new("grep")
        .arg("Monitor.*\n.*")
        .stdin(Stdio::from(monitors.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .expect("'grep' returned error");

    let output = grep.wait_with_output().expect("output returned error");
    let res = str::from_utf8(&output.stdout).expect("response returned error");

    let split = res.split("\n\n");

    let mut monitors: Vec<MonitorInfo> = Vec::new();

    for monitor in split {
        let lines: Vec<&str> = monitor.split('\n').collect();

        if lines[0].is_empty() {
            break;
        }

        let split_on_space: Vec<&str> = lines[0].split(' ').collect();
        let name = split_on_space[1];

        let monitor_info = get_monitor_info(name, lines[1], lines[9]).expect("get monitor info returned error");

        monitors.push(monitor_info);
    }

    Ok(monitors)
}

fn get_monitor_info(name: &str, line: &str, transform: &str) -> Result<MonitorInfo, Error> {
    let split: Vec<&str> = line.split(" at ").collect();

    let dimensions_string_raw = split[0].trim();
    let re = Regex::new(r"@.*").expect("regex could not be made");
    let dimensions_string = re.replace_all(dimensions_string_raw, "");
    let split_dimensions_string: Vec<&str> = dimensions_string.split('x').collect();

    let width = split_dimensions_string[0].parse::<u16>().unwrap();
    let height = split_dimensions_string[1].parse::<u16>().unwrap();

    let dimensions = Vector {
        x: width,
        y: height,
    };

    let position_string: Vec<&str> = split[1].trim().split('x').collect();
    let x = position_string[0].parse::<u16>().expect("parsing of position x failed");
    let y = position_string[1].parse::<u16>().expect("parsing of position x failed");

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

    Ok(MonitorInfo {
        name: name.to_owned(),
        position,
        rotation,
        dimensions,
    })
}