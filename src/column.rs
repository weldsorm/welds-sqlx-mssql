use crate::ext::ustr::UStr;
use crate::protocol::col_meta_data::{ColumnData, Flags};
use crate::{Mssql, MssqlTypeInfo};
use sqlx_core::column::Column;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: MssqlTypeInfo,
    pub(crate) flags: Flags,
}

impl MssqlColumn {
    pub(crate) fn new(meta: ColumnData, ordinal: usize) -> Self {
        Self {
            name: UStr::from(meta.col_name),
            type_info: MssqlTypeInfo(meta.type_info),
            ordinal,
            flags: meta.flags,
        }
    }
}

impl Column for MssqlColumn {
    type Database = Mssql;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &MssqlTypeInfo {
        &self.type_info
    }
}
