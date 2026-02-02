use console::style;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use vacs_data_diagnostics::log;
use vacs_protocol::vatsim::{PositionId, StationId};
use vacs_vatsim::coverage::position::PositionConfigFile;
use vacs_vatsim::coverage::station::{StationConfigFile, StationRaw};
use vacs_vatsim::{FacilityType, coverage};

#[derive(Deserialize)]
pub struct VatglassesData {
    pub airspace: Vec<Airspace>,
    pub positions: HashMap<String, Position>,
}

impl std::fmt::Debug for VatglassesData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VatglassesData")
            .field("airspace", &self.airspace.len())
            .field("positions", &self.positions.len())
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct Airspace {
    id: String,
    group: String,
    owner: Vec<String>,
}

#[derive(Deserialize)]
pub struct Position {
    pre: Vec<String>,
    r#type: String,
    frequency: Option<String>,
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("pre", &self.pre.len())
            .field("type", &self.r#type)
            .field("frequency", &self.frequency)
            .finish()
    }
}

pub trait TryFromRef<T: ?Sized>: Sized {
    type Error;
    fn try_from_ref(value: &T) -> Result<Self, Self::Error>;
}

impl TryFromRef<VatglassesData> for PositionConfigFile {
    type Error = String;
    fn try_from_ref(value: &VatglassesData) -> Result<Self, Self::Error> {
        Ok(Self {
            positions: value
                .positions
                .iter()
                .map(|(id, p)| coverage::position::PositionRaw {
                    id: PositionId::from(id.clone()),
                    facility_type: p.r#type.parse().unwrap(), // TODO handle error
                    frequency: p.frequency.clone().unwrap_or("199.998".to_string()),
                    prefixes: p.pre.iter().cloned().collect(),
                    profile_id: None,
                })
                .collect(),
        })
    }
}

impl TryFromRef<VatglassesData> for StationConfigFile {
    type Error = String;
    fn try_from_ref(value: &VatglassesData) -> Result<Self, Self::Error> {
        let mut seen = HashSet::new();
        let mut duplicates = 0;

        let stations = value
            .airspace
            .iter()
            .map(|a| {
                let facility_type = FacilityType::from(a.group.clone());

                if !seen.insert((a.id.clone(), facility_type)) {
                    log::warn(format_args!(
                        "Duplicate airspace ID {} ({})",
                        style(format!("`{}`", a.id)).cyan(),
                        facility_type.as_str()
                    ));
                    duplicates += 1;
                }

                StationRaw {
                    id: StationId::from(a.id.clone()),
                    parent_id: None,
                    controlled_by: a
                        .owner
                        .iter()
                        .map(|o| PositionId::from(o.clone()))
                        .collect(),
                }
            })
            .collect();

        Ok(Self { stations })
    }
}
