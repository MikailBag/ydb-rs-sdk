use crate::errors::YdbError;
use crate::types::{Bytes, Value, ValueOptional};
use crate::{ValueList, YdbResult};
use itertools::Itertools;
use std::any::type_name;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec::IntoIter;

macro_rules! simple_convert {
    ($native_type:ty, $ydb_value_kind_first:path $(,$ydb_value_kind:path)* $(,)?) => {
        impl From<$native_type> for Value {
            fn from(value: $native_type)->Self {
                $ydb_value_kind_first(value)
            }
        }

        impl TryFrom<Value> for $native_type {
            type Error = YdbError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    $ydb_value_kind_first(val) => Ok(val.into()),
                    $($ydb_value_kind(val) => Ok(val.into()),)*
                    value => Err(YdbError::Convert(format!(
                        "failed to convert from {} to {}",
                        value.kind_static(),
                        type_name::<Self>(),
                    ))),
                }
            }
        }

        impl TryFrom<Value> for Option<$native_type> {
            type Error = YdbError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Optional(opt_val) => {
                        if let Err(err) = <$native_type as TryFrom<Value>>::try_from(opt_val.t) {
                            return Err(err);
                        };

                        match opt_val.value {
                            Some(val) => {
                                let res_val: $native_type = val.try_into()?;
                                Ok(Some(res_val))
                            }
                            None => Ok(None),
                        }
                    }
                    value => Ok(Some(value.try_into()?)),
                }
            }
        }

    };
}

impl<T: Into<Value> + Default> From<Option<T>> for Value {
    fn from(from_value: Option<T>) -> Self {
        let t = T::default().into();
        let value = match from_value {
            Some(val) => Some(val.into()),
            None => None,
        };

        return Value::Optional(Box::new(ValueOptional { t, value }));
    }
}

impl<T: Into<Value> + Default> FromIterator<T> for Value {
    fn from_iter<T2: IntoIterator<Item = T>>(iter: T2) -> Self {
        let t: Value = T::default().into();
        let values: Vec<Value> = iter.into_iter().map(|item| item.into()).collect();
        return Value::List(Box::new(ValueList { t, values }));
    }
}

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = YdbError>,
{
    type Error = YdbError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let value = match value {
            Value::List(inner) => inner,
            value => {
                return Err(YdbError::from_str(format!(
                    "can't convert from {} to Vec",
                    value.kind_static()
                )));
            }
        };

        // check list type compatible - for prevent false positive convert empty list
        let list_item_type = value.t.kind_static();
        if TryInto::<T>::try_into(value.t).is_err() {
            let vec_item_type = type_name::<i32>();
            return Err(YdbError::from_str(format!(
                "can't convert list item type '{}' to vec item type '{}'",
                list_item_type, vec_item_type
            )));
        };

        let res: Vec<T> = value
            .values
            .into_iter()
            .map(|item| item.try_into())
            .try_collect()?;
        return Ok(res);
    }
}

simple_convert!(i8, Value::Int8);
simple_convert!(u8, Value::Uint8);
simple_convert!(i16, Value::Int16, Value::Int8, Value::Uint8);
simple_convert!(u16, Value::Uint16, Value::Uint8);
simple_convert!(
    i32,
    Value::Int32,
    Value::Int16,
    Value::Uint16,
    Value::Int8,
    Value::Uint8,
);
simple_convert!(u32, Value::Uint32, Value::Uint16, Value::Uint8);
simple_convert!(
    i64,
    Value::Int64,
    Value::Int32,
    Value::Uint32,
    Value::Int16,
    Value::Uint16,
    Value::Int8,
    Value::Uint8,
);
simple_convert!(
    u64,
    Value::Uint64,
    Value::Uint32,
    Value::Uint16,
    Value::Uint8,
);
simple_convert!(
    String,
    Value::Utf8,
    Value::Json,
    Value::JsonDocument,
    Value::Yson
);
simple_convert!(
    Bytes,
    Value::String,
    Value::Utf8,
    Value::Json,
    Value::JsonDocument,
    Value::Yson
);
simple_convert!(f32, Value::Float);
simple_convert!(f64, Value::Double, Value::Float);
simple_convert!(Duration, Value::Timestamp, Value::Date, Value::DateTime);
