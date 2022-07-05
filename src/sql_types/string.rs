use crate::{
    filters::{
        Sqlize,
        Escapable,
        InFilterValue,
        // Gather,
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

    fn get_arg(&self) -> String {
        if let Some(value) = &self.equals {
            return equals(&value.escape());
        }
        if let Some(value) = &self.not {
            return not_equal(&value.escape());
        }
        if let Some(value) = &self.lt {
            return lt(&value.escape());
        }
        if let Some(value) = &self.lte {
            return lte(&value.escape());
        }
        if let Some(value) = &self.gt {
            return gt(&value.escape());
        }
        if let Some(value) = &self.gte {
            return gte(&value.escape());
        }
        if let Some(filter) = &self.is_in {
            return filter.get_args();
        }
        if let Some(value) = &self.starts_with {
            // if let Some(mode) = &self.mode {

            // }
            return start_with(value);
        }
        if let Some(value) = &self.ends_with {
            // if let Some(mode) = &self.mode {

            // }
            return ends_with(value);
        }
        if let Some(value) = &self.search {
            // if let Some(mode) = &self.mode {

            // }
            return search(value);
        }
        return "".to_string()
    }

}

impl<'a> Sqlize for StringFilter {
    fn to_sql(&self, column: &str) -> String {
        prepend_column(column, &self.get_arg())
    }

    fn to_nullable_sql(&self, column: &str) -> Option<String> {
        let sql = self.get_arg();
        if sql.is_empty() {
            return None;
        }
        Some(prepend_column(column, &sql))
    }
}
