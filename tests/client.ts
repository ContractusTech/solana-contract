import * as anchor from "@coral-xyz/anchor";
import { Program, BN, IdlTypes } from "@coral-xyz/anchor";
import { PublicKey, Keypair, TransactionInstruction, VersionedTransaction, Signer, AddressLookupTableAccount, TransactionMessage, Connection, ComputeBudgetProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { DealContract } from "../target/types/deal_contract";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

export const DEAL_STATE_SEED: string = "deal_state";

export const ENCODER = anchor.utils.bytes.utf8;

export const SERVICE_FEE_OWNER_KP: Keypair = Keypair.generate(); // FIXME
export const SERVICE_FEE_MINT_KP: Keypair = Keypair.generate();

export const SERVICE_FEE_OWNER: PublicKey = new PublicKey("C8pHACh7SAZWVZvgppM6VkWEDC6voLGGctch5Vr5hkEz"); // FIXME
export const SERVICE_FEE_MINT: PublicKey = new PublicKey("62PtWFh2dQ69LKbHumBpMa7wG71r7i7Damwo2wMYfcR1");
export const SERVICE_FEE_TA: PublicKey = getAssociatedTokenAddressSync(SERVICE_FEE_MINT, SERVICE_FEE_OWNER);

export const HOLDER_MINT: PublicKey = new PublicKey("64esx9p99rgwzmBCFCaUDCKJL2b2WrgdFe7chyaDyrKD"); // FIXME

export const DEAL_CONTRACT_PROGRAM_ID: PublicKey = new PublicKey("GKNkN4uDJWmidEC9h5Q9GQXNg48Go6q5bdnkDj6bSopz");


export function uuidTodealIdBuf (uuid: string): Buffer {
  const uuidhex = uuid.replace(/-/g, '');
  if (uuid.length != 36 || uuidhex.length != 32) throw new Error(`Invalid uuid UUID: ${uuid}`);
  return Buffer.from(uuidhex, 'hex') 
}

export function getDealStatePk(dealId: Buffer, clientPk: PublicKey, executorPk: PublicKey, programId?: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([
    dealId,
    ENCODER.encode(DEAL_STATE_SEED),
    clientPk.toBuffer(),
    executorPk.toBuffer(),
  ], programId ? programId : DEAL_CONTRACT_PROGRAM_ID)
}

export async function signAndSendIxs(
  connection: Connection,
  instructions: TransactionInstruction[], 
  signers: Signer[] = [], 
  payer?: Signer,
  lookupTable?: AddressLookupTableAccount[])
: Promise<anchor.web3.SimulatedTransactionResponse | string> {
  if (!payer && signers.length == 0) { throw new Error("no payer has been assigned")};
  const blockhash = (await connection.getLatestBlockhash());
  const msg = new TransactionMessage({
    payerKey: payer ? payer.publicKey : signers[0].publicKey,
    recentBlockhash: blockhash.blockhash,
    instructions
  }).compileToV0Message(lookupTable);

  const tx = new VersionedTransaction(msg);
  tx.sign(signers)

  const simulation = await connection.simulateTransaction(tx);
  if (simulation.value.err) {throw simulation.value}

  const rawTx = tx.serialize();
  if (rawTx.length > 1200) {
    console.error(`TX_LENGTH: ${rawTx.length}`)
  }
  const signature = await connection.sendRawTransaction(rawTx)
  const txStatus = (await connection.confirmTransaction({signature,...blockhash}, "confirmed")).value;

  if (!!txStatus.err) {throw signature}
  return signature
}

export const getTotalComputeIxs = (
  compute: number,
  priorityMicroLamports = 1
) => {
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
    units: compute,
  });
  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: priorityMicroLamports,
  });
  return [modifyComputeUnits, addPriorityFee];
};

