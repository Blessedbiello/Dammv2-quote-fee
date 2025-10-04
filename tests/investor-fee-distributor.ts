import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "../target/types/investor_fee_distributor";

describe("investor-fee-distributor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.InvestorFeeDistributor as Program<InvestorFeeDistributor>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
