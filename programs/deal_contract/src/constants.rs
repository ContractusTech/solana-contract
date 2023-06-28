use anchor_lang::prelude::*;
use solana_program::pubkey;

pub const DEAL_STATE_SEED: &[u8] = b"state";

// NEED CHECK: CTUS address
// Devnet: CyhjLfsfDz7rtszqBGaHiFrBbck2LNKEXQkywqNrGVyw
// Mainnet: ---
// Localnet: 67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V
pub const SERVICE_TOKEN_ADDRESS_MINT: Pubkey =
    pubkey!("67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V");

// NEED CHECK: Admin level
// Devnet: 3aDaxu2XwsGmj7amUnrxaHoKTtKJUqebkYP9HJTkP434
// Mainnet: ---
// Localnet: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
pub const SERVICE_ACCOUNT_ADDRESS: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// NEED CHECK: 10_000 CTUS
pub const HOLDER_MODE_AMOUNT: u64 = 10000000000000;

pub const HOLDER_MINT: Pubkey = pubkey!("HbebLQBTUiquq6UpGutJo3Fq5Logqf7Ww6tgCBkxLgi9");
