mod early;
mod late;
mod middle;

use serde::{Serialize, Serializer, ser::SerializeMap};

use super::FilamentOptions;

impl Serialize for FilamentOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(122))?;
        early::serialize_entries(&mut map, self)?;
        middle::serialize_entries(&mut map, self)?;
        late::serialize_entries(&mut map, self)?;
        map.end()
    }
}
