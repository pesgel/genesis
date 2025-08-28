use anyhow::Result;
use arrow::{
    datatypes::{DataType, Field, Schema},
    json::{ReaderBuilder, WriterBuilder, writer::JsonArray},
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fs::File, io::BufReader, sync::Arc};
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    username: String,
    #[serde(deserialize_with = "deserialize_string_date")]
    created_at: DateTime<Utc>,
    created_by: Option<String>,
    // finished: Vec<i32>,
}

fn main() -> Result<()> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("username", DataType::Utf8, false),
        Field::new("created_at", DataType::Date64, false),
        Field::new("created_by", DataType::Utf8, true),
        // Field::new(
        //     "finished",
        //     DataType::List(Arc::new(Field::new("finished", DataType::Int32, false))),
        //     true,
        // ),
    ]);
    let reader = BufReader::new(File::open("assets/user.ndjson")?);
    let reader = ReaderBuilder::new(Arc::new(schema)).build(reader)?;
    for batch in reader {
        let batch = batch?;
        let data: Vec<u8> = Vec::new();
        let mut writer = WriterBuilder::new()
            .with_explicit_nulls(true)
            .build::<_, JsonArray>(data);
        writer.write_batches(&[&batch])?;
        writer.finish()?;
        let data = writer.into_inner();
        // deserialize the data
        let users: Vec<User> = serde_json::from_slice(&data)?;
        for user in users {
            println!("{user:?}");
        }
    }
    Ok(())
}

fn deserialize_string_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    // 若时间字符串是带有时区的
    //    let s = String::deserialize(deserializer)?;
    //    let dt = DateTime::parse_from_rfc3339(&s)
    //         .map_err(serde::de::Error::custom)?
    //         .with_timezone(&Utc);
    // 无时区操作,先转换为NaiveDateTime
    let s = String::deserialize(deserializer)?;
    // format 2019-12-28T05:35:42.771
    let from: NaiveDateTime = s.parse().map_err(serde::de::Error::custom)?;
    let date_time = Utc.from_local_datetime(&from).unwrap();

    Ok(date_time)
}
//
// fn deserialize_string_date_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s = Option::<String>::deserialize(deserializer)?;
//     match s {
//         Some(s) => {
//             let from: NaiveDateTime = s.parse().map_err(serde::de::Error::custom)?;
//             let date_time = Utc.from_local_datetime(&from).unwrap();
//             Ok(Some(date_time))
//         }
//         None => Ok(None),
//     }
// }
