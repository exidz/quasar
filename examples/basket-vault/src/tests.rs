extern crate std;
use {
    alloc::{vec, vec::Vec},
    mollusk_svm::{program::keyed_account_for_system_program, Mollusk},
    quasar_basket_vault_client::{accounts, args, instructions},
    solana_account::Account,
    solana_address::Address,
    solana_instruction::Instruction,
    solana_program_pack::Pack,
    spl_token_interface::state::{Account as TokenAccount, Mint},
    std::println,
};

fn setup() -> Mollusk {
    let mut mollusk = Mollusk::new(&crate::ID, "../../target/deploy/quasar_basket_vault");
    mollusk_svm_programs_token::token::add_program(&mut mollusk);
    mollusk
}

/// Some init-if-needed token accounts are created by CPI and need to be marked
/// as transaction signers in the Mollusk test harness.
fn with_signers(mut ix: Instruction, indices: &[usize]) -> Instruction {
    for &i in indices {
        ix.accounts[i].is_signer = true;
    }
    ix
}

fn pack_token(mint: Address, owner: Address, amount: u64) -> Vec<u8> {
    let token = TokenAccount {
        mint,
        owner,
        amount,
        delegate: None.into(),
        state: spl_token_interface::state::AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };
    let mut data = vec![0u8; TokenAccount::LEN];
    Pack::pack(token, &mut data).unwrap();
    data
}

