use crate::{bill::Bills, calc::calculate_from};

#[test]
fn test_no_zero_amount_transactions() {
    let mut bills = Bills::default();

    // Create some bills where some transaction amounts might be 0
    // A pays 30, split among A, B, C (10 each)
    bills.add_bill("A", "Lunch", 30., vec!["A", "B", "C"]);
    // B pays 30, split among A, B (15 each)
    bills.add_bill("B", "Coffee", 30., vec!["A", "B"]);

    let result = calculate_from(bills);
    assert!(result.is_ok(), "calculate should be success");
    let result = result.unwrap();

    // Check if there are any transactions with amount 0 in the complete record (items)
    let all_items = result.get_all_bills();
    for (payer, bills_list) in all_items {
        for bill in bills_list {
            assert_ne!(
                bill.bill, 0.0,
                "There should be no transactions with amount 0 in the complete record: {} to {} amount is 0",
                payer, bill.payee
            );
        }
    }

    // Check if there are any transactions with amount 0 in the simplified result (final_result)
    let final_result = result.get_final_result();
    for ((payer, payee), amount) in final_result {
        assert_ne!(
            *amount, 0.0,
            "There should be no transactions with amount 0 in the simplified result: {} to {} amount is 0",
            payer, payee
        );
    }

    // Verify transaction count
    // Should have: C->A (10), A->B (5) after netting
    assert_eq!(
        final_result.len(),
        2,
        "There should be 2 non-zero transactions"
    );
    assert!(
        final_result.contains_key(&("C".into(), "A".into())),
        "Should contain transaction C->A"
    );
    assert!(
        final_result.contains_key(&("A".into(), "B".into())),
        "Should contain transaction A->B"
    );

    // Verify specific amounts
    let c_to_a = final_result.get(&("C".into(), "A".into())).unwrap();
    assert_eq!(*c_to_a, 10.0, "C should pay A 10");

    let a_to_b = final_result.get(&("A".into(), "B".into())).unwrap();
    assert_eq!(*a_to_b, 5.0, "A should pay B 5");
}

#[test]
fn test_zero_amount_edge_cases() {
    let mut bills = Bills::default();

    // Test perfectly balanced case: A and B prepay the same amount for each other
    // A prepays 20 for A, B (10 each)
    bills.add_bill("A", "Dinner", 20., vec!["A", "B"]);
    // B prepays 20 for A, B (10 each)
    bills.add_bill("B", "Movie", 20., vec!["A", "B"]);

    let result = calculate_from(bills);
    assert!(result.is_ok(), "calculate should be success");
    let result = result.unwrap();

    // Check complete record
    let all_items = result.get_all_bills();
    for (payer, bills_list) in all_items {
        for bill in bills_list {
            assert_ne!(
                bill.bill, 0.0,
                "There should be no transactions with amount 0 in the complete record: {} to {} amount is 0",
                payer, bill.payee
            );
        }
    }

    // Check simplified result - should be empty because all transactions cancel out
    let final_result = result.get_final_result();
    assert_eq!(
        final_result.len(),
        0,
        "The simplified result should be empty because all transaction amounts are 0"
    );

    // Verify no transactions are included
    assert!(
        !final_result.contains_key(&("A".into(), "B".into())),
        "Should not contain A->B zero amount transaction"
    );
    assert!(
        !final_result.contains_key(&("B".into(), "A".into())),
        "Should not contain B->A zero amount transaction"
    );
}

#[test]
fn test_items_count() {
    let mut bills = Bills::default();

    // Add 3 bill items
    let id1 = bills.add_bill("A", "Lunch", 30., vec!["A", "B"]);
    let id2 = bills.add_bill("B", "Coffee", 20., vec!["B", "C"]);
    let id3 = bills.add_bill("C", "Snack", 15., vec!["A", "C"]);

    // Verify items count
    assert_eq!(bills.get_all_items().len(), 3, "Should have 3 items");

    // Verify each ID exists
    assert!(bills.contains_item(&id1), "Item 1 should exist");
    assert!(bills.contains_item(&id2), "Item 2 should exist");
    assert!(bills.contains_item(&id3), "Item 3 should exist");

    // Verify count after deleting one item
    let removed = bills.delete_item(&id2);
    assert!(removed.is_some(), "Should remove item 2");
    assert_eq!(
        bills.get_all_items().len(),
        2,
        "Should have 2 items after removal"
    );

    // Verify deleted item no longer exists
    assert!(
        !bills.contains_item(&id2),
        "Item 2 should not exist after removal"
    );

    // Verify count after clearing
    bills.clear_items();
    assert_eq!(
        bills.get_all_items().len(),
        0,
        "Should have 0 items after clear"
    );
}

