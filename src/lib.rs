#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("infostring requires at least one of the 'std' and 'alloc' features to be enabled.");

pub mod de;
pub mod error;
pub mod ser;

pub use de::{Deserializer, from_str};
pub use error::{Error, Result};
pub use ser::{Serializer, to_string};

#[cfg(test)]
mod tests {

    #[cfg(not(feature = "std"))]
    use super::alloc::{
        string::{String, ToString},
        vec,
    };

    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct PlayerInfo {
        name: String,
        ping: i32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct ServerInfo {
        hostname: String,
        max_players: i32,
    }

    #[test]
    fn test_serialize_flattened_tuple() {
        // Tuples naturally shed their container shape and merge data sequences
        let data = (
            PlayerInfo {
                name: "Doomguy".to_string(),
                ping: 15,
            },
            ServerInfo {
                hostname: "DM6".to_string(),
                max_players: 16,
            },
        );

        let result = to_string(&data).unwrap();
        assert_eq!(
            result,
            "\\name\\Doomguy\\ping\\15\\hostname\\DM6\\max_players\\16"
        );
    }

    #[test]
    fn test_serialize_flattened_seq() {
        // Sequences output field-by-field, endlessly chained
        let frags = vec![
            PlayerInfo {
                name: "Ranger".to_string(),
                ping: 20,
            },
            PlayerInfo {
                name: "Visor".to_string(),
                ping: 25,
            },
        ];

        let result = to_string(&frags).unwrap();
        assert_eq!(result, "\\name\\Ranger\\ping\\20\\name\\Visor\\ping\\25");
    }

    #[test]
    fn test_serialize_tuple_struct() {
        #[derive(Serialize)]
        struct Entity(PlayerInfo, ServerInfo);

        let entity = Entity(
            PlayerInfo {
                name: "Bitterman".to_string(),
                ping: 10,
            },
            ServerInfo {
                hostname: "Q2DM1".to_string(),
                max_players: 8,
            },
        );

        let result = to_string(&entity).unwrap();
        assert_eq!(
            result,
            "\\name\\Bitterman\\ping\\10\\hostname\\Q2DM1\\max_players\\8"
        );
    }
}
