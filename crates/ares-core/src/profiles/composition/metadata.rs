use super::MergedProfileMetadata;

#[derive(Clone, Debug, PartialEq)]
pub struct ProfileGroupMetadata {
    inherits_group: Option<Vec<String>>,
    compatible_machine_expression_group: Option<Vec<String>>,
    compatible_process_expression_group: Option<Vec<String>>,
}

impl ProfileGroupMetadata {
    pub fn inherits_group(&self) -> Option<&[String]> {
        self.inherits_group.as_deref()
    }

    pub fn compatible_machine_expression_group(&self) -> Option<&[String]> {
        self.compatible_machine_expression_group.as_deref()
    }

    pub fn compatible_process_expression_group(&self) -> Option<&[String]> {
        self.compatible_process_expression_group.as_deref()
    }

    pub(super) fn from_profiles(
        process: &MergedProfileMetadata,
        filaments: &[MergedProfileMetadata],
        machine: &MergedProfileMetadata,
    ) -> Self {
        let inherits_group = positional_group(
            std::iter::once(process)
                .chain(filaments)
                .chain(std::iter::once(machine))
                .map(|metadata| metadata.inherits().unwrap_or_default().to_owned()),
        );
        let compatible_machine_expression_group =
            positional_group(std::iter::once(process).chain(filaments).map(|metadata| {
                metadata
                    .compatible_printers_condition()
                    .unwrap_or_default()
                    .to_owned()
            }));
        let compatible_process_expression_group =
            positional_group(filaments.iter().map(|metadata| {
                metadata
                    .compatible_prints_condition()
                    .unwrap_or_default()
                    .to_owned()
            }));

        Self {
            inherits_group,
            compatible_machine_expression_group,
            compatible_process_expression_group,
        }
    }
}

fn positional_group(values: impl IntoIterator<Item = String>) -> Option<Vec<String>> {
    let values = values.into_iter().collect::<Vec<_>>();
    values
        .iter()
        .any(|value| !value.is_empty())
        .then_some(values)
}
