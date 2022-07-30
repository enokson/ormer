use postgres::types::ToSql;
use std::{
    fmt::{Display, Write},
};

pub mod conditions;
pub mod operators;
pub mod update;

#[macro_export]
macro_rules! compose {
    (|| $index:expr, $params:expr, $sql:expr,) => {{
        ($index, $params, $sql)
    }};

    (|| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let mod_sql = move |mut sql: String| {
            write!(&mut sql, " ").unwrap();
            sql
        };
        let sql = mod_sql($sql);
        let (index, params, sql) = $head($index, $params, sql);
        let (index, params, sql) = compose!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    (| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let (index, params, sql) = $head($index, $params, $sql);
        let (index, params, sql) = compose!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    ($($F:expr),*) => {{
        move |index, params, sql| {
            let (index, params, sql) = compose!(| index, params, sql, $($F,)*);
            (index, params, sql)
        }}
    }
}
pub use crate::compose;

#[macro_export]
macro_rules! columns {
    (|| $index:expr, $params:expr, $sql:expr,) => {{
        ($index, $params, $sql)
    }};

    (|| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let mod_sql = move |mut sql: String| {
            write!(&mut sql, " ").unwrap();
            sql
        };
        let sql = mod_sql($sql);
        let (index, params, sql) = $head($index, $params, sql);
        let (index, params, sql) = columns!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    (| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let (index, params, sql) = $head($index, $params, $sql);
        let (index, params, sql) = columns!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    ($($F:expr),*) => {{
        move |index, params, sql| {
            let (index, params, sql) = columns!(| index, params, sql, $($F,)*);
            (index, params, sql)
        }}
    }
}
pub use crate::columns;

#[macro_export]
macro_rules! conditions {
    (|| $index:expr, $params:expr, $sql:expr,) => {{
        ($index, $params, $sql)
    }};

    (|| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let mod_sql = move |mut sql: String| {
            write!(&mut sql, " ").unwrap();
            sql
        };
        let sql = mod_sql($sql);
        let (index, params, sql) = $head($index, $params, sql);
        let (index, params, sql) = conditions!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    (| $index:expr, $params:expr, $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let (index, params, sql) = $head($index, $params, $sql);
        let (index, params, sql) = conditions!(|| index, params, sql, $($tail,)*);
        (index, params, sql)
    }};

    ($($F:expr),*) => {{
        move |index, params, mut sql: String| {
            write!(&mut sql, "WHERE ").unwrap();
            let (index, params, sql) = conditions!(| index, params, sql, $($F,)*);
            (index, params, sql)
        }}
    }
}
pub use crate::conditions;

pub fn columns<'a, 'b>(
    columns: &'b [ &'b (dyn Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)) ]
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, params, sql| {
        let (first, rest) = columns.split_first().unwrap();
        let (mut index, mut params, mut sql) = first(index, params, sql);
        for c in rest {
            write!(&mut sql, ", ").unwrap();
            let (i, p, s) = c(index, params, sql);
            index = i;
            params = p;
            sql = s;
        }
        (index, params, sql)
    }
}

pub fn compose<'a, 'b>(
    n: &'b [ &'b (dyn Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)) ]
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, params, sql| {
        let (first, rest) = n.split_first().unwrap();
        let (mut index, mut params, mut sql) = first(index, params, sql);
        for param in rest {
            write!(&mut sql, " ").unwrap();
            let (i, p, s) = param(index, params, sql);
            sql = s;
            params = p;
            index = i;
        }
        (index, params, sql)
    }
}

pub fn esc<'a, 'b>(
    string: &'b str
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, params, mut sql| {
        write!(&mut sql, "`{}`", string).unwrap();
        (index, params, sql)
    }
}

pub fn from<'a, 'b>(index: i32, params: Vec<&'a (dyn ToSql + Sync)>, mut sql: String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
    write!(&mut sql, "FROM").unwrap();
    (index, params, sql)
}


pub fn select<'a, 'b>(index: i32, params: Vec<&'a (dyn ToSql + Sync)>, mut sql: String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
    write!(&mut sql, "SELECT").unwrap();
    (index, params, sql)
}

pub fn select_n<'b>(param: impl Fn(String) -> String + 'b) -> impl Fn(String) -> String + 'b {
    move |mut sql| {
        write!(&mut sql, "SELECT ").unwrap();
        let mut sql = param(sql);
        write!(&mut sql, " ").unwrap();
        sql
    }
}

pub fn space<'b>(param: impl Fn(String) -> String + 'b) -> impl Fn(String) -> String + 'b {
    move |mut sql| {
        write!(&mut sql, " ").unwrap();
        param(sql)
    }
}

pub fn table<'b>(name: &'b str) -> impl Fn(String) -> String + 'b {
    move |mut sql| {
        write!(&mut sql, " {}", name).unwrap();
        sql
    }
}

pub const NAME_AND_RANK: &'static (dyn Fn(i32, Vec<& (dyn ToSql + Sync)>, String) -> (i32, Vec<&(dyn ToSql + Sync)>, String)) = &columns!(
    esc("name"), esc("rank") 
);

#[cfg(test)]
mod test {
    use postgres::{Client, NoTls};
    use super::*;
    use conditions::*;
    use operators::*;


    #[test]
    fn select_test<'b>() {
        fn users<'a>(
            index: i32, 
            params: Vec<&'a (dyn ToSql + Sync)>, 
            mut sql: String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
            write!(&mut sql, "users").unwrap();
            (index, params, sql)
        }

        let (index, params, sql) = compose(&[
            &select, 
            &columns(&[ 
                &esc("name"), 
                &esc("rank") 
            ]),
            &from, &users,
            &condition(eqls("id", &1))
        ])(1, Vec::with_capacity(1), String::with_capacity(100));
        assert_eq!("SELECT `name`, `rank` FROM users WHERE id=$1", sql);

        fn try_to_query(client: &mut Client) {
            let select_name_and_rank_from_users = compose!(select, NAME_AND_RANK, from, users);
            let where_id_eqls_input = conditions!(
                eqls("id", &1)
            );
            let (index, params, sql) = compose!( 
                select_name_and_rank_from_users,
                where_id_eqls_input
            )(1, Vec::with_capacity(10), String::with_capacity(100));
            assert_eq!(
                "SELECT `name`, `rank` FROM users WHERE id=$1",
                sql
            );
            client.query(&sql, &params).unwrap();

        }

    }

}

