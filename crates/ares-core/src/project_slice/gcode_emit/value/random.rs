use super::{Config, Value};

impl Config {
    /// PlaceholderParser.cpp:647-657,1708-1716: typed distributions over ContextData.
    pub(in crate::project_slice::gcode_emit) fn random(
        &self,
        args: &[Value],
    ) -> Result<Value, String> {
        let rng = self
            .random
            .as_ref()
            .ok_or("Random number generator not available in this context.")?;
        let [minimum, maximum] = args else {
            return Err("random requires two numeric arguments".to_owned());
        };
        let mut rng = rng.borrow_mut();
        let bounds = match (minimum.scalar(), maximum.scalar()) {
            (Value::Integer(minimum), Value::Integer(maximum)) if minimum <= maximum => {
                // Upstream expr::as_i and uniform_int_distribution<> use int32.
                return Ok(Value::Integer(i64::from(
                    rng.integer(*minimum as i32, *maximum as i32),
                )));
            }
            (Value::Number(minimum), Value::Number(maximum)) => (*minimum, *maximum),
            (Value::Integer(minimum), Value::Number(maximum)) => (*minimum as f64, *maximum),
            (Value::Number(minimum), Value::Integer(maximum)) => (*minimum, *maximum as f64),
            _ => return Err("invalid random arguments".to_owned()),
        };
        let (minimum, maximum) = bounds;
        if minimum > maximum {
            return Err("invalid random interval".to_owned());
        }
        // libstdc++ uniform_real_distribution<double>: canonical * (b-a) + a.
        Ok(Value::Number(rng.unit() * (maximum - minimum) + minimum))
    }
}
