#[cfg(test)]
mod test {
    use soroban_sdk::{Env, Address, BytesN};

    use crate::{TimeLockedVault, TimeLockedVaultClient};

    #[test]
    fn test_deposit_and_withdraw() {
        let env = Env::default();
        
        // Create test token
        let token_address = env.register_stellar_asset_contract(Address::random(&env));
        let token_client = token::Client::new(&env, &token_address);
        
        // Deploy the vault contract
        let contract_id = env.register_contract(None, TimeLockedVault);
        let client = TimeLockedVaultClient::new(&env, &contract_id);
        
        // Initialize the vault
        let admin = Address::random(&env);
        client.initialize(&token_address, &admin);
        
        // Mint some test tokens to a user
        let user = Address::random(&env);
        token_client.mint(&user, &1000);
        
        // User deposits tokens with 1000 seconds lock
        let unlock_time = env.ledger().timestamp() + 1000;
        client.deposit(&user, 500, unlock_time);
        
        // Admin adds beneficiary
        let beneficiary = Address::random(&env);
        client.add_beneficiary(&admin, &beneficiary, 0);
        
        // Try to withdraw too early (should fail)
        env.ledger().set_timestamp(unlock_time - 1);
        assert!(env.try_invoke_contract::<()>(&contract_id, "withdraw", (beneficiary.clone(), 0)).is_err());
        
        // Wait until unlock time and withdraw
        env.ledger().set_timestamp(unlock_time);
        client.withdraw(&beneficiary, 0);
        
    
        assert_eq!(token_client.balance(&beneficiary), 500);
    }
}
