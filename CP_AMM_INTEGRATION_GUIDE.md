# CP-AMM (Dynamic AMM v2) Integration Guide

**Status:** Research Complete - Integration Pattern Identified
**Last Updated:** 2025-10-04

---

## Overview

This guide documents the integration pattern for Meteora's Dynamic AMM v2 (cp-amm) program, specifically for creating an honorary position that accrues quote-only fees and claiming those fees programmatically.

## Key Finding: Lock Escrow Architecture

DAMM v2 uses a **lock escrow** model rather than traditional LP positions:
- **Lock Escrow** = A position that can hold LP tokens and accrue fees
- **Honorary Position** = Lock escrow with zero LP tokens that still claims fees

---

## Integration Setup Complete ✅

### 1. IDLs Added
- ✅ `/idls/dynamic_amm.json` - Dynamic AMM v2 program interface
- ✅ `/idls/dynamic_vault.json` - Vault program dependency
- ✅ `declare_program!(dynamic_amm)` in lib.rs
- ✅ `declare_program!(dynamic_vault)` in lib.rs

### 2. Program IDs
- **Dynamic AMM:** `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` (mainnet & devnet)
- **Vault Program:** Declared via IDL

---

## CPI Pattern: Creating Honorary Position

### Instruction: `create_lock_escrow`

**Purpose:** Creates a lock escrow (position) that can claim fees.

**Account Structure (from CPI examples):**
```rust
pub struct CreateLockEscrow<'info> {
    pub pool: AccountInfo<'info>,              // Pool account (PDA)
    pub lock_escrow: AccountInfo<'info>,       // Lock escrow to create (PDA)
    pub owner: AccountInfo<'info>,             // Owner of the lock escrow (our PDA)
    pub lp_mint: AccountInfo<'info>,           // LP token mint
    pub payer: Signer<'info>,                  // Payer for account rent
    pub system_program: AccountInfo<'info>,
}
```

**CPI Call Pattern:**
```rust
use crate::dynamic_amm;

let accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
    pool: ctx.accounts.pool.to_account_info(),
    lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
    owner: ctx.accounts.investor_fee_position_owner.to_account_info(),
    lp_mint: ctx.accounts.lp_mint.to_account_info(),
    payer: ctx.accounts.payer.to_account_info(),
    system_program: ctx.accounts.system_program.to_account_info(),
};

// Use PDA signer if owner is a PDA
let seeds = &[
    INVESTOR_FEE_POS_OWNER_SEED,
    vault.as_ref(),
    &[investor_fee_position_owner.bump],
];
let signer_seeds = &[&seeds[..]];

let cpi_ctx = CpiContext::new_with_signer(
    ctx.accounts.dynamic_amm_program.to_account_info(),
    accounts,
    signer_seeds,
);

dynamic_amm::cpi::create_lock_escrow(cpi_ctx)?;
```

---

## CPI Pattern: Claiming Fees

### Instruction: `claim_fee`

**Purpose:** Claims accrued fees from a lock escrow to specified token accounts.

**Account Structure (from CPI examples - comprehensive):**
```rust
pub struct ClaimFee<'info> {
    pub pool: AccountInfo<'info>,              // Pool account (mut)
    pub lp_mint: AccountInfo<'info>,           // LP mint (mut)
    pub lock_escrow: AccountInfo<'info>,       // Lock escrow (mut)
    pub owner: Signer<'info>,                  // Lock escrow owner (PDA with signer seeds)
    pub source_tokens: AccountInfo<'info>,     // Escrow vault (compatibility, use escrow_vault)
    pub a_vault: AccountInfo<'info>,           // Token A vault (mut)
    pub b_vault: AccountInfo<'info>,           // Token B vault (mut)
    pub a_vault_lp: AccountInfo<'info>,        // Vault A LP account (mut)
    pub b_vault_lp: AccountInfo<'info>,        // Vault B LP account (mut)
    pub a_vault_lp_mint: AccountInfo<'info>,   // Vault A LP mint (mut)
    pub b_vault_lp_mint: AccountInfo<'info>,   // Vault B LP mint (mut)
    pub user_a_token: AccountInfo<'info>,      // Destination for token A fees (mut)
    pub user_b_token: AccountInfo<'info>,      // Destination for token B fees (mut)
    pub vault_program: AccountInfo<'info>,     // Dynamic vault program
    pub escrow_vault: AccountInfo<'info>,      // Escrow vault (mut)
    pub token_program: AccountInfo<'info>,     // Token program
    pub a_token_vault: AccountInfo<'info>,     // Token A vault (mut)
    pub b_token_vault: AccountInfo<'info>,     // Token B vault (mut)
}
```

