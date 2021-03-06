// TODO - remove once schemars stops causing warning.
#![allow(clippy::field_reassign_with_default)]

use std::fmt::{self, Display, Formatter};

use datasize::DataSize;
#[cfg(test)]
use rand::Rng;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

use casper_types::bytesrepr::{self, FromBytes, ToBytes};

use super::GlobalStateUpdate;

use crate::components::consensus::EraId;
#[cfg(test)]
use crate::testing::TestRng;

/// The first era to which the upgrade applies.
#[derive(Copy, Clone, DataSize, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
pub struct ActivationPoint {
    pub(crate) era_id: EraId,
}

impl ActivationPoint {
    /// Returns whether we should upgrade the node due to the next era being at or after the upgrade
    /// activation point.
    pub(crate) fn should_upgrade(&self, era_being_deactivated: &EraId) -> bool {
        era_being_deactivated.0 + 1 >= self.era_id.0
    }
}

impl Display for ActivationPoint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "upgrade activation point {}", self.era_id)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, DataSize, Debug)]
pub struct ProtocolConfig {
    #[data_size(skip)]
    pub(crate) version: Version,
    /// Whether we need to clear latest blocks back to the switch block just before the activation
    /// point or not.
    pub(crate) hard_reset: bool,
    /// This protocol config applies starting at the era specified in the activation point.
    pub(crate) activation_point: ActivationPoint,
    /// Any arbitrary updates we might want to make to the global state at the start of the era
    /// specified in the activation point.
    pub(crate) global_state_update: Option<GlobalStateUpdate>,
}

#[cfg(test)]
impl ProtocolConfig {
    /// Generates a random instance using a `TestRng`.
    pub fn random(rng: &mut TestRng) -> Self {
        let protocol_version = Version::new(
            rng.gen_range(0, 10),
            rng.gen::<u8>() as u64,
            rng.gen::<u8>() as u64,
        );
        let activation_point = ActivationPoint {
            era_id: EraId(rng.gen::<u8>() as u64),
        };

        ProtocolConfig {
            version: protocol_version,
            hard_reset: rng.gen(),
            activation_point,
            global_state_update: None,
        }
    }
}

impl ToBytes for ProtocolConfig {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        buffer.extend(self.version.to_string().to_bytes()?);
        buffer.extend(self.hard_reset.to_bytes()?);
        buffer.extend(self.activation_point.era_id.to_bytes()?);
        buffer.extend(self.global_state_update.to_bytes()?);
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        self.version.to_string().serialized_length()
            + self.hard_reset.serialized_length()
            + self.activation_point.era_id.serialized_length()
            + self.global_state_update.serialized_length()
    }
}

impl FromBytes for ProtocolConfig {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (protocol_version_string, remainder) = String::from_bytes(bytes)?;
        let protocol_version =
            Version::parse(&protocol_version_string).map_err(|_| bytesrepr::Error::Formatting)?;
        let (hard_reset, remainder) = bool::from_bytes(remainder)?;
        let (era_id, remainder) = EraId::from_bytes(remainder)?;
        let activation_point = ActivationPoint { era_id };
        let (global_state_update, remainder) = Option::<GlobalStateUpdate>::from_bytes(remainder)?;
        let protocol_config = ProtocolConfig {
            version: protocol_version,
            activation_point,
            global_state_update,
            hard_reset,
        };
        Ok((protocol_config, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_config_bytesrepr_roundtrip() {
        let mut rng = crate::new_rng();
        let config = ProtocolConfig::random(&mut rng);
        bytesrepr::test_serialization_roundtrip(&config);
    }

    #[test]
    fn toml_roundtrip() {
        let mut rng = crate::new_rng();
        let config = ProtocolConfig::random(&mut rng);
        let encoded = toml::to_string_pretty(&config).unwrap();
        let decoded = toml::from_str(&encoded).unwrap();
        assert_eq!(config, decoded);
    }
}
