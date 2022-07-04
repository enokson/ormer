use crate::{
    filters::{
        Sqlize,
        Escapable,
        InFilterValue,
        Gather,
    },
    helper_functions::*
};
use serde::Deserialize;
use std:: {
    collections::{
        BTreeMap,
    },
};

impl Escapable for String {}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct StringFilter {
    pub equals: Option<String>,
    pub not: Option<String>,
    pub lt: Option<String>,
    pub lte: Option<String>,
    pub gt: Option<String>,
    pub gte: Option<String>,
    pub contains: Option<String>,
    pub search: Option<String>,
    pub mode: Option<String>,
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
    #[serde(rename="in")]
    pub is_in: Option<InFilterValue<String>>
}

impl StringFilter {
    pub fn gather_args_with_key(&self, key: &str) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        let mut values = self.gather_args();
        if let Some(value) = values.remove("not") {
            args.push(not(&prepend_column(key, &value)));
        }
        for (_, value) in values {
            args.push(prepend_column(key, &value));
        }
        args
    }
}

impl Gather for StringFilter {
    fn gather_args(&self) -> BTreeMap<&str, String> {
        let mut args: BTreeMap<&str, String> = BTreeMap::new();
        if let Some(value) = &self.equals {
            args.insert("eqls", equals(&value.escape()));
            // args.push(equals(column, &value.escape()))
        }
        if let Some(value) = &self.not {
            args.insert("not", equals(&value.escape()));
            // args.push(not(column, &value.escape()))
        }
        if let Some(value) = &self.lt {
            args.insert("lt", lt(&value.escape()));
            // args.push(lt(column, &value.escape()))
        }
        if let Some(value) = &self.lte {
            args.insert("lte", lte(&value.escape()));
            // args.push(lte(column, &value.escape()))
        }
        if let Some(value) = &self.gt {
            args.insert("gt", gt(&value.escape()));
            // args.push(gt(column, &value.escape()))
        }
        if let Some(value) = &self.gte {
            args.insert("gte", gte(&value.escape()));
            // args.push(gte(column, &value.escape()))
        }
        // if let Some(value) = &self.is_in {
        //     // TODO: add is_in values
        //     // if let Some(arg) = value.to_nullable_sql(column) {
        //     //     args.push(arg)
        //     // }
        // }
        args
    }
}

impl<'a> Sqlize for StringFilter {
    fn to_sql(&self, column: &str) -> String {
        let args: Vec<String> = self.gather_args_with_key(column);
        if args.is_empty() {
            panic!("Filter not found.");
        }
        if args.len() == 1 {
            return args[0].clone()
        }
        enclose::<String>(&args.join(" AND "))
    }

    fn to_nullable_sql(&self, column: &str) -> Option<String> {
        let args: Vec<String> = self.gather_args_with_key(column);
        if args.is_empty() {
            return None;
        }
        if args.len() == 1 {
            return Some(args[0].clone())
        }
        Some(enclose::<String>(&args.join(" AND ")))
    }
}