export async function getInitializeIx ({
  dealContractProgram,
  dealId,
  amount,
  serviceFee,
  clientPk,
  executorPk,
  payerPk,
  dealMint,
  holderMode,
  deadline,
  withChecker = null,
  clientBond = null,
  executorBond = null,
}: {
  dealContractProgram: Program<DealContract>,
  dealId: string | Buffer,
  amount: number,
  serviceFee: {
      amount: number,
      mint?: PublicKey
  },
  clientPk: PublicKey,
  executorPk: PublicKey,
  payerPk: PublicKey,
  dealMint: PublicKey,
  deadline?: number,
  holderMode?: boolean,
  withChecker?: {
    checkerFee: BN,
    checkerKey: PublicKey
  },
  clientBond?: IdlTypes<DealContract>["Bond"],
  executorBond?: IdlTypes<DealContract>["Bond"],
}) {
  if (!(dealId instanceof Buffer)) {dealId = uuidTodealIdBuf(dealId)}
  dealId = dealId as Buffer;
  const dealState = getDealStatePk(dealId, clientPk, executorPk)[0];

  const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealState, true);
  const clientDealTa = getAssociatedTokenAddressSync(dealMint, clientPk);
  const executorDealTa = getAssociatedTokenAddressSync(dealMint, executorPk);

  const serviceFeeTa = serviceFee.mint ? getAssociatedTokenAddressSync(serviceFee.mint, SERVICE_FEE_OWNER) : SERVICE_FEE_TA;
  
  return dealContractProgram.methods.initialize({
    id: Array.from(dealId),
    dealAmount: new anchor.BN(amount),
    serviceFee: new anchor.BN(serviceFee.amount),
    deadlineTs: deadline !== undefined ? new BN(deadline) : null,
    holderMode: !!holderMode,
    checkerFee: !!withChecker ? withChecker.checkerFee : null,
    clientBond: clientBond ? clientBond.amount : null,
    executorBond: executorBond ? executorBond.amount : null,
  })
  .accountsStrict({
    client: clientPk,
    executor: executorPk,
    checker: withChecker ? withChecker.checkerKey : payerPk,
    payer: payerPk,

    dealMint,
    clientBondMint: clientBond ? clientBond.mint : dealMint,
    executorBondMint: executorBond ? executorBond.mint : dealMint,
    serviceMint: !!serviceFee.mint ? serviceFee.mint : SERVICE_FEE_MINT,

    clientHolderTa: holderMode ? getAssociatedTokenAddressSync(HOLDER_MINT, clientPk) : clientDealTa,
    clientBondTa: clientBond ? getAssociatedTokenAddressSync(clientBond.mint, clientPk) : clientDealTa,
    clientDealTa,
    clientServiceTa: getAssociatedTokenAddressSync(!!serviceFee.mint ? serviceFee.mint : SERVICE_FEE_MINT, clientPk),
  
    dealStateClientBondTa: clientBond ? getAssociatedTokenAddressSync(clientBond.mint, dealState, true) : dealStateDealTa,
    dealStateDealTa,
    dealStateExecutorBondTa: executorBond ? getAssociatedTokenAddressSync(executorBond.mint, dealState, true) : dealStateDealTa,
    dealStateHolderTa: holderMode ? getAssociatedTokenAddressSync(HOLDER_MINT, dealState, true) : dealStateDealTa,
  
    executorBondTa: executorBond ? getAssociatedTokenAddressSync(executorBond.mint, executorPk) : executorDealTa,
    executorDealTa,
  
    serviceFeeOwner: SERVICE_FEE_OWNER,
    serviceFeeTa,
  
    dealState,
    holderMint: HOLDER_MINT,
    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  }).preInstructions([getTotalComputeIxs(400000)[0]]);
}


