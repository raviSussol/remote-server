use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum SyncType {
    Delete,
    Update,
    Insert,
}
#[derive(Debug)]
struct SyncRecord {
    sync_type: SyncType,
    record_type: String,
    data: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct SimpleTranslatedRecord {
    id: String,
    #[serde(rename = "string_to_change")]
    new_name: Option<String>,
    #[serde(rename = "number_to_change")]
    new_number: Option<i32>,
    this_one_might_be_null: Option<bool>,
}

impl<'a> SimpleSyncTranslation<'a> for SimpleTranslatedRecord {
    type RecordType = SimpleTranslatedRecord;

    fn get_original_type() -> &'static str {
        "original_record"
    }
}

impl Mutatable for SimpleTranslatedRecord {
    fn make_sync_mutation(&self, sync_type: &SyncType) -> Query {
        match sync_type {
            SyncType::Delete => Query {
                query: "this will be a DELETE query, not a string",
                record: Box::new(self),
            },
            _ => Query {
                query: "this will be a UPDATE query, not a string",
                record: Box::new(self),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct SimpleTranslatedRecord2 {
    id: String,
    #[serde(rename = "was_something_else")]
    something: Option<String>,
}

impl<'a> SimpleSyncTranslation<'a> for SimpleTranslatedRecord2 {
    type RecordType = SimpleTranslatedRecord2;

    fn get_original_type() -> &'static str {
        "original_record2"
    }
}

impl Mutatable for SimpleTranslatedRecord2 {
    fn make_sync_mutation(&self, sync_type: &SyncType) -> Query {
        match sync_type {
            SyncType::Delete => Query {
                query: "this will be a DELETE query, not a string",
                record: Box::new(self),
            },
            _ => Query {
                query: "this will be a UPDATE query, not a string",
                record: Box::new(self),
            },
        }
    }
}

fn get_record_for_translation(sync_record: &SyncRecord) -> Result<Box<dyn Mutatable>, String> {
    if let Some(record) = SimpleTranslatedRecord::try_get_record(&sync_record) {
        return Ok(Box::new(record));
    }

    if let Some(record) = SimpleTranslatedRecord2::try_get_record(&sync_record) {
        return Ok(Box::new(record));
    }

    Err("Cannot find matching translation".to_string())
}

#[derive(Debug)]
struct Query<'a> {
    query: &'static str,
    record: Box<&'a dyn std::fmt::Debug>,
}

trait Mutatable {
    fn make_sync_mutation(&self, sync_type: &SyncType) -> Query;
}

fn main() {
    // Insert or Update original_record -> SimpleTranslatedRecord
    let sync_record = SyncRecord {
        sync_type: SyncType::Insert,
        record_type: "original_record".to_owned(),
        data: r#"{
            "id": "ABC",
            "string_to_change": "CBA",
            "number_to_change": 12
        }"#
        .to_owned(),
    };

    let translated_record = get_record_for_translation(&sync_record).unwrap();

    println!(
        "{:#?}",
        &translated_record.make_sync_mutation(&sync_record.sync_type)
    );

    // Delete original_record -> SimpleTranslatedRecord
    let sync_record = SyncRecord {
        sync_type: SyncType::Delete,
        record_type: "original_record".to_owned(),
        data: r#"{
            "id": "ABC"
        }"#
        .to_owned(),
    };

    let translated_record = get_record_for_translation(&sync_record).unwrap();

    println!(
        "{:#?}",
        &translated_record.make_sync_mutation(&sync_record.sync_type)
    );

    // Insert or Update original_record2 -> SimpleTranslatedRecord2
    let sync_record = SyncRecord {
        sync_type: SyncType::Update,
        record_type: "original_record2".to_owned(),
        data: r#"{
            "id": "CBA",
            "was_something_else": "nemaste"
        }"#
        .to_owned(),
    };

    let translated_record = get_record_for_translation(&sync_record).unwrap();

    println!(
        "{:#?}",
        &translated_record.make_sync_mutation(&sync_record.sync_type)
    );

    // And it can turn back to the original

    println!(
        "{}",
        serde_json::to_string_pretty(&SimpleTranslatedRecord {
            id: "ABC".to_owned(),
            new_name: Some("value".to_owned()),
            new_number: None,
            this_one_might_be_null: None
        })
        .unwrap()
    )
}

trait SimpleSyncTranslation<'a> {
    type RecordType: Deserialize<'a> + Mutatable + SimpleSyncTranslation<'a>;
    fn get_original_type() -> &'static str;

    fn try_get_record(sync_record: &'a SyncRecord) -> Option<Self::RecordType> {
        match sync_record.record_type.as_str() {
            record_type if record_type == Self::RecordType::get_original_type() => {
                Some(serde_json::from_str::<'a, Self::RecordType>(&sync_record.data).unwrap())
            }

            _ => None,
        }
    }
}
