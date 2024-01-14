use std::collections::HashMap;

use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};

use crate::models::{
    database_condition::{ConditionOperator, DatabaseCondition},
    database_error::DatabaseError,
    database_field::{DatabaseField, FieldType},
    database_insertable::DatabaseInsertable,
    database_type::DatabaseType,
};

pub struct PostgresDB {
    client: Client,
}

impl PostgresDB {
    pub async fn new(params: &str) -> Result<Self, DatabaseError> {
        let (client, connection) = match tokio_postgres::connect(params, NoTls).await {
            Ok(r) => r,
            Err(e) => {
                return Err(DatabaseError {
                    error: "CannotConnectToDatabase".to_owned(),
                    details: e.to_string(),
                })
            }
        };
        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        return Ok(Self { client });
    }

    fn get_string_operator(operator: ConditionOperator) -> &'static str {
        match operator {
            ConditionOperator::Eq => "=",
            ConditionOperator::Gt => ">",
            ConditionOperator::Gte => ">=",
            ConditionOperator::Lt => "<",
            ConditionOperator::Lte => "<=",
        }
    }

    fn stringify_condition(cond: &DatabaseCondition) -> String {
        match cond.value() {
            FieldType::String(val) => {
                return format!(
                    "{} {} \'{}\'",
                    cond.name(),
                    Self::get_string_operator(cond.operator()),
                    val
                );
            }
            FieldType::Number(val) => {
                return format!(
                    "{} {} {}",
                    cond.name(),
                    Self::get_string_operator(cond.operator()),
                    val
                );
            }
            FieldType::Bool(val) => {
                return format!(
                    "{} {} {}",
                    cond.name(),
                    Self::get_string_operator(cond.operator()),
                    val
                );
            }
        };
    }

    fn stringify_datafield_in_key_val_string(fields: Vec<DatabaseField>) -> (String, String) {
        let mut joined_values = String::new();
        let mut joined_key = String::new();
        for (idx, field) in fields.iter().enumerate() {
            joined_key = format!("{}{}", joined_key, field.field_name());
            match field.field_type() {
                FieldType::Number(val) => {
                    joined_values = format!("{}{}", joined_values, val);
                }
                FieldType::String(val) => {
                    joined_values = format!("{}\'{}\'", joined_values, val);
                }
                FieldType::Bool(val) => {
                    joined_values = format!("{}{}", joined_values, val);
                }
            };
            if idx + 1 < fields.len() {
                joined_key = format!("{}, ", joined_key);
                joined_values = format!("{}, ", joined_values);
            }
        }
        return (joined_key, joined_values);
    }
}

#[async_trait]
impl DatabaseType for PostgresDB {
    async fn initialize<D: DatabaseInsertable>(&mut self) -> Result<(), DatabaseError> {
        let default_D = D::default();
        let fields = default_D.fields_value();
        let mut table_fields = Vec::new();
        let mut constraints = Vec::new();
        let mut list_primary_key = Vec::new();
        let mut list_foreign_key = Vec::new();
        for field in fields {
            let field_type = match field.field_type() {
                FieldType::Number(_) => "INTEGER",
                FieldType::String(_) => "TEXT",
                FieldType::Bool(_) => "BOOLEAN",
            };
            let mandatory = match field.is_mandatory() {
                true => " NOT NULL".to_owned(),
                false => String::new(),
            };
            let unique = match field.unique() {
                true => " UNIQUE".to_owned(),
                false => String::new(),
            };
            if field.is_primary_key() {
                list_primary_key.push(field.field_name());
            }
            if field.is_foreign_key().is_some() {
                list_foreign_key.push(field.is_foreign_key().unwrap())
            }
            table_fields.push(format!(
                "{} {}{}{}",
                field.field_name(),
                field_type,
                mandatory,
                unique
            ));
        }
        if list_primary_key.len() > 0 {
            let joined_primary_key = list_primary_key.join(",");
            constraints.push(format!("PRIMARY KEY ({})", joined_primary_key));
        }
        if list_foreign_key.len() > 0 {
            for key in list_foreign_key {
                constraints.push(format!(
                    "FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE SET NULL",
                    key.1, key.0, key.1
                ));
            }
        }
        let req = format!(
            "CREATE TABLE IF NOT EXISTS {} (\n{},\n{});",
            D::database_name(),
            table_fields.join(",\n"),
            constraints.join(",\n")
        );
        match self.client.batch_execute(&req).await {
            Ok(()) => Ok(()),
            Err(e) => Err(DatabaseError {
                error: "CannotCreateTable".to_owned(),
                details: e.to_string(),
            }),
        }
    }
    async fn insert<D: DatabaseInsertable>(&mut self, data: D) -> Result<(), DatabaseError> {
        let fields = data.fields_value();
        let (joined_key, joined_values) = Self::stringify_datafield_in_key_val_string(fields);
        let req = format!(
            "INSERT INTO {}({}) VALUES({})",
            D::database_name(),
            joined_key,
            joined_values
        );
        match self.client.batch_execute(&req).await {
            Ok(()) => Ok(()),
            Err(e) => Err(DatabaseError {
                error: "CannotInsertInTable".to_owned(),
                details: e.to_string(),
            }),
        }
    }

