use crate::types::GameState;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::warn;

/// Thread-safe state tracker for all games managed by Cruce.
/// Unknown game names are implicitly in the `GameState::Idle` state.
#[derive(Debug, Default)]
pub struct GameStateTracker {
    states: Mutex<HashMap<String, GameState>>,
}

impl GameStateTracker {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Return the current lifecycle state for `game_name` (defaults to `GameState::Idle`).
    pub fn get(&self, game_name: &str) -> GameState {
        let lock = self.states.lock().unwrap_or_else(|p| p.into_inner());
        lock.get(game_name).copied().unwrap_or(GameState::Idle)
    }

    /// Attempt to atomically move `game_name` to `target` state.
    /// Returns `true` on success, or `false` if the transition is invalid.
    pub fn transition(&self, game_name: &str, target: GameState) -> bool {
        let mut lock = self.states.lock().unwrap_or_else(|p| p.into_inner());
        let current = lock.get(game_name).copied().unwrap_or(GameState::Idle);

        if !current.can_transition_to(target) {
            warn!(
                "Invalid state transition for '{}': {:?} -> {:?}",
                game_name, current, target
            );
            return false;
        }

        if target == GameState::Idle {
            lock.remove(game_name);
        } else {
            lock.insert(game_name.to_string(), target);
        }

        true
    }

    /// Unconditionally resets `game_name` to `GameState::Idle` (used in error/crash recovery).
    pub fn force_idle(&self, game_name: &str) {
        let mut lock = self.states.lock().unwrap_or_else(|p| p.into_inner());
        lock.remove(game_name);
    }

    /// Returns a snapshot list of all non-idle games.
    pub fn active_games(&self) -> Vec<(String, GameState)> {
        let lock = self.states.lock().unwrap_or_else(|p| p.into_inner());
        lock.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }
}
