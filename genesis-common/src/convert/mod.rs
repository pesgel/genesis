use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

pub fn copy<S, D>(source: &S) -> Result<D, Box<dyn Error>>
where
    S: Serialize,
    D: DeserializeOwned + Serialize + Default,
{
    // 序列化源对象为 `serde_json::Value`
    let value = serde_json::to_value(source)?;
    // 获取目标类型的字段类型映射
    let type_map = get_type_map::<D>()?;
    // 自定义转换 `serde_json::Value` 为目标类型
    let converted_value = convert_value(value, type_map)?;
    // 将转换后的值反序列化为目标类型
    let destination = serde_json::from_value(converted_value)?;
    Ok(destination)
}

pub fn get_type_map<D>() -> Result<HashMap<String, Value>, Box<dyn Error>>
where
    D: Serialize + DeserializeOwned + Default,
{
    // 用 Default 构造结构体，避免缺字段报错
    let empty: D = Default::default();
    // 转换为 JSON 对象
    let Value::Object(obj) = serde_json::to_value(empty)? else {
        return Err("Expected a JSON object".into());
    };
    let mut type_map = HashMap::with_capacity(obj.len() * 2);
    for (key, val) in obj {
        type_map.insert(key.clone(), val.clone());
    }
    Ok(type_map)
}

fn convert_value(value: Value, type_map: HashMap<String, Value>) -> Result<Value, Box<dyn Error>> {
    let Value::Object(obj) = value else {
        return Err("Expected a JSON object".into());
    };

    let mut new_map = Map::with_capacity(obj.len());

    for (k, ty) in type_map {
        let camel = snake_to_camel(&k);
        let snake = camel_to_snake(&k);
        let value = match obj.get(&k) {
            None => match obj.get(&camel) {
                None => match obj.get(&snake) {
                    None => ty,
                    Some(d) => convert_single_value(d, &ty)?,
                },
                Some(v) => convert_single_value(v, &ty)?,
            },
            Some(v) => convert_single_value(v, &ty)?,
        };
        new_map.insert(k, value.clone());
        new_map.insert(camel, value.clone());
        new_map.insert(snake, value.clone());
    }
    Ok(Value::Object(new_map))
}

// 处理单个值的类型转换
fn convert_single_value(value: &Value, field_type: &Value) -> Result<Value, Box<dyn Error>> {
    // 若原始值为null,设置为默认值
    if value.is_null() {
        return Ok(field_type.to_owned());
    }

    match field_type {
        Value::String(_) => match value {
            Value::String(_) => Ok(value.clone()),
            Value::Number(n) => Ok(Value::String(n.to_string())),
            Value::Bool(b) => Ok(Value::String(b.to_string())),
            _ => Ok(Value::String(String::new())),
        },
        Value::Number(_) => match value {
            Value::Number(_) => Ok(value.clone()),
            Value::String(s) => serde_json::Number::from_str(s)
                .map(Value::Number)
                .or(Ok(field_type.to_owned())),
            _ => Ok(field_type.to_owned()),
        },
        Value::Bool(_) => match value {
            Value::Bool(_) => Ok(value.clone()),
            Value::String(s) => match s.as_ref() {
                "true" | "1" => Ok(Value::Bool(true)),
                "false" | "0" => Ok(Value::Bool(false)),
                _ => Ok(Value::Bool(false)),
            },
            _ => Ok(Value::Bool(false)),
        },
        _ => Ok(value.clone()),
    }
}

pub fn camel_to_snake(input: &str) -> String {
    let mut snake_case = String::with_capacity(input.len());
    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                snake_case.push('_');
            }
            snake_case.extend(c.to_lowercase());
        } else {
            snake_case.push(c);
        }
    }
    snake_case
}

pub fn snake_to_camel(input: &str) -> String {
    let mut camel_case = String::with_capacity(input.len());
    let mut uppercase_next = false;
    for (i, c) in input.chars().enumerate() {
        if c == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            camel_case.extend(c.to_uppercase());
            uppercase_next = false;
        } else if i == 0 {
            camel_case.push(c); // 首字母保留小写
        } else {
            camel_case.push(c);
        }
    }
    camel_case
}
