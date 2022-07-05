use crate::{
    helper_functions::*
};
use serde::{
    Deserialize,
    de::{
        DeserializeOwned
    }
};
use std::{
    collections::{
        BTreeMap
    },
    fmt::{
        Display,
    },
};
// use uuid::Uuid;

pub trait Sqlize/* : Display */ {
    fn to_sql(&self, column: &str) -> String;
    fn to_nullable_sql(&self, column: &str) -> Option<String>;
}

pub trait FromList {
    fn combine_args(&self) -> String;
}

pub trait Escapable: Display {
    fn escape(&self) -> String {
        format!("'{}'", self)
    }
}

pub trait Filter {
    fn to_sql(&self) -> String;
    fn to_nullable_sql(&self) -> Option<String>;
}

// pub trait Gather {
//     fn gather_args(&self) -> BTreeMap<&str, String>;
// }

// impl Escapable for Uuid {}



// equals
// not
// in
// notIn
// lt
// lte
// gt
// gte
// contains
// search
// mode
// startsWith
// endsWith

pub struct SqlValue<T: Sqlize>(pub T);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum FilterType<A: Escapable, B: Sqlize> {
    Filter(B),
    Eq(A),    
}

// impl<A: Display, B: Sqlize> FilterType<A, B> {
//     pub fn fmt_value
// }

impl<A: Escapable, B: Sqlize> Sqlize for FilterType<A, B> {
    fn to_sql(&self, column: &str) -> String {
        match self {
            Self::Eq(value) => prepend_column(column, &equals(&value.escape())),
            Self::Filter(value) => value.to_sql(column)
        }
    }
    fn to_nullable_sql(&self, column: &str) -> Option<String> {
        match self {
            Self::Eq(value) => Some(prepend_column(column, &equals(&value.escape()))),
            Self::Filter(value) => value.to_nullable_sql(column)
        }
    }

}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="camelCase")]
pub struct InFilterValue<T: Escapable> {
    values: Option<Vec<T>>,
    queries: Option<Vec<String>>,
}
impl<T: Escapable> InFilterValue<T> {

    pub fn get_args(&self) -> String {
        let mut temp_args: Vec<String> = vec![];
        if let Some(values) = &self.values {
            for value in values {
                temp_args.push(value.escape());
            }
        }
        if let Some(queries) = &self.queries {
            for query in queries {
                temp_args.push(enclose::<String>(query))
            }
        }
        if temp_args.is_empty() {
            return "".to_string();
        }
        is_in(&temp_args.join(","))
    }

}

impl<T: Escapable> Sqlize for InFilterValue<T> {
    fn to_sql(&self, column: &str) -> String {
        prepend_column(column, &is_in(&self.get_args()))
    }
    fn to_nullable_sql(&self, column: &str) -> Option<String> {
        let sql = self.get_args();
        if sql.is_empty() {
            return None;
        }
        Some(prepend_column(column, &sql))
    }
}

#[cfg(test)]
mod test {
    // use serde::Serialize;
    use serde_json::{
        json,
        from_value,
    };
    use super::*;
    use crate::sql_types::{
        number::*,
        string::StringFilter,
    };
    use uuid::Uuid;

    #[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
    pub struct UserFilter {
        uuid: Option<FilterType<Uuid, UuidFilter>>,
        name: Option<FilterType<String, StringFilter>>,
        all: Option<Vec<UserFilter>>,
        any: Option<Vec<UserFilter>>,
        not: Option<Vec<UserFilter>>,
    }