#[test]
fn test_result() {
    let mut bills = Bills::default();

    // Define data
    bills.add_bill("A", "BBQ", 90., vec!["A", "B", "C"]);
    bills.add_bill("B", "Water", 21., vec!["A", "B", "C"]);

    // Calculate
    let result = calculate_from(bills);

    // Check result
    assert!(result.is_ok(), "calculate should be success");

    let result = result.unwrap();

    // Verify split results
    let c_to_a = result
        .get_final_result_item("C".into(), "A".into())
        .expect("Item C to A should be exist");
    assert_eq!(c_to_a, 30.0, "C should pay A 30 for BBQ");

    let c_to_b = result
        .get_final_result_item("C".into(), "B".into())
        .expect("Item C to B should be exist");
    assert_eq!(c_to_b, 7.0, "C should pay B 7 for Water");

    let b_to_a = result
        .get_final_result_item("B".into(), "A".into())
        .expect("Item B to A should be exist");
    assert_eq!(b_to_a, 23.0, "B should pay A 23 (30 - 7)");

    // Verify count
    let final_result = result.get_final_result();
    assert_eq!(final_result.len(), 3, "Should have exactly 3 transactions");
}

#[test]
fn test_complex_bills() {
    let mut bills = Bills::default();

    // A prepays 50 for B and C, B and C should each pay A 25
    bills.add_bill("A", "Dinner", 50., vec!["B", "C"]);

    let result = calculate_from(bills);
    assert!(result.is_ok(), "calculate should be success");
    let result = result.unwrap();

    let b_to_a = result
        .get_final_result_item("B".into(), "A".into())
        .expect("Item B to A should be exist");
    assert_eq!(b_to_a, 25.0, "B should pay A 25");

    let c_to_a = result
        .get_final_result_item("C".into(), "A".into())
        .expect("Item C to A should be exist");
    assert_eq!(c_to_a, 25.0, "C should pay A 25");

    let final_result = result.get_final_result();
    assert_eq!(final_result.len(), 2, "Should have exactly 2 transactions");
}

#[test]
fn test_unrelated_bills() {
    let mut bills = Bills::default();

    // A prepays 30 split among A, B, C, each should pay A 10
    // B prepays 30 split among A, B, each should pay B 15
    // Final result: C only has transaction with A, not with B
    bills.add_bill("A", "Lunch", 30., vec!["A", "B", "C"]);
    bills.add_bill("B", "Coffee", 30., vec!["A", "B"]);

    let result = calculate_from(bills);
    assert!(result.is_ok(), "calculate should be success");
    let result = result.unwrap();

    // Verify C only has transaction with A, not with B
    let c_to_a = result.get_final_result_item("C".into(), "A".into());
    assert!(c_to_a.is_some(), "C should pay A");
    assert_eq!(c_to_a.unwrap(), 10.0, "C should pay A 10");

    let c_to_b = result.get_final_result_item("C".into(), "B".into());
    assert!(c_to_b.is_none(), "C should not have any transaction with B");

    // Verify transaction between A and B
    let a_to_b = result.get_final_result_item("A".into(), "B".into());
    assert!(a_to_b.is_some(), "A should pay B");
    assert_eq!(a_to_b.unwrap(), 5.0, "A should pay B 5 (15 - 10)");

    // Verify total transaction count
    let final_result = result.get_final_result();
    assert_eq!(final_result.len(), 2, "Should have exactly 2 transactions");
}

#[test]
fn test_duplicate_split_members() {
    let mut bills = Bills::default();

    // Duplicate members in split list, should return Error
    bills.add_bill("Alice", "Lunch", 60., vec!["Bob", "Bob", "Charlie"]);

    let result = calculate_from(bills);
    assert!(
        result.is_err(),
        "Should return error for duplicate split members"
    );
}

#[test]
fn test_negative_paid_amount() {
    let mut bills = Bills::default();

    // Negative prepaid amount, should return Error
    bills.add_bill("Alice", "Refund?", -30., vec!["Alice", "Bob"]);

    let result = calculate_from(bills);
    assert!(
        result.is_err(),
        "Should return error for negative paid amount"
    );
}

