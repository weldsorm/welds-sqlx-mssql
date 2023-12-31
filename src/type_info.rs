use std::fmt::{self, Display, Formatter};

use crate::protocol::type_info::{DataType, TypeInfo as ProtocolTypeInfo};
use sqlx_core::type_info::TypeInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlTypeInfo(pub(crate) ProtocolTypeInfo);

impl TypeInfo for MssqlTypeInfo {
    fn is_null(&self) -> bool {
        matches!(self.0.ty, DataType::Null)
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

impl Display for MssqlTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}
