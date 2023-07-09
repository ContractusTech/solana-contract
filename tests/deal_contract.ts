import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, IdlTypes, BN } from "@coral-xyz/anchor";
import { PublicKey, Keypair, Signer, SystemProgram, Transaction, Commitment, AddressLookupTableAccount, AddressLookupTableProgram, VersionedTransaction, VersionedMessage } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount, DEFAULT_ACCOUNT_STATE_SIZE, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { DealContract, IDL as DC_IDL } from "../target/types/deal_contract";
import { assert } from "chai";
import { v4 as uuid } from 'uuid'
import { DEAL_CONTRACT_PROGRAM_ID, getCancelIx, getDealStatePk, getFinishIx, getInitializeIx, getTotalComputeIxs, HOLDER_MINT, SERVICE_FEE_MINT, SERVICE_FEE_MINT_KP, SERVICE_FEE_OWNER, SERVICE_FEE_TA, signAndSendIxs as signAndSendIxs, uuidTodealIdBuf } from "./client";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import './keys';
import { checkerKp, clientKp, executorKp, holderMintKp, mintAuthorityKp, payerKp, serviceFeeMintKp, serviceKp } from "./keys";
import { seed } from "@coral-xyz/anchor/dist/cjs/idl";

const ADDRESS_LOOKUP_TABLE_ADDRESS: PublicKey = new PublicKey("9479m8V6EuvFKPC812s8BZ3g3hD5Ru63hkR23Y7DLvEs");
let ADDRESS_LOOKUP_TABLE_ACCOUNT: AddressLookupTableAccount | undefined = undefined;
async function getAddressLookupTable(pubkey?: PublicKey) {
  if (ADDRESS_LOOKUP_TABLE_ACCOUNT === undefined) {
    if (!pubkey) { throw new Error("addressLookupTable pubkey hasn't been passed")}
    const alt = await conn.getAddressLookupTable(pubkey);
    if (!alt.value) { throw new Error("addressLookupTable hasn't been properly created")}
    console.log(`ALT: ${pubkey.toString()}`);
    ADDRESS_LOOKUP_TABLE_ACCOUNT = alt.value as AddressLookupTableAccount
  } 
  return ADDRESS_LOOKUP_TABLE_ACCOUNT as AddressLookupTableAccount
}

console.log(`
payerKp: ${payerKp.publicKey.toString()}    
clientKp: ${clientKp.publicKey.toString()}    
executorKp: ${executorKp.publicKey.toString()}    
checkerKp: ${checkerKp.publicKey.toString()}    
mintAuthorityKp: ${mintAuthorityKp.publicKey.toString()}    
serviceKp: ${serviceKp.publicKey.toString()}    
serviceFeeMintKp: ${serviceFeeMintKp.publicKey.toString()}    
holderMintKp: ${holderMintKp.publicKey.toString()}    
`)

const COMMITMENT = 'confirmed'

const PROGRAM_ID = new PublicKey("GKNkN4uDJWmidEC9h5Q9GQXNg48Go6q5bdnkDj6bSopz")

const conn = new anchor.web3.Connection("http://0.0.0.0:8899", {commitment: COMMITMENT});
const wallet = NodeWallet.local();

const confirmOptions = {commitment: COMMITMENT as Commitment, skipPreflight: true};

const provider = new anchor.AnchorProvider(conn, wallet, { commitment: COMMITMENT, skipPreflight: true });
  // console.log(`provider: ${JSON.stringify(provider)}`)
anchor.setProvider(provider)
const program = new Program( DC_IDL as anchor.Idl, PROGRAM_ID, provider) as Program<DealContract>;


