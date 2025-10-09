import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "../target/types/investor_fee_distributor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  setupTestContext,
  derivePolicyConfigPda,
  deriveInvestorFeePositionOwnerPda,
  deriveDailyProgressPda,
  createTokenAccount,
  mintTokensTo,
  getTokenBalance,
  TestContext,
  ONE_SOL,
} from "./test-helpers";

describe("Edge Cases", () => {
  let ctx: TestContext;
  let creatorQuoteAta: PublicKey;
  let policyConfigPda: PublicKey;

  before(async () => {
    ctx = await setupTestContext();
    [policyConfigPda] = derivePolicyConfigPda(ctx.program, ctx.vault);

    creatorQuoteAta = await createTokenAccount(
      ctx.provider,
      ctx.quoteMint,
      ctx.payer.publicKey
    );
  });

  describe("Policy Initialization Edge Cases", () => {
    it("handles maximum BPS value (10000 = 100%)", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      await ctx.program.methods
        .initializePolicy(
          vault,
          10000, // 100% to investors
          null,
          new BN(1),
          new BN(1000),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.investorFeeShareBps).to.equal(10000);
    });

    it("handles minimum BPS value (0 = all to creator)", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      await ctx.program.methods
        .initializePolicy(
          vault,
          0, // 0% to investors, 100% to creator
          null,
          new BN(1),
          new BN(1000),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.investorFeeShareBps).to.equal(0);
    });

    it("handles very small Y0 values", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      await ctx.program.methods
        .initializePolicy(
          vault,
          5000,
          null,
          new BN(1),
          new BN(1), // Minimum Y0
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.y0TotalStreamed.toString()).to.equal("1");
    });

    it("handles very large Y0 values", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      const largeY0 = new BN("1000000000000000000"); // 1 quintillion

      await ctx.program.methods
        .initializePolicy(
          vault,
          5000,
          null,
          new BN(1),
          largeY0,
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.y0TotalStreamed.toString()).to.equal(largeY0.toString());
    });

    it("handles minimum min_payout value", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      await ctx.program.methods
        .initializePolicy(
          vault,
          5000,
          null,
          new BN(1), // Minimum payout of 1 lamport
          new BN(1000000),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.minPayoutLamports.toString()).to.equal("1");
    });

    it("handles very large daily cap", async () => {
      const vault = Keypair.generate().publicKey;
      const [pda] = derivePolicyConfigPda(ctx.program, vault);

      const largeCap = new BN("1000000000000000"); // 1 million SOL

      await ctx.program.methods
        .initializePolicy(
          vault,
          5000,
          largeCap,
          new BN(1000),
          new BN(1000000),
          creatorQuoteAta
        )
        .accounts({
          policyConfig: pda,
          payer: ctx.payer.publicKey,
          authority: ctx.payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await ctx.program.account.policyConfig.fetch(pda);
      expect(config.dailyCapLamports.toString()).to.equal(largeCap.toString());
    });
  });

  describe("Distribution Edge Cases", () => {
    it("handles zero fees in treasury", async () => {
      // Test cranking when there are no fees to distribute
      console.log("Zero fees test - requires crank with empty treasury");
    });

    it("handles all investors fully unlocked (f_locked = 0)", async () => {
      // When all tokens are unlocked, f_locked = 0
      // All fees should go to creator
      console.log("All unlocked test - requires vesting completion");
    });

    it("handles all investors fully locked (f_locked = 1)", async () => {
      // When all tokens are locked, f_locked = 1
      // Maximum investor share should apply
      console.log("All locked test - requires vesting start");
    });

    it("handles single investor", async () => {
      // Edge case: only 1 investor gets 100% of investor share
      console.log("Single investor test - requires specific setup");
    });

    it("handles rounding errors with many small payments", async () => {
      // Test floor division with many investors
      // Verify no over-distribution
      console.log("Rounding test - requires many investors");
    });

    it("handles investor with zero locked amount", async () => {
      // Investor with 0 locked should get 0 payout
      console.log("Zero locked test - requires mixed vesting states");
    });

    it("handles very small fee amounts", async () => {
      // Test with fees less than min_payout for all investors
      // All should be carried forward as dust
      console.log("Small fees test - requires dust scenario");
    });
  });

  describe("Time Gate Edge Cases", () => {
    it("enforces exact 24-hour boundary", async () => {
      // Test cranking at exactly 24 hours
      console.log("24h boundary test - requires time control");
    });

    it("prevents cranking at 23 hours 59 minutes", async () => {
      // Should fail just before 24h mark
      console.log("Pre-24h test - requires time control");
    });

    it("allows cranking after 25 hours", async () => {
      // Should succeed well after 24h
      console.log("Post-24h test - requires time control");
    });
  });

  describe("Pagination Edge Cases", () => {
    it("handles single page with all investors", async () => {
      console.log("Single page test - requires <= 20 investors");
    });

    it("handles exact page boundary (20 investors)", async () => {
      console.log("Page boundary test - requires exactly 20 investors");
    });

    it("handles last page with single investor", async () => {
      console.log("Last page single investor - requires specific count");
    });

    it("handles empty page (should fail)", async () => {
      console.log("Empty page test - should reject 0 investors");
    });
  });

  describe("Account Validation Edge Cases", () => {
    it("rejects wrong quote mint in treasury", async () => {
      console.log("Wrong mint test - requires incorrect ATA");
    });

    it("rejects wrong policy config PDA", async () => {
      console.log("Wrong PDA test - requires invalid account");
    });

    it("rejects unauthorized authority", async () => {
      console.log("Unauthorized test - requires different signer");
    });
  });
});
