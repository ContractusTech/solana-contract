import * as anchor from "@project-serum/anchor";
import { Program, AnchorProvider } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram, Transaction, Commitment } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { DealContract } from "../target/types/deal_contract";
import { assert } from "chai";
import * as fs from 'fs';

describe("Test deal contract with SPL token", () => {
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

  const dealAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();

  const clientAccount = anchor.web3.Keypair.generate();
  const executorAccount = anchor.web3.Keypair.generate();
  const checkerAccount = anchor.web3.Keypair.generate();
  const serviceFeeAccount = anchor.web3.Keypair.generate();
  const mintServiceAuthority = anchor.web3.Keypair.generate();
  const mintServiceKeypair: Keypair = (() => {
    let secret: Uint8Array = JSON.parse(fs.readFileSync(process.env.MINT_KEY_PATH, 'utf-8'))
    return Keypair.fromSecretKey(new Uint8Array(secret))
  })()

  var mint;

  var clientTokenAccount;
  var executorTokenAccount;
  var checkerTokenAccount;
  var serviceFeeTokenAccount;

  var mintService;
  var clientServiceTokenAccount;
  var serviceFeeServiceTokenAccount;

  const initDealWithParams = async (
    dealId,
    amount,
    checkerFee,
    serviceFee,
    clientAccount,
    executorAccount,
    checkerAccount,
    payer,
    serviceFeeTokenAccount,
    clientTokenAccount,
    clientServiceTokenAccount,
    executorTokenAccount,
    checkerTokenAccount,
    mint) => {
    const seed = Buffer.from(anchor.utils.bytes.utf8.encode(dealId))

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
      new anchor.BN(serviceFee),
      new anchor.BN(checkerFee)
    )
      .accounts({
        client: clientAccount.publicKey,
        executor: executorAccount.publicKey,
        checker: checkerAccount.publicKey,
        payer: payer.publicKey,
        serviceFeeAccount: serviceFeeTokenAccount,
        clientTokenAccount: clientTokenAccount,
        clientServiceTokenAccount: clientServiceTokenAccount,
        executorTokenAccount: executorTokenAccount,
        checkerTokenAccount: checkerTokenAccount,
        mint: mint,
        depositAccount: vault_account_pda,
        dealState: state_account_pda,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,

      })
      .signers([clientAccount, executorAccount, checkerAccount, payer])
      .rpc()

    return {
      vault_account_pda,
      state_account_pda,
      vault_account_bump,
      state_account_bump,
      vault_authority_pda,
      seed
    }
  }

  const initDeal = async (dealId, checkerFee, serviceFee) => {
    return await initDealWithParams(
      dealId,
      amount,
      checkerFee,
      serviceFee,
      clientAccount,
      executorAccount,
      checkerAccount,
      payer,
      serviceFeeTokenAccount,
      clientTokenAccount,
      clientServiceTokenAccount,
      executorTokenAccount,
      checkerTokenAccount,
      mint)
  }

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
    })(), [payer])
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

    mintService = await createMint(
      provider.connection,
      payer,
      mintServiceAuthority.publicKey,
      null,
      0, 
      mintServiceKeypair);

    clientTokenAccount = await createAccount(provider.connection, payer, mint, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    executorTokenAccount = await createAccount(provider.connection, payer, mint, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    checkerTokenAccount = await createAccount(provider.connection, payer, mint, checkerAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    serviceFeeTokenAccount = await createAccount(provider.connection, payer, mint, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

    clientServiceTokenAccount = await createAccount(provider.connection, payer, mintService, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
    serviceFeeServiceTokenAccount = await createAccount(provider.connection, payer, mintService, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

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
    const fee = 100
    let data = await initDeal(dealId, fee, service_fee)

    const state = await program.account.dealState.fetch(data.state_account_pda)
    const serviceFeeTokenAccountInfo = await getAccount(
      provider.connection,
      serviceFeeTokenAccount
    )

    assert.ok(serviceFeeTokenAccountInfo.amount.toString() == service_fee.toString())
    assert.ok(state.amount.toNumber().toString() == amount.toString())
    assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
    assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())

    // Try call Init again
    try {
      let _ = await initDeal(dealId, fee, service_fee)
      assert.ok(false)
    } catch {
      assert.ok(true)
    }
  });

  it("Finish deal", async () => {
    let dealId = "123456789"
    let fee = 100
    let data = await initDeal(dealId, fee, service_fee)


    const state = await program.account.dealState.fetch(data.state_account_pda)

    const depositAccount = await getAccount(
      provider.connection,
      state.depositKey
    )

    const clientTokenAccountInfoBefore = await getAccount(
      provider.connection,
      clientTokenAccount
    )

    const executorTokenAccountInfoBefore = await getAccount(
      provider.connection,
      executorTokenAccount
    )

    await program.methods
      .finish(data.seed)
      .accounts({
        initializer: checkerAccount.publicKey,
        authority: data.vault_authority_pda,
        depositAccount: data.vault_account_pda,
        executorTokenAccount: executorTokenAccount,
        checkerTokenAccount: checkerTokenAccount,
        dealState: data.state_account_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([checkerAccount])
      .rpc()

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

    assert.ok(Number(clientTokenAccountInfoBefore.amount).toString() == clientTokenAccountInfo.amount.toString())
    assert.ok((Number(executorTokenAccountInfo.amount)).toString() == (Number(otherTokenBalance) + amount).toString())
    assert.ok((Number(checkerTokenAccountInfo.amount)).toString() == (Number(otherTokenBalance) + fee).toString())

  });

  it("Cancel deal", async () => {

    let dealId = "12345678"
    let checkerFee = 100
    let data = await initDeal(dealId, checkerFee, service_fee)

    const state = await program.account.dealState.fetch(data.state_account_pda)

    const depositAccount = await getAccount(
      provider.connection,
      state.depositKey
    )

    const clientTokenAccountInfoBefore = await getAccount(
      provider.connection,
      clientTokenAccount
    )
    assert.ok(depositAccount.amount.toString() == (amount + checkerFee).toString())
    assert.ok(state.checkerFee.toString() == new anchor.BN(checkerFee).toString())
    assert.ok(state.amount.toNumber().toString() == amount.toString())
    assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
    assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())

    await program.methods
      .cancel(data.seed)
      .accounts({
        initializer: checkerAccount.publicKey,
        authority: data.vault_authority_pda,
        depositAccount: data.vault_account_pda,
        clientTokenAccount: clientTokenAccount,
        dealState: data.state_account_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([checkerAccount])
      .rpc()

    const clientTokenAccountInfo = await getAccount(
      provider.connection,
      clientTokenAccount
    )
    assert.ok((Number(clientTokenAccountInfoBefore.amount) + Number(amount + checkerFee)).toString() == clientTokenAccountInfo.amount.toString())
  });

  it("Try start deal with the same executor and client", async () => {
    var promise = initDealWithParams(
      "1234",
      amount,
      0,
      100,
      clientAccount,
      clientAccount,
      checkerAccount,
      payer,
      serviceFeeTokenAccount,
      clientTokenAccount,
      clientServiceTokenAccount,
      clientTokenAccount,
      checkerTokenAccount,
      mint)
    promise.then(() => {
      assert.ok(false)
    }).catch(() => {
      assert.ok(true)
    })
  })

  it("Try start deal with the zero fee", async () => {
    let promise = initDeal('1234500', 0, 0)
    promise.then(() => {
      assert.ok(false)
    }).catch((error) => {
      assert.ok(true)
    })
  })

  it("Start deal with zero fee. Payment by service token", async () => {
    let holder_mode_amount = 10000
    let amount = 1000
    try {
      await initDealWithParams(
        "1234",
        amount,
        0,
        0,
        clientAccount,
        executorAccount,
        checkerAccount,
        payer,
        serviceFeeTokenAccount,
        clientServiceTokenAccount,
        clientServiceTokenAccount,
        executorTokenAccount,
        checkerTokenAccount,
        mintService)
        assert.ok(false)
        return
    } catch { }
    
    await mintTo(provider.connection, payer, mintService, clientServiceTokenAccount, mintServiceAuthority.publicKey, holder_mode_amount, [mintServiceAuthority])
    const clientTokenAccountInfo = await getAccount(
      provider.connection,
      clientServiceTokenAccount
    )

    assert.ok(clientTokenAccountInfo.mint.toBase58() == mintService.toBase58())
    assert.ok(clientTokenAccountInfo.amount.toString() == holder_mode_amount.toString())
    let data = await initDealWithParams(
      "1234",
      amount,
      0,
      0,
      clientAccount,
      executorAccount,
      checkerAccount,
      payer,
      serviceFeeServiceTokenAccount,
      clientServiceTokenAccount,
      clientServiceTokenAccount,
      executorTokenAccount,
      checkerTokenAccount,
      mintService)

      const state = await program.account.dealState.fetch(data.state_account_pda)
      const serviceFeeTokenAccountInfo = await getAccount(
        provider.connection,
        serviceFeeServiceTokenAccount
      )
  
      assert.ok(serviceFeeTokenAccountInfo.amount.toString() == "0".toString())
      assert.ok(state.amount.toNumber().toString() == amount.toString())
      assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
      assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
  })

  it("Start deal with ivalid client token account", async () => {
    let amount = 1000
    try {
      await initDealWithParams(
        "1234",
        amount,
        0,
        0,
        clientAccount,
        executorAccount,
        checkerAccount,
        payer,
        serviceFeeTokenAccount,
        executorTokenAccount,
        clientServiceTokenAccount,
        executorTokenAccount,
        checkerTokenAccount,
        mintService)
        assert.ok(false)
    } catch { assert.ok(true) }
    
  })
  it("Start deal with ivalid zero amount, fee and service fee with custom token", async () => {

    let amount = 0
    
    try {
      await initDealWithParams(
        "1234333",
        amount,
        0,
        0,
        clientAccount,
        executorAccount,
        checkerAccount,
        payer,
        serviceFeeTokenAccount,
        clientTokenAccount,
        clientServiceTokenAccount,
        executorTokenAccount,
        checkerTokenAccount,
        mintService)
        assert.ok(false)
    } catch(error) {
      assert.ok(true)
    }
  })

  it("Start deal with ivalid zero service fee with custom token", async () => {

    let amount = 1000
    const clientTokenAccountInfo = await getAccount(
      provider.connection,
      clientTokenAccount
    )
    try {
      await initDealWithParams(
        "123433321312",
        amount,
        0,
        0,
        clientAccount,
        executorAccount,
        checkerAccount,
        payer,
        serviceFeeTokenAccount,
        clientTokenAccount,
        clientServiceTokenAccount,
        executorTokenAccount,
        checkerTokenAccount,
        mint)
        assert.ok(false)
    } catch(error) {
      assert.ok(true)
    }
  })
});
