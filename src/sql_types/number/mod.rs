// pub mod uuid;

use crate::{
    filters::{
        Sqlize,
        InFilterValue,
        Escapable,
        // Gather,
    },
    helper_functions::*
};
use serde::Deserialize;
use std::{
    collections::{
        BTreeMap
    },
};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct NumberFilter<T: Escapable> {
    pub equals: Option<T>,
    pub not: Option<T>,
    pub lt: Option<T>,
    pub lte: Option<T>,
    pub gt: Option<T>,
    pub gte: Option<T>,
    // pub contains: Option<T>,
    #[serde(rename="in")]
    pub is_in: Option<InFilterValue<T>>,
}

impl<T: Escapable> NumberFilter<T> {

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
        return "".to_string()
    }
}

impl<T: Escapable> Sqlize for NumberFilter<T> {
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

impl Escapable for Uuid {}

// impl<T: Sqlize> Display for NumberFilter<T> {

// }

// BIT(size)	A bit-value type. The number of bits per value is specified in size. The size parameter can hold a value from 1 to 64. The default value for size is 1.
// TINYINT(size)	A very small integer. Signed range is from -128 to 127. Unsigned range is from 0 to 255. The size parameter specifies the maximum display width (which is 255)
// BOOL	Zero is considered as false, nonzero values are considered as true.
// BOOLEAN	Equal to BOOL
// SMALLINT(size)	A small integer. Signed range is from -32768 to 32767. Unsigned range is from 0 to 65535. The size parameter specifies the maximum display width (which is 255)
// MEDIUMINT(size)	A medium integer. Signed range is from -8388608 to 8388607. Unsigned range is from 0 to 16777215. The size parameter specifies the maximum display width (which is 255)
// INT(size)	A medium integer. Signed range is from -2147483648 to 2147483647. Unsigned range is from 0 to 4294967295. The size parameter specifies the maximum display width (which is 255)
// INTEGER(size)	Equal to INT(size)
// BIGINT(size)	A large integer. Signed range is from -9223372036854775808 to 9223372036854775807. Unsigned range is from 0 to 18446744073709551615. The size parameter specifies the maximum display width (which is 255)
// FLOAT(size, d)	A floating point number. The total number of digits is specified in size. The number of digits after the decimal point is specified in the d parameter. This syntax is deprecated in MySQL 8.0.17, and it will be removed in future MySQL versions
// FLOAT(p)	A floating point number. MySQL uses the p value to determine whether to use FLOAT or DOUBLE for the resulting data type. If p is from 0 to 24, the data type becomes FLOAT(). If p is from 25 to 53, the data type becomes DOUBLE()
// DOUBLE(size, d)	A normal-size floating point number. The total number of digits is specified in size. The number of digits after the decimal point is specified in the d parameter
// DOUBLE PRECISION(size, d)	 
// DECIMAL(size, d)	An exact fixed-point number. The total number of digits is specified in size. The number of digits after the decimal point is specified in the d parameter. The maximum number for size is 65. The maximum number for d is 30. The default value for size is 10. The default value for d is 0.
// DEC(size, d)

pub type UuidFilter = NumberFilter<Uuid>;
pub type TinyIntFilter = NumberFilter<i8>;
pub type BoolFilter = NumberFilter<bool>;
pub type SmallIntFilter = NumberFilter<i16>;
pub type SmallSerial = SmallIntFilter;
pub type MediumIntFilter = NumberFilter<i32>;
pub type IntFilter = NumberFilter<i32>;
pub type SerialFilter = IntFilter;
pub type BigIntFilter = NumberFilter<i64>;
pub type BigSerial = BigIntFilter;
pub type FloatFilter = NumberFilter<f32>;
pub type RealFilter = FloatFilter;
pub type DoubleFilter = NumberFilter<f64>;
pub type OidFilter = NumberFilter<u32>;
pub type Bytea<'a> = NumberFilter<Vec<u8>>;
// pub type Bytea = NumberFilter<Vec<u8>>;

// DATE	A date. Format: YYYY-MM-DD. The supported range is from '1000-01-01' to '9999-12-31'
// DATETIME(fsp)	A date and time combination. Format: YYYY-MM-DD hh:mm:ss. The supported range is from '1000-01-01 00:00:00' to '9999-12-31 23:59:59'. Adding DEFAULT and ON UPDATE in the column definition to get automatic initialization and updating to the current date and time
// TIMESTAMP(fsp)	A timestamp. TIMESTAMP values are stored as the number of seconds since the Unix epoch ('1970-01-01 00:00:00' UTC). Format: YYYY-MM-DD hh:mm:ss. The supported range is from '1970-01-01 00:00:01' UTC to '2038-01-09 03:14:07' UTC. Automatic initialization and updating to the current date and time can be specified using DEFAULT CURRENT_TIMESTAMP and ON UPDATE CURRENT_TIMESTAMP in the column definition
// TIME(fsp)	A time. Format: hh:mm:ss. The supported range is from '-838:59:59' to '838:59:59'
// YEAR	A year in four-digit format. Values allowed in four-digit format: 1901 to 2155, and 0000.
// MySQL 8.0 does not support year in two-digit format.

// pub type DateFilter = NumberFilter<>