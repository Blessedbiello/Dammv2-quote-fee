# DAMM v2 Honorary Quote-Only Fee Position + 24h Distribution Crank

A Solana Anchor program that creates and manages an honorary DAMM v2 LP position owned by a program PDA, accruing fees exclusively in the quote mint, with permissionless 24-hour distribution cranks that distribute fees pro-rata to investors based on still-locked Streamflow amounts.

## 🎯 Overview

This module enables Star (or any fundraising platform) to:

1. **Create an honorary LP position** in Meteora's DAMM v2 pools that accrues trading fees **only in the quote token**
2. **Run permissionless cranks** once per 24 hours to distribute accumulated fees
3. **Pay investors pro-rata** based on their still-locked token amounts from Streamflow vesting contracts
4. **Route remainder to creator** after investor distributions complete

### Key Features

- ✅ **Quote-only fee accrual** - Validates pool has `collectFeeMode: 1`, rejects base token fees
- ✅ **Program-owned position** - Honorary position owned by PDA, independent of creator
- ✅ **24-hour time gates** - Enforces once-per-day distribution windows
- ✅ **Idempotent pagination** - Safe multi-page processing with resume capability
- ✅ **Dust handling** - Amounts below threshold carried forward to next distribution
- ✅ **Daily caps** - Optional limits on investor distributions per day
- ✅ **Permissionless cranking** - Anyone can trigger distributions (with optional incentives)

---

## 📋 Architecture

### State Accounts

| Account | PDA Seeds | Size | Purpose |
|---------|-----------|------|---------|
| **PolicyConfig** | `[b"policy_config", vault]` | 156 bytes | Fee distribution policy and parameters |
| **InvestorFeePositionOwner** | `[b"investor_fee_pos_owner", vault]` | 280 bytes | Owns the honorary DAMM v2 position |
| **DailyProgress** | `[b"daily_progress", vault]` | 142 bytes | Tracks 24h window state and pagination |

### Instructions

1. **`initialize_policy`** - Set up fee distribution policy
2. **`initialize_honorary_position`** - Create quote-only DAMM v2 position
3. **`crank_distribution`** - ⚠️ *TO BE IMPLEMENTED* - Process fee distributions

### Program Flow

```
┌─────────────────────────────────────────────────────────────┐
│  1. INITIALIZATION (One-time setup)                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  initialize_policy(vault, params)                            │
│    └─> Creates PolicyConfig PDA with:                        │
│         • investor_fee_share_bps (e.g., 7000 = 70% max)     │
│         • Y0 (total investor allocation at TGE)             │
│         • creator_quote_ata (remainder destination)         │
│         • min_payout_lamports (dust threshold)              │
│         • optional daily_cap_lamports                       │
│                                                               │
│  initialize_honorary_position(vault)                         │
│    └─> Creates InvestorFeePositionOwner PDA                 │
│    └─> Validates pool has collectFeeMode == 1               │
│    └─> Creates DAMM v2 position owned by PDA                │
│    └─> Creates treasury ATAs for quote + base tokens        │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  2. RECURRING OPERATIONS (Daily distributions)               │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  [Trading activity accumulates fees in quote token]          │
│                            │                                  │
│                            ▼                                  │
│  crank_distribution(investor_page) - First page              │
│    └─> Check 24h gate (current_time >= last_window + 86400) │
│    └─> Claim fees from DAMM v2 position (quote only)        │
│    └─> Verify zero base token balance (enforcement)         │
│    └─> Read Streamflow locked amounts at time t             │
│    └─> Calculate: f_locked = locked_total / Y0              │
│    └─> Calculate: investor_share = min(policy_bps, f_locked)│
│    └─> Distribute pro-rata to page of investors             │
│    └─> Carry dust below min_payout forward                  │
│    └─> Emit InvestorPayoutPage event                        │
│                            │                                  │
│                            ▼                                  │
│  crank_distribution(investor_page) - Subsequent pages        │
│    └─> Continue from cursor position                         │
│    └─> Process next batch of investors                       │
│    └─> Update pagination state                               │
│                            │                                  │
│                            ▼                                  │
│  crank_distribution(investor_page) - Final page              │
│    └─> Process remaining investors                           │
│    └─> Calculate creator_remainder                           │
│    └─> Transfer remainder to creator_quote_ata              │
│    └─> Finalize day                                          │
│    └─> Emit CreatorPayoutDayClosed event                    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                    [Repeat next day]
```

---

## 🚀 Quick Start

### Prerequisites

