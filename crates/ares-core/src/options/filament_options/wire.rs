use serde::{Serialize, Serializer, ser::SerializeMap};

use super::{FilamentOptions, gcode_source::wire::serialize_entries};

impl Serialize for FilamentOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(53))?;
        serialize_entries(&mut map, &self.gcode)?;
        map.end()
    }
}
