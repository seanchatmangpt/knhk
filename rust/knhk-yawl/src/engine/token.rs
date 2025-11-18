//! Token management for workflow execution
//!
//! Manages the distribution and movement of tokens in the workflow net.

use crate::core::Token;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Token manager
///
/// Manages tokens in the workflow net.
/// Uses minimal locking for performance (Covenant 5).
#[derive(Debug)]
pub struct TokenManager {
    /// Active tokens indexed by token ID
    active_tokens: Arc<Mutex<HashMap<String, Token>>>,

    /// Token queue for processing
    queue: Arc<Mutex<VecDeque<String>>>,
}

impl TokenManager {
    /// Create a new token manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_tokens: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Create a new token at the given position
    ///
    /// # Errors
    /// Returns error if token creation fails
    pub fn create_token(&self, position: impl Into<String>) -> crate::Result<String> {
        let token = Token::new(position);
        let token_id = token.id.clone();

        let mut active_tokens = self.active_tokens.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock active tokens: {}", e))
        })?;

        active_tokens.insert(token_id.clone(), token);

        let mut queue = self.queue.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock queue: {}", e))
        })?;

        queue.push_back(token_id.clone());

        tracing::debug!("Created token: {}", token_id);

        Ok(token_id)
    }

    /// Move a token to a new position
    ///
    /// # Errors
    /// Returns error if token not found or move fails
    pub fn move_token(
        &self,
        token_id: &str,
        new_position: impl Into<String>,
    ) -> crate::Result<()> {
        let new_position = new_position.into();

        let mut active_tokens = self.active_tokens.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock active tokens: {}", e))
        })?;

        if let Some(token) = active_tokens.get_mut(token_id) {
            token.move_to(new_position.clone());
            tracing::debug!("Moved token {} to {}", token_id, new_position);
            Ok(())
        } else {
            Err(crate::Error::Other(anyhow::anyhow!(
                "Token not found: {}",
                token_id
            )))
        }
    }

    /// Remove a token
    ///
    /// # Errors
    /// Returns error if token removal fails
    pub fn remove_token(&self, token_id: &str) -> crate::Result<()> {
        let mut active_tokens = self.active_tokens.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock active tokens: {}", e))
        })?;

        if active_tokens.remove(token_id).is_some() {
            tracing::debug!("Removed token: {}", token_id);
            Ok(())
        } else {
            Err(crate::Error::Other(anyhow::anyhow!(
                "Token not found: {}",
                token_id
            )))
        }
    }

    /// Get the next token from the queue
    ///
    /// # Errors
    /// Returns error if queue access fails
    pub fn next_token(&self) -> crate::Result<Option<String>> {
        let mut queue = self.queue.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock queue: {}", e))
        })?;

        Ok(queue.pop_front())
    }

    /// Get token count
    ///
    /// # Errors
    /// Returns error if lock acquisition fails
    pub fn token_count(&self) -> crate::Result<usize> {
        let active_tokens = self.active_tokens.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock active tokens: {}", e))
        })?;

        Ok(active_tokens.len())
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let manager = TokenManager::new();
        let token_id = manager.create_token("task1").unwrap();

        assert!(!token_id.is_empty());
        assert_eq!(manager.token_count().unwrap(), 1);
    }

    #[test]
    fn test_token_movement() {
        let manager = TokenManager::new();
        let token_id = manager.create_token("task1").unwrap();

        assert!(manager.move_token(&token_id, "task2").is_ok());
    }

    #[test]
    fn test_token_removal() {
        let manager = TokenManager::new();
        let token_id = manager.create_token("task1").unwrap();

        assert!(manager.remove_token(&token_id).is_ok());
        assert_eq!(manager.token_count().unwrap(), 0);
    }

    #[test]
    fn test_token_queue() {
        let manager = TokenManager::new();
        let token1 = manager.create_token("task1").unwrap();
        let token2 = manager.create_token("task2").unwrap();

        let next = manager.next_token().unwrap();
        assert_eq!(next, Some(token1));

        let next = manager.next_token().unwrap();
        assert_eq!(next, Some(token2));

        let next = manager.next_token().unwrap();
        assert_eq!(next, None);
    }
}
