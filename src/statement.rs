use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::HashMap;
use crate::{Mssql, MssqlArguments, MssqlColumn, MssqlTypeInfo};
use either::Either;
use sqlx_core::column::ColumnIndex;
use sqlx_core::statement::Statement;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MssqlStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) metadata: Arc<MssqlStatementMetadata>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MssqlStatementMetadata {
    pub(crate) columns: Vec<MssqlColumn>,
    pub(crate) column_names: HashMap<UStr, usize>,
}

impl<'q> Statement<'q> for MssqlStatement<'q> {
    type Database = Mssql;

    fn to_owned(&self) -> MssqlStatement<'static> {
        MssqlStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[MssqlTypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[MssqlColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(MssqlArguments);
}

impl ColumnIndex<MssqlStatement<'_>> for &'_ str {
    fn index(&self, statement: &MssqlStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
