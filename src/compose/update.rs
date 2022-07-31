use postgres::types::{ToSql, FromSql};
use postgres::Client;

pub fn set<'a>(col: &'a str, param: &'a impl ToSql) -> 
impl Fn(i32, Vec<&'a dyn ToSql>) -> (i32, Vec<&'a dyn ToSql>, String) {
    move |index, mut params| {
        let sql = format!("SET {}=${}",col,index);
        params.push(param as &dyn ToSql);
        (index + 1, params, sql)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_test() {
        let (index, _params, sql) = set("name", &"fizz \"the buzz\" lightyear")(1, Vec::with_capacity(1));
        assert_eq!(2, index);
        assert_eq!("SET name=$1", sql)
    }

}
