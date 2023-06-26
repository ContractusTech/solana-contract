import * as anchor from "@project-serum/anchor";
import { Program, AnchorProvider } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram, Transaction, Commitment } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { DealContract } from "../target/types/deal_contract";
import { assert } from "chai";
import { v4 as uuid } from 'uuid'
import * as fs from 'fs';

describe("🤖 Tests Contractus smart-contract", () => {
  const commitment: Commitment = 'processed';
  const options = AnchorProvider.defaultOptions();
  const program = anchor.workspace.DealContract as Program<DealContract>;
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  
  describe("👽️ Deals with third party checker (no performance bond)", ()=> {
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

    const _createDeal = async (
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
      mint,
      holderMint,
      holderMode
    ) => {
      const seed = Buffer.from(anchor.utils.bytes.utf8.encode(dealId)).subarray(0, 32)
  
      const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit")), clientAccount.publicKey.toBuffer(),  executorAccount.publicKey.toBuffer()],
       
        program.programId
      );
      const [holder_vault_account_pda, _holder_vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("holder_deposit")), clientAccount.publicKey.toBuffer(),  executorAccount.publicKey.toBuffer()],
        program.programId
      );
      var vault_account_pda = _vault_account_pda;
      var vault_account_bump = _vault_account_bump;
  
      const [vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("auth")), clientAccount.publicKey.toBuffer(),  executorAccount.publicKey.toBuffer()],
        program.programId
      );
  
      const [state_account_pda, _state_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("state")), clientAccount.publicKey.toBuffer(),  executorAccount.publicKey.toBuffer()],
        program.programId
      );
  
      await program.methods.initializeWithChecker(
        seed,
        new anchor.BN(amount),
        new anchor.BN(serviceFee),
        new anchor.BN(checkerFee),
        holderMode
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
          authority: vault_authority_pda,
          depositAccount: vault_account_pda,
          dealState: state_account_pda,
          holderDepositAccount: holder_vault_account_pda,
          holderMint: holderMint.publicKey,
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
        vault_authority_pda,
        seed,
        holder_vault_account_pda
      }
    }

    const createDeal = async (dealId, amount, checkerFee, serviceFee, holderMode) => {
      return await _createDeal(
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
        mint, 
        mintServiceKeypair,
        holderMode)
    }

    before( async()=>{
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
  
    })

    it("Validating state", async () => {

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

    it("Create deal", async () => {

      const dealId = uuid()
      const fee = 100
      const amount = 1000
      const serviceFee = 50
      let data = await createDeal(dealId, amount, fee, serviceFee, false)
     
      const state = await program.account.dealState.fetch(data.state_account_pda)
  
      const serviceFeeTokenAccountInfo = await getAccount(
        provider.connection,
        serviceFeeTokenAccount
      )
  
      const depositInfo = await getAccount(
        provider.connection,
        data.vault_account_pda
      )
  
      assert.ok(serviceFeeTokenAccountInfo.amount.toString() == serviceFee.toString())
      assert.ok(state.amount.toNumber().toString() == amount.toString())
      assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
      assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
    });

    it("Try recreate deal with same ID", async () => {


      var serviceFeeTokenAccountInfo = await getAccount(
        provider.connection,
        serviceFeeTokenAccount
      )
      const amountBefore = serviceFeeTokenAccountInfo.amount
      const dealId = uuid()
      const fee = 100
      const amount = 1000
      const serviceFee = 50
      let data = await createDeal(dealId, amount, fee, serviceFee, false)
     
      const state = await program.account.dealState.fetch(data.state_account_pda)
  
      serviceFeeTokenAccountInfo = await getAccount(
        provider.connection,
        serviceFeeTokenAccount
      )
  
      const depositInfo = await getAccount(
        provider.connection,
        data.vault_account_pda
      )
  
      assert.ok(serviceFeeTokenAccountInfo.amount.toString() == (amountBefore + BigInt(serviceFee)).toString())
      assert.ok(state.amount.toNumber().toString() == amount.toString())
      assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
      assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
  
      // Try call Init again
      try {
        let _ = await createDeal(dealId, amount, fee, serviceFee, false)
        assert.ok(false)
      } catch {
        assert.ok(true)
      }
    });

    it("Create deal and finish", async () => {

      const dealId = uuid()
      const fee = 100
      const amount = 1000
      const serviceFee = 50
      let data = await createDeal(dealId, amount, fee, serviceFee, false)
  
      const state = await program.account.dealState.fetch(data.state_account_pda)
  
      const depositAccount = await getAccount(
        provider.connection,
        state.depositKey
      )

      assert.ok(Number(depositAccount.amount) == Number(amount + fee))

      const clientTokenAccountInfoBefore = await getAccount(
        provider.connection,
        clientTokenAccount
      )

      try {
        await program.methods
        .finish(data.seed)
        .accounts({
          initializer: checkerAccount.publicKey,
          depositAccount: data.vault_account_pda,
          executorTokenAccount: executorTokenAccount,
          holderDepositAccount: data.holder_vault_account_pda,
          authority: data.vault_authority_pda,
          checkerTokenAccount: checkerTokenAccount,
          dealState: data.state_account_pda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([checkerAccount])
        .rpc()
      } catch(error) {
        console.log(error)
        assert.ok(false)
      }
      
  
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
  
      assert.ok(clientTokenAccountInfoBefore.amount.toString() == clientTokenAccountInfo.amount.toString())
      assert.ok(executorTokenAccountInfo.amount.toString() == (Number(otherTokenBalance) + amount).toString())
      assert.ok(checkerTokenAccountInfo.amount.toString() == (Number(otherTokenBalance) + fee).toString())
  
    });

    it("Create deal and cancel", async () => {

      const dealId = uuid()
      const checkerFee = 100
      const amount = 1000
      const serviceFee = 50
      let data = await createDeal(dealId, amount, checkerFee, serviceFee, false)
    

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
          depositAccount: data.vault_account_pda,
          authority: data.vault_authority_pda,
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

    it("Try create deal with the same executor and client", async () => {
      var promise = _createDeal(
        uuid(),
        1000,
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
        mint, 
        mintServiceKeypair, 
        false)
      promise.then(() => {
        assert.ok(false)
      }).catch((error) => {
        assert.ok(error.error.errorCode.code == "ConstraintRaw")
      })
    })

    it("Try create deal with the zero fee (holder mode off)", async () => {
      let promise = createDeal(uuid(), 1000, 0, 0, false)
      promise.then(() => {
        assert.ok(false)
      }).catch((error) => {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      })
    })

    it("Try create deal with the zero fee (holder mode on, but not fund)", async () => {
      try {
        await _createDeal(
          uuid(),
          1000,
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
          mintService,
          mintServiceKeypair, 
          true)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })

    it("Create deal with ivalid client token account", async () => {
      try {
        await _createDeal(
          uuid(),
          1000,
          0,
          0,
          clientAccount,
          executorAccount,
          checkerAccount,
          payer,
          serviceFeeTokenAccount,
          executorTokenAccount, // <- Invalid here
          clientServiceTokenAccount,
          executorTokenAccount,
          checkerTokenAccount,
          mintService, mintServiceKeypair, 
          false)
          assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
      
    })

    it("Create deal with zero amount, fee and service fee with custom token", async () => {

      let amount = 0
      
      try {
        await _createDeal(
          uuid(),
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
          mintService, 
          mintServiceKeypair, 
          false)
          assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })

    it("Create deal with zero service fee with custom token", async () => {

      let amount = 1000

      try {
        await _createDeal(
          uuid(),
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
          mint, mintServiceKeypair, 
          false)
          assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })
  
  })

  describe("👻 Deals with performance bond (no checker)", ()=> {
    const amount = 1000;
    const service_fee = 50;
    const clientTokenBalance = 10000;
    const otherTokenBalance = 500;
    const serviceFeeTokenBalance = 0;
    const bondTokenBalance = 0;

    const dealAccount = anchor.web3.Keypair.generate();
    const payer = anchor.web3.Keypair.generate();
    const mintAuthority = anchor.web3.Keypair.generate();

    const clientAccount = anchor.web3.Keypair.generate();
    const executorAccount = anchor.web3.Keypair.generate();
    const checkerAccount = anchor.web3.Keypair.generate();
    const serviceFeeAccount = anchor.web3.Keypair.generate();
    const mintServiceAuthority = anchor.web3.Keypair.generate();
    const bondMintAuthority = anchor.web3.Keypair.generate();
    const mintServiceKeypair: Keypair = (() => {
      let secret: Uint8Array = JSON.parse(fs.readFileSync(process.env.MINT_KEY_PATH, 'utf-8'))
      return Keypair.fromSecretKey(new Uint8Array(secret))
    })()

    var mint;
    var mintBond;

    var clientTokenAccount;
    var executorTokenAccount;
    var checkerTokenAccount;
    var serviceFeeTokenAccount;

    var bondClientTokenAccount;
    var bondExecutorTokenAccount;

    var mintService;
    var clientServiceTokenAccount;
    var serviceFeeServiceTokenAccount;

    const createDeal = async (
      dealId,
      amount,
      clientBondAmount,
      executorBondAmount,
      serviceFee,
      clientAccount,
      executorAccount,
      payer,
      serviceFeeTokenAccount,
      clientTokenAccount,
      clientServiceTokenAccount,
      executorTokenAccount,
      clientBondTokenAccount,
      clientBondMint,
      executorBondTokenAccount,
      executorBondMint,
      mint,
      holderMint,
      holderMode,
      deadline
      ) => {
      const seed = Buffer.from(anchor.utils.bytes.utf8.encode(dealId)).subarray(0, 32)

      const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );
      const [_holder_vault_account_pda, _holder_vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("holder_deposit")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );
      var vault_account_pda = _vault_account_pda;
      var vault_account_bump = _vault_account_bump;

      const [_vault_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("auth")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );

      const [_state_account_pda, _state_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("state")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );

      const [executor_bond_vault_account_pda, _executor_bond_vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit_bond_executor")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );

      const [client_bond_vault_account_pda, _client_bond_vault_account_bump] = await PublicKey.findProgramAddress(
        [seed, Buffer.from(anchor.utils.bytes.utf8.encode("deposit_bond_client")), clientAccount.publicKey.toBuffer(), executorAccount.publicKey.toBuffer()],
        program.programId
      );

      var state_account_bump = _state_account_bump
      var state_account_pda = _state_account_pda

      var vault_authority_pda = _vault_authority_pda;
      await program.methods.initializeWithBond(
        seed,
        new anchor.BN(amount),
        new anchor.BN(clientBondAmount),
        new anchor.BN(executorBondAmount),
        new anchor.BN(serviceFee),
        new anchor.BN(deadline),
        holderMode
      ).accounts({
          client: clientAccount.publicKey,
          executor: executorAccount.publicKey,
          payer: payer.publicKey,
          serviceFeeAccount: serviceFeeTokenAccount,
          clientTokenAccount: clientTokenAccount,
          clientServiceTokenAccount: clientServiceTokenAccount,
          executorTokenAccount: executorTokenAccount,
          clientBondAccount: clientBondTokenAccount,
          executorBondAccount: executorBondTokenAccount,
          clientBondMint: clientBondMint,
          executorBondMint: executorBondMint,
          authority: _vault_authority_pda,
          mint: mint,
          depositAccount: vault_account_pda,
          dealState: state_account_pda,
          holderDepositAccount: _holder_vault_account_pda,
          holderMint: holderMint,
          depositClientBondAccount: client_bond_vault_account_pda,
          depositExecutorBondAccount: executor_bond_vault_account_pda,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([clientAccount, executorAccount, payer])
        .rpc()

      return {
        vault_account_pda,
        state_account_pda,
        vault_account_bump,
        state_account_bump,
        vault_authority_pda,
        seed,
        executor_bond_vault_account_pda,
        client_bond_vault_account_pda
      }
    }

    before(async () => {
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

      mintBond = await createMint(
        provider.connection,
        payer,
        bondMintAuthority.publicKey,
        null,
        0);

      try {
        mintService = await createMint(
          provider.connection,
          payer,
          mintServiceAuthority.publicKey,
          null,
          0,
          mintServiceKeypair);
      } catch {
        mintService = mintServiceKeypair.publicKey
      }
      

      clientTokenAccount = await createAccount(provider.connection, payer, mint, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      executorTokenAccount = await createAccount(provider.connection, payer, mint, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      checkerTokenAccount = await createAccount(provider.connection, payer, mint, checkerAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      serviceFeeTokenAccount = await createAccount(provider.connection, payer, mint, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

      bondClientTokenAccount = await createAccount(provider.connection, payer, mintBond, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      bondExecutorTokenAccount = await createAccount(provider.connection, payer, mintBond, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      
      clientServiceTokenAccount = await createAccount(provider.connection, payer, mintService, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      serviceFeeServiceTokenAccount = await createAccount(provider.connection, payer, mintService, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

      await mintTo(provider.connection, payer, mint, clientTokenAccount, mintAuthority.publicKey, clientTokenBalance, [mintAuthority])
      await mintTo(provider.connection, payer, mint, executorTokenAccount, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
      await mintTo(provider.connection, payer, mint, checkerTokenAccount, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
      await mintTo(provider.connection, payer, mint, serviceFeeTokenAccount, mintAuthority.publicKey, serviceFeeTokenBalance, [mintAuthority])
    })

    it("Validate state", async () => {

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
    
    it("Create deal with holder mode (no CTUS fund)", async () => {
      try {
        var data = await createDeal(
          uuid(),
          amount,
          0,
          0,
          0,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          true,
          new Date().getTime() / 1000)
    
          const state = await program.account.dealState.fetch(data.state_account_pda)
          const serviceFeeTokenAccountInfo = await getAccount(
            provider.connection,
            serviceFeeTokenAccount
          )
          assert.ok(serviceFeeTokenAccountInfo.amount.toString() == service_fee.toString())
          assert.ok(state.amount.toNumber().toString() == amount.toString())
          assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
          assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
      } catch(err) {
        assert.ok(err.error.origin == "client_service_token_account")
      }
    })

    it("Create deal with executor bond", async () => {

      await mintTo(provider.connection, payer, mintBond, bondExecutorTokenAccount, bondMintAuthority.publicKey, 100, [bondMintAuthority])
    
      let serviceFee = BigInt(100)
      let executorBond = BigInt(56)
      const serviceFeeTokenAccountInfo = await getAccount(
        provider.connection,
        serviceFeeTokenAccount
      )
      var serviceAccountAmount = serviceFeeTokenAccountInfo.amount
      
      try {
        var data = await createDeal(
          uuid(),
          amount,
          0,
          executorBond,
          serviceFee,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          false,
          new Date().getTime() / 1000)
    
          const state = await program.account.dealState.fetch(data.state_account_pda)
          const serviceFeeTokenAccountInfo = await getAccount(
            provider.connection,
            serviceFeeTokenAccount
          )
          const executorBondTokenAccountInfo = await getAccount(
            provider.connection,
            data.executor_bond_vault_account_pda
          )
          assert.ok(serviceFeeTokenAccountInfo.amount.toString() == (serviceAccountAmount + serviceFee).toString())
          assert.ok(executorBondTokenAccountInfo.amount.toString() == executorBond.toString())
          assert.ok(state.amount.toNumber().toString() == amount.toString())
          assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
          assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
      } catch(err) {
        console.log(err)
        assert.ok(false)
      }
    })

    it("Try create deal twice", async () => {

      await mintTo(provider.connection, payer, mintBond, bondExecutorTokenAccount, bondMintAuthority.publicKey, 100, [bondMintAuthority])
    
      let serviceFee = BigInt(100)
      let executorBond = BigInt(56)
      let dealId = uuid()
      try {
        await createDeal(
          dealId,
          amount,
          0,
          executorBond,
          serviceFee,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          false,
          new Date().getTime() / 1000)

          assert.ok(true)
        await createDeal(
          dealId,
          amount,
          0,
          executorBond,
          serviceFee,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          false,
          new Date().getTime() / 1000)
          assert.ok(false)
      } catch(err) {
        assert.ok(err.error.origin == "deposit_account")
      }
    })

    it("Create deal with bond and try cancel", async () => {
      await mintTo(provider.connection, payer, mintBond, bondExecutorTokenAccount, bondMintAuthority.publicKey, 100, [bondMintAuthority])
    
      let serviceFee = BigInt(100)
      let executorBond = BigInt(56)

      var executorTokenAccountInfo = await getAccount(
        provider.connection,
        executorTokenAccount
      )

      try {
        let data = await createDeal(
          uuid(),
          amount,
          0,
          executorBond,
          serviceFee,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          false,
          (new Date().getTime() / 1000) + 1000
        )
        executorTokenAccountInfo = await getAccount(
          provider.connection,
          executorTokenAccount
        )
        try {
          await program.methods
          .cancel(data.seed)
          .accounts({
            initializer: clientAccount.publicKey,
            depositAccount: data.vault_account_pda,
            authority: data.vault_authority_pda,
            clientTokenAccount: clientTokenAccount,
            dealState: data.state_account_pda,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([clientAccount])
          .rpc()
        } catch(error) {
          assert.ok(error.error.errorCode.code == 'NeedCancelWithBond')
        }
        try {
          await program.methods
          .cancelWithBond(data.seed)
          .accounts({
            clientBondAccount: bondClientTokenAccount,
            executorBondAccount: bondExecutorTokenAccount,
            depositClientBondAccount: data.client_bond_vault_account_pda,
            depositExecutorBondAccount: data.executor_bond_vault_account_pda,
            initializer: clientAccount.publicKey,
            depositAccount: data.vault_account_pda,
            authority: data.vault_authority_pda,
            clientTokenAccount: clientTokenAccount,
            dealState: data.state_account_pda,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([clientAccount])
          .rpc()
        } catch(error) {
          assert.ok(error.error.errorCode.code == 'DeadlineNotCome')
        }
      } catch(error) { 
        console.log(error)
        assert.ok(false)
      }
    })

    it("Create and finish deal with bond and deadline", async () => {

      await mintTo(provider.connection, payer, mintBond, bondClientTokenAccount, bondMintAuthority.publicKey, 100, [bondMintAuthority])
      await mintTo(provider.connection, payer, mintBond, bondExecutorTokenAccount, bondMintAuthority.publicKey, 100, [bondMintAuthority])

      var bondClientTokenAccountInfo = await getAccount(
        provider.connection,
        bondClientTokenAccount
      )
      var bondClientTokenAmountBefore = bondClientTokenAccountInfo.amount

      var bondExecutorTokenAccountInfo = await getAccount(
        provider.connection,
        bondExecutorTokenAccount
      )
      var bondExecutorTokenAmountBefore = bondExecutorTokenAccountInfo.amount

      var clientTokenAccountInfo = await getAccount(
        provider.connection,
        clientTokenAccount
      )
      var clientTokenAmountBefore = clientTokenAccountInfo.amount
      
      let amount = BigInt(100)
      let serviceFee = BigInt(100)
      let executorBond = BigInt(56)
      let clientBond = BigInt(40)
      let data
      let deadline = new Date().getTime() / 1000
      try {
        data = await createDeal(
          uuid(),
          amount,
          clientBond,
          executorBond,
          serviceFee,
          clientAccount,
          executorAccount,
          payer,
          serviceFeeTokenAccount,
          clientTokenAccount,
          clientServiceTokenAccount,
          executorTokenAccount,
          bondClientTokenAccount,
          mintBond,
          bondExecutorTokenAccount,
          mintBond,
          mint,
          mintService,
          false,
          deadline
        )
        
        bondClientTokenAccountInfo = await getAccount(
          provider.connection,
          bondClientTokenAccount
        )
        var bondClientTokenAmountAfter = bondClientTokenAccountInfo.amount
  
        bondExecutorTokenAccountInfo = await getAccount(
          provider.connection,
          bondExecutorTokenAccount
        )
        var bondExecutorTokenAmountAfter = bondExecutorTokenAccountInfo.amount
        
       let depositBondExecutorTokenAccountInfo = await getAccount(
          provider.connection,
          data.executor_bond_vault_account_pda
        )

        let depositBondClientTokenAccountInfo = await getAccount(
          provider.connection,
          data.client_bond_vault_account_pda
        )

        clientTokenAccountInfo = await getAccount(
          provider.connection,
          clientTokenAccount
        )
        var clientTokenAmountAfter = clientTokenAccountInfo.amount

        assert.ok(bondExecutorTokenAmountAfter < bondExecutorTokenAmountBefore)
        assert.ok(clientTokenAmountAfter < clientTokenAmountBefore)
        assert.ok(bondClientTokenAmountAfter < bondClientTokenAmountBefore)
        assert.ok(depositBondExecutorTokenAccountInfo.amount == executorBond)
        assert.ok(depositBondClientTokenAccountInfo.amount == clientBond)

        let before_deadline = new Date().getTime() / 1000
        assert.ok(before_deadline > deadline)

        await program.methods
          .cancelWithBond(data.seed)
          .accounts({
            clientBondAccount: bondClientTokenAccount,
            executorBondAccount: bondExecutorTokenAccount,
            depositClientBondAccount: data.client_bond_vault_account_pda,
            depositExecutorBondAccount: data.executor_bond_vault_account_pda,
            initializer: clientAccount.publicKey,
            depositAccount: data.vault_account_pda,
            authority: data.vault_authority_pda,
            clientTokenAccount: clientTokenAccount,
            dealState: data.state_account_pda,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([clientAccount])
          .rpc()

          bondClientTokenAccountInfo = await getAccount(
            provider.connection,
            bondClientTokenAccount
          )
          var bondClientTokenAmountAfterCancel = bondClientTokenAccountInfo.amount
    
          bondExecutorTokenAccountInfo = await getAccount(
            provider.connection,
            bondExecutorTokenAccount
          )
          var bondExecutorTokenAmountAfterCancel = bondExecutorTokenAccountInfo.amount

          clientTokenAccountInfo = await getAccount(
            provider.connection,
            clientTokenAccount
          )
          var clientTokenAmountAfterCancel = clientTokenAccountInfo.amount

          assert.ok(bondClientTokenAmountAfterCancel == bondClientTokenAmountBefore)
          assert.ok(bondExecutorTokenAmountAfterCancel == bondExecutorTokenAmountBefore)
          assert.ok(clientTokenAmountAfterCancel == clientTokenAmountBefore - serviceFee)

      } catch(error) { 
        assert.ok(false)
      }
    })
  })
});
