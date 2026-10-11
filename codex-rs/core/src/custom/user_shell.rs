use crate::session::turn_context::TurnContext;

pub(crate) fn no_inject(turn_context: &TurnContext) -> bool {
    turn_context.config.permissions.custom.user_shell_no_inject
}