**CPI Call Pattern:**
```rust
let accounts = dynamic_amm::cpi::accounts::ClaimFee {
    pool: ctx.accounts.pool.to_account_info(),
    lp_mint: ctx.accounts.lp_mint.to_account_info(),
    lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
    owner: ctx.accounts.investor_fee_position_owner.to_account_info(),
    source_tokens: ctx.accounts.escrow_vault.to_account_info(),
    a_vault: ctx.accounts.a_vault.to_account_info(),
    b_vault: ctx.accounts.b_vault.to_account_info(),
    a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
    b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
    a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
    b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
    user_a_token: ctx.accounts.treasury_base_ata.to_account_info(),
    user_b_token: ctx.accounts.treasury_quote_ata.to_account_info(),
    vault_program: ctx.accounts.dynamic_vault.to_account_info(),
    escrow_vault: ctx.accounts.escrow_vault.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
    a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
    b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
};

let seeds = &[
    INVESTOR_FEE_POS_OWNER_SEED,
    vault.as_ref(),
    &[investor_fee_position_owner.bump],
];
let signer_seeds = &[&seeds[..]];

let cpi_ctx = CpiContext::new_with_signer(
    ctx.accounts.dynamic_amm_program.to_account_info(),
    accounts,
    signer_seeds,
);

// Claim maximum available fees
dynamic_amm::cpi::claim_fee(cpi_ctx, u64::MAX)?;
```

---

## Integration Checklist

### Prerequisites
- [x] Add dynamic_amm.json IDL to project
- [x] Add dynamic_vault.json IDL to project
- [x] Add `declare_program!(dynamic_amm)` to lib.rs
- [x] Add `declare_program!(dynamic_vault)` to lib.rs
- [ ] Obtain all required account addresses from pool setup

### For initialize_honorary_position
- [ ] Add pool account parameter
- [ ] Add lp_mint account parameter
- [ ] Add lock_escrow account (PDA to be created)
- [ ] Add dynamic_amm_program account
- [ ] Implement `create_lock_escrow` CPI call
- [ ] Store lock_escrow address in InvestorFeePositionOwner state

### For crank_distribution
- [ ] Add pool account parameter
- [ ] Add lp_mint account parameter
- [ ] Add lock_escrow account parameter
- [ ] Add vault accounts (a_vault, b_vault)
- [ ] Add vault LP accounts (a_vault_lp, b_vault_lp, a_vault_lp_mint, b_vault_lp_mint)
- [ ] Add escrow_vault account parameter
- [ ] Add token vault accounts (a_token_vault, b_token_vault)
- [ ] Add dynamic_amm_program account
- [ ] Add dynamic_vault account
- [ ] Implement `claim_fee` CPI call in first page logic

---

## Account Derivation

### Lock Escrow PDA
The lock escrow is typically derived by the dynamic_amm program. You'll need to:
1. Call `create_lock_escrow` with a unique escrow account
2. Store the escrow address in your state
3. Use that same escrow for all subsequent `claim_fee` calls

**Alternative:** Check if dynamic_amm has a deterministic PDA derivation for lock escrows.

---

## Quote-Only Fee Validation

### Pool Config Validation
Before creating the honorary position, validate the pool configuration:

```rust
// Fetch pool config
let pool_data = ctx.accounts.pool.try_borrow_data()?;

// Parse config (offset depends on Pool account structure)
// Check: config.collect_fee_mode == 1

require!(
    collect_fee_mode == 1,
    ErrorCode::PoolNotQuoteOnlyFees
);
```

