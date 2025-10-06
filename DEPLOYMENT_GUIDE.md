# Deployment Guide: DAMM v2 Quote-Only Fee Distribution

## Build Status ✅

**Program**: `investor_fee_distributor.so` (379KB)
**Build Date**: 2025-10-06
**Rust Version**: 1.90.0
**Anchor Version**: 0.30.1
**Status**: Compiled successfully, ready for deployment

---

## Prerequisites

### 1. Environment Setup
```bash
# Verify versions
rustc --version    # Should be 1.80+
anchor --version   # Should be 0.30.1+
solana --version   # Should be 1.18+

# Check Solana configuration
solana config get
# Should show: RPC URL: https://api.devnet.solana.com
```

### 2. Wallet Setup
```bash
# Check wallet balance (need ~5 SOL for deployment)
solana balance

# If low, airdrop devnet SOL
solana airdrop 2

# Verify program keypair
solana address -k target/deploy/investor_fee_distributor-keypair.json
```

### 3. Required Accounts
Before deployment, ensure you have:
- [ ] DAMM v2 pool with `collectFeeMode: 1` (quote-only)
- [ ] Vault address (fundraising vault)
- [ ] Creator quote token ATA
- [ ] Y0 total streamed amount calculated
- [ ] Policy parameters decided (investor_fee_share_bps, daily_cap, min_payout)

---

## Deployment Steps

### Step 1: Deploy Program to Devnet

```bash
# Build first (already done)
anchor build

# Deploy program
solana program deploy target/deploy/investor_fee_distributor.so

# Note the program ID that's returned
# Should match: target/deploy/investor_fee_distributor-keypair.json
```

**Expected Output:**
```
Program Id: <PROGRAM_ID>
Signature: <TX_SIGNATURE>
```

### Step 2: Verify Deployment

```bash
# Check program deployment
PROGRAM_ID=$(solana address -k target/deploy/investor_fee_distributor-keypair.json)
solana program show $PROGRAM_ID

# Should show:
# - Program Id: <PROGRAM_ID>
# - Owner: BPFLoaderUpgradeab1e11111111111111111111111
# - ProgramData Address: <ADDRESS>
# - Authority: <YOUR_WALLET>
# - Data Length: ~379KB
```

### Step 3: Initialize Policy Configuration

**TypeScript Example:**
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvestorFeeDistributor } from "./target/types/investor_fee_distributor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.InvestorFeeDistributor as Program<InvestorFeeDistributor>;

// Configuration parameters
const vault = new PublicKey("<VAULT_ADDRESS>");
const investorFeeShareBps = 5000;  // 50%
const dailyCapLamports = 1_000_000_000;  // 1 SOL (optional)
const minPayoutLamports = 1_000_000;  // 0.001 SOL
const y0TotalStreamed = 10_000_000_000;  // 10 SOL total Y0
const creatorQuoteAta = new PublicKey("<CREATOR_QUOTE_ATA>");

// Derive PDA
const [policyConfig] = PublicKey.findProgramAddressSync(
  [Buffer.from("policy_config"), vault.toBuffer()],
  program.programId
);

