use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Node {
    pub name: String,

    pub lines_covered: usize,

    pub lines_missed: usize,

    pub lines_total: usize,

    pub coverage_percent: f32,

    #[serde(default, deserialize_with = "deserialize_kv_vec")]
    pub children: Vec<(String, Node)>,
}

fn deserialize_kv_vec<'de, D>(deserializer: D) -> Result<Vec<(String, Node)>, D::Error>
where
    D: Deserializer<'de>,
{
    let items = KeyValueVec::deserialize(deserializer)?;
    Ok(items.into())
}

#[derive(Debug)]
struct KeyValueVec<K, V>(Vec<(K, V)>);

impl<K, V> From<KeyValueVec<K, V>> for Vec<(K, V)> {
    fn from(src: KeyValueVec<K, V>) -> Self {
        src.0
    }
}

struct KeyValueVisitor<K, V> {
    marker: PhantomData<fn() -> KeyValueVec<K, V>>,
}

impl<K, V> KeyValueVisitor<K, V> {
    fn new() -> Self {
        KeyValueVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for KeyValueVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = KeyValueVec<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut items = Vec::with_capacity(access.size_hint().unwrap_or(1));
        while let Some((key, value)) = access.next_entry()? {
            items.push((key, value));
        }

        Ok(KeyValueVec(items))
    }
}

impl<'de, K, V> Deserialize<'de> for KeyValueVec<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyValueVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_map() {
        let res = deserialize_kv_vec(json!({
            "a": {
                "name": "a",
                "linesCovered": 12,
                "linesMissed": 34,
                "linesTotal": 46,
                "coveragePercent": 12.34
            },
            "b": {
                "name": "b",
                "linesCovered": 56,
                "linesMissed": 78,
                "linesTotal": 134,
                "coveragePercent": 56.78
            }
        }));

        assert!(res.is_ok());
        let values = res.unwrap();
        assert_eq!(values.first().map(|v| v.0.as_ref()), Some("a"));
        assert_eq!(values.get(1).map(|v| v.0.as_ref()), Some("b"));
    }
}