export async function getCancelIx ({
  dealContractProgram, initializer, dealId, clientPk, executorPk, dealMint, checkerKey = null, clientBondMint, executorBondMint
}: {
  dealContractProgram: Program<DealContract>,

  initializer: PublicKey,
  dealId: string | Buffer,
  clientPk: PublicKey,
  executorPk: PublicKey,
  dealMint: PublicKey,

  checkerKey?: PublicKey,
  clientBondMint?: PublicKey,
  executorBondMint?: PublicKey,
}) {
  if (!(dealId instanceof Buffer)) {dealId = uuidTodealIdBuf(dealId)}
  dealId = dealId as Buffer;
  const dealState = getDealStatePk(dealId, clientPk, executorPk)[0];

  const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealState, true);
  const clientDealTa = getAssociatedTokenAddressSync(dealMint, clientPk);
  const executorDealTa = getAssociatedTokenAddressSync(dealMint, executorPk);

  const checkerDealTa = getAssociatedTokenAddressSync(dealMint, checkerKey ? checkerKey: initializer);

  return dealContractProgram.methods.cancel()
  .accountsStrict({
    initializer,
    checkerDealTa,
    checker: checkerKey ? checkerKey : initializer,

    dealMint,
    clientBondMint: clientBondMint ? clientBondMint : dealMint,
    executorBondMint: executorBondMint ? executorBondMint : dealMint,

    clientBondTa: clientBondMint ? getAssociatedTokenAddressSync(clientBondMint, clientPk) : clientDealTa,
    clientDealTa,
  
    dealStateClientBondTa: clientBondMint ? getAssociatedTokenAddressSync(clientBondMint, dealState, true) : dealStateDealTa,
    dealStateDealTa,
    dealStateExecutorBondTa: executorBondMint ? getAssociatedTokenAddressSync(executorBondMint, dealState, true) : dealStateDealTa,
  
    executorBondTa: executorBondMint ? getAssociatedTokenAddressSync(executorBondMint, executorPk) : executorDealTa,
  
    dealState,
    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  }).preInstructions([getTotalComputeIxs(400000)[0]])
}

export async function getFinishIx ({
  dealContractProgram,
  initializer,
  dealId,
  clientPk,
  executorPk,
  dealMint,
  holderMode,
  checkerKey = null,
  clientBond = null,
  executorBond = null,
}: {
  dealContractProgram: Program<DealContract>,
  dealId: string | Buffer,
    initializer: PublicKey,
  clientPk: PublicKey,
  executorPk: PublicKey,
  dealMint: PublicKey,
  holderMode?: boolean,
  checkerKey?: PublicKey,
  clientBond?: IdlTypes<DealContract>["Bond"],
  executorBond?: IdlTypes<DealContract>["Bond"],
}) {
  if (!(dealId instanceof Buffer)) {dealId = uuidTodealIdBuf(dealId)}
  dealId = dealId as Buffer;
  const dealState = getDealStatePk(dealId, clientPk, executorPk)[0];

  const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealState, true);
  const clientDealTa = getAssociatedTokenAddressSync(dealMint, clientPk);
  const executorDealTa = getAssociatedTokenAddressSync(dealMint, executorPk);
  const checkerDealTa = checkerKey ? getAssociatedTokenAddressSync(dealMint, checkerKey) : executorDealTa;

  return dealContractProgram.methods.finish()
  .accountsStrict({
    checkerDealTa,
    initializer,
    client: clientPk,
    executor: executorPk,
    checker: checkerKey ? checkerKey : initializer,

    clientHolderTa: getAssociatedTokenAddressSync(HOLDER_MINT, clientPk),

    dealMint,
    clientBondMint: clientBond ? clientBond.mint : dealMint,
    executorBondMint: executorBond ? executorBond.mint : dealMint,

    clientBondTa: clientBond ? getAssociatedTokenAddressSync(clientBond.mint, clientPk) : clientDealTa,
  
    dealStateClientBondTa: clientBond ? getAssociatedTokenAddressSync(clientBond.mint, dealState, true) : dealStateDealTa,
    dealStateDealTa,
    dealStateExecutorBondTa: executorBond ? getAssociatedTokenAddressSync(clientBond.mint, dealState, true) : dealStateDealTa,
    dealStateHolderTa: holderMode ? getAssociatedTokenAddressSync(HOLDER_MINT, dealState, true) : dealStateDealTa,
  
    executorBondTa: executorBond ? getAssociatedTokenAddressSync(executorBond.mint, executorPk) : executorDealTa,
    executorDealTa,
  
    dealState,

    holderMint: HOLDER_MINT,
    serviceFee: SERVICE_FEE_OWNER,
    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  }).preInstructions([getTotalComputeIxs(400000)[0]])
}