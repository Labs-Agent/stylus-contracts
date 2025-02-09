#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    block,
    prelude::*,
    evm,
    msg,
};

// Define events using Solidity ABI
sol! {
    event StatsUpdated(address indexed user, string stats, uint256 timestamp);
    
    error InvalidStats();
}

// Define storage using Solidity style
sol_storage! {
    #[entrypoint]
    pub struct UserStats {
        // Mapping from user address to array of stats entries
        mapping(address => StatEntry[]) userStats;
    }

    pub struct StatEntry {
        string stats_json;
        uint256 timestamp;
    }
}

#[public]
impl UserStats {
    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }

    pub fn update_stats(&mut self, stats_json: String) -> Result<bool, Vec<u8>> {
        let user = msg::sender();
        let current_time = U256::from(block::timestamp());

        let mut user_stats = self.userStats.setter(user);
        let current_len = user_stats.len();

        if current_len >= 50 {
            // Shift elements to the left, removing the oldest entry
            let mut next_stats = Vec::new();
            let mut next_timestamps = Vec::new();
            
            for i in 1..50 {
                if let Some(next) = user_stats.get(i) {
                    next_stats.push(next.stats_json.get_string());
                    next_timestamps.push(next.timestamp.get());
                }
            }

            for i in 1..50 {
                if let Some(mut current) = user_stats.get_mut(i - 1) {
                    current.stats_json.set_str(&next_stats[i - 1]);
                    current.timestamp.set(next_timestamps[i - 1]);
                }
            }
            // Add new entry at the end (49th position)
            if let Some(mut last) = user_stats.get_mut(49) {
                last.stats_json.set_str(&stats_json);
                last.timestamp.set(current_time);
            }
        } else {
            // If less than 50 entries, just add new entry
            let mut new_entry = user_stats.grow();
            new_entry.stats_json.set_str(&stats_json);
            new_entry.timestamp.set(current_time);
        }

        // Emit event
        evm::log(StatsUpdated {
            user,
            stats: stats_json,
            timestamp: current_time,
        });

        Ok(true)
    }

    pub fn get_recent_stats(&self, user: Address) -> Result<Vec<(String, U256)>, Vec<u8>> {
        let current_time = U256::from(block::timestamp());
        let two_minutes_ago = current_time.saturating_sub(U256::from(120)); // 2 minutes in seconds

        let user_stats = self.userStats.get(user);
        let mut recent_stats = Vec::new();

        // Collect stats from last 2 minutes
        for i in 0..user_stats.len() {
            if let Some(entry) = user_stats.get(i) {
                if entry.timestamp.get() >= two_minutes_ago {
                    recent_stats.push((
                        entry.stats_json.get_string(),
                        entry.timestamp.get(),
                    ));
                }
            }
        }

        Ok(recent_stats)
    }
}

impl Default for UserStats {
    fn default() -> Self {
        unsafe { Self::new(Default::default(), Default::default()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_stats_management() {
        let mut contract = UserStats::default();
        let user = Address::default();

        // Test adding stats
        let stat1 = "{\"cpu\":80,\"memory\":70}".to_string();
        let stat2 = "{\"cpu\":85,\"memory\":75}".to_string();
        
        contract.update_stats(stat1.clone()).unwrap();
        contract.update_stats(stat2.clone()).unwrap();

        // Test getting recent stats
        let recent_stats = contract.get_recent_stats(user).unwrap();
        assert!(!recent_stats.is_empty());
        assert_eq!(recent_stats.len(), 2);
        assert_eq!(recent_stats[0].0, stat1);
        assert_eq!(recent_stats[1].0, stat2);
    }

    #[test]
    fn test_max_entries() {
        let mut contract = UserStats::default();
        let user = Address::default();

        // Add 51 entries
        for i in 0..51 {
            let stat = format!("{{\"test\":{}}}", i);
            contract.update_stats(stat).unwrap();
        }

        // Check that only 50 entries are kept
        let stats = contract.get_recent_stats(user).unwrap();
        assert_eq!(stats.len(), 50);
        
        // Verify the oldest entry was removed
        let first_stat = &stats[49].0;
        assert_eq!(first_stat, "{\"test\":1}");
    }
}
