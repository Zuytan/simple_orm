use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(DatabaseInsertable, attributes(simple_orm))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    return match data {
        Data::Struct(data_values) => {
            // Generate code for `fields_value` function
            let fields_value_fn = {
                let field_names = data_values.fields.iter().map(|field| {
                    let mut is_primary_key = false;
                    for attr in &field.attrs {
                        let attrs_str = attr.meta.to_token_stream().to_string();
                        let simple_orm_attrs = attrs_str.split("(").collect::<Vec<&str>>();
                        if simple_orm_attrs[0] == "simple_orm" && simple_orm_attrs.len() > 1 {
                            let attrs = simple_orm_attrs[1].split(" ").collect::<Vec<&str>>();
                            for attr in attrs {
                                if attr == "primary_key" || attr == "primary_key)" {
                                    is_primary_key = true;
                                }
                            }
                        }
                    }
                    (&field.ident, is_primary_key)
                });
                let field_value_initializers = field_names.clone().map(|(name, is_primary)| {
                    let mut is_primary_quote = quote! {};
                    if is_primary {
                        is_primary_quote = quote! {
                            .is_primary_key()
                        }
                    }
                    quote! { DatabaseField::builder(stringify!(#name), FieldType::from(self.#name.clone()))#is_primary_quote.build() }
                });

                quote! {
                    fn fields_value(&self) -> Vec<DatabaseField> {
                        return vec![
                            #( #field_value_initializers ),*
                        ];
                    }
                }
            };
            let from_fields_fn = {
                let field_names = data_values.fields.iter().map(|field| &field.ident);
                let field_types = data_values.fields.iter().map(|field| &field.ty);
                let field_type_checks = field_names.clone().zip(field_types).map(|(name, ty)| {
                    let type_check = if ty.to_token_stream().to_string() == "u8" || ty.to_token_stream().to_string() == "u16" || ty.to_token_stream().to_string() == "u32" || ty.to_token_stream().to_string() == "u64" || ty.to_token_stream().to_string() == "i8" || ty.to_token_stream().to_string() == "i16" || ty.to_token_stream().to_string() == "i32" || ty.to_token_stream().to_string() == "i64" {
                        quote! {
                            match f.field_type() {
                                FieldType::Number(val) => val.try_into().unwrap(),
                                _ => return Err(format!("Mismatched field type for '{}'",stringify!(id))),
                            }
                        }
                    } else if ty.to_token_stream().to_string() == "&str" || ty.to_token_stream().to_string() == "String" {
                        quote! {
                            match f.field_type() {
                                FieldType::String(val) => val,
                                _ => return Err(format!("Mismatched field type for '{}'",stringify!(id))),
                            }
                        }
                    } else if ty.to_token_stream().to_string() == "bool" {
                        quote! {
                            match f.field_type() {
                                FieldType::Bool(val) => val,
                                _ => return Err(format!("Mismatched field type for '{}'",stringify!(id))),
                            }
                        }
                    } else {
                        panic!("Type {} is not handled", ty.to_token_stream().to_string());
                    };
                    quote! {
                        #name: match fields.iter().find(|field| field.field_name() == stringify!(#name)) {
                            Some(f) => #type_check,
                            None => return Err(format!("Field '{}' not found in fields vector", stringify!(#name))),
                        }
                    }
                });

                quote! {
                    fn from_fields(fields: Vec<DatabaseField>) -> Result<Self, String>
                    where
                        Self: Sized,
                    {
                        Ok(Self {
                            #( #field_type_checks ),*
                        })
                    }
                }
            };
            let output = quote! {
                use crate::models::{
                    database_insertable::DatabaseInsertable,
                    database_field::{DatabaseField, FieldType}
                };
                impl DatabaseInsertable for #ident {
                    fn database_name() -> String
                    where
                        Self: Sized,
                    {
                        return "aled".to_owned();
                    }
                    #fields_value_fn
                    #from_fields_fn
                }
            };
            output.into()
        }
        Data::Enum(_) => {
            panic!("Derive trait \"DatabaseInsertable\" is only available for structs")
        }
        Data::Union(_) => {
            panic!("Derive trait \"DatabaseInsertable\" is only available for structs")
        }
    };
}
