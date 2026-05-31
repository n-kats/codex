use codex_protocol::models::ActivePermissionProfile;
use codex_protocol::models::PermissionProfile;
use codex_utils_absolute_path::AbsolutePathBuf;

/// Local snapshot of a permission profile plus its active-profile metadata.
///
/// This keeps the TUI call sites decoupled from the internal config resolver
/// types while preserving the legacy `active(...)` / `legacy(...)` constructors
/// used throughout the widget tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PermissionProfileSnapshot {
    permission_profile: PermissionProfile,
    active_permission_profile: Option<ActivePermissionProfile>,
    profile_workspace_roots: Vec<AbsolutePathBuf>,
}

impl PermissionProfileSnapshot {
    pub(crate) fn legacy(permission_profile: PermissionProfile) -> Self {
        Self {
            permission_profile,
            active_permission_profile: None,
            profile_workspace_roots: Vec::new(),
        }
    }

    pub(crate) fn active(
        permission_profile: PermissionProfile,
        active_permission_profile: ActivePermissionProfile,
    ) -> Self {
        Self {
            permission_profile,
            active_permission_profile: Some(active_permission_profile),
            profile_workspace_roots: Vec::new(),
        }
    }

    pub(crate) fn active_with_profile_workspace_roots(
        permission_profile: PermissionProfile,
        active_permission_profile: ActivePermissionProfile,
        profile_workspace_roots: Vec<AbsolutePathBuf>,
    ) -> Self {
        Self {
            permission_profile,
            active_permission_profile: Some(active_permission_profile),
            profile_workspace_roots,
        }
    }

    pub(crate) fn from_session_snapshot(
        permission_profile: PermissionProfile,
        active_permission_profile: Option<ActivePermissionProfile>,
    ) -> Self {
        Self {
            permission_profile,
            active_permission_profile,
            profile_workspace_roots: Vec::new(),
        }
    }

    pub(crate) fn permission_profile(&self) -> &PermissionProfile {
        &self.permission_profile
    }

    pub(crate) fn active_permission_profile(&self) -> Option<ActivePermissionProfile> {
        self.active_permission_profile.clone()
    }

    pub(crate) fn profile_workspace_roots(&self) -> &[AbsolutePathBuf] {
        self.profile_workspace_roots.as_slice()
    }
}
