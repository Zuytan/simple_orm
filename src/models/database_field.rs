use super::database_insertable::DatabaseInsertable;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FieldType {
    Number(i64),
    String(String),
    Bool(bool),
}

impl Default for FieldType {
    fn default() -> Self {
        return Self::String("".to_owned());
    }
}

impl From<String> for FieldType {
    fn from(val: String) -> Self {
        return Self::String(val);
    }
}
impl From<&str> for FieldType {
    fn from(val: &str) -> Self {
        return Self::String(val.to_owned());
    }
}
impl From<u8> for FieldType {
    fn from(val: u8) -> Self {
        return Self::Number(val.into());
    }
}
impl From<i8> for FieldType {
    fn from(val: i8) -> Self {
        return Self::Number(val.into());
    }
}
impl From<i16> for FieldType {
    fn from(val: i16) -> Self {
        return Self::Number(val.into());
    }
}
impl From<i32> for FieldType {
    fn from(val: i32) -> Self {
        return Self::Number(val.into());
    }
}
impl From<bool> for FieldType {
    fn from(val: bool) -> Self {
        return Self::Bool(val);
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct DatabaseField {
    field_name: String,
    field_type: FieldType,
    unique: bool,
    mandatory: bool,
    primary_key: bool,
    foreign_key: Option<(String, String)>,
}

impl DatabaseField {
    pub fn new(field_name: &str, field_type: FieldType) -> Self {
        return Self {
            field_name: field_name.to_owned(),
            field_type,
            unique: false,
            mandatory: false,
            primary_key: false,
            foreign_key: None,
        };
    }
    pub fn builder(field_name: &str, field_type: FieldType) -> DatabaseFieldBuilder {
        return DatabaseFieldBuilder::new(Self::new(field_name, field_type));
    }
    pub fn field_name(&self) -> String {
        return self.field_name.clone();
    }
    pub fn unique(&self) -> bool {
        return self.unique;
    }
    pub fn field_type(&self) -> FieldType {
        return self.field_type.clone();
    }
    pub fn set_field_type(&mut self, new_field_type: FieldType) {
        self.field_type = new_field_type;
    }
    pub fn is_mandatory(&self) -> bool {
        return self.mandatory;
    }
    pub fn is_primary_key(&self) -> bool {
        return self.primary_key;
    }
    pub fn is_foreign_key(&self) -> Option<(String, String)> {
        return self.foreign_key.clone();
    }
}

pub struct DatabaseFieldBuilder {
    dbf: DatabaseField,
}

impl DatabaseFieldBuilder {
    fn new(dbf: DatabaseField) -> Self {
        return Self { dbf };
    }
    pub fn is_mandatory(mut self) -> Self {
        self.dbf.mandatory = true;
        return self;
    }
    pub fn is_primary_key(mut self) -> Self {
        self.dbf.primary_key = true;
        return self;
    }
    pub fn is_foreign_key(mut self, foreign_db: &str, foreign_field: &str) -> Self {
        self.dbf.foreign_key = Some((foreign_db.to_owned(), foreign_field.to_owned()));
        return self;
    }
    pub fn is_unique(mut self) -> Self {
        self.dbf.unique = true;
        return self;
    }
    pub fn build(self) -> DatabaseField {
        return self.dbf;
    }
}
