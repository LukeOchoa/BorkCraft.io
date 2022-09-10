mod borkcraft;
mod errors;
mod login;
mod pages;

pub use borkcraft::*;
pub use pages::nether_portals::*;

use serde::Serialize;
pub fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(cereal).unwrap()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn
// }
