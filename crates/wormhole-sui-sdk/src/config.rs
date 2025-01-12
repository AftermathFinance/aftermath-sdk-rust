use af_sui_types::{ObjectId, TypeTag};
use serde::{Deserialize, Serialize};

use crate::vaa::VAA;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub package: ObjectId,
    pub state: ObjectId,
}

impl Config {
    pub fn vaa_type_tag(&self) -> TypeTag {
        VAA::type_(self.package.into()).into()
    }
}