fn pack_mint(authority: Address, decimals: u8) -> Vec<u8> {
    let mint = Mint {
        mint_authority: Some(authority).into(),
        supply: 1_000_000_000,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let mut data = vec![0u8; Mint::LEN];
    Pack::pack(mint, &mut data).unwrap();
    data
}

fn build_basket_data(
    manager: Address,
    mint_a: Address,
    mint_b: Address,
    mint_c: Address,
    bump: u8,
) -> Vec<u8> {
    let mut data = vec![0u8; 130];
    data[0] = 1;
    data[1..33].copy_from_slice(manager.as_ref());
    data[33..65].copy_from_slice(mint_a.as_ref());
    data[65..97].copy_from_slice(mint_b.as_ref());
    data[97..129].copy_from_slice(mint_c.as_ref());
    data[129] = bump;
    data
}

fn build_vault_data(basket: Address, user: Address, mint: Address, bump: u8) -> Vec<u8> {
    let mut data = vec![0u8; 98];
    data[0] = 2;
    data[1..33].copy_from_slice(basket.as_ref());
    data[33..65].copy_from_slice(user.as_ref());
    data[65..97].copy_from_slice(mint.as_ref());
    data[97] = bump;
    data
}

fn derive_basket(manager: Address, basket_seed: Address) -> (Address, u8) {
    Address::find_program_address(
        &[b"basket", manager.as_ref(), basket_seed.as_ref()],
        &crate::ID,
    )
}

fn derive_vault(basket: Address, user: Address, mint: Address) -> (Address, u8) {
    Address::find_program_address(
        &[b"basket-vault", basket.as_ref(), user.as_ref(), mint.as_ref()],
        &crate::ID,
    )
}

#[test]
fn test_create_basket() {
    let mollusk = setup();
    let (system_program, system_program_account) = keyed_account_for_system_program();

    let manager = Address::new_unique();
    let manager_account = Account::new(1_000_000_000, 0, &system_program);
    let basket_seed = Address::new_unique();
    let basket_seed_account = Account::default();
    let mint_a = Address::new_unique();
    let mint_b = Address::new_unique();
    let mint_c = Address::new_unique();
    let (basket, basket_bump) = derive_basket(manager, basket_seed);
    let basket_account = Account::default();

    let instruction = instructions::create_basket(
        accounts::CreateBasket {
            manager,
            basket_seed,
        },
        args::CreateBasket {
            mint_a,
            mint_b,
            mint_c,
        },
    );

    let result = mollusk.process_instruction(
        &instruction,
        &[
            (manager, manager_account),
            (basket_seed, basket_seed_account),
            (basket, basket_account),
            (system_program, system_program_account),
        ],
    );

    assert!(
        result.program_result.is_ok(),
        "create_basket failed: {:?}",
        result.program_result
    );

    assert_eq!(
        result.resulting_accounts[2].1.data,
        build_basket_data(manager, mint_a, mint_b, mint_c, basket_bump)
    );

    println!("\n========================================");
    println!("  CREATE_BASKET CU: {}", result.compute_units_consumed);
    println!("========================================\n");
}

#[test]
fn test_deposit_allowed_mint() {
    let mollusk = setup();

    let (token_program, token_program_account) = mollusk_svm_programs_token::token::keyed_account();
    let (system_program, system_program_account) = keyed_account_for_system_program();
    let (rent, rent_account) = mollusk.sysvars.keyed_account_for_rent_sysvar();

    let manager = Address::new_unique();
    let basket_seed = Address::new_unique();
    let user = Address::new_unique();
    let mint = Address::new_unique();
    let other_mint = Address::new_unique();
    let third_mint = Address::new_unique();
    let (basket, basket_bump) = derive_basket(manager, basket_seed);
    let user_account = Account::new(1_000_000_000, 0, &system_program);
    let basket_account = Account {
        lamports: 2_000_000,
        data: build_basket_data(manager, mint, other_mint, third_mint, basket_bump),
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    };
    let mint_account = Account {
        lamports: 1_000_000,
        data: pack_mint(user, 6),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let user_ta = Address::new_unique();
    let user_ta_account = Account {
        lamports: 1_000_000,
        data: pack_token(mint, user, 1_000_000),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let (vault, _vault_bump) = derive_vault(basket, user, mint);
    let vault_account = Account::default();
    let vault_ta = Address::new_unique();
    let vault_ta_account = Account::new(0, 0, &system_program);

    let deposit_amount: u64 = 500_000;

    let instruction = with_signers(
        instructions::deposit(
            accounts::Deposit {
                user,
                basket,
                mint,
                user_ta,
                vault_ta,
            },
            args::Deposit {
                amount: deposit_amount,
            },
        ),
        &[5],
    );

    let result = mollusk.process_instruction(
        &instruction,
        &[
            (user, user_account),
            (basket, basket_account),
            (mint, mint_account),
            (user_ta, user_ta_account),
            (vault, vault_account),
            (vault_ta, vault_ta_account),
            (rent, rent_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );

    assert!(
        result.program_result.is_ok(),
        "deposit failed: {:?}",
        result.program_result
    );

    let user_ta_after: TokenAccount = Pack::unpack(&result.resulting_accounts[3].1.data).unwrap();
    let vault_ta_after: TokenAccount = Pack::unpack(&result.resulting_accounts[5].1.data).unwrap();
    let vault_data = &result.resulting_accounts[4].1.data;

    assert_eq!(user_ta_after.amount, 1_000_000 - deposit_amount);
    assert_eq!(vault_ta_after.amount, deposit_amount);
    assert_eq!(vault_ta_after.owner, vault);
    assert_eq!(vault_data, &build_vault_data(basket, user, mint, vault_data[97]));

    println!("\n========================================");
    println!("  BASKET DEPOSIT CU: {}", result.compute_units_consumed);
    println!("========================================\n");
}

#[test]
fn test_withdraw_allowed_mint() {
    let mollusk = setup();

    let (token_program, token_program_account) = mollusk_svm_programs_token::token::keyed_account();
    let (system_program, system_program_account) = keyed_account_for_system_program();

    let manager = Address::new_unique();
    let basket_seed = Address::new_unique();
    let user = Address::new_unique();
    let mint = Address::new_unique();
    let other_mint = Address::new_unique();
    let third_mint = Address::new_unique();
    let (basket, basket_bump) = derive_basket(manager, basket_seed);
    let user_account = Account::new(1_000_000_000, 0, &system_program);
    let basket_account = Account {
        lamports: 2_000_000,
        data: build_basket_data(manager, mint, other_mint, third_mint, basket_bump),
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    };
    let mint_account = Account {
        lamports: 1_000_000,
        data: pack_mint(user, 6),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let user_ta = Address::new_unique();
    let user_ta_account = Account {
        lamports: 1_000_000,
        data: pack_token(mint, user, 250_000),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let (vault, vault_bump) = derive_vault(basket, user, mint);
    let vault_account = Account {
        lamports: 2_000_000,
        data: build_vault_data(basket, user, mint, vault_bump),
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    };
    let vault_ta = Address::new_unique();
    let vault_ta_account = Account {
        lamports: 1_000_000,
        data: pack_token(mint, vault, 750_000),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };

    let withdraw_amount: u64 = 300_000;

    let instruction = instructions::withdraw(
        accounts::Withdraw {
            user,
            basket,
            mint,
            user_ta,
            vault_ta,
        },
        args::Withdraw {
            amount: withdraw_amount,
        },
    );

    let result = mollusk.process_instruction(
        &instruction,
        &[
            (user, user_account),
            (basket, basket_account),
            (mint, mint_account),
            (user_ta, user_ta_account),
            (vault, vault_account),
            (vault_ta, vault_ta_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );

    assert!(
        result.program_result.is_ok(),
        "withdraw failed: {:?}",
        result.program_result
    );

    let user_ta_after: TokenAccount = Pack::unpack(&result.resulting_accounts[3].1.data).unwrap();
    let vault_ta_after: TokenAccount = Pack::unpack(&result.resulting_accounts[5].1.data).unwrap();

    assert_eq!(user_ta_after.amount, 250_000 + withdraw_amount);
    assert_eq!(vault_ta_after.amount, 750_000 - withdraw_amount);

    println!("\n========================================");
    println!("  BASKET WITHDRAW CU: {}", result.compute_units_consumed);
    println!("========================================\n");
}

#[test]
// Same basket fund, same token, different users => distinct user vault PDAs.
fn test_same_index_fund_gives_each_user_a_distinct_vault() {
    let basket = Address::new_unique();
    let mint = Address::new_unique();
    let user_a = Address::new_unique();
    let user_b = Address::new_unique();

    let (vault_a, _) = derive_vault(basket, user_a, mint);
    let (vault_b, _) = derive_vault(basket, user_b, mint);

    assert_ne!(vault_a, vault_b, "each user should derive a distinct vault PDA");

    let deposit_a = instructions::deposit(
        accounts::Deposit {
            user: user_a,
            basket,
            mint,
            user_ta: Address::new_unique(),
            vault_ta: Address::new_unique(),
        },
        args::Deposit { amount: 100 },
    );

    let deposit_b = instructions::deposit(
        accounts::Deposit {
            user: user_b,
            basket,
            mint,
            user_ta: Address::new_unique(),
            vault_ta: Address::new_unique(),
        },
        args::Deposit { amount: 100 },
    );

    assert_eq!(deposit_a.accounts[4].pubkey, vault_a);
    assert_eq!(deposit_b.accounts[4].pubkey, vault_b);
}

#[test]
// Same basket fund, same user, different tokens => distinct token vault PDAs.
fn test_same_basket_supports_multiple_token_vaults_for_one_user() {
    let user = Address::new_unique();
    let basket = Address::new_unique();
    let mint_a = Address::new_unique();
    let mint_b = Address::new_unique();

    let (vault_a, _) = derive_vault(basket, user, mint_a);
    let (vault_b, _) = derive_vault(basket, user, mint_b);

    assert_ne!(vault_a, vault_b, "one basket should support multiple token vaults");

    let deposit_a = instructions::deposit(
        accounts::Deposit {
            user,
            basket,
            mint: mint_a,
            user_ta: Address::new_unique(),
            vault_ta: Address::new_unique(),
        },
        args::Deposit { amount: 100 },
    );
    let deposit_b = instructions::deposit(
        accounts::Deposit {
            user,
            basket,
            mint: mint_b,
            user_ta: Address::new_unique(),
            vault_ta: Address::new_unique(),
        },
        args::Deposit { amount: 100 },
    );

    assert_eq!(deposit_a.accounts[4].pubkey, vault_a);
    assert_eq!(deposit_b.accounts[4].pubkey, vault_b);
}

#[test]
// Same user, different baskets => separate vault namespaces.
fn test_same_user_gets_distinct_vaults_for_different_baskets() {
    let user = Address::new_unique();
    let basket_a = Address::new_unique();
    let basket_b = Address::new_unique();
    let mint = Address::new_unique();

    let (vault_a, _) = derive_vault(basket_a, user, mint);
    let (vault_b, _) = derive_vault(basket_b, user, mint);

    assert_ne!(vault_a, vault_b, "basket address should be part of the vault PDA");
}

#[test]
fn test_deposit_rejects_mint_outside_basket_allowlist() {
    let mollusk = setup();

    let (token_program, token_program_account) = mollusk_svm_programs_token::token::keyed_account();
    let (system_program, system_program_account) = keyed_account_for_system_program();
    let (rent, rent_account) = mollusk.sysvars.keyed_account_for_rent_sysvar();

    let manager = Address::new_unique();
    let basket_seed = Address::new_unique();
    let allowed_mint_a = Address::new_unique();
    let allowed_mint_b = Address::new_unique();
    let allowed_mint_c = Address::new_unique();
    let disallowed_mint = Address::new_unique();
    let user = Address::new_unique();
    let (basket, basket_bump) = derive_basket(manager, basket_seed);
    let basket_account = Account {
        lamports: 2_000_000,
        data: build_basket_data(
            manager,
            allowed_mint_a,
            allowed_mint_b,
            allowed_mint_c,
            basket_bump,
        ),
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    };
    let user_account = Account::new(1_000_000_000, 0, &system_program);
    let mint_account = Account {
        lamports: 1_000_000,
        data: pack_mint(user, 6),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let user_ta = Address::new_unique();
    let user_ta_account = Account {
        lamports: 1_000_000,
        data: pack_token(disallowed_mint, user, 1_000_000),
        owner: token_program,
        executable: false,
        rent_epoch: 0,
    };
    let (vault, _vault_bump) = derive_vault(basket, user, disallowed_mint);
    let vault_account = Account::default();
    let vault_ta = Address::new_unique();
    let vault_ta_account = Account::new(0, 0, &system_program);

    let instruction = with_signers(
        instructions::deposit(
            accounts::Deposit {
                user,
                basket,
                mint: disallowed_mint,
                user_ta,
                vault_ta,
            },
            args::Deposit { amount: 1 },
        ),
        &[5],
    );

    let result = mollusk.process_instruction(
        &instruction,
        &[
            (user, user_account),
            (basket, basket_account),
            (disallowed_mint, mint_account),
            (user_ta, user_ta_account),
            (vault, vault_account),
            (vault_ta, vault_ta_account),
            (rent, rent_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );

    assert!(
        result.program_result.is_err(),
        "deposit should fail for a mint outside the basket allowlist"
    );

    println!("\n========================================");
    println!("  REJECT_DISALLOWED_MINT CU: {}", result.compute_units_consumed);
    println!("========================================\n");
}
