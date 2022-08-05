use postgres::types::ToSql;
use std::fmt::{Display, Write};

pub mod conditions;
pub mod operators;
pub mod update;

pub fn append<'a, T: Display + AsRef<str> + 'a>(string: T) -> impl Fn(String) -> String + 'a {
    move |mut sql| {
        write!(&mut sql, "{}", string).unwrap();
        sql
    }
    
}

#[macro_export]
macro_rules! compose {
    (|| $sql:expr,) => {{
        $sql
    }};
    (|| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = ($head)($sql);
        let sql = compose!(|| sql, $($tail,)*);
        sql
    }};
    (| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = ($head)($sql);
        let sql = compose!(|| sql, $($tail,)*);
        sql
    }};
    ($($F:expr),*) => {{
        move |sql| {
            // compose(&[ $( &$F as &dyn Fn(String) -> String, )* ])(sql)
            let sql = compose!(| sql, $($F,)*);
            sql
        }}
    }
}
pub use crate::compose;

#[macro_export]
macro_rules! ormer {
    (|| $sql:expr,) => {{
        $sql
    }};
    (|| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = append(" ")($sql);
        let sql = (&$head)(sql);
        let sql = ormer!(|| sql, $($tail,)*);
        sql
    }};
    (| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = (&$head)($sql);
        let sql = ormer!(|| sql, $($tail,)*);
        sql
    }};
    ($($F:expr),*) => {{
        move |sql| {
            let sql = ormer!(| sql, $($F,)*);
            sql
        }}
    }
}
pub use crate::ormer;

#[macro_export]
macro_rules! columns {
    (|| $sql:expr,) => {{
        $sql
    }};
    (|| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = append(", ")($sql);
        let sql = $head(sql);
        let sql = columns!(|| sql, $($tail,)*);
        sql
    }};
    (| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = $head($sql);
        let sql = columns!(|| sql, $($tail,)*);
        sql
    }};
    ($($F:expr),*) => {{
        move |sql| {
            let sql = columns!(| sql, $($F,)*);
            sql
        }}
    }
}
pub use crate::columns;

#[macro_export]
macro_rules! conditions {
    (|| $sql:expr,) => {{
        $sql
    }};
    (|| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = append(",")($sql);
        let sql = $head(sql);
        let sql = columns!(|| sql, $($tail,)*);
        (index, params, sql)
    }};
    (| $sql:expr, $head:expr, $($tail:expr,)*) => {{
        let sql = $head($sql);
        let sql = columns!(|| sql, $($tail,)*);
        sql
    }};
    ($($F:expr),*) => {{
        move |mut sql: String| {
            write!(&mut sql, "WHERE ").unwrap();
            let sql = columns!(| sql, $($F,)*);
            sql
        }}
    }
}
pub use crate::conditions;

