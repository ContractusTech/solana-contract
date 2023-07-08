use anchor_lang::prelude::*;
use solana_program::pubkey;

pub const DEAL_STATE_SEED: &[u8] = b"deal_state";

// NEED CHECK: CTUS address
// Devnet: CyhjLfsfDz7rtszqBGaHiFrBbck2LNKEXQkywqNrGVyw
// Mainnet: ---
// Localnet: 67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V
pub const SERVICE_FEE_MINT: Pubkey = pubkey!("62PtWFh2dQ69LKbHumBpMa7wG71r7i7Damwo2wMYfcR1");
// pub const SERVICE_FEE_TA: Pubkey = pubkey!("11111111111111111111111111111111");
pub const SERVICE_FEE_OWNER: Pubkey = pubkey!("C8pHACh7SAZWVZvgppM6VkWEDC6voLGGctch5Vr5hkEz");

// NEED CHECK: Admin level
// Devnet: 3aDaxu2XwsGmj7amUnrxaHoKTtKJUqebkYP9HJTkP434
// Mainnet: ---
// Localnet: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
pub const SERVICE_ACCOUNT_ADDRESS: Pubkey = pubkey!("C8pHACh7SAZWVZvgppM6VkWEDC6voLGGctch5Vr5hkEz");

// NEED CHECK: 10_000 CTUS
#[constant]
pub const HOLDER_MODE_AMOUNT: u64 = 10000000000000;

pub const HOLDER_MINT: Pubkey = pubkey!("64esx9p99rgwzmBCFCaUDCKJL2b2WrgdFe7chyaDyrKD");
