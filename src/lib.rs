//! Microsoft SQL (MSSQL) database driver.

#[macro_use]
extern crate sqlx_core;

use crate::executor::Executor;

#[cfg(feature = "any")]
pub mod any;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
//mod ext;
mod io;
mod options;
mod protocol;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::MssqlArguments;
pub use column::MssqlColumn;
pub use connection::MssqlConnection;
pub use database::Mssql;
pub use error::MssqlDatabaseError;
pub use options::{MssqlConnectOptions, MssqlSslMode};
pub use query_result::MssqlQueryResult;
pub use row::MssqlRow;
pub use statement::MssqlStatement;
pub use transaction::MssqlTransactionManager;
pub use type_info::MssqlTypeInfo;
pub use value::{MssqlValue, MssqlValueRef};

pub(crate) use sqlx_core::driver_prelude::*;

/// An alias for [`Pool`][crate::pool::Pool], specialized for MSSQL.
pub type MssqlPool = sqlx_core::pool::Pool<Mssql>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for MSSQL.
pub type MssqlPoolOptions = sqlx_core::pool::PoolOptions<Mssql>;

/// An alias for [`Executor<'_, Database = Mssql>`][Executor].
pub trait MssqlExecutor<'c>: Executor<'c, Database = Mssql> {}
impl<'c, T: Executor<'c, Database = Mssql>> MssqlExecutor<'c> for T {}

impl_into_arguments_for_arguments!(MssqlArguments);
impl_acquire!(Mssql, MssqlConnection);
impl_column_index_for_row!(MssqlRow);
impl_column_index_for_statement!(MssqlStatement);

// // NOTE: required due to the lack of lazy normalization
// impl_into_arguments_for_arguments!(MssqlArguments);
// impl_executor_for_pool_connection!(Mssql, MssqlConnection, MssqlRow);
// impl_executor_for_transaction!(Mssql, MssqlRow);
// impl_acquire!(Mssql, MssqlConnection);
// impl_column_index_for_row!(MssqlRow);
// impl_column_index_for_statement!(MssqlStatement);
// impl_into_maybe_pool!(Mssql, MssqlConnection);
