# Simple ORM
A simple ORM library.

# Example
1) Once you have a struct you want to save in DB, make it derive `DatabaseInsertable` and define it's `primary_key` :
```rust
#[derive(Debug, Default, DatabaseInsertable)]
struct User {
  #[simple_orm(primary_key)]
  id: String,
  name: String,
  age: u8,
  activated: bool,
}
```
2) Create one of the available database connexion (Only PostgresDB currently) and initialize the struct in the database :
```rust
let mut pg_db = PostgresDB::new("host=localhost user=postgres password=postgres").await?;
pg_db.initialize::<User>().await?;
```
3) Finally, make the action you want :
   1) Get
   ```rust
   let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "MY_SUPER_ID")];
   let res = pg_db.get::<User>(&conds).await?;
   ``` 
   2) Insert:
   ```rust
   let user = User {
     id: "MY_SUPER_ID".to_owned(),
     name: "MY_SUPER_NAME".to_owned(),
     age: 99,
     activated: true,
   };
   pg_db.insert(user).await?;
   ```
   3) Update:
   ```rust
   let user = User {
     id: "MY_SUPER_ID".to_owned(),
     name: "MY_AMAZING_NAME".to_owned(),
     age: 100,
     activated: false,
   };
   let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "MY_SUPER_ID")];
   pg_db.update::<User>(user, &conds).await?;
   ```
  
   4) Delete
   ```rust
   let conds = vec![DatabaseCondition::new("id", ConditionOperator::Eq, "MY_SUPER_ID")];
   pg_db.delete::<User>(&conds).await?;
   ```
# Roadmap
V1.0.0 :
- [x] Get, insert, update, delete object in Postgres DB
- [ ] Handle foreign key
- [ ] SQLite support

# WARNING
⚠ Mainly a study project rather than a real library, use at your own risks. If you encounter any problem, please create an issue, I'd love to fix everything anyone can encounter.
