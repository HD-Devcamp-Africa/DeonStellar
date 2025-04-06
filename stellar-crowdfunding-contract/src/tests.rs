#[cfg(test)]
mod test {
    use soroban_sdk::{Env, Address};

    use crate::{CrowdfundingContract, CrowdfundingContractClient};

    #[test]
    fn test_campaign_creation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingContract);
        let client = CrowdfundingContractClient::new(&env, &contract_id);

        // Initialize the contract
        client.initialize();

        // Create a test campaign
        let creator = Address::random(&env);
        client.create_campaign(
            &creator,
            "Test Campaign".to_string(),
            "Description".to_string(),
            1000,
            env.ledger().timestamp() + 1000,
        );

        // Verify the campaign exists
        let campaign = client.get_campaign(0);
        assert_eq!(campaign.get(0).unwrap(), creator);
        assert_eq!(campaign.get(1).unwrap(), "Test Campaign");
    }

    #[test]
    fn test_contribution() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingContract);
        let client = CrowdfundingContractClient::new(&env, &contract_id);

        // Initialize and create a campaign
        client.initialize();
        let creator = Address::random(&env);
        client.create_campaign(
            &creator,
            "Test Campaign".to_string(),
            "Description".to_string(),
            1000,
            env.ledger().timestamp() + 1000,
        );

        // Make a contribution
        let contributor = Address::random(&env);
        client.contribute(&contributor, 0, 100);

        // Verify the contribution was recorded
        let contributions = client.get_user_contributions(&contributor, 0);
        assert_eq!(contributions, 100);

        let campaign = client.get_campaign(0);
        let amount_raised: i64 = campaign.get(6).unwrap().unwrap().try_into().unwrap();
        assert_eq!(amount_raised, 100);
    }
}
