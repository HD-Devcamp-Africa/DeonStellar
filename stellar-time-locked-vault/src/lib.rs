#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Address, BytesN, Val, IntoVal, token};
use soroban_sdk::auth::{Context, CustomAccountInterface};
use soroban_sdk::xdr::{ScVal, ScVec};


const DEPOSIT_EVENT: Symbol = symbol_short!("DEPOSIT");
const WITHDRAWAL_EVENT: Symbol = symbol_short!("WITHDRAW");
const BENEFICIARY_ADDED: Symbol = symbol_short!("BENADDED");

#[contract]
pub struct TimeLockedVault;

#[contractimpl]
impl TimeLockedVault {
   
    pub fn initialize(env: Env, token_address: Address, admin: Address) {
        
        env.storage().persistent().set(&symbol_short!("token"), &token_address);
        env.storage().persistent().set(&symbol_short!("admin"), &admin);
        
       
        env.storage().persistent().set(&symbol_short!("deposits"), &Vec::new(&env));
        env.storage().persistent().set(&symbol_short!("beneficiaries"), &Map::new(&env));
    }

    // Deposit tokens into the vault with a lock period
    pub fn deposit(
        env: Env,
        from: Address,
        amount: i64,
        unlock_timestamp: u64,
    ) {
        // Verify the amount is positive
        assert!(amount > 0, "Amount must be positive");
        
        // Verify the unlock timestamp is in the future
        let current_timestamp = env.ledger().timestamp();
        assert!(
            unlock_timestamp > current_timestamp,
            "Unlock timestamp must be in the future"
        );

        // Get the token client
        let token_address: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("token"))
            .unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // Transfer tokens from the sender to the contract
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        // Create deposit record
        let deposit = vec![
            &env,
            from.clone(),
            amount.into_val(&env),
            unlock_timestamp.into_val(&env),
        ];

        // Store the deposit
        let mut deposits: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("deposits"))
            .unwrap_or_else(|| Vec::new(&env));
        deposits.push_back(deposit);
        env.storage().persistent().set(&symbol_short!("deposits"), &deposits);

        // Emit deposit event
        env.events().publish(
            (DEPOSIT_EVENT, from),
            (amount, unlock_timestamp),
        );
    }

  
    pub fn add_beneficiary(
        env: Env,
        admin: Address,
        beneficiary: Address,
        deposit_index: u32,
    ) {
        // Verify the caller is the admin
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap();
        assert!(admin == stored_admin, "Only admin can add beneficiaries");

     
        let deposits: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("deposits"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(
            deposit_index < deposits.len(),
            "Deposit does not exist"
        );

        let mut beneficiaries: Map<u32, Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("beneficiaries"))
            .unwrap_or_else(|| Map::new(&env));
        beneficiaries.set(deposit_index, beneficiary.clone());
        env.storage().persistent().set(&symbol_short!("beneficiaries"), &beneficiaries);

        // Emit event
        env.events().publish(
            (BENEFICIARY_ADDED, admin),
            (deposit_index, beneficiary),
        );
    }

    pub fn withdraw(env: Env, beneficiary: Address, deposit_index: u32) {
       
        let mut deposits: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("deposits"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(
            deposit_index < deposits.len(),
            "Deposit does not exist"
        );
        let deposit = deposits.get(deposit_index).unwrap();

   
        let beneficiaries: Map<u32, Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("beneficiaries"))
            .unwrap_or_else(|| Map::new(&env));
        let authorized_beneficiary = beneficiaries.get(deposit_index).unwrap();
        assert!(
            beneficiary == authorized_beneficiary,
            "Not authorized to withdraw this deposit"
        );

        let unlock_timestamp: u64 = deposit.get(3).unwrap().try_into().unwrap();
        let current_timestamp = env.ledger().timestamp();
        assert!(
            current_timestamp >= unlock_timestamp,
            "Lock period has not passed yet"
        );

       
        let token_address: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("token"))
            .unwrap();
        let token_client = token::Client::new(&env, &token_address);

        
        let amount: i64 = deposit.get(2).unwrap().try_into().unwrap();

      
        token_client.transfer(
            &env.current_contract_address(),
            &beneficiary,
            &amount,
        );

    
        deposits.remove(deposit_index);
        env.storage().persistent().set(&symbol_short!("deposits"), &deposits);

    
        let mut beneficiaries: Map<u32, Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("beneficiaries"))
            .unwrap_or_else(|| Map::new(&env));
        beneficiaries.remove(deposit_index);
        env.storage().persistent().set(&symbol_short!("beneficiaries"), &beneficiaries);

    
        env.events().publish(
            (WITHDRAWAL_EVENT, beneficiary),
            (deposit_index, amount),
        );
    }

    pub fn get_deposit(env: Env, deposit_index: u32) -> Vec<Val> {
        let deposits: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("deposits"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(
            deposit_index < deposits.len(),
            "Deposit does not exist"
        );
        deposits.get(deposit_index).unwrap()
    }


    pub fn get_beneficiary(env: Env, deposit_index: u32) -> Address {
        let beneficiaries: Map<u32, Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("beneficiaries"))
            .unwrap_or_else(|| Map::new(&env));
        beneficiaries.get(deposit_index).unwrap()
    }
}
