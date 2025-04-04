#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Address, Vec, Map, Val, IntoVal, BytesN};
use soroban_sdk::auth::Context;


const PROPOSAL_CREATED: Symbol = symbol_short!("PROPCREAT");
const VOTE_CAST: Symbol = symbol_short!("VOTECAST");
const PROPOSAL_EXECUTED: Symbol = symbol_short!("PROPEXEC");


const STATUS_PENDING: u32 = 0;
const STATUS_ACTIVE: u32 = 1;
const STATUS_PASSED: u32 = 2;
const STATUS_FAILED: u32 = 3;
const STATUS_EXECUTED: u32 = 4;

#[contract]
pub struct DaoVotingSystem;

#[contractimpl]
impl DaoVotingSystem {
   
    pub fn initialize(env: Env, governance_token: Address, admin: Address, voting_period: u64) {
        
        env.storage().persistent().set(&symbol_short!("gov_token"), &governance_token);
        env.storage().persistent().set(&symbol_short!("admin"), &admin);
        env.storage().persistent().set(&symbol_short!("voting_period"), &voting_period);
        env.storage().persistent().set(&symbol_short!("proposals"), &Vec::new(&env));
        env.storage().persistent().set(&symbol_short!("members"), &Map::new(&env));
    }

    // Add a member with voting power (only admin)
    pub fn add_member(env: Env, admin: Address, member: Address) {
        // Verify the caller is the admin
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap();
        assert!(admin == stored_admin, "Only admin can add members");

        // Add member to the map
        let mut members: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("members"))
            .unwrap_or_else(|| Map::new(&env));
        members.set(member, true);
        env.storage().persistent().set(&symbol_short!("members"), &members);
    }

    // Create a new proposal
    pub fn create_proposal(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        action: BytesN<32>, 
    ) -> u32 {
        
        let members: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("members"))
            .unwrap_or_else(|| Map::new(&env));
        assert!(members.get(creator.clone()).unwrap_or(false), "Only members can create proposals");

        let voting_period: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("voting_period"))
            .unwrap();

        let proposal = vec![
            &env,
            creator.clone().into_val(&env),
            title.into_val(&env),
            description.into_val(&env),
            action.into_val(&env),
            STATUS_PENDING.into_val(&env),
            0i64.into_val(&env), 
            0i64.into_val(&env), 
            env.ledger().timestamp().into_val(&env), 
            0u64.into_val(&env), 
            0u64.into_val(&env), 
        ];

    
        let mut proposals: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("proposals"))
            .unwrap_or_else(|| Vec::new(&env));
        proposals.push_back(proposal);
        let proposal_id = proposals.len() - 1;
        env.storage().persistent().set(&symbol_short!("proposals"), &proposals);

        
        env.events().publish(
            (PROPOSAL_CREATED, creator),
            (proposal_id, title),
        );

        proposal_id
    }


    pub fn start_voting(env: Env, admin: Address, proposal_id: u32) {
        
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap();
        assert!(admin == stored_admin, "Only admin can start voting");

        
        let voting_period: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("voting_period"))
            .unwrap();

        
        let mut proposals: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("proposals"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(proposal_id < proposals.len(), "Proposal does not exist");

        let mut proposal = proposals.get(proposal_id).unwrap();
        let status: u32 = proposal.get(5).unwrap().try_into().unwrap();
        assert!(status == STATUS_PENDING, "Proposal must be pending to start voting");

        
        proposal.set(5, STATUS_ACTIVE.into_val(&env));
        proposal.set(9, env.ledger().timestamp().into_val(&env)); // voting start
        proposal.set(10, (env.ledger().timestamp() + voting_period).into_val(&env)); // voting end

        
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&symbol_short!("proposals"), &proposals);
    }


    pub fn vote(env: Env, voter: Address, proposal_id: u32, support: bool) {
        
        let members: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("members"))
            .unwrap_or_else(|| Map::new(&env));
        assert!(members.get(voter.clone()).unwrap_or(false), "Only members can vote");

        
        let token_address: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("gov_token"))
            .unwrap();
        let token_client = token::Client::new(&env, &token_address);
        let voting_power = token_client.balance(&voter);
        assert!(voting_power > 0, "Voter has no voting power");

        
        let mut proposals: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("proposals"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(proposal_id < proposals.len(), "Proposal does not exist");

        let mut proposal = proposals.get(proposal_id).unwrap();
        let status: u32 = proposal.get(5).unwrap().try_into().unwrap();
        assert!(status == STATUS_ACTIVE, "Proposal must be active for voting");

        
        let voting_end_at: u64 = proposal.get(10).unwrap().try_into().unwrap();
        assert!(
            env.ledger().timestamp() < voting_end_at,
            "Voting period has ended"
        );


        
        if support {
            let mut yes_votes: i64 = proposal.get(6).unwrap().try_into().unwrap();
            yes_votes += voting_power;
            proposal.set(6, yes_votes.into_val(&env));
        } else {
            let mut no_votes: i64 = proposal.get(7).unwrap().try_into().unwrap();
            no_votes += voting_power;
            proposal.set(7, no_votes.into_val(&env));
        }

        // Save the updated proposal
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&symbol_short!("proposals"), &proposals);

        // Emit event
        env.events().publish(
            (VOTE_CAST, voter),
            (proposal_id, support, voting_power),
        );
    }

    // Execute a passed proposal
    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u32) {
        // Verify the executor is a member
        let members: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("members"))
            .unwrap_or_else(|| Map::new(&env));
        assert!(members.get(executor.clone()).unwrap_or(false), "Only members can execute proposals");

        // Get the proposal
        let mut proposals: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("proposals"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(proposal_id < proposals.len(), "Proposal does not exist");

        let mut proposal = proposals.get(proposal_id).unwrap();
        let status: u32 = proposal.get(5).unwrap().try_into().unwrap();
        assert!(status == STATUS_ACTIVE || status == STATUS_PASSED, "Proposal must be active or passed");

        // Check voting period has ended
        let voting_end_at: u64 = proposal.get(10).unwrap().try_into().unwrap();
        assert!(
            env.ledger().timestamp() >= voting_end_at,
            "Voting period has not ended yet"
        );

        // vote counts
        let yes_votes: i64 = proposal.get(6).unwrap().try_into().unwrap();
        let no_votes: i64 = proposal.get(7).unwrap().try_into().unwrap();

        // Determine if proposal passed (simple majority)
        if yes_votes > no_votes {
            proposal.set(5, STATUS_PASSED.into_val(&env));
            
            // Get the action to execute
            let action: BytesN<32> = proposal.get(4).unwrap().try_into().unwrap();

            proposal.set(5, STATUS_EXECUTED.into_val(&env));
            
            // Emit event
            env.events().publish(
                (PROPOSAL_EXECUTED, executor),
                (proposal_id, action),
            );
        } else {
            proposal.set(5, STATUS_FAILED.into_val(&env));
        }

    
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&symbol_short!("proposals"), &proposals);
    }


    pub fn get_proposal(env: Env, proposal_id: u32) -> Vec<Val> {
        let proposals: Vec<Vec<Val>> = env
            .storage()
            .persistent()
            .get(&symbol_short!("proposals"))
            .unwrap_or_else(|| Vec::new(&env));
        assert!(proposal_id < proposals.len(), "Proposal does not exist");
        proposals.get(proposal_id).unwrap()
    }
}
