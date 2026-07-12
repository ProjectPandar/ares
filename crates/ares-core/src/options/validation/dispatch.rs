use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

use super::super::SliceOptions;

impl SliceOptions {
    pub fn validate_print_config(
        &self,
        under_cli: bool,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        match self.printer_technology()? {
            PrinterTechnology::Fff => self.validate_fff_options(under_cli),
            PrinterTechnology::Sla => Ok(BTreeMap::new()),
        }
    }

    fn printer_technology(&self) -> Result<PrinterTechnology, SliceError> {
        match self.values().get("printer_technology") {
            None => Ok(PrinterTechnology::Fff),
            Some(Value::String(value)) if value == "FFF" => Ok(PrinterTechnology::Fff),
            Some(Value::String(value)) if value == "SLA" => Ok(PrinterTechnology::Sla),
            Some(Value::String(_)) => Err(SliceError::InvalidInput(
                "printer_technology must be FFF or SLA".to_owned(),
            )),
            Some(_) => Err(SliceError::InvalidInput(
                "printer_technology must be a string".to_owned(),
            )),
        }
    }
}

enum PrinterTechnology {
    Fff,
    Sla,
}
