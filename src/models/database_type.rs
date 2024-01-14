use async_trait::async_trait;

use super::{
    database_condition::DatabaseCondition, database_error::DatabaseError,
    database_insertable::DatabaseInsertable,
};

#[async_trait]
pub trait DatabaseType: Send {
    async fn initialize<D: DatabaseInsertable>(&mut self) -> Result<(), DatabaseError>;
    async fn insert<D: DatabaseInsertable>(&mut self, data: D) -> Result<(), DatabaseError>;
    async fn update<D: DatabaseInsertable>(
        &mut self,
        data: D,
        conditions: &[DatabaseCondition],
    ) -> Result<(), DatabaseError>;
    async fn delete<D: DatabaseInsertable>(
        &mut self,
        conditions: &[DatabaseCondition],
    ) -> Result<(), DatabaseError>;
    async fn get<D: DatabaseInsertable>(
        &mut self,
        conditions: &[DatabaseCondition],
    ) -> Result<Vec<D>, DatabaseError>;
}
