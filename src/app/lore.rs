use crate::app::App;

impl App {
    pub fn unlock_lore(&mut self, key: &str) {
        self.unlocked_lore.insert(key.to_string());
    }

    #[allow(dead_code)]
    pub fn has_unlocked(&self, key: &str) -> bool {
        self.unlocked_lore.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lore_unlock_starts_empty() {
        let app = App::new_test(42);
        assert!(!app.has_unlocked("nihil_first_contact"));
    }

    #[test]
    fn test_lore_unlock_and_query() {
        let mut app = App::new_test(42);
        app.unlock_lore("nihil_first_contact");
        assert!(app.has_unlocked("nihil_first_contact"));
        assert!(!app.has_unlocked("solari_first_contact"));
    }

    #[test]
    fn test_lore_unlock_idempotent() {
        let mut app = App::new_test(42);
        app.unlock_lore("key");
        app.unlock_lore("key");
        assert!(app.has_unlocked("key"));
    }
}
