#![no_std]
/*
Explanation:
Data Structures:
SwapEventData: A struct representing the swap event data, including the user address, amount, and nonce.
Storage Keys:
LOCKED_AMOUNTS: Stores locked USDC amounts per user.
PROCESSED_TXS: Keeps track of processed Ethereum transaction hashes.
NONCE: A counter to ensure uniqueness in events.
Functions:
verify_signature: Verifies the BLS signature from Ethereum.
swap_to_eth: Locks USDC on Stellar, increments the nonce, and emits an event with swap details.
swap_from_eth: Verifies the BLS signature, mints USDC on Stellar, and marks the transaction as processed.
Helper Functions:
parse_message: Parses the message from Ethereum to extract swap details.
Storage accessors for locked_amounts, processed_txs, and increment_nonce.
*/
use soroban_sdk::{
    contractimpl, contracttype, symbol_short, vec, Bytes, Env, Map, Symbol, Vec,
};
use soroban_sdk::crypto::bls::{Bls12_381, G1Point, G2Point};

#[derive(Clone)]
#[contracttype]
pub struct SwapEventData {
    pub user: Bytes,
    pub amount: i128,
    pub nonce: u64,
}

pub struct StellarEthBridge;

#[contractimpl]
impl StellarEthBridge {
    // Storage keys
    const LOCKED_AMOUNTS: Symbol = symbol_short!("locked_amts");
    const PROCESSED_TXS: Symbol = symbol_short!("processed_txs");
    const NONCE: Symbol = symbol_short!("nonce");

    /// Verifies a BLS signature from Ethereum on the Stellar network.
    /// Returns true if the signature is valid, false otherwise.
    pub fn verify_signature(
        env: Env,
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> bool {
        let bls = Bls12_381::new(&env);

        // Deserialize public key and signature
        let pk = match bls.g2_from_bytes(&public_key) {
            Ok(point) => point,
            Err(_) => return false,
        };
        let sig = match bls.g1_from_bytes(&signature) {
            Ok(point) => point,
            Err(_) => return false,
        };

        // Hash the message to G2 using Ethereum's DST
        let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
        let msg_hash = bls.hash_to_g2(&message, dst);

        // Perform pairing check
        bls.pairing_check(&sig, &pk, &msg_hash)
    }

    /// Swaps USDC from Stellar to Ethereum.
    /// Locks the USDC amount on Stellar and emits an event for Ethereum to unlock equivalent USDC.
    pub fn swap_to_eth(env: Env, user: Bytes, amount: i128) -> bool {
        // Validate amount
        if amount <= 0 {
            return false;
        }

        // Lock USDC tokens
        let mut balances = Self::locked_amounts(&env);
        let user_balance = balances.get(user.clone()).unwrap_or(0);

        // Update balance
        balances.set(user.clone(), user_balance + amount);

        // Increment nonce
        let nonce = Self::increment_nonce(&env);

        // Create event data
        let event_data = SwapEventData {
            user: user.clone(),
            amount,
            nonce,
        };

        // Emit event
        env.events().publish(
            (Symbol::short("swap_to_eth"),),
            event_data.clone(),
        );

        true
    }

    /// Swaps USDC from Ethereum to Stellar.
    /// Verifies the BLS signature from Ethereum and mints equivalent USDC on Stellar.
    pub fn swap_from_eth(
        env: Env,
        eth_tx_hash: Bytes,
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> bool {
        // Check if transaction has already been processed
        let mut processed_txs = Self::processed_txs(&env);
        if processed_txs.contains(&eth_tx_hash) {
            return false; // Transaction already processed
        }

        // Verify the Ethereum signature
        if !Self::verify_signature(env.clone(), public_key, message.clone(), signature) {
            return false;
        }

        // Parse the message to extract user and amount
        let event_data = match Self::parse_message(&message) {
            Some(data) => data,
            None => return false,
        };

        // Mint USDC to the user's Stellar account
        let mut balances = Self::locked_amounts(&env);
        let user_balance = balances.get(event_data.user.clone()).unwrap_or(0);
        balances.set(event_data.user.clone(), user_balance + event_data.amount);

        // Mark transaction as processed
        processed_txs.push_back(eth_tx_hash.clone());

        // Emit event
        env.events().publish(
            (Symbol::short("swap_from_eth"),),
            event_data.clone(),
        );

        true
    }

    // Helper function to parse the message
    fn parse_message(message: &Bytes) -> Option<SwapEventData> {
        // For simplicity, assume message is serialized SwapEventData
        // In practice, ensure secure deserialization and validation
        let env = message.env();
        SwapEventData::try_from_val(&env, message).ok()
    }

    // Helper functions for storage
    fn locked_amounts(env: &Env) -> Map<Bytes, i128> {
        env.storage().persistent(Map::<Bytes, i128>::new(Self::LOCKED_AMOUNTS))
    }

    fn processed_txs(env: &Env) -> Vec<Bytes> {
        env.storage().persistent(Vec::<Bytes>::new(Self::PROCESSED_TXS))
    }

    fn increment_nonce(env: &Env) -> u64 {
        let nonce: u64 = env
            .storage()
            .get(Self::NONCE)
            .unwrap_or(Ok(0))
            .unwrap_or(0);
        let new_nonce = nonce + 1;
        env.storage().set(Self::NONCE, &new_nonce);
        new_nonce
    }
}
