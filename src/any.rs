use crate::protocol::text::ColumnType;

use crate::{
    Mssql, MssqlColumn, MssqlConnectOptions, MssqlConnection, MssqlQueryResult, MssqlRow,
    MssqlTransactionManager, MssqlTypeInfo,
};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{StreamExt, TryFutureExt, TryStreamExt};
use sqlx_core::any::{
    Any, AnyArguments, AnyColumn, AnyConnectOptions, AnyConnectionBackend, AnyQueryResult, AnyRow,
    AnyStatement, AnyTypeInfo, AnyTypeInfoKind,
};
use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;
use sqlx_core::transaction::TransactionManager;

sqlx_core::declare_driver_with_optional_migrate!(DRIVER = Mssql);

impl AnyConnectionBackend for MssqlConnection {
    fn name(&self) -> &str {
        <Mssql as Database>::NAME
    }

    fn close(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close(*self)
    }

    fn close_hard(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close_hard(*self)
    }

    fn ping(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::ping(self)
    }

    fn begin(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::begin(self)
    }

    fn commit(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::commit(self)
    }

    fn rollback(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::rollback(self)
    }

    fn start_rollback(&mut self) {
        MssqlTransactionManager::start_rollback(self)
    }

    fn shrink_buffers(&mut self) {
        Connection::shrink_buffers(self);
    }

    fn flush(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::flush(self)
    }

    fn should_flush(&self) -> bool {
        Connection::should_flush(self)
    }

    #[cfg(feature = "migrate")]
    fn as_migrate(
        &mut self,
    ) -> sqlx_core::Result<&mut (dyn sqlx_core::migrate::Migrate + Send + 'static)> {
        Ok(self)
    }

    fn fetch_many<'q>(
        &'q mut self,
        query: &'q str,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxStream<'q, sqlx_core::Result<Either<AnyQueryResult, AnyRow>>> {
        let persistent = arguments.is_some();
        let args = arguments.as_ref().map(AnyArguments::convert_to);

        Box::pin(
            self.run(query, args, persistent)
                .try_flatten_stream()
                .map(|res| {
                    Ok(match res? {
                        Either::Left(result) => Either::Left(map_result(result)),
                        Either::Right(row) => Either::Right(AnyRow::try_from(&row)?),
                    })
                }),
        )
    }

    fn fetch_optional<'q>(
        &'q mut self,
        query: &'q str,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxFuture<'q, sqlx_core::Result<Option<AnyRow>>> {
        let persistent = arguments.is_some();
        let args = arguments.as_ref().map(AnyArguments::convert_to);

        Box::pin(async move {
            let stream = self.run(query, args, persistent).await?;
            futures_util::pin_mut!(stream);

            if let Some(Either::Right(row)) = stream.try_next().await? {
                return Ok(Some(AnyRow::try_from(&row)?));
            }

            Ok(None)
        })
    }

    fn prepare_with<'c, 'q: 'c>(
        &'c mut self,
        sql: &'q str,
        _parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'c, sqlx_core::Result<AnyStatement<'q>>> {
        Box::pin(async move {
            let statement = Executor::prepare_with(self, sql, &[]).await?;
            AnyStatement::try_from_statement(
                sql,
                &statement,
                statement.metadata.column_names.clone(),
            )
        })
    }

    fn describe<'q>(&'q mut self, sql: &'q str) -> BoxFuture<'q, sqlx_core::Result<Describe<Any>>> {
        Box::pin(async move {
            let describe = Executor::describe(self, sql).await?;
            describe.try_into_any()
        })
    }
}

impl<'a> TryFrom<&'a MssqlTypeInfo> for AnyTypeInfo {
    type Error = sqlx_core::Error;

    fn try_from(type_info: &'a MssqlTypeInfo) -> Result<Self, Self::Error> {
        Ok(AnyTypeInfo {
            kind: match &type_info.r#type {
                ColumnType::Null => AnyTypeInfoKind::Null,
                ColumnType::Short => AnyTypeInfoKind::SmallInt,
                ColumnType::Long => AnyTypeInfoKind::Integer,
                ColumnType::LongLong => AnyTypeInfoKind::BigInt,
                ColumnType::Float => AnyTypeInfoKind::Real,
                ColumnType::Double => AnyTypeInfoKind::Double,
                ColumnType::Blob
                | ColumnType::TinyBlob
                | ColumnType::MediumBlob
                | ColumnType::LongBlob => AnyTypeInfoKind::Blob,
                ColumnType::String | ColumnType::VarString | ColumnType::VarChar => {
                    AnyTypeInfoKind::Text
                }
                _ => {
                    return Err(sqlx_core::Error::AnyDriverError(
                        format!("Any driver does not support Mssql type {:?}", type_info).into(),
                    ))
                }
            },
        })
    }
}

impl<'a> TryFrom<&'a MssqlColumn> for AnyColumn {
    type Error = sqlx_core::Error;

    fn try_from(column: &'a MssqlColumn) -> Result<Self, Self::Error> {
        let type_info = AnyTypeInfo::try_from(&column.type_info)?;

        Ok(AnyColumn {
            ordinal: column.ordinal,
            name: column.name.clone(),
            type_info,
        })
    }
}

impl<'a> TryFrom<&'a MssqlRow> for AnyRow {
    type Error = sqlx_core::Error;

    fn try_from(row: &'a MssqlRow) -> Result<Self, Self::Error> {
        AnyRow::map_from(row, row.column_names.clone())
    }
}

impl<'a> TryFrom<&'a AnyConnectOptions> for MssqlConnectOptions {
    type Error = sqlx_core::Error;

    fn try_from(any_opts: &'a AnyConnectOptions) -> Result<Self, Self::Error> {
        use sqlx_core::connection::ConnectOptions;
        let mut opts = Self::from_url(&any_opts.database_url)?;
        opts.log_settings = any_opts.log_settings.clone();
        Ok(opts)
    }
}

fn map_result(result: MssqlQueryResult) -> AnyQueryResult {
    AnyQueryResult {
        rows_affected: result.rows_affected,
        last_insert_id: None,
    }
}
