use std::fmt::Write;
use postgres::types::ToSql;

pub fn eqls<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}=${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn neqls<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}<>${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn lt<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}<${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn lte<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}<=${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn gt<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}>${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn gte<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{}>=${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn like<'a: 'b, 'b>(col: &'b str, param: &'a (impl ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        write!(&mut sql, "{} LIKE ${}", col, index).unwrap();
        params.push(param as &(dyn ToSql + Sync));
        (index + 1, params, sql)
    }
}

pub fn between<'a: 'b, 'b>(col: &'b str, param1: &'a (dyn ToSql + Sync), param2: &'a (dyn ToSql + Sync)) -> 
impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, mut params, mut sql| {
        let index1 = index;
        let index2 = index + 1;
        write!(&mut sql, "{} BETWEEN ${} AND ${}", col, index1, index2).unwrap();
        params.push(param1);
        params.push(param2);
        (index2 + 1, params, sql)
    }
}

pub fn not<'a, C>(
    cond: impl Fn(i32, Vec<&'a (dyn ToSql +  Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
    move |index, params, mut sql| {
        write!(&mut sql, "NOT ").unwrap();
        let (index, params, sql) = cond(index, params, sql);
        (index, params, sql)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eqls_test() {
        assert_eq!(
            "id=$1",
            eqls("id", &1)(1, Vec::with_capacity(1), String::with_capacity(5)).2);
    }

}
