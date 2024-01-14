use super::database_field::DatabaseField;

pub trait DatabaseInsertable: Send + Default {
    fn database_name() -> String
    where
        Self: Sized;
    fn fields_value(&self) -> Vec<DatabaseField>;

    fn from_fields(fields: Vec<DatabaseField>) -> Result<Self, String>
    where
        Self: Sized;
}
