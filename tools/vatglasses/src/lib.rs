pub mod types;

use crate::types::{TryFromRef, VatglassesData};
use std::path::PathBuf;
use vacs_data_diagnostics::log;
use vacs_vatsim::coverage::position::PositionConfigFile;
use vacs_vatsim::coverage::station::StationConfigFile;

pub fn parse(
    input: &PathBuf,
    output: &PathBuf,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info(format_args!(
        "Parsing VATglasses data from {:?} to {:?}",
        input, output
    ));

    if !input.exists() {
        log::error(format_args!("Input file {:?} does not exist", input));
        return Err("Input file does not exist".into());
    }

    if output.exists() {
        if !output.is_dir() {
            log::error(format_args!("Output {:?} is not a directory", output));
            return Err("Output is not a directory".into());
        }
    } else if let Err(err) = std::fs::create_dir_all(output) {
        log::error(format_args!(
            "Failed to create output directory {:?}: {:?}",
            output, err
        ));
        return Err(err.into());
    }

    let output_stations = output.join("stations.toml");
    if output_stations.exists() {
        if overwrite {
            log::warn(format_args!(
                "Overwriting existing stations output file: {:?}",
                output_stations
            ));
        } else {
            log::error(format_args!(
                "Stations output file {:?} already exists",
                output_stations
            ));
            return Err("Stations output file already exists".into());
        }
    }

    let output_positions = output.join("positions.toml");
    if output_positions.exists() {
        if overwrite {
            log::warn(format_args!(
                "Overwriting existing positions output file: {:?}",
                output_positions
            ));
        } else {
            log::error(format_args!(
                "Positions output file {:?} already exists",
                output_positions
            ));
            return Err("Positions output file already exists".into());
        }
    }

    let file = match std::fs::File::open(input) {
        Ok(f) => f,
        Err(err) => {
            log::error(format_args!(
                "Failed to open input file {:?}: {:?}",
                input, err
            ));
            return Err(err.into());
        }
    };

    let data: VatglassesData = match serde_json::from_reader(file) {
        Ok(d) => d,
        Err(err) => {
            log::error(format_args!(
                "Failed to parse input file {:?}: {:?}",
                input, err
            ));
            return Err(err.into());
        }
    };

    log::info(format_args!("Parsed VATglasses data: {:?}", data));

    let mut stations = match StationConfigFile::try_from_ref(&data) {
        Ok(s) => s,
        Err(err) => {
            log::error(format_args!(
                "Failed to convert VATglasses data to stations: {:?}",
                err
            ));
            return Err(err.into());
        }
    };

    stations.stations.sort_by(|a, b| a.id.cmp(&b.id));

    let serialized_stations = match toml::to_string_pretty(&stations) {
        Ok(s) => s,
        Err(err) => {
            log::error(format_args!("Failed to serialize stations: {:?}", err));
            return Err(err.into());
        }
    };

    if let Err(err) = std::fs::write(&output_stations, serialized_stations) {
        log::error(format_args!(
            "Failed to write stations output file {:?}: {:?}",
            output_stations, err
        ));
        return Err(err.into());
    }

    let mut positions = match PositionConfigFile::try_from_ref(&data) {
        Ok(p) => p,
        Err(err) => {
            log::error(format_args!(
                "Failed to convert VATglasses data to positions: {:?}",
                err
            ));
            return Err(err.into());
        }
    };

    positions.positions.sort_by(|a, b| {
        a.facility_type
            .cmp(&b.facility_type)
            .reverse()
            .then_with(|| a.id.cmp(&b.id))
    });

    let serialized_positions = match toml::to_string_pretty(&positions) {
        Ok(s) => s,
        Err(err) => {
            log::error(format_args!("Failed to serialize positions: {:?}", err));
            return Err(err.into());
        }
    };

    if let Err(err) = std::fs::write(&output_positions, serialized_positions) {
        log::error(format_args!(
            "Failed to write positions output file {:?}: {:?}",
            output_positions, err
        ));
        return Err(err.into());
    }

    log::info(format_args!("Wrote output files to {:?}", output));
    Ok(())
}
