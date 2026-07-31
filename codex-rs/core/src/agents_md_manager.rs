use crate::agents_md::LoadedAgentsMd;
use crate::agents_md::load_project_instructions;
use crate::config::Config;
use crate::environment_selection::TurnEnvironmentSnapshot;
use codex_extension_api::UserInstructions;
use codex_protocol::protocol::TurnEnvironmentSelection;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Owns the inputs and cached result of AGENTS.md discovery for a session.
pub(crate) struct AgentsMdManager {
    user_instructions: Option<UserInstructions>,
    cache: Mutex<AgentsMdCache>,
}

#[derive(Default)]
struct AgentsMdCache {
    selections: Option<Vec<TurnEnvironmentSelection>>,
    project_doc_paths: Option<Vec<AbsolutePathBuf>>,
    loaded: Option<Arc<LoadedAgentsMd>>,
}

impl AgentsMdManager {
    pub(crate) fn new(user_instructions: Option<UserInstructions>) -> Self {
        Self {
            user_instructions: user_instructions
                .filter(|instructions| !instructions.text.trim().is_empty()),
            cache: Mutex::new(AgentsMdCache::default()),
        }
    }

    #[tracing::instrument(name = "agents_md.refresh", skip_all)]
    pub(crate) async fn refresh(&self, config: &Config, environments: &TurnEnvironmentSnapshot) {
        let selections = environments.to_selections();
        let project_doc_paths = config.project_doc_paths.clone();
        let cache = self.cache.lock().await;
        if cache.selections.as_ref() == Some(&selections)
            && cache.project_doc_paths.as_ref() == Some(&project_doc_paths)
        {
            return;
        }
        drop(cache);

        let loaded =
            load_project_instructions(config, self.user_instructions.clone(), environments)
                .await
                .map(Arc::new);
        let mut cache = self.cache.lock().await;
        cache.selections = Some(selections);
        cache.project_doc_paths = Some(project_doc_paths);
        cache.loaded = loaded;
    }

    pub(crate) async fn get_loaded(&self) -> Option<Arc<LoadedAgentsMd>> {
        self.cache.lock().await.loaded.clone()
    }

    pub(crate) fn user_instructions(&self) -> Option<UserInstructions> {
        self.user_instructions.clone()
    }
}