macro_rules! simple_sql_line {
    ($(#[$attr:meta])* $name:ident, $text:expr) => {
        $(#[$attr])*
        pub fn $name(sql: String) -> String {
            append($text)(sql)
        }
    };
}

macro_rules! simple_sql_fn {
    ($(#[$attr:meta])* $name:ident, $text:expr) => {
        $(#[$attr])*
        pub fn $name(arg: impl Fn(String) -> String) -> impl Fn(String) -> String {
            compose!(append($text), enclose(&arg))
        }
    };
}

simple_sql_line!(add, "ADD");
simple_sql_line!(all, "ALL");
simple_sql_line!(alter, "ALTER");
simple_sql_line!(alter_table, "ALTER TABLE");
simple_sql_fn!(and, "AND");
simple_sql_fn!(any, "ANY");
simple_sql_line!(as_sql, "AS");
simple_sql_line!(asc, "ASC");
simple_sql_fn!(avg, "AVG");
simple_sql_line!(boolean, "BOOL");
simple_sql_line!(between, "BETWEEN");
simple_sql_line!(ch, "CHAR");
simple_sql_line!(conflict, "CONFLICT");
simple_sql_line!(case, "CASE");
simple_sql_line!(constraint, "CONSTRAINT");
simple_sql_fn!(count, "COUNT");
simple_sql_line!(create, "CREATE");
simple_sql_line!(create_index, "CREATE INDEX");
simple_sql_line!(create_table, "CREATE TABLE");
simple_sql_line!(create_table_if_not_exists, "CREATE TABLE IF NOT EXISTS");
simple_sql_line!(create_unique_index, "CREATE UNIQUE INDEX");
simple_sql_line!(create_view, "CREATE VIEW");
simple_sql_line!(cross_join, "CROSS JOIN");
simple_sql_line!(cube, "CUBE");
simple_sql_line!(current_date, "CURRENT_DATE");
simple_sql_line!(current_time, "CURRENT_TIME");
simple_sql_fn!(current_time_fn, "CURRENT_TIME");
simple_sql_line!(date, "DATE");
simple_sql_line!(day, "DAY");
simple_sql_line!(default, "DEFAULT");
simple_sql_line!(delete, "DELETE");
simple_sql_line!(desc, "DESC");
simple_sql_line!(distinct, "DISTINCT");
simple_sql_line!(drop, "DROP");
simple_sql_line!(drop_column, "DROP COLUMN");
simple_sql_line!(drop_constraint, "DROP CONSTRAINT");
simple_sql_line!(drop_database, "DROP DATABASE");
simple_sql_line!(drop_default, "DROP DEFAULT");
simple_sql_line!(drop_index, "DROP INDEX");
simple_sql_line!(drop_table, "DROP TABLE");
simple_sql_line!(drop_view, "DROP VIEW");
simple_sql_line!(do_sql, "DO");
simple_sql_line!(except, "EXCEPT");
simple_sql_line!(exists, "EXISTS");
simple_sql_line!(extract, "EXTRACT");
simple_sql_line!(fetch, "FETCH");
simple_sql_line!(first, "FIRST");
simple_sql_line!(foreign, "FOREIGN");
simple_sql_line!(foreign_key, "FOREIGN KEY");
simple_sql_line!(from, "FROM");
simple_sql_line!(full_join, "FULL JOIN");
simple_sql_line!(full_outer_join, "FULL OUTER JOIN");
simple_sql_line!(group_by, "GROUP BY");
simple_sql_line!(having, "HAVING");
simple_sql_line!(if_sql, "IF");
simple_sql_line!(ilike, "ILIKE");
simple_sql_line!(in_sql, "IN");
simple_sql_line!(index, "INDEX");
simple_sql_line!(inner_join, "INNER JOIN");
simple_sql_line!(
    /// appends INSERT to the sql string
    /// sql -> "INSERT...sql"
    insert, "INSERT");
simple_sql_line!(
    /// appends INSERT INTO to the sql string
    /// sql -> "INSERT INTO...sql"
    insert_into, "INSERT INTO");
simple_sql_line!(
    /// appends INSERT INTO SELECT to the sql string
    /// sql -> "INSERT INTO...sql"
    insert_into_select, "INSERT INTO SELECT");
simple_sql_line!(intersect, "INTERSECT");
simple_sql_line!(
    /// appends "INTO"  onto the sql string
    /// sql -> "INTO...sql"
    into, "INTO");
simple_sql_line!(is, "IS");
simple_sql_line!(is_null, "IS NULL");
simple_sql_line!(is_not_null, "IS NOT NULL");
simple_sql_line!(join, "JOIN");
simple_sql_line!(join_cross, "CROSS JOIN");
simple_sql_line!(join_inner, "INNER JOIN");
simple_sql_line!(join_left, "LEFT JOIN");
simple_sql_line!(join_left_outer, "LEFT OUTER JOIN");
simple_sql_line!(join_right, "RIGHT JOIN");
simple_sql_line!(join_outer_right, "RIGHT OUTER JOIN");
simple_sql_line!(join_full, "FULL OUTER JOIN");
simple_sql_line!(key, "KEY");
simple_sql_line!(left_join, "LEFT JOIN");
simple_sql_line!(left_outer_join, "LEFT OUTER JOIN");
simple_sql_line!(like, "LIKE");
simple_sql_line!(limit, "LIMIT");
simple_sql_line!(localtime, "LOCALTIME");
simple_sql_line!(natural, "NATURAL");
simple_sql_line!(not, "NOT");
simple_sql_line!(not_like, "NOT LIKE");
simple_sql_line!(not_null, "NOT NULL");
simple_sql_line!(nothing, "NOTHING");
simple_sql_line!(now, "NOW()");
simple_sql_line!(null, "NULL");
simple_sql_fn!(max, "MAX");
simple_sql_fn!(min, "MIN");
simple_sql_line!(month, "MONTH");
simple_sql_line!(on, "ON");
simple_sql_line!(only, "ONLY");
simple_sql_line!(or, "OR");
simple_sql_line!(order_by, "ORDER BY");
simple_sql_line!(primary, "PRIMARY");
simple_sql_line!(primary_key, "PRIMARY KEY");
simple_sql_line!(references, "REFERENCES");
simple_sql_line!(returning, "RETURNING");
simple_sql_line!(right_join, "RIGHT JOIN");
simple_sql_line!(right_outer_join, "RIGHT OUTER JOIN");
simple_sql_line!(rollup, "ROLLUP");
simple_sql_line!(row, "ROW");
simple_sql_line!(select, "SELECT");
simple_sql_line!(select_distinct, "SELECT DISTINCT");
simple_sql_line!(select_into, "SELECT INTO");
simple_sql_line!(set, "SET");
simple_sql_line!(some, "SOME");
simple_sql_fn!(sum, "SUM");
simple_sql_line!(table, "TABLE");
simple_sql_line!(text, "TEXT");
simple_sql_line!(time, "TIME");
simple_sql_fn!(time_fn, "TIME");
simple_sql_line!(truncate_table, "TRUNCATE TABLE");
simple_sql_line!(union_sql, "UNION");
simple_sql_line!(union_all, "UNION ALL");
simple_sql_line!(unique, "UNIQUE");
simple_sql_line!(update, "UPDATE");
simple_sql_line!(using, "USING");
simple_sql_line!(varchar, "VARCHAR");
simple_sql_fn!(varchar_fn, "VARCHAR");
simple_sql_line!(
    /// appends "VALUES" to the sql string
    /// sql -> "VALUES...sql",
    values, "VALUES");
simple_sql_line!(view, "VIEW");
simple_sql_line!(union, "UNION");
simple_sql_line!(where_sql, "WHERE");
simple_sql_line!(year, "YEAR");

pub fn r_postgres_indexer<'a>(
    param: &'a (impl ToSql + Sync)
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>) -> (i32, Vec<&'a (dyn ToSql + Sync)>) {
    move |mut index, mut params| {
        index += 1;
        params.push(param);
        (index, params)
    }
}

pub fn prepare<'a>(col: &'a str) -> impl Fn(i32) -> (&'a str, i32) + 'a {
    move |index| {
        (&col, index + 1)
    }
}

pub fn join_text<'a: 'b, 'b>(
    sep: impl Fn(String) -> String + 'b, 
    list: &'a [&'a dyn Fn(String) -> String]) -> impl Fn(String) -> String + 'b {
    move |sql| {
        let sep = &sep;
        let (first, rest) = list.split_first().unwrap();
        let mut sql = first(sql);
        for c in rest {
            sql = compose!(sep, c)(sql);
        }
        sql
    }
}

