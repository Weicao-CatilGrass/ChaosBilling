use std::collections::BTreeMap;

use crate::{
    bill::{Bills, SplitResult, SplitResultItem},
    error::BillSplitError,
    who::Who,
};

pub fn calculate_from(item: Bills) -> Result<SplitResult, BillSplitError> {
    // Validate input data
    precheck(&item)?;

    // Calculate each person's net balance and original transactions
    let (direct_transactions, items) = calculate_balances_and_transactions(&item);

    // Generate the simplest result: net settlement between each pair
    let final_result = calculate_net_settlements(&direct_transactions);

    // Add "Total" reason to final_result
    let mut items = items;
    add_total_reason(&mut items);

    Ok(SplitResult {
        items,
        final_result,
    })
}

fn precheck(item: &Bills) -> Result<(), BillSplitError> {
    for (_, bill_item) in &item.items {
        // Check if the paid amount is negative
        if bill_item.paid < 0.0 {
            return Err(BillSplitError::NegativePaidAmount);
        }

        // Check for duplicate members in the split list
        let mut seen = std::collections::HashSet::new();
        for person in &bill_item.split {
            if !seen.insert(person) {
                return Err(BillSplitError::DuplicateSplitMembers);
            }
        }
    }
    Ok(())
}

fn calculate_balances_and_transactions(
    item: &Bills,
) -> (
    BTreeMap<(Who, Who), f64>,
    BTreeMap<Who, Vec<SplitResultItem>>,
) {
    let mut direct_transactions: BTreeMap<(Who, Who), f64> = BTreeMap::new();
    let mut items: BTreeMap<Who, Vec<SplitResultItem>> = BTreeMap::new();

    for (_, bill_item) in &item.items {
        let who_paid = &bill_item.who_paid;
        let paid = bill_item.paid;
        let split_count = bill_item.split.len() as f64;

        if split_count == 0.0 {
            continue;
        }

        // Round
        let share = (paid / split_count * 100.0).round() / 100.0;

        // Calculate the amount each person should pay
        for person in &bill_item.split {
            // If the payer is also in the split list, deduct their own share
            if person != who_paid {
                // Record direct transaction
                let key = (person.clone(), who_paid.clone());
                *direct_transactions.entry(key).or_insert(0.0) += share;

                // Add to full record
                let bill_result_item = SplitResultItem {
                    payee: who_paid.clone(),
                    bill: share,
                    reason: bill_item.reason.clone(),
                };

                items
                    .entry(person.clone())
                    .or_insert_with(Vec::new)
                    .push(bill_result_item);
            }
        }
    }

    (direct_transactions, items)
}

fn calculate_net_settlements(
    direct_transactions: &BTreeMap<(Who, Who), f64>,
) -> BTreeMap<(Who, Who), f64> {
    let mut final_result: BTreeMap<(Who, Who), f64> = BTreeMap::new();

    // First, calculate net amounts for each transaction pair
    let mut net_transactions: BTreeMap<(Who, Who), f64> = BTreeMap::new();
    for ((from, to), amount) in direct_transactions {
        let key = (from.clone(), to.clone());
        *net_transactions.entry(key).or_insert(0.0) += amount;
    }

    // Now process net transactions, ensuring correct direction
    let mut processed_pairs = std::collections::HashSet::new();

    for ((from, to), amount) in &net_transactions {
        // Create a normalized transaction pair key (sorted alphabetically)
        let pair_key = if from < to {
            (from.clone(), to.clone())
        } else {
            (to.clone(), from.clone())
        };

        // If this pair has already been processed, skip it
        if processed_pairs.contains(&pair_key) {
            continue;
        }
        processed_pairs.insert(pair_key.clone());

        // Check for reverse transaction
        let reverse_key = (to.clone(), from.clone());
        if let Some(reverse_amount) = net_transactions.get(&reverse_key) {
            // There is a reverse transaction, calculate net amount
            let net_amount = *amount - *reverse_amount;

            if net_amount > 0.0001 {
                // from owes to (net)
                final_result.insert(
                    (from.clone(), to.clone()),
                    (net_amount * 100.0).round() / 100.0,
                );
            } else if net_amount < -0.0001 {
                // to owes from (net)
                final_result.insert(
                    (to.clone(), from.clone()),
                    (-net_amount * 100.0).round() / 100.0,
                );
            }
            // If net amount is close to 0, don't add any transaction
        } else {
            // No reverse transaction, add directly
            if *amount > 0.0001 {
                final_result.insert(
                    (from.clone(), to.clone()),
                    (*amount * 100.0).round() / 100.0,
                );
            }
        }
    }

    final_result
}

fn add_total_reason(items: &mut BTreeMap<Who, Vec<SplitResultItem>>) {
    for (_payer, bills_list) in items.iter_mut() {
        for bill in bills_list {
            bill.reason = format!("{} (Total)", bill.reason);
        }
    }
}