describe("🤖 Tests Contractus smart-contract", () => {
  let addressLookupTablePk: PublicKey;
  it("create addressLookupTable", async()=>{
    const alt = await conn.getAccountInfo(new PublicKey(ADDRESS_LOOKUP_TABLE_ADDRESS));
    if (alt.data.length > 0) { 
      getAddressLookupTable(ADDRESS_LOOKUP_TABLE_ADDRESS); 
      return 
    }
    
    const [ix, pk] = AddressLookupTableProgram.createLookupTable({
      authority: payerKp.publicKey, 
      payer: payerKp.publicKey, 
      recentSlot: (await conn.getSlot())
    });

    addressLookupTablePk = pk;

    const tx = new Transaction();
    tx.add(ix);
    tx.recentBlockhash = (await conn.getLatestBlockhash()).blockhash;
    tx.feePayer = payerKp.publicKey;
    tx.sign(payerKp);
    const createTxSig = await conn.sendTransaction(tx, [payerKp], {skipPreflight: true})
    await conn.confirmTransaction(createTxSig, COMMITMENT);
  })

  it("extendLookupTable", async() => {
    if (!addressLookupTablePk) { addressLookupTablePk = ADDRESS_LOOKUP_TABLE_ADDRESS; return }
    const extendInstruction = AddressLookupTableProgram.extendLookupTable({
      payer: payerKp.publicKey,
      authority: payerKp.publicKey,
      lookupTable: addressLookupTablePk,
      addresses: [
        DEAL_CONTRACT_PROGRAM_ID,
        HOLDER_MINT,
        SERVICE_FEE_MINT,
        SERVICE_FEE_OWNER,
        SERVICE_FEE_TA,
        SystemProgram.programId,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_PROGRAM_ID,
        clientKp.publicKey,
        executorKp.publicKey,
      ],
    });

    const tx = new Transaction();
    tx.add(extendInstruction);
    tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
    tx.feePayer = payerKp.publicKey;
    tx.sign(payerKp);
    console.log("We have to wait until extendLookupTable transaction is finalized...");
    const extendTxSig = await conn.sendTransaction(tx, [payerKp], {skipPreflight: true})
    await conn.confirmTransaction(extendTxSig, "finalized");

    await getAddressLookupTable(addressLookupTablePk);
  })

  
  it("transfer SOL from payer to another accounts", async()=>{
    await provider.sendAndConfirm((() => {
      const tx = new Transaction();
      tx.add(SystemProgram.transfer({fromPubkey: payerKp.publicKey,toPubkey: clientKp.publicKey,lamports: 1000000000,}));
      tx.add(SystemProgram.transfer({fromPubkey: payerKp.publicKey,toPubkey: executorKp.publicKey,lamports: 1000000000,}));
      tx.add(SystemProgram.transfer({fromPubkey: payerKp.publicKey,toPubkey: mintAuthorityKp.publicKey,lamports: 1000000000,}));
      tx.add(SystemProgram.transfer({fromPubkey: payerKp.publicKey,toPubkey: serviceKp.publicKey,lamports: 1000000000,}));
      tx.add(SystemProgram.transfer({fromPubkey: payerKp.publicKey,toPubkey: checkerKp.publicKey,lamports: 1000000000,}));
      return tx;
    })(), [payerKp])
  })
  
  it("create serviceFee token", async() => {
    await createMint(
      provider.connection,
      payerKp,
      mintAuthorityKp.publicKey,
      null,
      0,
      serviceFeeMintKp,
      {commitment: COMMITMENT}
    );
    await createAccount(provider.connection, payerKp, serviceFeeMintKp.publicKey, serviceKp.publicKey, null, confirmOptions, TOKEN_PROGRAM_ID);
  }) 

  it("create `holder` token", async() => {
    await createMint(
      provider.connection,
      payerKp,
      mintAuthorityKp.publicKey,
      null,
      0,
      holderMintKp,
      {commitment: COMMITMENT}
    );
  })
  
  describe("👽️ Deals with third party checker (no performance bond)", ()=> {
    const clientDealTokenBalance = 10000;
    const otherTokenBalance = 500;

    let dealMint: PublicKey;

    let clientDealTa: PublicKey;
    let clientHolderTa: PublicKey;
    let executorDealTa: PublicKey;

    const createDeal = async ({dealId, amount, serviceFee, withChecker, clientBond, executorBond, 
      holderMode, signers, executor, client}: {
      dealId: string | Buffer,
      amount: number,
      serviceFee: {
          amount: number,
          mint?: PublicKey
      },
      signers: Signer[],
      deadline?: number,
      withChecker?: {
        checkerFee: BN | number,
        checkerKey: PublicKey
      },
      clientBond?: IdlTypes<DealContract>["Bond"],
      executorBond?: IdlTypes<DealContract>["Bond"],
      holderMode?: boolean,
      executor?: PublicKey,
      client?: PublicKey,
    }) => {
       const instruction = (await getInitializeIx({
        dealContractProgram: program,
        dealId,
        amount,
        serviceFee,
        clientPk: client ? client : clientKp.publicKey,
        executorPk: executor ? executor : executorKp.publicKey,
        payerPk: payerKp.publicKey,
        dealMint,
        holderMode: !!holderMode ? holderMode : false,
        withChecker: !!withChecker ? {
          checkerKey: withChecker.checkerKey,
          checkerFee: new BN(withChecker.checkerFee)
        } : undefined,
        clientBond,
        executorBond      
      })).instruction();
      return await signAndSendIxs(conn, [getTotalComputeIxs(400000)[0], await instruction], signers, payerKp, [await getAddressLookupTable()])
    };

    let clientServiceFeeTa: PublicKey;
    let serviceServiceFeeTa: PublicKey;
    
    before( async()=>{
      dealMint = await createMint(
        provider.connection,
        payerKp,
        mintAuthorityKp.publicKey,
        null,
        0,
        Keypair.generate(),
        {commitment: COMMITMENT}
      );
      console.log(`dealMint: ${dealMint.toString()}`);

      clientDealTa = getAssociatedTokenAddressSync(dealMint, clientKp.publicKey);
      executorDealTa = getAssociatedTokenAddressSync(dealMint, executorKp.publicKey);
      clientHolderTa = getAssociatedTokenAddressSync(HOLDER_MINT, clientKp.publicKey);
 
      clientServiceFeeTa = await createAccount(provider.connection, payerKp, serviceFeeMintKp.publicKey, clientKp.publicKey, undefined, confirmOptions, TOKEN_PROGRAM_ID);
      clientDealTa = await createAccount(provider.connection, payerKp, dealMint, clientKp.publicKey, undefined, confirmOptions, TOKEN_PROGRAM_ID);
      clientHolderTa = await createAccount(provider.connection, payerKp, HOLDER_MINT, clientKp.publicKey, undefined, confirmOptions, TOKEN_PROGRAM_ID);

      const mintDeal = mintTo(provider.connection, mintAuthorityKp, dealMint, clientDealTa, mintAuthorityKp.publicKey, clientDealTokenBalance, undefined, confirmOptions, TOKEN_PROGRAM_ID);
      const mintHolder = mintTo(provider.connection, mintAuthorityKp, HOLDER_MINT, clientHolderTa, mintAuthorityKp.publicKey, 1000000000, undefined, confirmOptions, TOKEN_PROGRAM_ID);
      await Promise.all([mintDeal, mintHolder]);
    })

    it("Validating state", async () => {
      getAccount(provider.connection,clientDealTa).then(r=>{
        assert.ok(r.mint.toBase58() === dealMint.toBase58(), "invalid client dealMint")
        assert.ok(r.amount.toString() == clientDealTokenBalance.toString(), "invalid client dealAmount")
      });
      getAccount(provider.connection,clientHolderTa).then(r=>{
        assert.ok(r.mint.toBase58() === HOLDER_MINT.toBase58(), "invalid client clientHolderTa.mint")
        assert.ok(r.amount.toString() != "0", "invalid client clientHolderTa.amount")
      });
    });

    it("Create deal", async () => {
      const dealId = uuidTodealIdBuf(uuid())
      const checkerFee = 100
      const amount = 1000
      const serviceFee = 50

      const clientDealTaData = await conn.getTokenAccountBalance(clientDealTa, COMMITMENT);
      const clientHolderTaData = await conn.getTokenAccountBalance(clientHolderTa, COMMITMENT);

      await createDeal({
        dealId, 
        amount, 
        serviceFee: {
          amount: serviceFee,
          mint: dealMint
        }, 
        holderMode: false,
        withChecker: {
          checkerFee: new BN(checkerFee),
          checkerKey: checkerKp.publicKey
        }, 
        signers: [clientKp, executorKp, checkerKp, payerKp]
      });
     
      const dealStatePk = getDealStatePk(dealId, clientKp.publicKey, executorKp.publicKey)[0];
      const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealStatePk, true);
      const dealStateDealTaInfo = await getAccount(provider.connection, dealStateDealTa );

      serviceServiceFeeTa = getAssociatedTokenAddressSync(dealMint, SERVICE_FEE_OWNER);

      const dealStateData = await program.account.dealState.fetch(dealStatePk, "processed");
      const serviceFeeTaInfo = await getAccount(provider.connection, serviceServiceFeeTa );
    
      assert.ok(serviceFeeTaInfo.amount.toString() == serviceFee.toString(), 
        `invalid serviceFee: expected ${serviceFee.toString()}. got ${serviceFeeTaInfo.amount.toString()}`)
      assert.ok(dealStateData.amount.toString() == amount.toString(), 
        `invalid dealStateData.amount: expected ${amount.toString()}. got ${dealStateData.amount.toString()}`)
      assert.ok(dealStateData.clientKey.toString() == clientKp.publicKey.toString(),
        `dealStateData.clientkey: expected: ${clientKp.publicKey}. got ${dealStateData.clientKey.toString()}`)
      assert.ok(dealStateData.executorKey.toString() == executorKp.publicKey.toString(),
        `dealStateData.executorkey: expected: ${executorKp.publicKey}. got ${dealStateData.executorKey.toString()}`)
      assert.ok(dealStateDealTaInfo.amount.toString() == (amount + checkerFee).toString(),
        `dealStateDealTaInfo.amount: expected ${(amount + checkerFee).toString()}. got ${dealStateDealTaInfo.amount.toString()}`)
    });

    it("Try recreate deal with same ID", async () => {
      const dealId = uuidTodealIdBuf(uuid())
      const amount = 1000
      const serviceFee = 50

      serviceServiceFeeTa = getAssociatedTokenAddressSync(dealMint, SERVICE_FEE_OWNER);

      let serviceFeeAmountBefore;
      serviceFeeAmountBefore = (await getAccount(provider.connection, serviceServiceFeeTa, "processed")).amount

      const sig = await createDeal({
        dealId, 
        amount, 
        serviceFee: {
          amount: serviceFee,
          mint: dealMint
        }, 
        signers: [clientKp, executorKp, payerKp]
      });
     

      const dealStatePk = getDealStatePk(dealId, clientKp.publicKey, executorKp.publicKey)[0];
      const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealStatePk, true);
      const dealStateDealTaInfo = await getAccount(provider.connection, dealStateDealTa );
      const dealStateData = await program.account.dealState.fetch(dealStatePk, "processed");
  
      const serviceFeeAmountAfter = (await getAccount(provider.connection, serviceServiceFeeTa, "processed" )).amount
      const serviceFeePaid = serviceFeeAmountAfter - serviceFeeAmountBefore;
    
      try {
        assert.ok(serviceFeePaid.toString() == serviceFee.toString(), `invalid serviceFeePaid` );
        assert.ok(dealStateData.amount.toString() == amount.toString(), 
          `invalid dealStateData.amount: expected ${amount.toString()}. got ${dealStateData.amount.toString()}`)
        assert.ok(dealStateData.clientKey.toString() == clientKp.publicKey.toString(),
          `dealStateData.clientkey: expected: ${clientKp.publicKey}. got ${dealStateData.clientKey.toString()}`)
        assert.ok(dealStateData.executorKey.toString() == executorKp.publicKey.toString(),
          `dealStateData.executorkey: expected: ${executorKp.publicKey}. got ${dealStateData.executorKey.toString()}`)
        assert.ok(dealStateDealTaInfo.amount.toString() == amount .toString(),
          `dealStateDealTaInfo.amount: expected ${amount.toString()}. got ${dealStateDealTaInfo.amount.toString()}`)
      } catch (e) {
        console.log(`failed assertion tx signature: ${sig}`);
        throw e
      }
  
      // Try call Init again
      try {
        await createDeal({
          dealId, 
          amount, 
          serviceFee: {
            amount: serviceFee,
            mint: dealMint
          }, 
          signers: [clientKp, executorKp, payerKp]
        });
        assert.ok(false)
      } catch {
        assert.ok(true)
      }
    });

    it("Create deal and finish with checker", async () => {
      const dealId = uuidTodealIdBuf(uuid())
      const checkerFee = 100
      const amount = 1000
      const serviceFee = 50

      const clientDealTaInfoBefore = (await getAccount(provider.connection, clientDealTa, "processed" ));
      const executorDealTaInfoBefore = (await getAccount(provider.connection, executorDealTa, "processed" ));
      const checkerDealTaAmountBefore = (await getAccount(provider.connection, executorDealTa, "processed" )).amount | BigInt(0);

      await createDeal({
        dealId, 
        amount, 
        serviceFee: {
          amount: serviceFee,
          mint: dealMint
        }, 
        signers: [clientKp, executorKp, checkerKp, payerKp],
        withChecker: {
          checkerKey: checkerKp.publicKey,
          checkerFee: new BN(checkerFee)
        }
      });
  
      const dealStatePk = getDealStatePk(dealId, clientKp.publicKey, executorKp.publicKey)[0];
      const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealStatePk, true);
      const dealStateDealTaInfoBefore = await getAccount(provider.connection, dealStateDealTa, "processed" );
      assert.ok(Number(dealStateDealTaInfoBefore.amount) == amount + checkerFee, 
        `invalid dealStateDealTaInfoBefore.amount. expected ${amount + checkerFee} got ${dealStateDealTaInfoBefore.amount}`)

      const dealStateHolderTa = getAssociatedTokenAddressSync(HOLDER_MINT, dealStatePk, true);

      const checkerDealTa = getAssociatedTokenAddressSync(dealMint, checkerKp.publicKey);
      
      const instruction = (await getFinishIx({          
        initializer: checkerKp.publicKey,
        dealMint,
        clientPk: clientKp.publicKey,
        dealContractProgram: program,
        dealId,
        executorPk: executorKp.publicKey,
        checkerKey: checkerKp.publicKey,
        payerPk: payerKp.publicKey,
      })).instruction()
      await signAndSendIxs(conn, [await instruction], [checkerKp, payerKp], payerKp, [await getAddressLookupTable()])

      const dealStateDealTaInfoAfter = await conn.getAccountInfo(dealStateDealTa, "processed" );
      assert.ok(dealStateDealTaInfoAfter == null, `dealStateDealTaInfoAfter hadn't been closed`)

      const clientDealTaInfo = await getAccount(provider.connection, clientDealTa, "processed" )
      assert.ok((Number(clientDealTaInfoBefore.amount) - checkerFee - serviceFee - amount).toString() == clientDealTaInfo.amount.toString(),
        `invalid clientDealTaInfo.amount. expected ${(Number(clientDealTaInfoBefore.amount) - checkerFee - serviceFee - amount)} got ${clientDealTaInfo.amount}`)

      const executorDealTaInfoAfter = await getAccount(provider.connection, executorDealTa, "processed" );
      assert.ok(executorDealTaInfoAfter.amount.toString() == (Number(executorDealTaInfoBefore.amount) + amount).toString(),
        `invalid executorDealTaInfo.amount. expected ${Number(executorDealTaInfoBefore.amount) + amount} got ${executorDealTaInfoAfter.amount}`)

      const checkerDealTaInfo = await getAccount(provider.connection, checkerDealTa, "processed" );
      assert.ok(checkerDealTaInfo.amount.toString() == (Number(checkerDealTaAmountBefore) + checkerFee).toString(),
        `invalid checkerDealTaInfo.amount. expected ${Number(checkerDealTaAmountBefore) + checkerFee} got ${checkerDealTaInfo.amount}`)
    });

    it("Create deal and cancel as checker", async () => {
      const dealId = uuidTodealIdBuf(uuid())
      const checkerFee = 100
      const amount = 1000
      const serviceFee = 50

      await createDeal({
        dealId, 
        amount, 
        serviceFee: {
          amount: serviceFee,
          mint: dealMint
        }, 
        signers: [clientKp, executorKp, checkerKp, payerKp],
        withChecker: {
          checkerKey: checkerKp.publicKey,
          checkerFee: new BN(checkerFee)
        },
        deadline: 1
      });

      const dealStatePk = getDealStatePk(dealId, clientKp.publicKey, executorKp.publicKey)[0];
      const dealStateDealTa = getAssociatedTokenAddressSync(dealMint, dealStatePk, true);
      const dealStateDealTaInfo = await getAccount(provider.connection, dealStateDealTa );
    
      const dealStateData = await program.account.dealState.fetch(dealStatePk);

      const dealStateDealTaData = await getAccount(provider.connection, dealStateDealTa, "processed");

      const clientDealTaInfoBefore = await getAccount(provider.connection, clientDealTa, "processed" );
      
      assert.ok(dealStateDealTaData.amount.toString() == (amount + checkerFee).toString(),
        `invalid dealStateDealTaData.amount. expected ${amount + checkerFee} got ${dealStateDealTaData.amount}`)
      assert.ok(dealStateData.checker.checkerFee.toString() == new anchor.BN(checkerFee).toString(),
        `invalid dealStateData.checker.checkerFee. expected ${checkerFee} got ${dealStateData.checker.checkerFee}`)
      assert.ok(dealStateData.amount.toNumber().toString() == amount.toString(),
        `invalid dealStateData.amount. expected ${amount} got ${dealStateData.amount}`)
      assert.ok(dealStateData.clientKey.toBase58() == clientKp.publicKey.toBase58(),
        `invalid dealStateData.clientKey. expected ${clientKp.publicKey} got ${dealStateData.clientKey}`)
      assert.ok(dealStateData.executorKey.toBase58() == executorKp.publicKey.toBase58(),
        `invalid dealStateData.executorKey. expected ${executorKp.publicKey} got ${dealStateData.executorKey}`)

      const instruction = (await getCancelIx({
          initializer: checkerKp.publicKey,
          dealMint,
          clientPk: clientKp.publicKey,
          dealContractProgram: program,
          dealId,
          executorPk: executorKp.publicKey,
          checkerKey: checkerKp.publicKey,
        payerPk: payerKp.publicKey
      })).instruction();
      await signAndSendIxs(conn, [await instruction], [payerKp, checkerKp], checkerKp, [await getAddressLookupTable()])
      
      const clientDealTaInfo = await getAccount(provider.connection, clientDealTa, "processed");
      assert.ok((Number(clientDealTaInfoBefore.amount) + Number(amount)).toString() == clientDealTaInfo.amount.toString(),
        `invalid clientDealTaInfo.amount. expected ${(Number(clientDealTaInfoBefore.amount) + Number(amount + checkerFee)).toString()} got ${clientDealTaInfo.amount}`)
    });

    it("Try create deal with the same executor and client", async () => {
      createDeal(
      {
        dealId: uuid(),
        amount: 1000,
        serviceFee: {
          amount: 100,
          mint: dealMint
        }, 
        client: clientKp.publicKey,
        executor: clientKp.publicKey,
        holderMode: false,
        signers: [clientKp]
      }).then(() => {
        assert.ok(false)
      }).catch((error) => {
        assert.ok(error.error.errorCode.code == "ConstraintRaw")
      })
    })

    it("Try create deal with the zero fee (holder mode off)", async () => {
      createDeal({
        dealId: uuid(), 
        amount: 1000, 
        serviceFee: {
          amount: 0,
          mint: dealMint
        }, 
        signers: [clientKp, executorKp]
      }).then(() => {
        assert.ok(false)
      }).catch((error) => {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      })
    })

    it("Try create deal with the zero fee (holder mode on, but not fund)", async () => {
      try {
        await createDeal({
          dealId: uuid(),
          amount: 1000,
        serviceFee: {
          amount: 0,
          mint: dealMint
        }, 
          withChecker: {
            checkerFee: 0,
            checkerKey: checkerKp.publicKey
          },
          holderMode: true,
          signers: [clientKp, executorKp, checkerKp]
        })
        assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })

    it("Create deal with zero amount, fee and service fee with custom token", async () => {
      try {
        await createDeal({
          dealId: uuid(),
          amount: 0,
        serviceFee: {
          amount: 0,
          mint: dealMint
        }, 
          signers: [clientKp, executorKp, checkerKp, payerKp],
          holderMode: false
        })
        assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })

    it("Create deal with zero service fee with custom token", async () => {
      try {
        await createDeal({
          dealId: uuid(),
          amount: 1000,
        serviceFee: {
          amount: 0,
          mint: dealMint
        }, 
          signers: [clientKp, executorKp, checkerKp, payerKp],
          holderMode: false
        })
        assert.ok(false)
      } catch(error) {
        // TODO: - Add validation by errorCode
        assert.ok(true)
      }
    })
  })

  // describe("👻 Deals with performance bond (no checker)", ()=> {
  //   const amount = 1000;
  //   const service_fee = 50;
  //   const clientTokenBalance = 10000;
  //   const otherTokenBalance = 500;
  //   const serviceFeeTokenBalance = 0;
  //   const bondTokenBalance = 0;

  //   // const dealAccount = anchor.web3.Keypair.generate();
  //   const payer = anchor.web3.Keypair.generate();
  //   const mintAuthority = anchor.web3.Keypair.generate();

  //   const clientAccount = anchor.web3.Keypair.generate();
  //   const executorAccount = anchor.web3.Keypair.generate();
  //   const checkerAccount = anchor.web3.Keypair.generate();
  //   const serviceFeeAccount = anchor.web3.Keypair.generate();
  //   const serviceFeeMintAuthority = anchor.web3.Keypair.generate();
  //   const bondMintAuthority = anchor.web3.Keypair.generate();
  //   const serviceFeeMintKeypair: Keypair = serviceFeeMintKp;

  //   var mint;
  //   var mintBond;

  //   var clientTa;
  //   var executorTa;
  //   var checkerTa;
  //   var serviceFeeTa;

  //   var bondClientTa;
  //   var bondExecutorTa;

  //   var serviceFeeMint;
  //   var clientServiceTa;
  //   var serviceFeeServiceTa;

  //   const createDeal = async (
  //     dealId,
  //     amount,
  //     clientBondAmount,
  //     executorBondAmount,
  //     serviceFee,
  //     clientAccount,
  //     executorAccount,
  //     payer,
  //     serviceFeeTa,
  //     clientTa,
  //     clientServiceTa,
  //     executorTa,
  //     clientBondTa,
  //     clientBondMint,
  //     executorBondTa,
  //     executorBondMint,
  //     mint,
  //     holderMint,
  //     holderMode,
  //     deadline
  //   ) => {
  //     dealId = uuidTodealIdBuf(dealId);
      
  //     (await getInitializeIx({
  //       dealContractProgram: program,
  //       dealId,
  //       amount,
  //       serviceFee,
  //       clientPk: clientAccount ? clientAccount : clientKp.publicKey,
  //       executorPk: executorAccount ? executorAccount : executorKp.publicKey,
  //       payerPk: payerKp.publicKey,
  //       dealMint: mint,
  //       holderMode,
  //       clientBond: {
  //         mint: clientBondMint,
  //         amount: clientBondAmount,
  //       },
  //       executorBond: {
  //         mint: executorBondMint,
  //         amount: executorBondAmount
  //       }
  //     })).preInstructions([getTotalComputeIxs(800000)[0]]).rpc();

  //     const dealStatePk = getDealStatePk(dealId, clientAccount, executorAccount, DEAL_CONTRACT_PROGRAM_ID)[0];
  //     const dealStateDealTa = getAssociatedTokenAddressSync(mint, dealStatePk, true);
  //     const dealStateExecutorBondTa = getAssociatedTokenAddressSync(executorBondMint, dealStatePk, true);
  //     const dealStateClientBondTa = getAssociatedTokenAddressSync(clientAccount, dealStatePk, true);
    
  //     return {
  //       dealId,
  //       dealMint: mint,
  //       dealStateDealTa,
  //       dealStatePk,
  //       dealStateExecutorBondTa,
  //       dealStateClientBondTa,
  //       clientBondMint,
  //       executorBondMint
  //     }
  //   }

  //   before(async () => {
  //     await provider.connection.confirmTransaction(
  //       await provider.connection.requestAirdrop(payer.publicKey, 2000000000),
  //       "processed"
  //     );

  //     await provider.sendAndConfirm((() => {
  //       const tx = new Transaction();
  //       tx.add(
  //         SystemProgram.transfer({
  //           fromPubkey: payer.publicKey,
  //           toPubkey: clientAccount.publicKey,
  //           lamports: 100000000,
  //         })
  //       );
  //       return tx;
  //     })(), [payer])
  //     const accountInfo = await provider.connection.getAccountInfo(
  //       clientAccount.publicKey
  //     )
  //     assert.ok(accountInfo.lamports == 100000000)
  //     mint = await createMint(
  //       provider.connection,
  //       payer,
  //       mintAuthority.publicKey,
  //       null,
  //       0);

  //     mintBond = await createMint(
  //       provider.connection,
  //       payer,
  //       bondMintAuthority.publicKey,
  //       null,
  //       0);

  //     try {
  //       serviceFeeMint = await createMint(
  //         provider.connection,
  //         payer,
  //         serviceFeeMintAuthority.publicKey,
  //         null,
  //         0,
  //         serviceFeeMintKeypair);
  //     } catch {
  //       serviceFeeMint = serviceFeeMintKeypair.publicKey
  //     }
      

  //     clientTa = await createAccount(provider.connection, payer, mint, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
  //     executorTa = await createAccount(provider.connection, payer, mint, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
  //     checkerTa = await createAccount(provider.connection, payer, mint, checkerAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
  //     serviceFeeTa = await createAccount(provider.connection, payer, mint, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

  //     bondClientTa = await createAccount(provider.connection, payer, mintBond, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
  //     bondExecutorTa = await createAccount(provider.connection, payer, mintBond, executorAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
      
  //     clientServiceTa = await createAccount(provider.connection, payer, serviceFeeMint, clientAccount.publicKey, null, null, TOKEN_PROGRAM_ID);
  //     serviceFeeServiceTa = await createAccount(provider.connection, payer, serviceFeeMint, serviceFeeAccount.publicKey, null, null, TOKEN_PROGRAM_ID);

  //     await mintTo(provider.connection, payer, mint, clientTa, mintAuthority.publicKey, clientTokenBalance, [mintAuthority])
  //     await mintTo(provider.connection, payer, mint, executorTa, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
  //     await mintTo(provider.connection, payer, mint, checkerTa, mintAuthority.publicKey, otherTokenBalance, [mintAuthority])
  //     await mintTo(provider.connection, payer, mint, serviceFeeTa, mintAuthority.publicKey, serviceFeeTokenBalance, [mintAuthority])
  //   })

  //   it("Validate state", async () => {

  //     const clientTaInfo = await getAccount(
  //       provider.connection,
  //       clientTa
  //     )
  //     const executorTaInfo = await getAccount(
  //       provider.connection,
  //       executorTa
  //     )
  //     const checkerTaInfo = await getAccount(
  //       provider.connection,
  //       checkerTa
  //     )

  //     assert.ok(clientTaInfo.mint.toBase58() == mint.toBase58())
  //     assert.ok(clientTaInfo.amount.toString() == clientTokenBalance.toString())
  //     assert.ok(executorTaInfo.amount.toString() == otherTokenBalance.toString())
  //     assert.ok(checkerTaInfo.amount.toString() == otherTokenBalance.toString())
  //   });
    
  //   it("Create deal with holder mode (no CTUS fund)", async () => {
  //     try {
  //       var data = await createDeal(
  //         uuid(),
  //         amount,
  //         0,
  //         0,
  //         0,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         true,
  //         new Date().getTime() / 1000)
    
  //         const state = await program.account.dealState.fetch(data.dealStatePk)
  //         const serviceFeeTaInfo = await getAccount(
  //           provider.connection,
  //           serviceFeeTa
  //         )
  //         assert.ok(serviceFeeTaInfo.amount.toString() == service_fee.toString())
  //         assert.ok(state.amount.toNumber().toString() == amount.toString())
  //         assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
  //         assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
  //     } catch(err) {
  //       assert.ok(err.error.origin == "client_service_token_account")
  //     }
  //   })

  //   it("Create deal with executor bond", async () => {

  //     await mintTo(provider.connection, payer, mintBond, bondExecutorTa, bondMintAuthority.publicKey, 100, [bondMintAuthority])
    
  //     let serviceFee = BigInt(100)
  //     let executorBond = BigInt(56)
  //     const serviceFeeTaInfo = await getAccount(
  //       provider.connection,
  //       serviceFeeTa
  //     )
  //     var serviceAccountAmount = serviceFeeTaInfo.amount
      
  //     try {
  //       var data = await createDeal(
  //         uuid(),
  //         amount,
  //         0,
  //         executorBond,
  //         serviceFee,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         false,
  //         new Date().getTime() / 1000)
    
  //         const state = await program.account.dealState.fetch(data.dealStatePk)
  //         const serviceFeeTaInfo = await getAccount(
  //           provider.connection,
  //           serviceFeeTa
  //         )
  //         const executorBondTaInfo = await getAccount(
  //           provider.connection,
  //           data.dealStateExecutorBondTa
  //         )
  //         assert.ok(serviceFeeTaInfo.amount.toString() == (serviceAccountAmount + serviceFee).toString())
  //         assert.ok(executorBondTaInfo.amount.toString() == executorBond.toString())
  //         assert.ok(state.amount.toNumber().toString() == amount.toString())
  //         assert.ok(state.clientKey.toBase58() == clientAccount.publicKey.toBase58())
  //         assert.ok(state.executorKey.toBase58() == executorAccount.publicKey.toBase58())
  //     } catch(err) {
  //       console.log(err)
  //       assert.ok(false)
  //     }
  //   })

  //   it("Try create deal twice", async () => {

  //     await mintTo(provider.connection, payer, mintBond, bondExecutorTa, bondMintAuthority.publicKey, 100, [bondMintAuthority])
    
  //     let serviceFee = BigInt(100)
  //     let executorBond = BigInt(56)
  //     let dealId = uuid()
  //     try {
  //       await createDeal(
  //         dealId,
  //         amount,
  //         0,
  //         executorBond,
  //         serviceFee,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         false,
  //         new Date().getTime() / 1000)

  //         assert.ok(true)
  //       await createDeal(
  //         dealId,
  //         amount,
  //         0,
  //         executorBond,
  //         serviceFee,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         false,
  //         new Date().getTime() / 1000)
  //         assert.ok(false)
  //     } catch(err) {
  //       assert.ok(err.error.origin == "deposit_account")
  //     }
  //   })

  //   it("Create deal with bond and try cancel", async () => {
  //     await mintTo(provider.connection, payer, mintBond, bondExecutorTa, bondMintAuthority.publicKey, 100, [bondMintAuthority])
  //     let serviceFee = BigInt(100)
  //     let executorBond = BigInt(56)

  //     var executorTaInfo = await getAccount(
  //       provider.connection,
  //       executorTa
  //     )

  //     try {
  //       let data = await createDeal(
  //         uuid(),
  //         amount,
  //         0,
  //         executorBond,
  //         serviceFee,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         false,
  //         (new Date().getTime() / 1000) + 1000
  //       )
  //       executorTaInfo = await getAccount(
  //         provider.connection,
  //         executorTa
  //       )
  //       try {
  //         (await getCancelIx({
  //           dealContractProgram: program,
  //           dealId: data.dealId,
  //           clientPk: clientAccount.publicKey,
  //           dealMint: data.dealMint,
  //           executorPk: executorAccount.publicKey,
  //           initializer: clientAccount.publicKey,
  //           clientBondMint: data.clientBondMint,
  //           executorBondMint: data.executorBondMint,
  //         })).signers([clientAccount]).rpc()
  //       } catch(error) {
  //         assert.ok(error.error.errorCode.code == 'DeadlineNotCome')
  //       }
  //     } catch(error) { 
  //       console.log(error)
  //       assert.ok(false)
  //     }
  //   })

  //   it("Create and finish deal with bond and deadline", async () => {

  //     await mintTo(provider.connection, payer, mintBond, bondClientTa, bondMintAuthority.publicKey, 100, [bondMintAuthority])
  //     await mintTo(provider.connection, payer, mintBond, bondExecutorTa, bondMintAuthority.publicKey, 100, [bondMintAuthority])

  //     var bondClientTaInfo = await getAccount(
  //       provider.connection,
  //       bondClientTa
  //     )
  //     var bondClientTokenAmountBefore = bondClientTaInfo.amount

  //     var bondExecutorTaInfo = await getAccount(
  //       provider.connection,
  //       bondExecutorTa
  //     )
  //     var bondExecutorTokenAmountBefore = bondExecutorTaInfo.amount

  //     var clientTaInfo = await getAccount(
  //       provider.connection,
  //       clientTa
  //     )
  //     var clientTokenAmountBefore = clientTaInfo.amount
      
  //     let amount = BigInt(100)
  //     let serviceFee = BigInt(100)
  //     let executorBond = BigInt(56)
  //     let clientBond = BigInt(40)
  //     let data
  //     let deadline = new Date().getTime() / 1000
  //     try {
  //       data = await createDeal(
  //         uuid(),
  //         amount,
  //         clientBond,
  //         executorBond,
  //         serviceFee,
  //         clientAccount,
  //         executorAccount,
  //         payer,
  //         serviceFeeTa,
  //         clientTa,
  //         clientServiceTa,
  //         executorTa,
  //         bondClientTa,
  //         mintBond,
  //         bondExecutorTa,
  //         mintBond,
  //         mint,
  //         serviceFeeMint,
  //         false,
  //         deadline
  //       )
        
  //       bondClientTaInfo = await getAccount(
  //         provider.connection,
  //         bondClientTa
  //       )
  //       var bondClientTokenAmountAfter = bondClientTaInfo.amount
  
  //       bondExecutorTaInfo = await getAccount(
  //         provider.connection,
  //         bondExecutorTa
  //       )
  //       var bondExecutorTokenAmountAfter = bondExecutorTaInfo.amount
        
  //      let depositBondExecutorTaInfo = await getAccount(
  //         provider.connection,
  //         data.executor_bond_vault_account_pda
  //       )

  //       let depositBondClientTaInfo = await getAccount(
  //         provider.connection,
  //         data.client_bond_vault_account_pda
  //       )

  //       clientTaInfo = await getAccount(
  //         provider.connection,
  //         clientTa
  //       )
  //       var clientTokenAmountAfter = clientTaInfo.amount

  //       assert.ok(bondExecutorTokenAmountAfter < bondExecutorTokenAmountBefore)
  //       assert.ok(clientTokenAmountAfter < clientTokenAmountBefore)
  //       assert.ok(bondClientTokenAmountAfter < bondClientTokenAmountBefore)
  //       assert.ok(depositBondExecutorTaInfo.amount == executorBond)
  //       assert.ok(depositBondClientTaInfo.amount == clientBond)

  //       let before_deadline = new Date().getTime() / 1000
  //       assert.ok(before_deadline > deadline);

  //       (await getCancelIx({
  //         clientPk: clientAccount.publicKey,
  //         dealContractProgram: program,
  //         dealId: data.dealId,
  //         dealMint: data.dealMint,
  //         executorPk: executorAccount.publicKey,
  //         initializer: executorAccount.publicKey,
  //         checkerKey: checkerAccount.publicKey,
  //         clientBondMint: mintBond,
  //         executorBondMint: mintBond,
  //       })).signers([clientAccount])
  //         .rpc()

  //         bondClientTaInfo = await getAccount(
  //           provider.connection,
  //           bondClientTa
  //         )
  //         var bondClientTokenAmountAfterCancel = bondClientTaInfo.amount
    
  //         bondExecutorTaInfo = await getAccount(
  //           provider.connection,
  //           bondExecutorTa
  //         )
  //         var bondExecutorTokenAmountAfterCancel = bondExecutorTaInfo.amount

  //         clientTaInfo = await getAccount(
  //           provider.connection,
  //           clientTa
  //         )
  //         var clientTokenAmountAfterCancel = clientTaInfo.amount

  //         assert.ok(bondClientTokenAmountAfterCancel == bondClientTokenAmountBefore)
  //         assert.ok(bondExecutorTokenAmountAfterCancel == bondExecutorTokenAmountBefore)
  //         assert.ok(clientTokenAmountAfterCancel == clientTokenAmountBefore - serviceFee)

  //     } catch(error) { 
  //       assert.ok(false)
  //     }
  //   })
  // })
});
