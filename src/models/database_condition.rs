use super::database_field::FieldType;

#[derive(Clone, PartialEq, Debug)]
pub enum ConditionOperator {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

pub struct DatabaseCondition {
    name: String,
    value: FieldType,
    operator: ConditionOperator,
}

impl DatabaseCondition {
    pub fn new<V: ToString + Clone>(name: &str, operator: ConditionOperator, value: V) -> Self
    where
        FieldType: From<V>,
    {
        return Self {
            name: name.to_owned(),
            value: FieldType::from(value.clone()),
            operator: operator,
        };
    }

    pub fn name(&self) -> String {
        return self.name.clone();
    }

    pub fn operator(&self) -> ConditionOperator {
        return self.operator.clone();
    }

    pub fn value(&self) -> FieldType {
        return self.value.clone();
    }
}

#[cfg(test)]
pub mod tests {
    use crate::models::{
        database_condition::{ConditionOperator, DatabaseCondition},
        database_field::FieldType,
    };

    #[test]
    pub fn new() {
        let cond = DatabaseCondition::new("id", ConditionOperator::Eq, 32);
        assert_eq!(cond.name, "id");
        assert_eq!(cond.operator, ConditionOperator::Eq);
        assert_eq!(cond.value, FieldType::Number(32));
    }
}
