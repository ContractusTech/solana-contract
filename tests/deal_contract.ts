import * as anchor from "@project-serum/anchor";
import { Program, AnchorProvider, AccountClient, SplToken } from "@project-serum/anchor";

import { PublicKey, SystemProgram, Keypair, Transaction, Connection, Commitment,  AccountInfo } from '@solana/web3.js';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { DealContract } from "../target/types/deal_contract";
import { assert } from "chai";

describe("Contractus contract tests", () => {
  const commitment: Commitment = 'processed';
  const options = AnchorProvider.defaultOptions();
  const program = anchor.workspace.DealContract as Program<DealContract>;

  const provider = anchor.AnchorProvider.env()

  anchor.setProvider(provider)
  const amount = 1000;
  const service_fee = 50;
  const clientTokenBalance = 10000;
  const otherTokenBalance = 500;
  const serviceFeeTokenBalance = 0;
  // const ownerAccount = anchor.web3.Keypair.generate();
  const dealAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const clientAccount = anchor.web3.Keypair.generate();
  const executorAccount = anchor.web3.Keypair.generate();
  const checkerAccount = anchor.web3.Keypair.generate();
  const serviceFeeAccount = anchor.web3.Keypair.generate();

  let mint = null as PublicKey;
  let clientTokenAccount = null;
  let executorTokenAccount = null;
  let checkerTokenAccount = null;
  let serviceFeeTokenAccount = null;
  // let vault_account_pda = null;
  // let vault_account_bump = null;
  // let vault_authority_pda = null;
  // let state_account_pda = null;
  // let state_account_bump = null;

  it("Initialize state", async () => {
  
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 2000000000),
      "processed"
    );

    await provider.sendAndConfirm((() => {
      const tx = new Transaction();
      tx.add(
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: clientAccount.publicKey,
          lamports: 100000000,
        })
      );
      return tx;
    })(),[payer])
    const accountInfo = await provider.connection.getAccountInfo(
      clientAccount.publicKey
    )
    assert.ok(accountInfo.lamports == 100000000)
    mint = await createMint(
      provider.connection,
      payer, 
      mintAuthority.publicKey, 
      null,
      0);

    clientTokenAccount = await createAccount(provider.connection, payer, mint, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    executorTokenAccount = await createAccount(provider.connection, payer, mint, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    checkerTokenAccount = await createAccount(provider.connection, payer, mint, checkerAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    serviceFeeTokenAccount = await createAccount(provider.connection, payer, mint, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

    await mintTo(provider.connection, payer, mint, clientTokenAccount, mintAuthority.publicKey, clientTokenBalance, [mintAuthority])
    await mintTo(provider.connection, payer, mint, executorTokenAccount, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
    await mintTo(provider.connection, payer, mint, checkerTokenAccount, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
    await mintTo(provider.connection, payer, mint, serviceFeeTokenAccount, mintAuthority.publicKey, serviceFeeTokenBalance, [mintAuthority])

    const clientTokenAccountInfo = await getAccount(
      provider.connection, 
      clientTokenAccount
    )
    const executorTokenAccountInfo = await getAccount(
      provider.connection, 
      executorTokenAccount
    )
    const checkerTokenAccountInfo = await getAccount(
      provider.connection, 
      checkerTokenAccount
    )

    assert.ok(clientTokenAccountInfo.mint.toBase58() == mint.toBase58())
    assert.ok(clientTokenAccountInfo.amount.toString() == clientTokenBalance.toString())
    assert.ok(executorTokenAccountInfo.amount.toString() == otherTokenBalance.toString())
    assert.ok(checkerTokenAccountInfo.amount.toString() == otherTokenBalance.toString())
  });

  it("Initialize deal", async () => { 
   
    const dealId = "12312456"
    const seed = Buffer.from(anchor.utils.bytes.utf8.encode(dealId))
    
    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit"))],
      program.programId
    );
    let vault_account_pda = _vault_account_pda;
    let vault_account_bump = _vault_account_bump;

    const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("auth"))],
      program.programId
    );

    const [_state_account_pda, _state_account_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("state"))],
      program.programId
    );

    let state_account_bump = _state_account_bump
    let state_account_pda = _state_account_pda

    let vault_authority_pda = _vault_authority_pda;
    
    await program.methods.initialize(
      vault_account_bump,
      state_account_bump,
      seed,
      new anchor.BN(amount),
      new anchor.BN(service_fee),
      new anchor.BN(0)
    )
    .accounts({
      client: clientAccount.publicKey,
      executor: executorAccount.publicKey,
      checker: checkerAccount.publicKey,
      payer: payer.publicKey,
      serviceFeeAccount: serviceFeeTokenAccount,
      clientTokenAccount: clientTokenAccount,
      mint: mint,
      depositAccount: vault_account_pda,
      dealState: state_account_pda,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      
    })
    .signers([clientAccount, executorAccount, checkerAccount, payer])
    .rpc()
    
    const state = await program.account.dealState.fetch(state_account_pda)
    const serviceFeeTokenAccountInfo = await getAccount(
      provider.connection, 
      serviceFeeTokenAccount
    )

    assert.ok(serviceFeeTokenAccountInfo.amount.toString() == service_fee.toString())
    assert.ok(state.amount.toNumber().toString() == amount.toString())
    assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
    assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())

      // Try call Init again
    await program.methods.initialize(
      vault_account_bump,
      state_account_bump,
      seed,
      new anchor.BN(amount),
      new anchor.BN(service_fee),
      new anchor.BN(0)
    )
    .accounts({
      client: clientAccount.publicKey,
      executor: executorAccount.publicKey,
      checker: checkerAccount.publicKey,
      payer: payer.publicKey,
      serviceFeeAccount: serviceFeeTokenAccount,
      clientTokenAccount: clientTokenAccount,
      mint: mint,
      depositAccount: vault_account_pda,
      dealState: state_account_pda,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      
    })
    .signers([clientAccount, executorAccount, checkerAccount, payer])
    .rpc()
    .then(()=>{
      assert.ok(false)
    })
    .catch((error)=>{
      assert.ok(true)
    })
  });

  it("Cancel deal", async () => { 

    const seed = Buffer.from(anchor.utils.bytes.utf8.encode("12345678"))
    
    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit"))],
      program.programId
    );
    var vault_account_pda = _vault_account_pda;
    var vault_account_bump = _vault_account_bump;

    const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("auth"))],
      program.programId
    );

    const [_state_account_pda, _state_account_bump] = await PublicKey.findProgramAddress(
      [seed, Buffer.from(anchor.utils.bytes.utf8.encode("state"))],
      program.programId
    );

    var state_account_bump = _state_account_bump
    var state_account_pda = _state_account_pda

    var vault_authority_pda = _vault_authority_pda;
    
    await program.methods.initialize(
      vault_account_bump,
      state_account_bump,
      seed,
      new anchor.BN(amount),
      new anchor.BN(service_fee),
      new anchor.BN(0)
    )
    .accounts({
      client: clientAccount.publicKey,
      executor: executorAccount.publicKey,
      checker: checkerAccount.publicKey,
      payer: payer.publicKey,
      serviceFeeAccount: serviceFeeTokenAccount,
      clientTokenAccount: clientTokenAccount,
      mint: mint,
      depositAccount: vault_account_pda,
      dealState: state_account_pda,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      
    })
    .signers([clientAccount, executorAccount, checkerAccount, payer])
    .rpc()

    const state = await program.account.dealState.fetch(state_account_pda)
    
    const depositAccount = await getAccount(
      provider.connection, 
      state.depositKey
    )
   
    const clientTokenAccountInfoBefore = await getAccount(
      provider.connection, 
      clientTokenAccount
    )
    assert.ok(depositAccount.amount.toString() == amount.toString())
    assert.ok(state.checkerFee.toString() == new anchor.BN(0).toString())
    assert.ok(state.amount.toNumber().toString() == amount.toString())
    assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
    assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())

    await program.methods
    .cancel(seed)
    .accounts({
      initializer: checkerAccount.publicKey,
      authority: vault_authority_pda,
      depositAccount: vault_account_pda,
      clientTokenAccount: clientTokenAccount,
      dealState: state_account_pda,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([checkerAccount])
    .rpc()
    
    const clientTokenAccountInfo = await getAccount(
      provider.connection, 
      clientTokenAccount
    )
    assert.ok((Number(clientTokenAccountInfoBefore.amount) + Number(amount)).toString() == clientTokenAccountInfo.amount.toString())
  });
});