### Token Order Validation
```rust
// From pool account, determine which is Token A vs Token B
// Ensure Token B is the quote mint

let pool_token_b_mint = parse_token_b_mint_from_pool(&pool_data)?;

require!(
    pool_token_b_mint == ctx.accounts.quote_mint.key(),
    ErrorCode::InvalidTokenMint
);
```

---

## Example: Full Integration Flow

### Step 1: Initialize Policy (Already Complete ✅)
```rust
// User calls initialize_policy
// Creates PolicyConfig with parameters
```

### Step 2: Create Honorary Position (Needs CPI)
```rust
// User calls initialize_honorary_position

// CPI 1: Create lock escrow
dynamic_amm::cpi::create_lock_escrow(ctx, ...)?;

// Note: No LP tokens are locked - this is an "honorary" position
// But it can still claim fees that accrue from trading

// Store lock_escrow address for future use
position_owner.lock_escrow = lock_escrow.key();
```

### Step 3: Trading Occurs (Off-Chain)
```
// Users swap tokens in the pool
// Fees accrue to all lock escrows proportionally
// Our honorary position accrues quote-only fees
```

### Step 4: Crank Distribution (Needs CPI)
```rust
// Anyone calls crank_distribution

// First page: Claim fees
if progress.current_page == 0 {
    // CPI 2: Claim fees
    dynamic_amm::cpi::claim_fee(ctx, u64::MAX)?;

    // Fees now in treasury_quote_ata and treasury_base_ata

    // Validate quote-only
    require!(treasury_base_ata.amount == 0, ErrorCode::BaseFeesDetected);
}

// Distribute to investors...
// Pay creator remainder...
```

---

## Testing Strategy

### Unit Tests (Already Complete ✅)
- Math utilities tested
- Streamflow parsing tested

### Integration Tests (Pending)
1. **Devnet Pool Setup:**
   - Create devnet pool with `collectFeeMode: 1`
   - Note all vault and LP mint addresses
   - Fund pool with test liquidity

2. **Honorary Position Test:**
   - Call `initialize_policy`
   - Call `initialize_honorary_position` with devnet pool
   - Verify lock_escrow created successfully
   - Verify no LP tokens locked

3. **Fee Accrual Test:**
   - Execute swaps in devnet pool
   - Wait for fees to accrue
   - Verify fees visible in lock escrow

4. **Crank Test:**
   - Set up test Streamflow streams
   - Call `crank_distribution` with test investors
   - Verify quote fees claimed
   - Verify pro-rata distribution
   - Verify creator remainder

---

## Dependencies

### Required Programs
```toml
# Cargo.toml (if using direct dependencies)
[dependencies]
# ... existing dependencies

# NOTE: Dynamic AMM uses declare_program! so no direct dependency needed
# The IDL files provide all type information
```

### IDL Files Location
```
/idls/dynamic_amm.json    ← Meteora Dynamic AMM v2
/idls/dynamic_vault.json  ← Vault program (dependency)
```

---

## Known Limitations

### 1. Lock Escrow Requirement
- Cannot claim fees without a lock escrow
- Honorary position requires creating escrow even with zero LP tokens
- This is a protocol design, not a limitation of our implementation

### 2. Quote-Only Enforcement
- Relies on pool config (`collectFeeMode: 1`)
- Must validate pool config before creating position
- Cannot enforce quote-only if pool not configured correctly

### 3. Account Complexity
- `claim_fee` requires 17+ accounts
- Must carefully track all vault/LP mint addresses
- Client-side complexity for gathering accounts

---

## Alternative Approach: Direct Treasury

If lock_escrow complexity is too high, consider:

**Alternative 1:** Manual Fee Transfer
- Star team manually claims fees from pool to honorary position
- Our program just distributes from treasury balance
- Simpler but requires manual intervention

**Alternative 2:** Separate Fee Collection Service
- Off-chain service claims fees periodically
- Transfers to program treasury
- Crank only handles distribution