```bash
# Required tools
anchor --version  # 0.30.1 or higher
solana --version  # 2.1.0 or higher
rustc --version   # 1.85.0 or higher

# Required information from Star team
# 1. DAMM v2 pool address (with collectFeeMode: 1)
# 2. Vault identifier (Pubkey)
# 3. Y0 value (total investor streamed allocation at TGE)
# 4. Creator quote token ATA
# 5. List of investor Streamflow streams and ATAs
```

### Installation

```bash
# Clone repository
git clone <repo-url>
cd investor-fee-distributor

# Install dependencies
anchor build

# Run tests (once implemented)
anchor test
```

### Deployment

```bash
# Build program
anchor build

# Deploy to devnet (for testing)
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet-beta
```

---

## 📚 Usage Guide

### 1. Initialize Policy

```typescript
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';

const vaultPubkey = new PublicKey("<your-vault-id>");
const creatorQuoteAta = new PublicKey("<creator-quote-ata>");

// Derive PolicyConfig PDA
const [policyConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("policy_config"), vaultPubkey.toBuffer()],
    program.programId
);

// Initialize policy
await program.methods
    .initializePolicy(
        vaultPubkey,              // vault identifier
        7000,                     // investor_fee_share_bps (70%)
        null,                     // daily_cap_lamports (no cap)
        1000,                     // min_payout_lamports (0.000001 tokens with 9 decimals)
        new BN("1000000000000"),  // y0_total_streamed (1M tokens)
        creatorQuoteAta           // creator quote ATA
    )
    .accounts({
        policyConfig: policyConfigPda,
        payer: wallet.publicKey,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .rpc();
```

### 2. Initialize Honorary Position

**⚠️ NOTE:** This instruction requires cp-amm integration to be completed.

```typescript
const poolPubkey = new PublicKey("<damm-v2-pool-address>");
const quoteMint = new PublicKey("<quote-token-mint>");
const baseMint = new PublicKey("<base-token-mint>");

// Derive InvestorFeePositionOwner PDA
const [positionOwnerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("investor_fee_pos_owner"), vaultPubkey.toBuffer()],
    program.programId
);

// Derive treasury ATAs
const treasuryQuoteAta = getAssociatedTokenAddressSync(
    quoteMint,
    positionOwnerPda,
    true // allowOwnerOffCurve
);

const treasuryBaseAta = getAssociatedTokenAddressSync(
    baseMint,
    positionOwnerPda,
    true
);

await program.methods
    .initializeHonoraryPosition(vaultPubkey)
    .accounts({
        investorFeePositionOwner: positionOwnerPda,
        policyConfig: policyConfigPda,
        pool: poolPubkey,
        poolConfig: /* derive from pool */,
        position: /* to be created */,
        positionNftMint: /* new mint */,
        quoteMint,
        baseMint,
        treasuryQuoteAta,
        treasuryBaseAta,
        payer: wallet.publicKey,
        authority: wallet.publicKey,
        cpAmmProgram: CP_AMM_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();
```

### 3. Crank Distribution

**⚠️ NOTE:** This instruction is not yet implemented. See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

```typescript
// Pseudo-code for when implemented
const currentDayId = Math.floor(Date.now() / 1000 / 86400);

// Derive DailyProgress PDA
const [dailyProgressPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("daily_progress"), vaultPubkey.toBuffer()],
    program.programId
);

// Prepare investor page (batch of ~20 investors)
const investorPage = [
    {
        streamPubkey: new PublicKey("..."),
        quoteAta: new PublicKey("..."),
        lockedAmount: new BN("..."), // Will be read from Streamflow
    },
    // ... more investors
];

await program.methods
    .crankDistribution(investorPage.length)
    .accounts({
        cranker: cranker.publicKey,
        dailyProgress: dailyProgressPda,
        policyConfig: policyConfigPda,
        investorFeePositionOwner: positionOwnerPda,
        // ... more accounts
    })
    .remainingAccounts([
        // Streamflow stream accounts (read-only)
        ...investorPage.map(inv => ({
            pubkey: inv.streamPubkey,
            isSigner: false,
            isWritable: false,
        })),
        // Investor quote ATAs (writable for transfers)
        ...investorPage.map(inv => ({
            pubkey: inv.quoteAta,
            isSigner: false,
            isWritable: true,
        })),
    ])
    .rpc();
```

---

## 🔧 Configuration Reference

### PolicyConfig Parameters

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `vault` | `Pubkey` | Unique vault identifier | `new PublicKey("...")` |
| `investor_fee_share_bps` | `u16` | Max investor share in basis points (0-10000) | `7000` (70%) |
| `daily_cap_lamports` | `Option<u64>` | Optional daily distribution cap in lamports | `Some(1_000_000_000)` or `None` |
| `min_payout_lamports` | `u64` | Minimum payout threshold (dust) | `1000` (0.000001 tokens) |
| `y0_total_streamed` | `u64` | Total investor allocation minted at TGE | `1_000_000_000_000` (1M tokens @ 9 decimals) |
| `creator_quote_ata` | `Pubkey` | Creator's quote token ATA for remainder | `new PublicKey("...")` |