    async fn update<D: DatabaseInsertable>(
        &mut self,
        data: D,
        conditions: &[DatabaseCondition],
    ) -> Result<(), DatabaseError> {
        let mut cond = String::new();
        if conditions.len() > 0 {
            cond = " WHERE ".to_owned();
            for (idx, curr_cond) in conditions.iter().enumerate() {
                cond = format!("{}{}", cond, Self::stringify_condition(curr_cond));
                if idx + 1 < conditions.len() {
                    cond = format!("{} AND ", cond);
                }
            }
        }
        let fields = data.fields_value();
        let (joined_keys, joined_values) = Self::stringify_datafield_in_key_val_string(fields);
        let req = format!(
            "UPDATE {} SET ({}) = ({}){};",
            D::database_name(),
            joined_keys,
            joined_values,
            cond
        );
        match self.client.batch_execute(&req).await {
            Ok(()) => Ok(()),
            Err(e) => Err(DatabaseError {
                error: "CannotUpdateInTable".to_owned(),
                details: e.to_string(),
            }),
        }
    }

    async fn delete<D: DatabaseInsertable>(
        &mut self,
        query: &[DatabaseCondition],
    ) -> Result<(), DatabaseError> {
        let mut cond = String::new();
        if query.len() > 0 {
            cond = " WHERE ".to_owned();
            for (idx, curr_cond) in query.iter().enumerate() {
                cond = format!("{}{}", cond, Self::stringify_condition(curr_cond));
                if idx + 1 < query.len() {
                    cond = format!("{} AND ", cond);
                }
            }
        }
        let req = format!("DELETE FROM {}{};", D::database_name(), cond);
        match self.client.batch_execute(&req).await {
            Ok(()) => Ok(()),
            Err(e) => Err(DatabaseError {
                error: "CannotDeleteFromTable".to_owned(),
                details: e.to_string(),
            }),
        }
    }

    async fn get<D: DatabaseInsertable>(
        &mut self,
        query: &[DatabaseCondition],
    ) -> Result<Vec<D>, DatabaseError> {
        let mut cond = String::new();
        if query.len() > 0 {
            cond = " WHERE ".to_owned();
            for (idx, curr_cond) in query.iter().enumerate() {
                cond = format!("{}{}", cond, Self::stringify_condition(curr_cond));
                if idx + 1 < query.len() {
                    cond = format!("{} AND ", cond);
                }
            }
        }
        let def_d = D::default();
        let mut fields = def_d.fields_value();
        let field_str = fields
            .iter()
            .map(|f| f.field_name())
            .collect::<Vec<String>>()
            .join(", ");
        let req = format!("SELECT {} from {}{};", field_str, D::database_name(), cond);
        let result = match self.client.query(&req, &[]).await {
            Ok(res) => res,
            Err(e) => {
                return Err(DatabaseError {
                    error: "InvalidQuery".to_owned(),
                    details: e.to_string(),
                })
            }
        };
        let mut objects = Vec::new();
        for row in result {
            let mut new_obj_fields: Vec<DatabaseField> = Vec::new();
            for (idx, field) in fields.iter_mut().enumerate() {
                match field.field_type() {
                    FieldType::Number(_) => {
                        let value: i32 = row.get(idx);
                        field.set_field_type(FieldType::from(value))
                    }
                    FieldType::String(_) => {
                        let value: String = row.get(idx);
                        field.set_field_type(FieldType::from(value))
                    }
                    FieldType::Bool(_) => {
                        let value: bool = row.get(idx);
                        field.set_field_type(FieldType::from(value))
                    }
                };
                new_obj_fields.push(field.clone());
            }
            let obj = match D::from_fields(new_obj_fields) {
                Ok(o) => o,
                Err(e) => {
                    return Err(DatabaseError {
                        error: "ExtractionFailed".to_owned(),
                        details: e,
                    })
                }
            };
            objects.push(obj)
        }
        return Ok(objects);
    }
}

#[cfg(test)]
pub mod tests {

    use simple_orm_derive::DatabaseInsertable;

    use crate::models::{
        database_condition::{ConditionOperator, DatabaseCondition},
        database_type::DatabaseType,
    };

    use super::PostgresDB;

    #[derive(Debug, Default, DatabaseInsertable)]
    struct User {
        #[simple_orm(primary_key)]
        id: String,
        name: String,
        age: u8,
        activated: bool,
    }

    #[tokio::test]
    async fn initialize() {
        let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres")
            .await
            .unwrap();
        let res = pg_db.initialize::<User>().await;
        assert_eq!(res.is_ok(), true);
    }

    #[tokio::test]
    async fn insert() {
        let user = User {
            id: "heyZ".to_owned(),
            name: "name".to_owned(),
            age: 25,
            activated: true,
        };
        let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres")
            .await
            .unwrap();
        let res = pg_db.insert(user).await;
        println!("res: {:?}", res)
    }

    #[tokio::test]
    async fn get() {
        let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres")
            .await
            .unwrap();
        let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "heyZ")];
        let res = pg_db.get::<User>(&conds).await.unwrap();
        println!("{:?}", res)
    }
    #[tokio::test]
    async fn delete() {
        let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres")
            .await
            .unwrap();
        let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "heyZ")];
        let _ = pg_db.delete::<User>(&conds).await.unwrap();
    }
    #[tokio::test]
    async fn update() {
        let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres")
            .await
            .unwrap();
        let user = User {
            id: "heyZ".to_owned(),
            name: "name".to_owned(),
            age: 26,
            activated: true,
        };
        let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "heyZ")];
        let _ = pg_db.update::<User>(user, &conds).await.unwrap();
    }
}
