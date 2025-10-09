import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "../target/types/investor_fee_distributor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { BN } from "bn.js";
import {
  setupTestContext,
  derivePolicyConfigPda,
  createTokenAccount,
  TestContext,
  ONE_SOL,
} from "./test-helpers";

describe("initialize_policy", () => {
  let ctx: TestContext;
  let creatorQuoteAta: PublicKey;
  let policyConfigPda: PublicKey;

  before(async () => {
    ctx = await setupTestContext();

    // Create creator's quote token account
    creatorQuoteAta = await createTokenAccount(
      ctx.provider,
      ctx.quoteMint,
      ctx.payer.publicKey
    );

    // Derive policy config PDA
    [policyConfigPda] = derivePolicyConfigPda(ctx.program, ctx.vault);
  });

  it("successfully initializes policy with valid parameters", async () => {
    const investorFeeShareBps = 7000; // 70%
    const dailyCapLamports = new BN(10 * ONE_SOL); // 10 SOL cap
    const minPayoutLamports = new BN(1000); // 0.000001 tokens
    const y0TotalStreamed = new BN(1_000_000 * ONE_SOL); // 1M tokens

    const tx = await ctx.program.methods
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

    console.log("Initialize policy tx:", tx);

    // Fetch and verify policy config
    const policyConfig = await ctx.program.account.policyConfig.fetch(
      policyConfigPda
    );

    expect(policyConfig.authority.toString()).to.equal(
      ctx.payer.publicKey.toString()
    );
    expect(policyConfig.vault.toString()).to.equal(ctx.vault.toString());
    expect(policyConfig.investorFeeShareBps).to.equal(investorFeeShareBps);
    expect(policyConfig.dailyCapLamports.toString()).to.equal(
      dailyCapLamports.toString()
    );
    expect(policyConfig.minPayoutLamports.toString()).to.equal(
      minPayoutLamports.toString()
    );
    expect(policyConfig.y0TotalStreamed.toString()).to.equal(
      y0TotalStreamed.toString()
    );
    expect(policyConfig.creatorQuoteAta.toString()).to.equal(
      creatorQuoteAta.toString()
    );
  });

  it("fails when investor_fee_share_bps exceeds 10000", async () => {
    const vault2 = anchor.web3.Keypair.generate().publicKey;
    const [policyConfigPda2] = derivePolicyConfigPda(ctx.program, vault2);

    try {
      await ctx.program.methods
        .initializePolicy(
          vault2,
          10001, // Invalid: > 10000 BPS
          null,
          new BN(1000),
          new BN(1_000_000 * ONE_SOL),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: policyConfigPda2,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      expect.fail("Should have failed with InvalidFeeShareBps");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("InvalidFeeShareBps");
    }
  });

  it("fails when y0_total_streamed is zero", async () => {
    const vault3 = anchor.web3.Keypair.generate().publicKey;
    const [policyConfigPda3] = derivePolicyConfigPda(ctx.program, vault3);

    try {
      await ctx.program.methods
        .initializePolicy(
          vault3,
          7000,
          null,
          new BN(1000),
          new BN(0), // Invalid: Y0 cannot be zero
          creatorQuoteAta
        )
        .accounts({
          policyConfig: policyConfigPda3,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      expect.fail("Should have failed with InvalidY0Amount");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("InvalidY0Amount");
    }
  });

  it("successfully initializes policy without daily cap", async () => {
    const vault4 = anchor.web3.Keypair.generate().publicKey;
    const [policyConfigPda4] = derivePolicyConfigPda(ctx.program, vault4);

    await ctx.program.methods
      .initializePolicy(
        vault4,
        5000, // 50%
        null, // No daily cap
        new BN(1000),
        new BN(500_000 * ONE_SOL),
        creatorQuoteAta
      )
      .accounts({
        policyConfig: policyConfigPda4,
        payer: ctx.payer.publicKey,
        authority: ctx.payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const policyConfig = await ctx.program.account.policyConfig.fetch(
      policyConfigPda4
    );

    expect(policyConfig.dailyCapLamports).to.be.null;
  });

  it("prevents reinitializing existing policy", async () => {
    // Try to reinitialize the first policy
    try {
      await ctx.program.methods
        .initializePolicy(
          ctx.vault,
          8000,
          null,
          new BN(2000),
          new BN(2_000_000 * ONE_SOL),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: policyConfigPda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      expect.fail("Should have failed - account already initialized");
    } catch (err) {
      // Anchor will throw error about account already existing
      expect(err).to.exist;
    }
  });
});
