mod rchain;
use rchain::{Blockchain, Block, Transaction, TransactionData, WorldState};
use std::borrow::BorrowMut;

fn main() {
    println!("Demo RChain Version 1\n---------");

    // Create a new Blockchain
    let mut bc = Blockchain::new();

    // Create an empty block (first block has no prev_block)
    let mut genesis = Block::new(None);

    let initial_users = vec!("alice", "bob");

    for user in initial_users {
        let create_transaction = Transaction::new(user.into(),
                                                 TransactionData::CreateUserAccount(user.into()),
                                                 0);

        let token_action = Transaction::new(user.into(),
                                            TransactionData::CreateTokens {receiver: user.into(), amount: 100_000_000},
                                            0);

        genesis.add_transaction(create_transaction);

        genesis.add_transaction(token_action);
    }

    let mut res = bc.append_block(genesis);
    //println!("Genesis block successfully added: {:?}", res);
    // println!("Full blockchain printout");
    // println!("{:#?}", bc);

    // Transfer 1 token from alice to bob
    let mut block2 = Block::new(bc.get_last_block_hash());
    block2.add_transaction(Transaction::new(
        "alice".into(),
        TransactionData::TransferTokens {to: "bob".into(), amount: 1}, 0));

    res = bc.append_block(block2);
    println!("Block added: {:?}", res);
    // println!("Full blockchain printout");
    // println!("{:#?}", bc);
    println!("Blockchain valid: {:?}", bc.check_validity());

    // Everything is fine until here

    // Now I write stuff
    println!("Number of tokens on chain: {:?}", bc.get_total_tokens());
    println!("Transactions for bob: {:?}", bc.get_transactions_for("bob".into()));
    println!("Transactions for alice: {:?}", bc.get_transactions_for("alice".into()));

    // In the next block, create 8 new users
    // As the accounts aren't on the chain yet, giving them tokens
    // will have to be on another block
    let new_users = ["carol", "dave", "ed", "frank", "gertrude", "ilhan"];
    let mut block3 = Block::new(bc.get_last_block_hash());

    for user in &new_users {
        let create_transaction = Transaction::new(user.to_string(),
                                                 TransactionData::CreateUserAccount(user.to_string()),
                                                 0);
        block3.add_transaction(create_transaction);
    }
    res = bc.append_block(block3);

    // Since this blockchain doesn't allow new tokens after genesis, alice will share
    let mut block4 = Block::new(bc.get_last_block_hash());
    for user in &new_users {
        let token_action = Transaction::new("alice".into(),
            TransactionData::TransferTokens {to: user.to_string(), amount: 5000}, 0);

        block4.add_transaction(token_action);
    }

    res = bc.append_block(block4);
    println!("Block added: {:?}", res);
    println!("Full blockchain printout");
    println!("{:#?}", bc);
    println!("Blockchain valid: {:?}", bc.check_validity());

    // Attack I: changing a transaction
    // Let's tamper the block chain. Maybe bob was not satisfied with the amount of coins alice sent
    // him, so he will tamper the blockchains transaction to transmit 100 Coins instead of 1

    // let's clone the current blockchain before tempering
    let mut bc_attack_1 = bc.clone();
    // get the transaction as mutable (second block, first transaction; the token transfer)
    let transaction_data = bc_attack_1.blocks[1].transactions[0].borrow_mut();

    // change the amount value of the transaction INSIDE the chain
    match transaction_data.record.borrow_mut() {
        &mut TransactionData::TransferTokens {to:_, ref mut amount} => {
            *amount = 100; // Actually change the value in place
        },

        _ => {} // We know that that recors is a TransferToken Action so we ignore the rest
    }

    println!("Changed transaction: {:?}", transaction_data.record);

    // Will print an error, since the blocks hash changes for the
    println!("Is the Blockchain still valid? {:#?}", bc_attack_1.check_validity());

    // Attack II: Changing transaction + updating the hash (increasing initial tokens in create
    // user action)
    let mut bc_attack_2 = bc.clone();

    // Alice tokens
    let transaction_data= bc_attack_2.blocks[0].transactions[1].borrow_mut();

    // change tokens
    match transaction_data.record.borrow_mut() {
        &mut TransactionData::CreateTokens {receiver: _, ref mut amount} => {
            *amount = 100_000_000_000; // Let's dont be small on that
        },
        _ => {} // We know that that record is a Token Create Action so we ignore the rest
    }

    // If we execute now, we'll see the same error as above, hashes dont match (this time 1st block)

    // Will print an error, since the blocks hash changes for the
    println!("Is the Blockchain still valid? {:#?}", bc_attack_2.check_validity());

    // But alice was smart, she also updated the first blocks' hash
    bc_attack_2.blocks[0].update_hash();

    // So the hash is correct now, however, block2 points now to sth which does not exists
    // Again, the blockchain is invalid but for a different reason
    println!("Is the Blockchain still valid? {:#?}", bc_attack_2.check_validity());


}
