use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{display::SimpleTable, string_vec, who::Who};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Bills {
    pub items: BTreeMap<String, BillItem>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BillItem {
    pub who_paid: Who,
    pub reason: String,
    pub paid: f64,
    pub split: Vec<Who>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SplitResult {
    pub items: BTreeMap<Who, Vec<SplitResultItem>>,
    pub final_result: BTreeMap<(Who, Who), f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitResultItem {
    pub payee: Who,
    pub bill: f64,
    pub reason: String,
}

impl Bills {
    /// Add a new bill item
    pub fn add_bill(&mut self, who_paid: &str, reason: &str, paid: f64, split: Vec<&str>) -> Uuid {
        let item = BillItem {
            who_paid: who_paid.into(),
            reason: reason.to_string(),
            paid,
            split: split.into_iter().map(|s| s.into()).collect(),
        };
        self.add_item(item)
    }

    /// Add a new bill item
    pub fn add_item(&mut self, item: BillItem) -> Uuid {
        let id = Uuid::new_v4();
        self.items.insert(id.to_string(), item);
        id
    }

    /// Get a bill item by ID (immutable reference)
    pub fn get_item(&self, id: &Uuid) -> Option<&BillItem> {
        self.items.get(&id.to_string())
    }

    /// Get a bill item by ID (mutable reference)
    pub fn get_item_mut(&mut self, id: &Uuid) -> Option<&mut BillItem> {
        self.items.get_mut(&id.to_string())
    }

    /// Update the bill item with the specified ID
    pub fn update_item(&mut self, id: &Uuid, item: BillItem) -> bool {
        if let std::collections::btree_map::Entry::Occupied(mut e) =
            self.items.entry(id.to_string())
        {
            e.insert(item);
            true
        } else {
            false
        }
    }

    /// Delete the bill item with the specified ID
    pub fn delete_item(&mut self, id: &Uuid) -> Option<BillItem> {
        self.items.remove(&id.to_string())
    }

    /// Get all bill items
    pub fn get_all_items(&self) -> &BTreeMap<String, BillItem> {
        &self.items
    }

    /// Check if a bill item with the specified ID exists
    pub fn contains_item(&self, id: &Uuid) -> bool {
        self.items.contains_key(&id.to_string())
    }

    /// Clear all bill items
    pub fn clear_items(&mut self) {
        self.items.clear();
    }
}

impl SplitResult {
    /// Add a bill (who pays whom, amount, reason)
    pub fn add_bill(&mut self, payer: Who, payee: Who, amount: f64, reason: String) {
        let result_item = SplitResultItem {
            payee,
            bill: amount,
            reason,
        };

        self.items.entry(payer).or_default().push(result_item);
    }

    /// Get all bill items for a specified payer (immutable reference)
    pub fn get_bills(&self, payer: &Who) -> Option<&Vec<SplitResultItem>> {
        self.items.get(payer)
    }

    /// Get all bill items for a specified payer (mutable reference)
    pub fn get_bills_mut(&mut self, payer: &Who) -> Option<&mut Vec<SplitResultItem>> {
        self.items.get_mut(payer)
    }

    /// Update the bill list for a specified payer
    pub fn update_bills(
        &mut self,
        payer: Who,
        bills: Vec<SplitResultItem>,
    ) -> Option<Vec<SplitResultItem>> {
        self.items.insert(payer, bills)
    }

    /// Delete all bill items for a specified payer
    pub fn delete_bills(&mut self, payer: &Who) -> Option<Vec<SplitResultItem>> {
        self.items.remove(payer)
    }

    /// Get all bill items for all payers
    pub fn get_all_bills(&self) -> &BTreeMap<Who, Vec<SplitResultItem>> {
        &self.items
    }

    /// Check if bill items exist for a specified payer
    pub fn contains_payer(&self, payer: &Who) -> bool {
        self.items.contains_key(payer)
    }

    /// Clear all bill items
    pub fn clear_bills(&mut self) {
        self.items.clear();
    }
    /// Set the simplified result
    pub fn set_final_result(&mut self, result: BTreeMap<(Who, Who), f64>) {
        self.final_result = result;
    }

    /// Get the simplified result (immutable reference)
    pub fn get_final_result(&self) -> &BTreeMap<(Who, Who), f64> {
        &self.final_result
    }

    /// Get the simplified result (mutable reference)
    pub fn get_final_result_mut(&mut self) -> &mut BTreeMap<(Who, Who), f64> {
        &mut self.final_result
    }

    /// Clear the simplified result
    pub fn clear_final_result(&mut self) {
        self.final_result.clear();
    }

    /// Get a specific item from the simplified result (who pays whom, returns Option<f64>)
    pub fn get_final_result_item(&self, payer: Who, payee: Who) -> Option<f64> {
        self.final_result.get(&(payer, payee)).copied()
    }
}

impl Bills {
    pub fn table(self) -> String {
        let mut table = SimpleTable::new(string_vec![
            "#", "Who", "|", "Paid", "|", "Split", "|", "Reason"
        ]);
        let mut items: Vec<_> = self.items.into_iter().collect();
        items.sort_by(|a, b| {
            b.1.paid
                .partial_cmp(&a.1.paid)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (_, items) in items {
            let split = items
                .split
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            table.push_item(string_vec![
                "",
                items.who_paid,
                "|",
                items.paid,
                "|",
                split,
                "|",
                items.reason
            ]);
        }
        table.to_string()
    }

    pub fn from_table_str(table_str: impl Into<String>) -> Bills {
        let mut bills = Bills::default();
        let table_str = table_str.into();

        for line in table_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                continue;
            }

            let who_paid = parts[0];
            let paid_str = parts[1];
            let split_str = parts[2];
            let reason = parts[3];

            let paid = paid_str.parse::<f64>().unwrap_or(0.0);

            let split: Vec<&str> = split_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            bills.add_bill(who_paid, reason, paid, split);
        }

        bills
    }

    pub fn get_members(&self) -> Vec<String> {
        let mut members = std::collections::HashSet::new();

        for item in self.items.values() {
            members.insert(item.who_paid.to_string());
            for who in &item.split {
                members.insert(who.to_string());
            }
        }

        let mut result: Vec<String> = members.into_iter().collect();
        result.sort();
        result
    }
}