    impl UserFilter {
        pub fn get_args(&self) -> String {
            let mut args: Vec<String> = vec![];
            if let Some(filter_type) = &self.uuid {
                if let Some(filter) = filter_type.to_nullable_sql("uuid") {
                    args.push(filter);
                    // args.insert("uuid", filter);
                }
            }
            if let Some(filter_type) = &self.name {
                if let Some(filter) = filter_type.to_nullable_sql("name") {
                    args.push(filter);
                    // args.insert("name", filter);
                }
            }
            if let Some(filters) = &self.all {
                if !filters.is_empty() {
                    let filter_list = filters
                        .iter()
                        .filter_map(|v| v.to_nullable_sql()).collect::<Vec<String>>();
                    if !filter_list.is_empty() {
                        let value = {
                            if filter_list.len() > 1 {
                                enclose::<String>(&filter_list.join(" AND "))
                            } else {
                                filter_list[0].clone()
                            }
                        };
                        args.push(value);
                        // args.insert("all", value);
                    }
                }
            }
            if let Some(filters) = &self.any {
                if !filters.is_empty() {
                    let filter_list = filters
                        .iter()
                        .filter_map(|v| v.to_nullable_sql()).collect::<Vec<String>>();
                    if !filter_list.is_empty() {
                        let value = {
                            if filter_list.len() > 1 {
                                enclose::<String>(&filter_list.join(" OR "))
                            } else {
                                filter_list[0].clone()
                            }
                        };
                        args.push(value);
                        // args.insert("any", value);
                    }
                }
            }
            args.join(" AND ")
        }
    }

    impl Filter for UserFilter {
        fn to_sql(&self) -> String {
            self.get_args()
        }
        fn to_nullable_sql(&self) -> Option<String> {
            let args = self.get_args();
            if args.is_empty() {
                return None
            }
            Some(args)
        }
    }

    #[test]
    pub fn string_filter_test() {
        let json_filter = json!({
            "uuid": {
                "equals": "4fac5dd0-06d6-451b-9fd6-20b386e5d9bd"
            }
        });
        let filter: UserFilter = from_value(json_filter).unwrap();
        // println!("{:#?}",  filter);
        assert_eq!(
            "uuid = '4fac5dd0-06d6-451b-9fd6-20b386e5d9bd'",
            filter.to_sql()
        );

        let json_filter = json!({
            "uuid": {
                "equals": "4fac5dd0-06d6-451b-9fd6-20b386e5d9bd"
            },
            "name": "joshua"

        });
        let filter: UserFilter = from_value(json_filter).unwrap();
        assert_eq!(
            "uuid = '4fac5dd0-06d6-451b-9fd6-20b386e5d9bd' AND name = 'joshua'",
            filter.to_sql()
        );

        let json_filter = json!({
            "uuid": {
                "in": {
                    "values": [ "4fac5dd0-06d6-451b-9fd6-20b386e5d9bd" ],
                    "queries": [ "select uuid from users where uuid = '4fac5dd0-06d6-451b-9fd6-20b386e5d9bd'" ]
                }
            }
        });
        let filter: UserFilter = from_value(json_filter).unwrap();
        // println!("{:#?}",  filter);
        assert_eq!(
            "uuid IN ('4fac5dd0-06d6-451b-9fd6-20b386e5d9bd',(select uuid from users where uuid = '4fac5dd0-06d6-451b-9fd6-20b386e5d9bd'))",
            filter.to_sql()
        );

        let json_filter = json!({
            "uuid": { }
        });
        let filter: UserFilter = from_value(json_filter).unwrap();
        assert_eq!(
            None,
            filter.to_nullable_sql()
        );

        let json_filter = json!({
            "all": [
                { "uuid": "4fac5dd0-06d6-451b-9fd6-20b386e5d9bd" },
                { "name": "joshua" }
            ],
            "any": [
                // { "uuid": "00000000-06d6-451b-9fd6-20b386e5d9bd" },
                { "name": "jimbob" }
            ]
        });
        let filter: UserFilter = from_value(json_filter).unwrap();
        assert_eq!(
            "(uuid = '4fac5dd0-06d6-451b-9fd6-20b386e5d9bd' AND name = 'joshua') AND name = 'jimbob'",
            filter.to_sql()
        );
    }

}