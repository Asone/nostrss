use nostr_sdk::Keys;

use self::relay::Relay;

// pub mod config;
pub mod relay;
pub mod service;

// Helper trait.
pub trait NostrProfile {
    fn get_keys(&self) -> Keys;
    fn get_display_name(self) -> Option<String>;
    fn get_description(self) -> Option<String>;
    fn get_picture(self) -> Option<String>;
    fn get_banner(self) -> Option<String>;
    fn get_nip05(self) -> Option<String>;
    fn get_lud16(self) -> Option<String>;
    fn get_relays(&self) -> Vec<Relay>;
}