pub fn columns<'a>(columns: &'a [ &'a dyn Fn(String) -> String ]) -> impl Fn(String) -> String + 'a {
    move |sql| { join_text(append(", "), columns)(sql) }
}

pub fn compose<'a>(args: &'a [ &'a dyn Fn(String) -> String ]) -> impl Fn(String) -> String + 'a {
    move |sql| { join_text(append(""), args)(sql) }
}

pub fn esc(string: impl Fn(String) -> String) -> impl Fn(String) -> String {
    compose!(append("'"), string, append("'"))
}

pub fn ormer<'a>(args: &'a [ &'a dyn Fn(String) -> String ]) -> impl Fn(String) -> String + 'a {
    move |sql| { join_text(append(" "), args)(sql) }
}


/// appends the "insert into (...) table values (...)" line to the sql string
/// fn(table, columns, args) -> fn(sql) -> "INSERT INTO table (columns) VALUES (args)...sql"
pub fn insert_values(
    table: impl Fn(String) -> String, 
    columns: impl Fn(String) -> String,
    args: impl Fn(String) -> String) -> impl Fn(String) -> String {
    ormer!(insert_into, table, enclose(&columns), values, enclose(&args))
}

pub fn is_null_fn(
    arg: impl Fn(String) -> String,
    default: impl Fn(String) -> String) -> impl Fn(String) -> String {
        let com = compose!(arg, append(","), default);
        compose!(append("ISNULL"), enclose(&com))
}

pub fn coalesce(
    arg: impl Fn(String) -> String,
    default: impl Fn(String) -> String) -> impl Fn(String) -> String {
        let com = compose!(arg, append(","), default);
        compose!(append("COALESCE"), enclose(&com))
}

pub fn update_args_dangerously(
    table: impl Fn(String) -> String,
    updates: impl Fn(String) -> String) -> impl Fn(String) -> String {
    ormer!(update, table, updates)
}

pub fn update_args(
    table: impl Fn(String) -> String,
    updates: impl Fn(String) -> String,
    conditions: impl Fn(String) -> String) -> impl Fn(String) -> String {
    ormer!(update, table, updates, conditions)
}

pub fn enclose<'a>(param: impl Fn(String) -> String + 'a) -> impl Fn(String) -> String + 'a {
    compose!(append("("), param, append(")"))
}

pub fn brackets(arg: impl Fn(String) -> String) -> impl Fn(String) -> String {
    compose!(append("["), arg, append("]"))
}

pub fn connect(
    a: impl Fn(String) -> String,
    b: impl Fn(String) -> String) -> impl Fn(String) -> String {
    compose!(a, append("."), b)
}

pub fn delete_rows_dangerously(table: impl Fn(String)  -> String) -> impl Fn(String) -> String {
    ormer!(delete, from, table)
}

pub fn delete_rows(
    table: impl Fn(String) -> String,
    conditions: impl Fn(String) -> String) -> impl Fn(String) -> String {
    ormer!(delete, from, table, conditions)
}

#[cfg(test)]
mod test {
    use postgres::{Client, NoTls};
    use super::*;
    use conditions::*;
    use operators::*;


    #[test]
    fn select_test<'b>() {

        let (id, args) = r_postgres_indexer(&30)(0, Vec::with_capacity(3));
        let (name, args) = r_postgres_indexer(&"foo")(id, args);
        let (rank, args) = r_postgres_indexer(&100)(name, args);

        let where_id_eqls_param = eqls("id", id);
        let users_table = append("users");

        let sql = ormer(&[
            &select, 
            &columns(&[ 
                &esc(append("name")), 
                &esc(append("rank")) 
            ]),
            &from, &users_table,
            &condition(&where_id_eqls_param)
        ])(String::with_capacity(100));
        assert_eq!("SELECT 'name', 'rank' FROM users WHERE id=$1", sql);

        fn try_to_query(client: &mut Client) {
            let id = 30;
            let name = "foo";
            let rank = 100;
            let (id, args) = r_postgres_indexer(&id)(0, Vec::with_capacity(3));
            let (name, args) = r_postgres_indexer(&name)(id, args);
            let (rank, args) = r_postgres_indexer(&rank)(name, args);
            let id_name_and_rank = |sql: String| {
                columns!(
                    esc(append("id")),
                    esc(append("name")),
                    esc(append("rank"))
                )(sql)
            };
            let users = append("users");
            let select_id_name_and_rank_from_users = ormer!(select, id_name_and_rank, from, users);
            let where_id_eqls_input = conditions!(
                eqls("id", id),
                eqls("name", name),
                eqls("rank", rank)
            );
            let sql = ormer!( 
                select_id_name_and_rank_from_users,
                where_id_eqls_input
            )(String::with_capacity(100));
            assert_eq!(
                "SELECT 'id', 'name', 'rank' FROM users WHERE id=$1 AND name=$2 AND rank=$3",
                sql
            );
            client.query(&sql, &args).unwrap();
        }

    }

}

