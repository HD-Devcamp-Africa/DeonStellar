#[cfg(test)]
mod test {
    use soroban_sdk::{Env, Address, BytesN};

    use crate::{DaoVotingSystem, DaoVotingSystemClient};

    #[test]
    fn test_dao_workflow() {
        let env = Env::default();
        
        // Create test governance token
        let token_address = env.register_stellar_asset_contract(Address::random(&env));
        let token_client = token::Client::new(&env, &token_address);
        
        // Deploy the DAO contract
        let contract_id = env.register_contract(None, DaoVotingSystem);
        let client = DaoVotingSystemClient::new(&env, &contract_id);
        
        // Initialize the DAO with 1 week voting period
        let admin = Address::random(&env);
        client.initialize(&token_address, &admin, 604800); // 7 days in seconds
        
        // Add members
        let member1 = Address::random(&env);
        let member2 = Address::random(&env);
        client.add_member(&admin, &member1);
        client.add_member(&admin, &member2);
        
        // Give members some governance tokens
        token_client.mint(&member1, &1000);
        token_client.mint(&member2, &500);
        
        // Member creates a proposal
        let action = BytesN::from_array(&env, &[0; 32]); // Mock action
        let proposal_id = client.create_proposal(
            &member1,
            "Upgrade contract".to_string(),
            "Upgrade to version 2.0".to_string(),
            action,
        );
        
        // Admin starts voting
        client.start_voting(&admin, proposal_id);
        
        // Members vote
        client.vote(&member1, proposal_id, true); // Yes vote with 1000 power
        client.vote(&member2, proposal_id, false); // No vote with 500 power
        
        // Fast forward time to end voting period
        env.ledger().set_timestamp(env.ledger().timestamp() + 604800);
        
        // Execute the proposal
        client.execute_proposal(&member1, proposal_id);
        
        // Verify proposal status is executed
        let proposal = client.get_proposal(proposal_id);
        let status: u32 = proposal.get(5).unwrap().try_into().unwrap();
        assert_eq!(status, 4); // STATUS_EXECUTED
    }
}
