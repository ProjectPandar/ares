mod early;
mod late;
mod middle;

use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProcessOptions;

impl Serialize for ProcessOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(275))?;
        early::serialize_entries(&mut map, &self.object, &self.region)?;
        middle::serialize_entries(&mut map, &self.object, &self.region)?;
        late::serialize_entries(&mut map, &self.object, &self.region)?;
        map.end()
    }
}