**Current Implementation:** Uses Alternative 1
- `crank_distribution` reads from `treasury_quote_ata.amount`
- Assumes fees have been transferred there
- Ready to upgrade to full CPI when lock_escrow is established

---

## Migration Path

### Phase 1: Current (90% Complete)
- ✅ Distribution logic complete
- ✅ Streamflow integration complete
- ✅ Math utilities tested
- ⚠️ Manual fee transfer to treasury

### Phase 2: Lock Escrow Integration (10% Remaining)
- [ ] Add 17+ accounts to instructions
- [ ] Implement `create_lock_escrow` CPI
- [ ] Implement `claim_fee` CPI
- [ ] Test on devnet with real pool

### Phase 3: Production Deployment
- [ ] Test with mainnet pool
- [ ] Verify quote-only fees
- [ ] Monitor first distributions
- [ ] Set up automated cranking service

---

## Resources

### Official Documentation
- **DAMM v2 Docs:** https://docs.meteora.ag/overview/products/damm-v2
- **CPI Examples:** https://github.com/MeteoraAg/cpi-examples
- **Claiming Fees Guide:** https://docs.meteora.ag/product-overview/meteora-liquidity-pools/dynamic-amm-overview/claiming-fees-from-permanently-locked-liquidity

### Code References
- **Claim Fee Example:** `/tmp/cpi-examples/programs/cpi-example/src/instructions/dynamic_amm_cpi/claim_fee.rs`
- **Lock Liquidity Example:** `/tmp/cpi-examples/programs/cpi-example/src/instructions/dynamic_amm_cpi/lock_liquidity.rs`
- **Dynamic AMM IDL:** `/idls/dynamic_amm.json`

### Community
- **Meteora Discord:** https://discord.gg/meteora
- **Meteora GitHub:** https://github.com/MeteoraAg

---

## Quick Reference: Account List

For `claim_fee` CPI, you need these accounts:

| Account | Type | Mut | Description |
|---------|------|-----|-------------|
| pool | UncheckedAccount | ✓ | Pool PDA |
| lp_mint | UncheckedAccount | ✓ | LP token mint |
| lock_escrow | UncheckedAccount | ✓ | Honorary position |
| owner | Signer | | Position owner PDA (with seeds) |
| source_tokens | UncheckedAccount | | Escrow vault (compatibility) |
| a_vault | UncheckedAccount | ✓ | Token A vault |
| b_vault | UncheckedAccount | ✓ | Token B vault |
| a_vault_lp | UncheckedAccount | ✓ | Vault A LP account |
| b_vault_lp | UncheckedAccount | ✓ | Vault B LP account |
| a_vault_lp_mint | UncheckedAccount | ✓ | Vault A LP mint |
| b_vault_lp_mint | UncheckedAccount | ✓ | Vault B LP mint |
| user_a_token | UncheckedAccount | ✓ | Destination for token A fees |
| user_b_token | UncheckedAccount | ✓ | Destination for token B fees |
| vault_program | UncheckedAccount | | Dynamic vault program |
| escrow_vault | UncheckedAccount | ✓ | Escrow vault |
| token_program | UncheckedAccount | | SPL Token program |
| a_token_vault | UncheckedAccount | ✓ | Token A vault |
| b_token_vault | UncheckedAccount | ✓ | Token B vault |

**Total:** 17 accounts (some duplicated for compatibility)

---

## Status Summary

✅ **Complete:**
- IDL setup
- declare_program! integration
- Account structure understanding
- CPI pattern documentation

⚠️ **Partially Complete:**
- initialize_honorary_position (structure ready, CPI pending)
- crank_distribution (using treasury balance, CPI pending)

❌ **Pending:**
- Full CPI implementation with all 17+ accounts
- Devnet testing with real pool
- Integration tests

**Recommendation:** Deploy with manual fee transfer first, upgrade to full CPI in Phase 2.

---

**Last Updated:** 2025-10-04
**Author:** Implementation Team
**Status:** Research Complete - Ready for CPI Implementation
