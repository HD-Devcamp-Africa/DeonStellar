# 1. Stellar Crowdfunding Contract


A Rust implementation of a crowdfunding smart contract for the Stellar blockchain.

## Features

- 🏗️ **Campaign Creation**: Users can create fundraising campaigns with targets and deadlines
- 💰 **Contribution Tracking**: Transparent record of all contributions
- ⏳ **Deadline Management**: Automatic enforcement of campaign time limits
- 📊 **Query Functions**: Check campaign status and user contributions

## Learning Outcomes

- Basic storage operations on Stellar
- Time-based conditional logic
- Payment handling and token transfers
- Event emission for on-chain transparency

## Usage

### Prerequisites
- Rust 1.60+
- Soroban CLI
- Stellar testnet account

### Deployment
```bash
cargo build --target wasm32-unknown-unknown --release
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/stellar_crowdfunding_contract.wasm --source YOUR_SECRET_KEY --network testnet
```

### Interacting with the Contract
```rust
// Create a campaign
client.create_campaign(
    &creator,
    "Medical Fund".to_string(),
    "Help with hospital bills".to_string(),
    10000,  // Target amount (in stroops)
    env.ledger().timestamp() + 86400,  // 24-hour deadline
);

// Contribute to campaign
client.contribute(&contributor, 0, 1000);  // 1000 stroops
```

---

# 2. Stellar Time-Locked Vault


A secure time-locked vault contract for Stellar assets.

## Features

- 🔒 **Token Locking**: Deposit tokens with custom unlock timestamps
- ⏱️ **Time Enforcement**: Strict withdrawal timing controls
- 👥 **Beneficiary Management**: Designate authorized withdrawers
- 🔍 **Transparency**: View all deposits and beneficiaries

## Learning Outcomes

- Time-based transaction logic
- Authorization and access control patterns
- Token handling and balance management
- Secure withdrawal workflows

## Usage

### Prerequisites
- Rust 1.60+
- Soroban CLI
- Stellar asset token

### Deployment
```bash
cargo build --target wasm32-unknown-unknown --release
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/stellar_time_locked_vault.wasm --source YOUR_SECRET_KEY --network testnet
```

### Example Workflow
```rust
// Initialize vault with governance token
client.initialize(&token_address, &admin);

// Deposit tokens for 30 days
let unlock_time = env.ledger().timestamp() + 2592000;  // 30 days
client.deposit(&user, 5000, unlock_time);

// Add beneficiary
client.add_beneficiary(&admin, &beneficiary, 0);

// Withdraw after unlock period
client.withdraw(&beneficiary, 0);
```

---

# 3. Stellar DAO Voting System


Decentralized governance system for Stellar-based organizations.

## Features

- 📜 **Proposal System**: Create, view, and manage governance proposals
- 🗳️ **Token-weighted Voting**: Votes proportional to governance token holdings
- ⏱️ **Time-bound Elections**: Configurable voting periods
- ⚙️ **Execution Engine**: Automatic execution of passed proposals

## Learning Outcomes

- Sophisticated access control patterns
- Time-based state transitions
- Complex event handling
- Governance token integration

## Usage

### Prerequisites
- Rust 1.60+
- Soroban CLI
- Governance token contract

### Deployment
```bash
cargo build --target wasm32-unknown-unknown --release
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/stellar_dao_voting.wasm --source YOUR_SECRET_KEY --network testnet
```

### Governance Workflow
```rust
// Initialize DAO with 7-day voting period
client.initialize(&gov_token, &admin, 604800);

// Member creates proposal
let proposal_id = client.create_proposal(
    &member,
    "Upgrade Treasury".to_string(),
    "Allocate funds for development".to_string(),
    action_data,
);

// Admin starts voting
client.start_voting(&admin, proposal_id);

// Members vote
client.vote(&member1, proposal_id, true);  // Yes
client.vote(&member2, proposal_id, false); // No

// Execute passed proposal
client.execute_proposal(&executor, proposal_id);
```

---

## General Project Structure for All Contracts

```
stellar-project/
├── src/
│   ├── lib.rs          # Main contract logic
│   └── test.rs         # Test cases
├── Cargo.toml          # Dependencies
├── README.md           # This documentation
└── .gitignore          # Ignore build artifacts
```

## Contributing
Pull requests welcome! For major changes, please open an issue first.

## License
[MIT](https://choosealicense.com/licenses/mit/)

