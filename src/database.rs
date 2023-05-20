use crate::{
    MssqlArguments, MssqlColumn, MssqlConnection, MssqlQueryResult, MssqlRow, MssqlStatement,
    MssqlTransactionManager, MssqlTypeInfo, MssqlValue, MssqlValueRef,
};
use sqlx_core::database::{Database, HasArguments, HasStatement, HasValueRef};

/// MSSQL database driver.
#[derive(Debug)]
pub struct Mssql;

impl Database for Mssql {
    type Connection = MssqlConnection;

    type TransactionManager = MssqlTransactionManager;

    type Row = MssqlRow;

    type QueryResult = MssqlQueryResult;

    type Column = MssqlColumn;

    type TypeInfo = MssqlTypeInfo;

    type Value = MssqlValue;

    const NAME: &'static str = "Mssql";

    const URL_SCHEMES: &'static [&'static str] = &["mssql"];
}

impl<'r> HasValueRef<'r> for Mssql {
    type Database = Mssql;

    type ValueRef = MssqlValueRef<'r>;
}

impl<'q> HasStatement<'q> for Mssql {
    type Database = Mssql;

    type Statement = MssqlStatement<'q>;
}

impl HasArguments<'_> for Mssql {
    type Database = Mssql;

    type Arguments = MssqlArguments;

    type ArgumentBuffer = Vec<u8>;
}
