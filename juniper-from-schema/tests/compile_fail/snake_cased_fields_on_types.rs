#![allow(dead_code, unused_variables, unused_must_use, unused_imports)]
include!("../setup.rs");

graphql_schema! {
    type Query {
        snake_cased: String!
    }

    schema { query: Query }
}

pub struct Query;

impl QueryFields for Query {
    fn field_snake_cased<'a>(&self, executor: &Executor<'a, Context>) -> FieldResult<&String> {
        unimplemented!()
    }
}
