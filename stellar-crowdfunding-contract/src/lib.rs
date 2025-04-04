#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec, Address, Map};

const CAMPAIGN_CREATED: Symbol = symbol_short!("CAMPCREATED");
const CONTRIBUTION_MADE: Symbol = symbol_short!("CONTRIBUTE");

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
impl CrowdfundingContract {
    
    pub fn initialize(env: Env) {
        
        env.storage().persistent().set(&symbol_short!("campaigns"), &Vec::new(&env));
        env.storage().persistent().set(&symbol_short!("contributions"), &Map::new(&env));
    }


    pub fn create_campaign(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        target_amount: i64,
        deadline: u64, 
    ) {
    
        let current_timestamp = env.ledger().timestamp();
        assert!(deadline > current_timestamp, "Deadline must be in the future");

        
        let campaign = vec![
            &env,
            creator.clone(),
            title,
            description,
            target_amount.into(),
            deadline.into(),
            0i64.into(), 
        ];

        
        let mut campaigns: Vec<Vec<soroban_sdk::Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("campaigns"))
            .unwrap_or_else(|| Vec::new(&env));
        campaigns.push_back(campaign);

        
        env.storage().persistent().set(&symbol_short!("campaigns"), &campaigns);

        env.events().publish(
            (CAMPAIGN_CREATED, creator),
            (title, target_amount, deadline),
        );
    }

    pub fn contribute(env: Env, contributor: Address, campaign_index: u32, amount: i64) {
        assert!(amount > 0, "Contribution amount must be positive");

        
        let mut campaigns: Vec<Vec<soroban_sdk::Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("campaigns"))
            .unwrap_or_else(|| Vec::new(&env));

        assert!(
            campaign_index < campaigns.len(),
            "Campaign does not exist"
        );

        let mut campaign = campaigns.get(campaign_index).unwrap();

        
        let deadline: u64 = campaign.get(5).unwrap().unwrap().try_into().unwrap();
        let current_timestamp = env.ledger().timestamp();
        assert!(
            current_timestamp < deadline,
            "Campaign deadline has passed"
        );


        let mut amount_raised: i64 = campaign.get(6).unwrap().unwrap().try_into().unwrap();
        amount_raised += amount;
        campaign.set(6, amount_raised.into());

        
        campaigns.set(campaign_index, campaign);
        env.storage().persistent().set(&symbol_short!("campaigns"), &campaigns);


        let mut contributions: Map<Address, Map<u32, i64>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("contributions"))
            .unwrap_or_else(|| Map::new(&env));

        let mut user_contributions = contributions.get(contributor.clone()).unwrap_or_else(|| Map::new(&env));
        let current_contribution: i64 = user_contributions.get(campaign_index).unwrap_or(0);
        user_contributions.set(campaign_index, current_contribution + amount);
        contributions.set(contributor.clone(), user_contributions);
        env.storage().persistent().set(&symbol_short!("contributions"), &contributions);

        // Emit event
        env.events().publish(
            (CONTRIBUTION_MADE, contributor),
            (campaign_index, amount),
        );
    }

    // Get campaign details
    pub fn get_campaign(env: Env, campaign_index: u32) -> Vec<soroban_sdk::Val> {
        let campaigns: Vec<Vec<soroban_sdk::Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("campaigns"))
            .unwrap_or_else(|| Vec::new(&env));

        assert!(
            campaign_index < campaigns.len(),
            "Campaign does not exist"
        );

        campaigns.get(campaign_index).unwrap()
    }

    // Get user contributions to a campaign
    pub fn get_user_contributions(env: Env, user: Address, campaign_index: u32) -> i64 {
        let contributions: Map<Address, Map<u32, i64>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("contributions"))
            .unwrap_or_else(|| Map::new(&env));

        contributions
            .get(user)
            .unwrap_or_else(|| Map::new(&env))
            .get(campaign_index)
            .unwrap_or(0)
    }
}