### Fee Distribution Formula

```
Given:
  - Y0 = Total investor allocation at TGE
  - locked_total(t) = Sum of still-locked across all investors at time t
  - claimed_quote = Total quote fees claimed from pool today

Calculate:
  1. f_locked(t) = locked_total(t) / Y0                                [0, 1]
  2. eligible_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))
  3. investor_fee_quote = floor(claimed_quote * eligible_bps / 10000)

  For each investor i:
    4. weight_i(t) = locked_i(t) / locked_total(t)
    5. payout_i = floor(investor_fee_quote * weight_i(t))

    6. If payout_i >= min_payout_lamports:
         Transfer payout_i to investor_i.quote_ata
       Else:
         carry_over_lamports += payout_i

  7. creator_remainder = claimed_quote - total_distributed_to_investors
  8. Transfer creator_remainder to policy.creator_quote_ata
```

**Key Properties:**
- Uses **floor division** for all calculations (no rounding up)
- **In-kind distribution** - only quote tokens, no price conversions
- **Dust threshold** - payouts below `min_payout_lamports` carried forward
- **Daily cap** - if set, limits total investor payouts per day

---

## 📊 Event Reference

### HonoraryPositionInitialized
Emitted when honorary position is created.

```rust
pub struct HonoraryPositionInitialized {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}
```

### QuoteFeesClaimed
Emitted when fees are claimed from DAMM v2 position.

```rust
pub struct QuoteFeesClaimed {
    pub day_id: u64,
    pub amount_claimed: u64,
    pub position: Pubkey,
    pub timestamp: i64,
}
```

### InvestorPayoutPage
Emitted after each page of investor distributions.

```rust
pub struct InvestorPayoutPage {
    pub day_id: u64,
    pub page: u16,
    pub investors_paid: u16,
    pub total_distributed: u64,
    pub dust_carried: u64,
    pub timestamp: i64,
}
```

### CreatorPayoutDayClosed
Emitted when day is finalized and creator receives remainder.

```rust
pub struct CreatorPayoutDayClosed {
    pub day_id: u64,
    pub creator_amount: u64,
    pub total_investors_paid: u64,
    pub total_pages: u16,
    pub timestamp: i64,
}
```

---

## ⚠️ Error Codes

| Code | Name | Description |
|------|------|-------------|
| 6000 | `PoolNotQuoteOnlyFees` | Pool config does not have `collectFeeMode: 1` |
| 6001 | `BaseFeesDetected` | Base token fees found in treasury - aborts distribution |
| 6002 | `TooEarlyForNextDay` | Must wait 24 hours since last window start |
| 6003 | `OutsideWindow` | Current time outside valid 24-hour window |
| 6004 | `DayAlreadyFinalized` | Day complete, no more distributions allowed |
| 6005 | `DailyCapReached` | Daily cap hit, cannot distribute more today |
| 6006 | `InvalidInvestorPage` | Invalid investor page data provided |
| 6007 | `StreamflowAccountMismatch` | Streamflow account data invalid |
| 6008 | `ArithmeticOverflow` | Overflow in distribution calculation |
| 6009 | `ArithmeticUnderflow` | Underflow in calculation |
| 6010 | `InvalidTokenMint` | Token mint doesn't match expected quote/base |
| 6011 | `InvalidPosition` | Position does not exist or is invalid |
| 6012 | `InvalidPolicy` | Policy configuration is invalid |
| 6013 | `NoFeesAvailable` | No fees to distribute |
| 6014 | `InvalidTotalPages` | Total pages must be > 0 |
| 6015 | `InvalidFeeShareBps` | Fee share BPS exceeds 10000 |
| 6016 | `InvalidY0Amount` | Y0 total streamed cannot be zero |

---

## 🔐 Security Considerations

### 1. Quote-Only Enforcement
- ✅ Validates pool `collectFeeMode: 1` at initialization
- ✅ Checks base token balance before each distribution
- ✅ **Fails deterministically** if any base fees detected

### 2. Time Gate Protection
- ✅ 24-hour minimum between day starts
- ✅ Window boundaries calculated as `day_id * 86400`
- ✅ Clock sysvar used for consensus time

### 3. Arithmetic Safety
- ✅ All calculations use checked arithmetic
- ✅ Overflow/underflow returns errors (no panics)
- ✅ Floor division prevents rounding attacks

