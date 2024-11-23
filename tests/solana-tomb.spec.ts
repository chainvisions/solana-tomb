import * as anchor from "@coral-xyz/anchor";
import { BanksClient, Clock, ProgramTestContext } from "solana-bankrun";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { createMint, createAccount, mintTo, getAccount } from "spl-token-bankrun";
import { Program } from "@coral-xyz/anchor";
import { SolanaTomb } from "../target/types/solana_tomb";
import { expect } from "@jest/globals";

const GENESIS_POOL_PROGRAM_ID = new anchor.web3.PublicKey(
  "EbUS5R37dPQRdDKgh7Kju9JhwUg6FPB1fSRAjbUj29EQ"
);

describe("solana-tomb", () => {
  let context: ProgramTestContext;
  let provider: BankrunProvider;
  let banksClient: BanksClient;
  let genesisPool: Program<SolanaTomb>;
  let rewardMint: anchor.web3.PublicKey;
  let farmingMint: anchor.web3.PublicKey;
  let signer: anchor.Wallet;
  let devshare: anchor.web3.Keypair;

  beforeEach(async () => {
    context = await startAnchor("./", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);
    signer = provider.wallet;
    banksClient = context.banksClient;
    genesisPool = anchor.workspace.SolanaTomb as Program<SolanaTomb>;
    // @ts-ignore
    rewardMint = await createMint(banksClient, provider.wallet.payer, provider.wallet.publicKey, null, 9); 
    // @ts-ignore
    farmingMint = await createMint(banksClient, provider.wallet.payer, provider.wallet.publicKey, null, 9);
    devshare = anchor.web3.Keypair.generate();

    console.log(rewardMint);
  });

  const initializePool = async () => {
    await genesisPool.methods.initialize().accounts({
      authority: signer.publicKey,
      devshare: devshare.publicKey,
      rewardMint: rewardMint
    }).signers([signer.payer, devshare]).rpc();

    // Add initial pool.
    await genesisPool.methods.addPool(new anchor.BN(50000), new anchor.BN(1732398368)).accounts({
      authority: signer.publicKey,
      tokenMint: farmingMint
    }).signers([signer.payer]).rpc();
  }

  test("Genesis pool properly initialized", async () => {
    await initializePool();
    const [statePda,__] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("state")], GENESIS_POOL_PROGRAM_ID);
    const [vaultPda,_] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault")], GENESIS_POOL_PROGRAM_ID);
    const stateAccount = await genesisPool.account.genesis.fetch(statePda);
    expect(stateAccount.vault.toString()).toBe(vaultPda.toString());
    expect(stateAccount.devShare.toString()).toBe(devshare.publicKey.toString());
    expect(stateAccount.authority.toString()).toBe(signer.publicKey.toString());
  });

  test("Add new pool", async () => {
    await initializePool();
    const [poolPda] = anchor.web3.PublicKey.findProgramAddressSync([farmingMint.toBuffer()], GENESIS_POOL_PROGRAM_ID);
    // @ts-ignore
    await genesisPool.methods.addPool(new anchor.BN(50000), new anchor.BN(1732398368)).accounts({
      authority: signer.publicKey,
      tokenMint: farmingMint
    }).signers([signer.payer]).rpc();
    const poolAccount = await genesisPool.account.pool.fetch(poolPda);
    expect(poolAccount.underlying).toBe(farmingMint);
    expect(poolAccount.totalShares).toBe(new anchor.BN(0));
    let clock = await context.banksClient.getClock();
    expect(poolAccount.lastUpdateAt).toBe(clock.unixTimestamp);
    expect(poolAccount.rewardRate).toBe(new anchor.BN(50000));
    expect(poolAccount.periodFinish).toBe(new anchor.BN(1732398368));
  });

  test("Deposit tokens", async () => {
    await initializePool();
    // Mint some new tokens and deposit them into the pool.
    // @ts-ignore
    const tokenAccount = await createAccount(banksClient, signer.payer, farmingMint, signer.publicKey);
    // @ts-ignore
    await mintTo(banksClient, signer.payer, farmingMint, tokenAccount, signer.payer, (5 * anchor.web3.LAMPORTS_PER_SOL));
    await genesisPool.methods.deposit(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL)).signers([signer.payer]).rpc();
    
    // Fetch stored pool info and make sure balances match up.
    const [poolPda] = anchor.web3.PublicKey.findProgramAddressSync([farmingMint.toBuffer()], GENESIS_POOL_PROGRAM_ID);
    const [depositPda] = anchor.web3.PublicKey.findProgramAddressSync([signer.publicKey.toBuffer(), poolPda.toBuffer()], GENESIS_POOL_PROGRAM_ID);
    const poolAcc = await genesisPool.account.pool.fetch(poolPda);
    const depositAcc = await genesisPool.account.depositor.fetch(depositPda);
    expect(poolAcc.totalShares).toBe(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL));
    expect(depositAcc.shares).toBe(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL));
    expect(depositAcc.authority.toString()).toBe(signer.publicKey.toString());
  });

  test("Withdraw tokens", async () => {
    await initializePool();
    // @ts-ignore
    const tokenAccount = await createAccount(banksClient, signer.payer, farmingMint, signer.publicKey);
    // @ts-ignore
    await mintTo(banksClient, signer.payer, farmingMint, tokenAccount, signer.payer, (5 * anchor.web3.LAMPORTS_PER_SOL));
    await genesisPool.methods.deposit(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL)).signers([signer.payer]).rpc();
    const [poolPda] = anchor.web3.PublicKey.findProgramAddressSync([farmingMint.toBuffer()], GENESIS_POOL_PROGRAM_ID);
    const [depositPda] = anchor.web3.PublicKey.findProgramAddressSync([signer.publicKey.toBuffer(), poolPda.toBuffer()], GENESIS_POOL_PROGRAM_ID);
    
    // Withdraw and check balances.
    await genesisPool.methods.withdraw(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL)).signers([signer.payer]).rpc();
    const poolAcc = await genesisPool.account.pool.fetch(poolPda);
    const depositAcc = await genesisPool.account.depositor.fetch(depositPda);
    expect(poolAcc.totalShares).toBe(new anchor.BN(0));
    expect(depositAcc.shares).toBe(new anchor.BN(0));
  })

  test("Reward accumulation", async () => {
    await initializePool();
    // Mint some new tokens and deposit them into the pool.
    // @ts-ignore
    const tokenAccount = await createAccount(banksClient, signer.payer, farmingMint, signer.publicKey);
    // @ts-ignore
    await mintTo(banksClient, signer.payer, farmingMint, tokenAccount, signer.payer, (5 * anchor.web3.LAMPORTS_PER_SOL));
    await genesisPool.methods.deposit(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL)).signers([signer.payer]).rpc();

    // Pass some time and check how much we've accumulated.
    let clock = await banksClient.getClock();
    provider.context.setClock(new Clock(clock.slot, clock.epochStartTimestamp, clock.epoch, clock.leaderScheduleEpoch, clock.unixTimestamp + BigInt(500)));
    let estEarnings = new anchor.BN(50000 * 500);
    // @ts-ignore
    let tokensSoFar = (await getAccount(banksClient, tokenAccount)).amount;
    await genesisPool.methods.claimRewards().signers([signer.payer]).rpc();
    // @ts-ignore
    let tokensAfter = (await getAccount(banksClient, tokenAccount)).amount;

    expect(tokensAfter).toBeGreaterThan(tokensSoFar);
    expect(tokensAfter - tokensSoFar).toBe(estEarnings);
  })
});
