use codex_protocol::config_types::CollaborationModeMask;
use codex_protocol::config_types::ModeKind;

use crate::model_catalog::ModelCatalog;
use codex_core::models_manager::manager::ModelsManager;

pub(crate) trait CollaborationModeCatalog {
    fn list_collaboration_modes(&self) -> Vec<CollaborationModeMask>;
}

impl CollaborationModeCatalog for ModelCatalog {
    fn list_collaboration_modes(&self) -> Vec<CollaborationModeMask> {
        ModelCatalog::list_collaboration_modes(self)
    }
}

impl CollaborationModeCatalog for ModelsManager {
    fn list_collaboration_modes(&self) -> Vec<CollaborationModeMask> {
        ModelsManager::list_collaboration_modes(self)
    }
}

impl<T> CollaborationModeCatalog for &T
where
    T: CollaborationModeCatalog + ?Sized,
{
    fn list_collaboration_modes(&self) -> Vec<CollaborationModeMask> {
        (**self).list_collaboration_modes()
    }
}

fn filtered_presets(model_catalog: &impl CollaborationModeCatalog) -> Vec<CollaborationModeMask> {
    model_catalog
        .list_collaboration_modes()
        .into_iter()
        .filter(|mask| mask.mode.is_some_and(ModeKind::is_tui_visible))
        .collect()
}

pub(crate) fn presets_for_tui(
    model_catalog: &impl CollaborationModeCatalog,
) -> Vec<CollaborationModeMask> {
    filtered_presets(model_catalog)
}

pub(crate) fn default_mask(
    model_catalog: &impl CollaborationModeCatalog,
) -> Option<CollaborationModeMask> {
    let presets = filtered_presets(model_catalog);
    presets
        .iter()
        .find(|mask| mask.mode == Some(ModeKind::Default))
        .cloned()
        .or_else(|| presets.into_iter().next())
}

pub(crate) fn mask_for_kind(
    model_catalog: &impl CollaborationModeCatalog,
    kind: ModeKind,
) -> Option<CollaborationModeMask> {
    if !kind.is_tui_visible() {
        return None;
    }
    filtered_presets(model_catalog)
        .into_iter()
        .find(|mask| mask.mode == Some(kind))
}

/// Cycle to the next collaboration mode preset in list order.
pub(crate) fn next_mask(
    model_catalog: &impl CollaborationModeCatalog,
    current: Option<&CollaborationModeMask>,
) -> Option<CollaborationModeMask> {
    let presets = filtered_presets(model_catalog);
    if presets.is_empty() {
        return None;
    }
    let current_kind = current.and_then(|mask| mask.mode);
    let next_index = presets
        .iter()
        .position(|mask| mask.mode == current_kind)
        .map_or(0, |idx| (idx + 1) % presets.len());
    presets.get(next_index).cloned()
}

pub(crate) fn default_mode_mask(
    model_catalog: &impl CollaborationModeCatalog,
) -> Option<CollaborationModeMask> {
    mask_for_kind(model_catalog, ModeKind::Default)
}

pub(crate) fn plan_mask(
    model_catalog: &impl CollaborationModeCatalog,
) -> Option<CollaborationModeMask> {
    mask_for_kind(model_catalog, ModeKind::Plan)
}
