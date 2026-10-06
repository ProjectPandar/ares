use super::{
    BoundedProjectUsage, BoundedResolvedProjectConfig, CandidatePass, CandidateSnapshot,
    GroupedPrintObjects, MaterializationPass, MaterializationSnapshot, NormalizeFdm2Pass,
    NormalizeFdm2Snapshot, PhaseEvent, PhaseObserver, Project, ProjectFdmNormalizationKey,
    ProjectSettings, ResolutionTrace, ResolvedProjectObject, SliceError, UsagePass, UsageSnapshot,
    resolve_with_observer,
};

impl PhaseObserver for ResolutionTrace {
    fn normalize_fdm_1(&mut self, _settings: &ProjectSettings) {
        self.events.push(PhaseEvent::NormalizeFdm1);
    }

    fn normalize_fdm_2(
        &mut self,
        pass: NormalizeFdm2Pass,
        (object_count, used_filament_count): (usize, usize),
        changed_keys: &[ProjectFdmNormalizationKey],
        _settings: &ProjectSettings,
    ) {
        self.events.push(PhaseEvent::NormalizeFdm2(pass));
        self.normalize_fdm_2.push(NormalizeFdm2Snapshot {
            pass,
            object_count,
            used_filament_count,
            changed_keys: changed_keys.to_vec(),
            enable_prime_tower: _settings.process.print.enable_prime_tower.0,
        });
    }

    fn materialize(
        &mut self,
        pass: MaterializationPass,
        source: &ProjectSettings,
        materialized: &ProjectSettings,
    ) {
        self.events.push(PhaseEvent::Materialize(pass));
        self.materializations.push(MaterializationSnapshot {
            pass,
            source_retraction_length: source.project.gcode.retraction_length.0.len(),
            materialized_retraction_length: materialized.project.gcode.retraction_length.0.len(),
            source_filament_retraction_length: source
                .filament
                .retract_overrides
                .filament_retraction_length
                .len(),
            materialized_filament_retraction_length: materialized
                .filament
                .retract_overrides
                .filament_retraction_length
                .len(),
            source_retracts_all_false: all_retracts_false(source),
            materialized_retracts_all_false: all_retracts_false(materialized),
            source_filament_retracts_all_false: all_filament_retracts_false(source),
            materialized_filament_retracts_all_false: all_filament_retracts_false(materialized),
            source_enable_prime_tower: source.process.print.enable_prime_tower.0,
            materialized_enable_prime_tower: materialized.process.print.enable_prime_tower.0,
        });
    }

    fn validate_and_group(&mut self, pass: MaterializationPass, _groups: &GroupedPrintObjects) {
        self.events.push(PhaseEvent::ValidateAndGroup(pass));
    }

    fn resolve_object_shells(&mut self, _resolved: &[ResolvedProjectObject]) {
        self.events.push(PhaseEvent::ResolveObjectShells);
    }

    fn candidates(&mut self, pass: CandidatePass, resolved: &[ResolvedProjectObject]) {
        self.events.push(PhaseEvent::Candidates(pass));
        self.candidates.push(CandidateSnapshot {
            pass,
            outer_wall_selectors: resolved
                .iter()
                .flat_map(|object| &object.layer_candidates)
                .flat_map(|candidate| &candidate.model_parts)
                .map(|part| part.region.outer_wall_filament_id.0)
                .collect(),
        });
    }

    fn usage(
        &mut self,
        pass: UsagePass,
        usage: &BoundedProjectUsage,
        resolved: &[ResolvedProjectObject],
        (source_settings, wipe_settings): (&ProjectSettings, &ProjectSettings),
    ) {
        self.events.push(PhaseEvent::Usage(pass));
        self.usages.push(UsageSnapshot {
            pass,
            supported_used_filaments: usage.supported_used_filaments.clone(),
            model_part_count: resolved
                .iter()
                .flat_map(|object| &object.layer_candidates)
                .map(|candidate| candidate.model_parts.len())
                .sum(),
            support_object_count: resolved
                .iter()
                .filter(|object| object.object.enable_support.0)
                .count(),
            source_enable_prime_tower: source_settings.process.print.enable_prime_tower.0,
            wipe_enable_prime_tower: wipe_settings.process.print.enable_prime_tower.0,
            wipe_selector: wipe_settings.process.print.wipe_tower_filament.0,
        });
    }

    fn resolve_views(&mut self, _settings: &ProjectSettings) {
        self.events.push(PhaseEvent::ResolveViews);
        self.view_resolutions += 1;
    }
}

fn all_retracts_false(settings: &ProjectSettings) -> bool {
    let retracts = &settings.project.print.retract_when_changing_layer.0;
    retracts.iter().all(|value| !value.0)
}

fn all_filament_retracts_false(settings: &ProjectSettings) -> bool {
    settings
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer
        .iter()
        .all(|value| matches!(value, crate::Nullable::Value(crate::OrcaBool(false))))
}

pub(crate) fn resolve_bounded_project_config_with_trace(
    project: &Project,
) -> Result<(BoundedResolvedProjectConfig, ResolutionTrace), SliceError> {
    let mut trace = ResolutionTrace::default();
    let resolved = resolve_with_observer(project, &mut trace)?;
    Ok((resolved, trace))
}
