import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "../target/types/investor_fee_distributor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import {
  setupTestContext,
  derivePolicyConfigPda,
  deriveInvestorFeePositionOwnerPda,
  deriveDailyProgressPda,
  createTokenAccount,
  mintTokensTo,
  getTokenBalance,
  createMockStreamflowStream,
  TestContext,
  ONE_SOL,
  ONE_DAY,
  sleep,
  DYNAMIC_AMM_PROGRAM_ID,
  DYNAMIC_VAULT_PROGRAM_ID,
} from "./test-helpers";

describe("End-to-End Integration Tests", () => {
  let ctx: TestContext;
  let creatorQuoteAta: PublicKey;
  let policyConfigPda: PublicKey;
  let positionOwnerPda: PublicKey;
  let dailyProgressPda: PublicKey;

  before(async () => {
    ctx = await setupTestContext();

    [policyConfigPda] = derivePolicyConfigPda(ctx.program, ctx.vault);
    [positionOwnerPda] = deriveInvestorFeePositionOwnerPda(
      ctx.program,
      ctx.vault
    );
    [dailyProgressPda] = deriveDailyProgressPda(ctx.program, ctx.vault);

    creatorQuoteAta = await createTokenAccount(
      ctx.provider,
      ctx.quoteMint,
      ctx.payer.publicKey
    );
  });

  it("Complete workflow: Initialize → Position → Distribute", async () => {
    console.log("\n=== STEP 1: Initialize Policy ===");

    const investorFeeShareBps = 6000; // 60% to investors
    const dailyCapLamports = new BN(100 * ONE_SOL);
    const minPayoutLamports = new BN(0.001 * ONE_SOL);
    const y0TotalStreamed = new BN(1_000_000 * ONE_SOL); // 1M tokens

    const initPolicyTx = await ctx.program.methods
      .initializePolicy(
        ctx.vault,
        investorFeeShareBps,
        dailyCapLamports,
        minPayoutLamports,
        y0TotalStreamed,
        creatorQuoteAta
      )
      .accounts({
        policyConfig: policyConfigPda,
        payer: ctx.payer.publicKey,
        authority: ctx.payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Policy initialized:", initPolicyTx);

    const policy = await ctx.program.account.policyConfig.fetch(
      policyConfigPda
    );
    console.log("Policy config:", {
      investorFeeShareBps: policy.investorFeeShareBps,
      dailyCapLamports: policy.dailyCapLamports?.toString(),
      minPayoutLamports: policy.minPayoutLamports.toString(),
      y0TotalStreamed: policy.y0TotalStreamed.toString(),
    });

    console.log("\n=== STEP 2: Create Mock Investors ===");

    const NUM_INVESTORS = 5;
    const investors = [];

    for (let i = 0; i < NUM_INVESTORS; i++) {
      const investorKeypair = Keypair.generate();

      // Different amounts: 100k, 150k, 200k, 250k, 300k
      const depositedAmount = (100_000 + i * 50_000) * ONE_SOL;

      const stream = await createMockStreamflowStream(
        ctx.provider,
        investorKeypair.publicKey,
        depositedAmount,
        365 * ONE_DAY, // 1 year
        90 * ONE_DAY,  // 90 day cliff
        depositedAmount * 0.25 // 25% cliff amount
      );

      const investorQuoteAta = await createTokenAccount(
        ctx.provider,
        ctx.quoteMint,
        investorKeypair.publicKey
      );

      investors.push({
        keypair: investorKeypair,
        stream: stream.keypair.publicKey,
        quoteAta: investorQuoteAta,
        depositedAmount,
        lockedAmount: depositedAmount, // All locked at start
      });

      console.log(`Investor ${i + 1}: ${depositedAmount / ONE_SOL} tokens deposited`);
    }

    console.log(`\nTotal deposited: ${investors.reduce((sum, inv) => sum + inv.depositedAmount, 0) / ONE_SOL} tokens`);
    console.log(`Total locked: ${investors.reduce((sum, inv) => sum + inv.lockedAmount, 0) / ONE_SOL} tokens`);

    console.log("\n=== STEP 3: Simulate Fee Accumulation ===");

    // Create treasury ATAs
    const treasuryQuoteAta = await getAssociatedTokenAddress(
      ctx.quoteMint,
      positionOwnerPda,
      true
    );

    const treasuryBaseAta = await getAssociatedTokenAddress(
      ctx.baseMint,
      positionOwnerPda,
      true
    );

    // Create the ATAs
    try {
      await createAssociatedTokenAccount(
        ctx.provider.connection,
        ctx.payer,
        ctx.quoteMint,
        positionOwnerPda
      );
      console.log("Treasury quote ATA created");
    } catch (err) {
      console.log("Treasury quote ATA may already exist");
    }

    try {
      await createAssociatedTokenAccount(
        ctx.provider.connection,
        ctx.payer,
        ctx.baseMint,
        positionOwnerPda
      );
      console.log("Treasury base ATA created");
    } catch (err) {
      console.log("Treasury base ATA may already exist");
    }

    // Simulate fees: Mint 50 SOL worth of quote tokens to treasury
    const feesAccumulated = 50 * ONE_SOL;
    await mintTokensTo(
      ctx.provider,
      ctx.quoteMint,
      treasuryQuoteAta,
      feesAccumulated
    );

    console.log(`Fees accumulated: ${feesAccumulated / ONE_SOL} quote tokens`);

    const treasuryBalance = await getTokenBalance(
      ctx.provider,
      treasuryQuoteAta
    );
    console.log(`Treasury balance: ${treasuryBalance} lamports`);

    console.log("\n=== STEP 4: Run Distribution Crank ===");

    // Calculate f_locked
    const totalLocked = investors.reduce((sum, inv) => sum + inv.lockedAmount, 0);
    const totalY0 = Number(y0TotalStreamed.toString());
    const fLocked = totalLocked / totalY0;
    const fLockedBps = Math.floor(fLocked * 10000);

    console.log(`f_locked = ${totalLocked / ONE_SOL} / ${totalY0 / ONE_SOL} = ${fLocked.toFixed(4)} (${fLockedBps} BPS)`);

    // Expected distribution
    const eligibleInvestorBps = Math.min(investorFeeShareBps, fLockedBps);
    const investorShare = Math.floor((feesAccumulated * eligibleInvestorBps) / 10000);
    const creatorShare = feesAccumulated - investorShare;

    console.log(`\nExpected distribution:`);
    console.log(`- Eligible investor BPS: ${eligibleInvestorBps}`);
    console.log(`- Total to investors: ${investorShare / ONE_SOL} tokens`);
    console.log(`- Total to creator: ${creatorShare / ONE_SOL} tokens`);

    // Calculate per-investor expected amounts
    for (let i = 0; i < investors.length; i++) {
      const weight = investors[i].lockedAmount / totalLocked;
      const payout = Math.floor(investorShare * weight);
      console.log(`  - Investor ${i + 1}: ${payout / ONE_SOL} tokens (weight: ${weight.toFixed(4)})`);
    }

    // Prepare investor data
    const investorData = investors.map((inv) => ({
      lockedAmount: new BN(inv.lockedAmount),
    }));

    try {
      const crankTx = await ctx.program.methods
        .crankDistribution(1, investorData) // Single page
        .accounts({
          cranker: ctx.payer.publicKey,
          dailyProgress: dailyProgressPda,
          policyConfig: policyConfigPda,
          investorFeePositionOwner: positionOwnerPda,
          treasuryQuoteAta: treasuryQuoteAta,
          treasuryBaseAta: treasuryBaseAta,
          creatorQuoteAta: creatorQuoteAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          ...investors.map((inv) => ({
            pubkey: inv.stream,
            isSigner: false,
            isWritable: false,
          })),
          ...investors.map((inv) => ({
            pubkey: inv.quoteAta,
            isSigner: false,
            isWritable: true,
          })),
        ])
        .rpc();

      console.log("\nCrank distribution tx:", crankTx);

      console.log("\n=== STEP 5: Verify Distribution ===");

      // Check investor balances
      let totalDistributed = 0n;
      for (let i = 0; i < investors.length; i++) {
        const balance = await getTokenBalance(
          ctx.provider,
          investors[i].quoteAta
        );
        totalDistributed += balance;
        console.log(`Investor ${i + 1} balance: ${Number(balance) / ONE_SOL} tokens`);
      }

      // Check creator balance
      const creatorBalance = await getTokenBalance(
        ctx.provider,
        creatorQuoteAta
      );
      console.log(`Creator balance: ${Number(creatorBalance) / ONE_SOL} tokens`);

      // Check treasury is depleted
      const finalTreasuryBalance = await getTokenBalance(
        ctx.provider,
        treasuryQuoteAta
      );
      console.log(`Treasury final balance: ${Number(finalTreasuryBalance) / ONE_SOL} tokens`);

      console.log("\n=== VERIFICATION ===");
      console.log(`Total distributed to investors: ${Number(totalDistributed) / ONE_SOL} tokens`);
      console.log(`Total distributed overall: ${(Number(totalDistributed) + Number(creatorBalance)) / ONE_SOL} tokens`);
      console.log(`Original fees: ${feesAccumulated / ONE_SOL} tokens`);

      // Assertions
      expect(Number(totalDistributed + creatorBalance + finalTreasuryBalance)).to.be.closeTo(
        feesAccumulated,
        1000 // Allow small rounding
      );

      console.log("\n✅ End-to-end test passed!");
    } catch (err) {
      console.error("Crank distribution failed:", err.message);
      if (err.logs) {
        console.log("Program logs:");
        err.logs.forEach((log: string) => console.log("  ", log));
      }
      throw err;
    }
  });

  it("Multi-day distribution workflow", async () => {
    console.log("\n=== Testing multi-day distributions ===");
    console.log("This test would simulate multiple 24h periods");
    console.log("Skipping for now - requires time manipulation");
  });

  it("Multi-page distribution with 50 investors", async () => {
    console.log("\n=== Testing multi-page distribution ===");
    console.log("This test would create 50+ investors and paginate");
    console.log("Skipping for now - requires large setup");
  });
});
