use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::protocol::type_info::{DataType, TypeInfo};
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};
use sqlx_core::types::Type;

impl Type<Mssql> for bool {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::BitN, 1))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Bit | DataType::BitN)
    }
}

impl Encode<'_, Mssql> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(if *self { 1 } else { 0 });

        IsNull::No
    }
}

impl Decode<'_, Mssql> for bool {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value.as_bytes()?[0] == 1)
    }
}
