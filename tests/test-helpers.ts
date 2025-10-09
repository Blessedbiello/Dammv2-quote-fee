import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "../target/types/investor_fee_distributor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { BN } from "bn.js";

export const DYNAMIC_AMM_PROGRAM_ID = new PublicKey(
  "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB"
);

export const DYNAMIC_VAULT_PROGRAM_ID = new PublicKey(
  "VAU1T7S5UuEHmMvXtXMVmpEoQtZ2ya7eRb7gcN47wDp"
);

export const STREAMFLOW_PROGRAM_ID = new PublicKey(
  "strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m"
);

export interface TestContext {
  provider: anchor.AnchorProvider;
  program: Program<InvestorFeeDistributor>;
  payer: Keypair;
  quoteMint: PublicKey;
  baseMint: PublicKey;
  vault: PublicKey;
}

export async function setupTestContext(): Promise<TestContext> {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .InvestorFeeDistributor as Program<InvestorFeeDistributor>;
  const payer = (provider.wallet as anchor.Wallet).payer;

  // Create test mints
  const quoteMint = await createMint(
    provider.connection,
    payer,
    payer.publicKey,
    null,
    9 // 9 decimals
  );

  const baseMint = await createMint(
    provider.connection,
    payer,
    payer.publicKey,
    null,
    9
  );

  // Generate unique vault identifier
  const vault = Keypair.generate().publicKey;

  return {
    provider,
    program,
    payer,
    quoteMint,
    baseMint,
    vault,
  };
}

export function derivePolicyConfigPda(
  program: Program<InvestorFeeDistributor>,
  vault: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("policy_config"), vault.toBuffer()],
    program.programId
  );
}

export function deriveInvestorFeePositionOwnerPda(
  program: Program<InvestorFeeDistributor>,
  vault: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("investor_fee_pos_owner"), vault.toBuffer()],
    program.programId
  );
}

export function deriveDailyProgressPda(
  program: Program<InvestorFeeDistributor>,
  vault: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("daily_progress"), vault.toBuffer()],
    program.programId
  );
}

export async function createTokenAccount(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  const payer = (provider.wallet as anchor.Wallet).payer;
  return await createAccount(
    provider.connection,
    payer,
    mint,
    owner,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
}

export async function mintTokensTo(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  destination: PublicKey,
  amount: number | bigint
): Promise<void> {
  const payer = (provider.wallet as anchor.Wallet).payer;
  await mintTo(
    provider.connection,
    payer,
    mint,
    destination,
    payer,
    amount,
    [],
    undefined,
    TOKEN_PROGRAM_ID
  );
}

export async function getTokenBalance(
  provider: anchor.AnchorProvider,
  tokenAccount: PublicKey
): Promise<bigint> {
  const account = await getAccount(
    provider.connection,
    tokenAccount,
    undefined,
    TOKEN_PROGRAM_ID
  );
  return account.amount;
}

export interface MockStreamflowStream {
  keypair: Keypair;
  recipient: PublicKey;
  depositedAmount: BN;
  startTime: BN;
  endTime: BN;
  cliffAmount: BN;
}

export async function createMockStreamflowStream(
  provider: anchor.AnchorProvider,
  recipient: PublicKey,
  depositedAmount: number,
  durationSeconds: number,
  cliffSeconds: number = 0,
  cliffAmount: number = 0
): Promise<MockStreamflowStream> {
  const streamKeypair = Keypair.generate();
  const currentTime = Math.floor(Date.now() / 1000);

  // Create account for mock stream data
  const streamAccount = streamKeypair.publicKey;

  // In real implementation, this would be created by Streamflow program
  // For testing, we'll create a mock account with similar structure
  const space = 512; // Enough space for StreamflowStream struct
  const rent = await provider.connection.getMinimumBalanceForRentExemption(space);

  const payer = (provider.wallet as anchor.Wallet).payer;

  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: streamAccount,
    lamports: rent,
    space,
    programId: STREAMFLOW_PROGRAM_ID,
  });

  const tx = new anchor.web3.Transaction().add(createAccountIx);
  await provider.sendAndConfirm(tx, [payer, streamKeypair]);

  return {
    keypair: streamKeypair,
    recipient,
    depositedAmount: new BN(depositedAmount),
    startTime: new BN(currentTime),
    endTime: new BN(currentTime + durationSeconds),
    cliffAmount: new BN(cliffAmount),
  };
}

export async function airdrop(
  provider: anchor.AnchorProvider,
  to: PublicKey,
  lamports: number
): Promise<void> {
  const signature = await provider.connection.requestAirdrop(to, lamports);
  await provider.connection.confirmTransaction(signature);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export const ONE_SOL = 1_000_000_000;
export const ONE_HOUR = 3600;
export const ONE_DAY = 86400;
