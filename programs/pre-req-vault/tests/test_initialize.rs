use {
    anchor_lang::{
        prelude::msg, solana_program::instruction::Instruction,
        system_program::ID as SYSTEM_PROGRAM_ID, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

const REGISTRATION_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM");

fn setup() -> (LiteSVM, Keypair) {
    let program_id = pre_req_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/pre_req_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    // Dumped from devnet: `solana program dump TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM tests/fixtures/registration.so`
    let registration = include_bytes!("../../../tests/fixtures/registration.so");
    svm.add_program(REGISTRATION_PROGRAM_ID, registration)
        .unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    (svm, payer)
}

/// Reads the `github` field from a registration `ApplicationAccount`:
/// 8 discriminator + 32 user + 1 bump + 1 pre_req_ts + 1 pre_req_rs, then a borsh string.
fn registered_github(data: &[u8]) -> String {
    let offset = 8 + 32 + 1 + 1 + 1;
    let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    String::from_utf8(data[offset + 4..offset + 4 + len].to_vec()).unwrap()
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &pre_req_vault::id());

    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &pre_req_vault::id());

    let (application_account, _) =
        Pubkey::find_program_address(&[b"prereqs", user.as_ref()], &REGISTRATION_PROGRAM_ID);

    // Initialize
    let init_ix = Instruction {
        program_id: pre_req_vault::id(),
        accounts: pre_req_vault::accounts::Initialize {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: pre_req_vault::instruction::Initialize {}.data(),
    };

    let message = Message::new(&[init_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx1 = svm.send_transaction(transaction).unwrap();

    msg!("Initialize transaction successful");
    msg!("Tx Signature: {}", tx1.signature);

    let vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let vault_state =
        pre_req_vault::state::VaultState::try_deserialize(&mut vault_state_account.data.as_ref())
            .unwrap();

    assert_eq!(vault_state.vault_bump, vault_bump);
    assert_eq!(vault_state.state_bump, state_bump);

    // Deposit 1 Sol

    let deposit_amount: u64 = 1_000_000_000;

    let deposit_ix = Instruction {
        program_id: pre_req_vault::id(),
        accounts: pre_req_vault::accounts::Deposit {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: pre_req_vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };

    let message = Message::new(&[deposit_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction2 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx2 = svm.send_transaction(transaction2).unwrap();

    msg!("Deposit transaction successful");
    msg!("Tx Signature: {}", tx2.signature);

    let vault_balance_after_deposit = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(vault_balance_after_deposit, deposit_amount);

    // Withdraw (also registers the GitHub handle via CPI)

    let withdraw_amount: u64 = 500_000_000;

    let withdraw_ix = Instruction {
        program_id: pre_req_vault::id(),
        accounts: pre_req_vault::accounts::Withdraw {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            application_account,
            application_program: REGISTRATION_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: pre_req_vault::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };

    let message = Message::new(&[withdraw_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction3 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx3 = svm.send_transaction(transaction3).unwrap();

    msg!("Withdraw transaction successful");
    msg!("Tx Signature: {}", tx3.signature);

    let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap();
    assert_eq!(vault_balance_after_withdraw, deposit_amount - withdraw_amount);

    let application = svm.get_account(&application_account).unwrap();
    assert_eq!(application.owner, REGISTRATION_PROGRAM_ID);
    assert_eq!(&application.data[8..40], user.as_ref());
    assert_eq!(
        registered_github(&application.data),
        pre_req_vault::GITHUB_USERNAME
    );

    // Close
    let close_amount = svm.get_balance(&vault_pda).unwrap();

    let close_ix = Instruction {
        program_id: pre_req_vault::id(),
        accounts: pre_req_vault::accounts::Close {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: pre_req_vault::instruction::Close {}.data(),
    };

    let message = Message::new(&[close_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction4 = Transaction::new(&[&payer], message, recent_blockhash);
    let tx4 = svm.send_transaction(transaction4).unwrap();

    msg!("Close transaction successful");
    msg!("Tx Signature: {}", tx4.signature);

    assert!(svm.get_account(&vault_pda).is_none());
    assert!(svm.get_account(&vault_state_pda).is_none());

    let user_balance_after_close = svm.get_balance(&user).unwrap();
    assert!(user_balance_after_close > close_amount);
}