#[test]
fn test_rounding() {
    let mut bills = Bills::default();

    // Test rounding: 51.0333333333 => 51.00, 51.599999999999 => 52.00
    // 100 / 3 = 33.333..., each should pay 33.33, payer gets back 66.67
    bills.add_bill("Alice", "Concert", 100., vec!["Alice", "Bob", "Charlie"]);

    let result = calculate_from(bills);
    assert!(result.is_ok(), "calculate should be success");
    let result = result.unwrap();

    let bob_to_alice = result
        .get_final_result_item("Bob".into(), "Alice".into())
        .expect("Item Bob to Alice should be exist");
    // 33.333... rounded to 2 decimal places => 33.33
    assert_eq!(bob_to_alice, 33.33, "Bob should pay Alice 33.33");

    let charlie_to_alice = result
        .get_final_result_item("Charlie".into(), "Alice".into())
        .expect("Item Charlie to Alice should be exist");
    assert_eq!(charlie_to_alice, 33.33, "Charlie should pay Alice 33.33");

    // Another test: 51.599999999999 => 52.00
    let mut bills2 = Bills::default();
    bills2.add_bill("Bob", "Dinner", 51.6, vec!["Alice", "Bob"]); // 51.6 / 2 = 25.8

    let result2 = calculate_from(bills2);
    assert!(result2.is_ok(), "calculate should be success");
    let result2 = result2.unwrap();

    let alice_to_bob = result2
        .get_final_result_item("Alice".into(), "Bob".into())
        .expect("Item Alice to Bob should be exist");
    // 25.8 rounded to 2 decimal places => 25.80
    assert_eq!(alice_to_bob, 25.8, "Alice should pay Bob 25.8");
}

#[test]
fn test_empty_bills() {
    let bills = Bills::default();
    let result = calculate_from(bills);
    assert!(
        result.is_ok(),
        "calculate should be success for empty bills"
    );
    let result = result.unwrap();

    assert_eq!(
        result.get_all_bills().len(),
        0,
        "Items should be empty for empty bills"
    );
    assert_eq!(
        result.get_final_result().len(),
        0,
        "Final result should be empty for empty bills"
    );
}

#[test]
fn test_single_person_bill() {
    let mut bills = Bills::default();

    // Single person bill: paying for oneself
    bills.add_bill("Alice", "Personal", 50., vec!["Alice"]);

    let result = calculate_from(bills);
    assert!(
        result.is_ok(),
        "calculate should be success for single person bill"
    );
    let result = result.unwrap();

    // Single person bill should not generate any transactions
    assert_eq!(
        result.get_final_result().len(),
        0,
        "Should have no transactions for single person bill"
    );
}

#[test]
fn test_split_not_include_payer() {
    let mut bills = Bills::default();

    // Payer not included in split list
    bills.add_bill("Alice", "Gift", 100., vec!["Bob", "Charlie"]);

    let result = calculate_from(bills);
    assert!(
        result.is_ok(),
        "calculate should be success when payer not in split"
    );
    let result = result.unwrap();

    // Bob and Charlie should each pay Alice 50
    let bob_to_alice = result
        .get_final_result_item("Bob".into(), "Alice".into())
        .expect("Bob should pay Alice");
    assert_eq!(bob_to_alice, 50.0, "Bob should pay Alice 50");

    let charlie_to_alice = result
        .get_final_result_item("Charlie".into(), "Alice".into())
        .expect("Charlie should pay Alice");
    assert_eq!(charlie_to_alice, 50.0, "Charlie should pay Alice 50");
}

#[test]
fn test_large_number_of_people() {
    let mut bills = Bills::default();

    // Test large group scenario
    let people = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    bills.add_bill("A", "Group Dinner", 1000., people.clone());

    let result = calculate_from(bills);
    assert!(
        result.is_ok(),
        "calculate should be success for large group"
    );
    let result = result.unwrap();

    // Each person should pay A 100 (1000/10)
    for person in &people {
        if *person != "A" {
            let amount = result
                .get_final_result_item(person.to_string().into(), "A".into())
                .expect(&format!("{} should pay A", person));
            assert_eq!(amount, 100.0, "{} should pay A 100", person);
        }
    }

    assert_eq!(
        result.get_final_result().len(),
        9,
        "Should have 9 transactions"
    );
}