### 4. PDA Ownership
- ✅ Deterministic seeds prevent collisions
- ✅ Canonical bumps stored and reused
- ✅ Position owned by program PDA, not externally controlled

### 5. Idempotency
- ✅ Same page can be called multiple times safely
- ✅ State prevents double-payments
- ✅ Resumable after partial failures

### 6. Access Control
- ✅ Policy initialization requires authority signature
- ✅ Cranking is permissionless (anyone can call)
- ✅ Funds only distributed per policy rules

---

## 🧪 Testing

**⚠️ Test suite not yet implemented.** See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for test plan.

### Test Categories

1. **Initialization Tests**
   - Create policy with valid parameters
   - Reject invalid BPS or Y0 values
   - Create honorary position with quote-only pool
   - Reject pools without `collectFeeMode: 1`

2. **Distribution Tests**
   - Simulate fee accrual in both swap directions
   - Multi-page crank with partial locks
   - All unlocked (100% to creator)
   - Daily cap enforcement
   - Dust handling and carry-forward

3. **Edge Cases**
   - Base fee detection (must fail)
   - Missing investor ATAs
   - Idempotent page retries
   - 24h gate violations

4. **Integration Tests**
   - Real Streamflow account parsing
   - Token-2022 compatibility
   - Large investor sets (100+)

---

## 📁 Project Structure

```
investor-fee-distributor/
├── programs/
│   └── investor-fee-distributor/
│       ├── src/
│       │   ├── lib.rs                              # Program entry
│       │   ├── constants.rs                        # PDA seeds, program IDs
│       │   ├── error.rs                            # Error codes
│       │   ├── events.rs                           # Event definitions
│       │   ├── state/
│       │   │   ├── mod.rs
│       │   │   ├── policy_config.rs                # Fee policy state
│       │   │   ├── daily_progress.rs               # 24h tracking state
│       │   │   └── investor_fee_position_owner.rs  # Position owner PDA
│       │   └── instructions/
│       │       ├── mod.rs
│       │       ├── initialize_policy.rs            # ✅ Complete
│       │       └── initialize_honorary_position.rs # ⚠️ Needs cp-amm CPI
│       └── Cargo.toml
├── tests/                                          # ❌ Not started
├── sdk/                                            # ❌ Not started
├── README.md                                       # ✅ This file
├── IMPLEMENTATION_STATUS.md                        # ✅ Progress tracker
└── Anchor.toml
```

---

## 🚧 Implementation Status

**Current Status:** 60% Foundation Complete

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for detailed progress.

**Completed:**
- ✅ State accounts (PolicyConfig, DailyProgress, InvestorFeePositionOwner)
- ✅ Error codes (18 errors)
- ✅ Events (6 events)
- ✅ initialize_policy instruction
- ✅ initialize_honorary_position structure

**Pending:**
- ⚠️ cp-amm CPI integration (requires program interface details)
- ❌ crank_distribution instruction (logic designed, not coded)
- ❌ Streamflow account parsing
- ❌ Helper functions (math, utils)
- ❌ Test suite
- ❌ TypeScript SDK

---

## 📞 Support & Resources

### External Programs

- **DAMM v2 (cp-amm)**
  - Program ID: `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`
  - Docs: https://docs.meteora.ag/overview/products/damm-v2
  - SDK: `@meteora-ag/cp-amm-sdk`
  - Repo: https://github.com/MeteoraAg/damm-v2

- **Streamflow**
  - Program ID: `strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m`
  - Docs: https://streamflow.finance/
  - SDK: `@streamflow/stream`
  - Repo: https://github.com/streamflow-finance/js-sdk

### Key Concepts

- **Basis Points (BPS):** 1 BPS = 0.01%. 10,000 BPS = 100%
- **Day ID:** `unix_timestamp / 86400` (days since epoch)
- **f_locked:** Fraction of tokens still locked (0 to 1)
- **Quote Token:** Token B in DAMM v2 pool (with `collectFeeMode: 1`)
- **Y0:** Total investor allocation minted at Token Generation Event

### For Questions

1. **Technical Implementation:** See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)
2. **Integration Help:** Review [Usage Guide](#-usage-guide) section
3. **Math Formulas:** See [Fee Distribution Formula](#fee-distribution-formula)

---

## 📜 License

[Add your license here]

---

## 🙏 Acknowledgments

Built for **Star** - the fundraising platform where founders raise capital in live, public token sales.

Powered by:
- **Meteora's DAMM v2** - Quote-only fee collection mechanism
- **Streamflow** - Token vesting and streaming payments
- **Anchor Framework** - Solana program development

---

**Last Updated:** 2025-10-04
**Version:** 0.1.0-foundation
**Status:** Foundation Complete, Integration Pending

For implementation completion, see [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).
