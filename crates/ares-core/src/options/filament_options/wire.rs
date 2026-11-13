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
        let mut map = serializer.serialize_map(Some(139))?;
        early::serialize_entries(&mut map, self)?;
        middle::serialize_entries(&mut map, self)?;
        late::serialize_entries(&mut map, self)?;
        super::preset_source::serialize_entries(&mut map, &self.preset)?;
        map.end()
    }
}