// Initialize policy
const tx = await program.methods
  .initializePolicy(
    vault,
    investorFeeShareBps,
    dailyCapLamports,
    minPayoutLamports,
    y0TotalStreamed,
    creatorQuoteAta
  )
  .accounts({
    policyConfig,
    payer: provider.wallet.publicKey,
    authority: provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

console.log("Policy initialized:", tx);
```

### Step 4: Initialize Honorary Position

```typescript
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";

// Accounts needed
const pool = new PublicKey("<DAMM_V2_POOL>");
const lpMint = new PublicKey("<LP_MINT>");
const quoteMint = new PublicKey("<QUOTE_MINT>");  // Token B
const baseMint = new PublicKey("<BASE_MINT>");    // Token A

// Derive PDAs
const [investorFeePositionOwner] = PublicKey.findProgramAddressSync(
  [Buffer.from("investor_fee_pos_owner"), vault.toBuffer()],
  program.programId
);

const [policyConfig] = PublicKey.findProgramAddressSync(
  [Buffer.from("policy_config"), vault.toBuffer()],
  program.programId
);

// Derive lock escrow (from DAMM v2 program)
const DYNAMIC_AMM_PROGRAM = new PublicKey("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
const [lockEscrow] = PublicKey.findProgramAddressSync(
  [Buffer.from("lock_escrow"), pool.toBuffer(), investorFeePositionOwner.toBuffer()],
  DYNAMIC_AMM_PROGRAM
);

// Derive treasury ATAs
const treasuryQuoteAta = anchor.utils.token.associatedAddress({
  mint: quoteMint,
  owner: investorFeePositionOwner
});

const treasuryBaseAta = anchor.utils.token.associatedAddress({
  mint: baseMint,
  owner: investorFeePositionOwner
});

// Initialize honorary position
const tx = await program.methods
  .initializeHonoraryPosition(vault)
  .accounts({
    investorFeePositionOwner,
    policyConfig,
    pool,
    lpMint,
    lockEscrow,
    quoteMint,
    baseMint,
    treasuryQuoteAta,
    treasuryBaseAta,
    payer: provider.wallet.publicKey,
    authority: provider.wallet.publicKey,
    dynamicAmmProgram: DYNAMIC_AMM_PROGRAM,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

console.log("Honorary position created:", tx);
console.log("Lock escrow:", lockEscrow.toString());
```

### Step 5: Test Distribution Crank

**Manual Version** (fees pre-transferred):
```typescript
// Prepare investor data
const investorData = [
  {
    index: 0,
    quoteAta: new PublicKey("<INVESTOR_1_QUOTE_ATA>")
  },
  {
    index: 1,
    quoteAta: new PublicKey("<INVESTOR_2_QUOTE_ATA>")
  }
  // ... more investors
];

const totalPages = 1;  // Adjust based on number of investors (20-30 per page)

// Derive daily progress PDA
const [dailyProgress] = PublicKey.findProgramAddressSync(
  [Buffer.from("daily_progress"), vault.toBuffer()],
  program.programId
);

// Prepare remaining accounts (streams + ATAs)
const remainingAccounts = [
  // First: Streamflow stream accounts (read-only)
  { pubkey: streamflowStream1, isSigner: false, isWritable: false },
  { pubkey: streamflowStream2, isSigner: false, isWritable: false },
  // Second: Investor quote ATAs (writable)
  { pubkey: investor1QuoteAta, isSigner: false, isWritable: true },
  { pubkey: investor2QuoteAta, isSigner: false, isWritable: true },
];

// Execute crank
const tx = await program.methods
  .crankDistribution(totalPages, investorData)
  .accounts({
    cranker: provider.wallet.publicKey,
    dailyProgress,
    policyConfig,
    investorFeePositionOwner,
    treasuryQuoteAta,
    treasuryBaseAta,
    creatorQuoteAta,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .remainingAccounts(remainingAccounts)
  .rpc();

console.log("Distribution completed:", tx);
```

**Full CPI Version** (automatic fee claiming):
```typescript
// Additional accounts for claim_fee CPI
const DYNAMIC_VAULT_PROGRAM = new PublicKey("<DYNAMIC_VAULT_PROGRAM_ID>");

// Get vault accounts from pool state
const aVault = new PublicKey("<A_VAULT>");
const bVault = new PublicKey("<B_VAULT>");
const aVaultLp = new PublicKey("<A_VAULT_LP>");
const bVaultLp = new PublicKey("<B_VAULT_LP>");
const aVaultLpMint = new PublicKey("<A_VAULT_LP_MINT>");
const bVaultLpMint = new PublicKey("<B_VAULT_LP_MINT>");
const aTokenVault = new PublicKey("<A_TOKEN_VAULT>");
const bTokenVault = new PublicKey("<B_TOKEN_VAULT>");
const escrowVault = new PublicKey("<ESCROW_VAULT>");

const tx = await program.methods
  .crankDistributionFull(totalPages, investorData)
  .accounts({
    cranker: provider.wallet.publicKey,
    dailyProgress,
    policyConfig,
    investorFeePositionOwner,
    // CPI accounts for claim_fee
    pool,
    lpMint,
    lockEscrow,
    escrowVault,
    aVault,
    bVault,
    aVaultLp,
    bVaultLp,
    aVaultLpMint,
    bVaultLpMint,
    aTokenVault,
    bTokenVault,
    dynamicAmmProgram: DYNAMIC_AMM_PROGRAM,
    dynamicVault: DYNAMIC_VAULT_PROGRAM,
    // Distribution accounts
    treasuryQuoteAta,
    treasuryBaseAta,
    creatorQuoteAta,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .remainingAccounts(remainingAccounts)
  .rpc();

console.log("Full CPI distribution completed:", tx);
```

---

## Testing Checklist

### Pre-Deployment Tests
- [x] Program builds successfully
- [x] All unit tests pass (math, streamflow utils)
- [ ] Deploy to devnet
- [ ] Verify program deployment

### Integration Tests
- [ ] Initialize policy with test parameters
- [ ] Create honorary position
- [ ] Verify lock escrow created
- [ ] Manually transfer test fees to treasury
- [ ] Execute crank with 1-2 test investors
- [ ] Verify pro-rata payouts correct
- [ ] Test quote-only enforcement (reject if base fees present)
- [ ] Test 24h time gate (reject if called too early)
- [ ] Test multi-page distribution
- [ ] Test creator remainder distribution

### Edge Case Tests
- [ ] All investors fully unlocked (100% to creator)
- [ ] Some investors locked, some unlocked
- [ ] Dust threshold handling
- [ ] Daily cap enforcement
- [ ] Zero fees scenario

---

## Monitoring & Maintenance

### Event Monitoring
Monitor these events for system health:
```typescript
// Listen for events
program.addEventListener("PolicyConfigCreated", (event) => {
  console.log("Policy created:", event);
});

program.addEventListener("HonoraryPositionInitialized", (event) => {
  console.log("Position initialized:", event);
});

program.addEventListener("QuoteFeesClaimed", (event) => {
  console.log("Fees claimed:", event.amountClaimed);
});

program.addEventListener("InvestorPayoutPage", (event) => {
  console.log(`Page ${event.page}: ${event.investorsPaid} paid, ${event.totalDistributed} distributed`);
});

program.addEventListener("CreatorPayoutDayClosed", (event) => {
  console.log("Day closed - creator received:", event.creatorAmount);
});
```

### Automated Cranking
Set up a cron job or service to call the crank every 24 hours:
```bash
# Example cron (runs daily at midnight UTC)
0 0 * * * cd /path/to/project && node scripts/crank-distribution.js
```

### State Queries
```typescript
// Check policy config
const policy = await program.account.policyConfig.fetch(policyConfigPda);
console.log("Investor share:", policy.investorFeeShareBps / 100, "%");
console.log("Y0:", policy.y0TotalStreamed);

// Check daily progress
const progress = await program.account.dailyProgress.fetch(dailyProgressPda);
console.log("Day ID:", progress.dayId);
console.log("Total claimed today:", progress.totalQuoteClaimedToday);
console.log("Investor distributed:", progress.investorDistributedToday);
console.log("Creator distributed:", progress.creatorDistributedToday);
console.log("Current page:", progress.currentPage, "/", progress.totalPages);
console.log("Finalized:", progress.isFinalized);

// Check position stats
const position = await program.account.investorFeePositionOwner.fetch(investorFeePositionOwnerPda);
console.log("Lock escrow:", position.lockEscrow.toString());
console.log("Lifetime fees claimed:", position.totalFeesClaimed);
```

---

## Troubleshooting

### Common Errors

**"Pool not quote-only fees"**
- Ensure DAMM v2 pool has `collectFeeMode: 1`
- Verify pool configuration

**"Base fees detected"**
- Quote-only validation failed
- Check `treasury_base_ata.amount == 0`
- Pool may not be configured correctly

**"Too early for next day"**
- 24h time gate enforced
- Must wait full 24 hours since last crank
- Current time: `clock.unix_timestamp`
- Last window: `progress.window_start + 86400`

**"Daily cap reached"**
- Hit the daily distribution limit
- Wait for next day or increase cap

**"Streamflow account mismatch"**
- Invalid or wrong Streamflow stream account
- Verify stream addresses match investors

**"Arithmetic overflow"**
- Check calculation inputs
- Ensure amounts within u64 range

---

## Security Considerations

### Access Control
- ✅ Permissionless cranking (anyone can call)
- ✅ PDA-owned position (no one can withdraw)
- ✅ Creator ATA locked in policy (immutable)
- ✅ Authority only needed for initialization

### Economic Security
- ✅ Quote-only enforcement prevents fund mixing
- ✅ Daily cap prevents over-distribution
- ✅ Minimum payout reduces spam
- ✅ Floor division ensures no over-payment

### State Security
- ✅ 24h time gate prevents premature cranking
- ✅ Idempotent pagination prevents double-payment
- ✅ Finalization flag prevents re-distribution

### Recommendations
1. Test thoroughly on devnet before mainnet
2. Start with small daily caps initially
3. Monitor events for anomalies
4. Have a contingency plan for failed cranks
5. Document all parameter choices

---

## Next Steps

1. **Deploy to Devnet** ✅ Ready
2. **Integration Testing** - Test all flows
3. **Security Audit** - Optional but recommended
4. **Mainnet Deployment** - After successful devnet testing
5. **Automated Cranker** - Set up 24h automation
6. **Monitoring Dashboard** - Track distributions

---

## Support & Resources

- **Anchor Docs**: https://www.anchor-lang.com/
- **Solana Cookbook**: https://solanacookbook.com/
- **DAMM v2**: https://github.com/MeteoraAg/damm-v2
- **Streamflow**: https://docs.streamflow.finance/

**Program Address**: Check `target/deploy/investor_fee_distributor-keypair.json`
**Repository**: https://github.com/Blessedbiello/Dammv2-quote-fee.git
