#![no_std]
use soroban_sdk::{
    contractimpl, contracttype, symbol, Address, Bytes, BytesN, Env, IntoVal, Symbol, Vec,
};

pub struct USDCAtomicSwap;

#[derive(Clone)]
#[contracttype]
pub struct Swap {
    initiator: Address,
    participant: Address,
    amount: i128,
    hash_lock: BytesN<32>,
    time_lock: u64,
    secret: Option<Bytes>,
    redeemed: bool,
    refunded: bool,
}

#[contractimpl]
impl USDCAtomicSwap {
    pub fn initiate_swap(
        env: Env,
        swap_id: BytesN<32>,
        participant: Address,
        amount: i128,
        hash_lock: BytesN<32>,
        time_lock: u64,
    ) {
        let initiator = env.invoker();
        let current_time = env.ledger().timestamp();

        // Ensure swap doesn't already exist
        assert!(
            !env.storage().has(&swap_id),
            "Swap with this ID already exists"
        );

        let swap = Swap {
            initiator: initiator.clone(),
            participant: participant.clone(),
            amount,
            hash_lock: hash_lock.clone(),
            time_lock: current_time + time_lock,
            secret: None,
            redeemed: false,
            refunded: false,
        };

        // Store the swap
        env.storage().set(&swap_id, &swap);

        // Transfer USDC from initiator to the contract
        Self::transfer_usdc(
            &env,
            &initiator,
            &env.current_contract_address(),
            amount,
        );
    }

    pub fn redeem(env: Env, swap_id: BytesN<32>, secret: Bytes) {
        let mut swap: Swap = env.storage().get_unchecked(&swap_id).unwrap();
        let participant = env.invoker();
        let current_time = env.ledger().timestamp();

        // Validations
        assert!(participant == swap.participant, "Not authorized");
        assert!(!swap.redeemed, "Already redeemed");
        assert!(!swap.refunded, "Already refunded");
        assert!(current_time < swap.time_lock, "Time lock expired");
        assert!(
            env.crypto().sha256(&secret) == swap.hash_lock,
            "Invalid secret"
        );

        swap.secret = Some(secret.clone());
        swap.redeemed = true;

        // Update storage
        env.storage().set(&swap_id, &swap);

        // Transfer USDC to participant
        Self::transfer_usdc(
            &env,
            &env.current_contract_address(),
            &participant,
            swap.amount,
        );
    }

    pub fn refund(env: Env, swap_id: BytesN<32>) {
        let mut swap: Swap = env.storage().get_unchecked(&swap_id).unwrap();
        let initiator = env.invoker();
        let current_time = env.ledger().timestamp();

        // Validations
        assert!(initiator == swap.initiator, "Not authorized");
        assert!(!swap.redeemed, "Already redeemed");
        assert!(!swap.refunded, "Already refunded");
        assert!(current_time >= swap.time_lock, "Time lock not expired");

        swap.refunded = true;

        // Update storage
        env.storage().set(&swap_id, &swap);

        // Transfer USDC back to initiator
        Self::transfer_usdc(
            &env,
            &env.current_contract_address(),
            &initiator,
            swap.amount,
        );
    }

    pub fn get_swap(env: Env, swap_id: BytesN<32>) -> Swap {
        env.storage().get_unchecked(&swap_id).unwrap()
    }

    fn transfer_usdc(env: &Env, from: &Address, to: &Address, amount: i128) {
        // USDC Token Contract ID on Soroban
        let usdc_token_contract_id = BytesN::from_array(env, &[
            // Replace with actual 32-byte USDC Token Contract ID
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ]);

        let usdc_client = soroban_sdk::token::Client::new(env, &usdc_token_contract_id);

        // Transfer USDC
        usdc_client.transfer(
            from.clone(),
            to.clone(),
            amount,
        ).unwrap();
    }
}
