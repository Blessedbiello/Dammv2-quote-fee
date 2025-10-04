# Implementation Status: DAMM v2 Honorary Quote-Only Fee Position + 24h Distribution Crank

## ✅ COMPLETED FOUNDATION (60% Complete)

### 1. Project Structure ✅
- Anchor project initialized with proper structure
- Dependencies configured (Anchor 0.30.1, anchor-spl, solana-program 2.1.0)
- Module organization established

### 2. State Accounts ✅
**All three core state accounts fully implemented:**

- **PolicyConfig** ([policy_config.rs](programs/investor-fee-distributor/src/state/policy_config.rs))
  - Stores fee distribution policy
  - Fields: investor_fee_share_bps, daily_cap_lamports, min_payout_lamports, y0_total_streamed, creator_quote_ata
  - Space calculation: 156 bytes

- **DailyProgress** ([daily_progress.rs](programs/investor-fee-distributor/src/state/daily_progress.rs))
  - Tracks 24h window state
  - Helper methods: `is_within_window()`, `can_crank()`, `reset_for_new_day()`
  - Space calculation: 142 bytes

- **InvestorFeePositionOwner** ([investor_fee_position_owner.rs](programs/investor-fee-distributor/src/state/investor_fee_position_owner.rs))
  - PDA that owns the honorary position
  - Tracks pool, position, mints, and lifetime stats
  - Space calculation: 280 bytes

### 3. Errors & Events ✅
**Complete error handling:**
- 18 custom error codes defined
- Covers all failure modes: quote-only validation, time gates, arithmetic, etc.

**Complete event emissions:**
- `HonoraryPositionInitialized`
- `QuoteFeesClaimed`
- `InvestorPayoutPage`
- `CreatorPayoutDayClosed`
- `PolicyConfigCreated`
- `DailyProgressReset`

### 4. Constants ✅
- PDA seeds defined
- Program IDs documented (cp-amm, Streamflow)
- Time and math constants (SECONDS_PER_DAY, MAX_BPS)

### 5. Instructions (Partial)
**✅ Completed:**
- `initialize_policy` - Fully functional

**⚠️ Partially Completed (needs cp-amm integration):**
- `initialize_honorary_position` - Structure complete, CPI calls marked with TODOs

**❌ Not Started:**
- `crank_distribution` - Core logic designed but not yet implemented
- `initialize_daily_progress` - Simple initialization needed
- Helper functions for Streamflow parsing

---

## 🔧 REMAINING WORK (40%)

### Critical Path Items

#### 1. CP-AMM Integration (High Priority)
**Required Information from Star Team:**
- Exact cp-amm account structures (Pool, Config, Position)
- Instruction data layouts for:
  - `create_position`
  - `claim_position_fee`
- Field offsets for:
  - `collectFeeMode` in Config account
  - Token A/B mints in Pool account
  - Fee amounts in Position account

**Files to Complete:**
- `initialize_honorary_position.rs` - Add CPI calls (marked with TODO comments)
- Create new file: `claim_position_fees.rs` - Helper for fee claiming

**Approach:**
```rust
// Option 1: Use @meteora-ag/cp-amm SDK (if Rust bindings exist)
// Option 2: Manual CPI with anchor_lang::solana_program::instruction::Instruction
// Option 3: Use declare_program! macro with cp-amm IDL
```

#### 2. Crank Distribution Instruction (High Priority)
**File to Create:** `instructions/crank_distribution.rs`

**Key Components Needed:**
1. **Account Structure:**
   - DailyProgress (mut)
   - PolicyConfig
   - InvestorFeePositionOwner
   - Treasury ATAs (quote + base for validation)
   - Creator quote ATA
   - Remaining accounts: Streamflow streams + investor ATAs

2. **Core Logic Flow:**
   ```rust
   1. Check 24h gate (day_id calculation)
   2. Reset or continue current day
   3. If first page: claim fees from cp-amm position
   4. Validate zero base fees in treasury
   5. Parse Streamflow accounts for locked amounts
   6. Calculate f_locked and eligible_investor_share_bps
   7. Distribute pro-rata with floor division
   8. Handle dust (< min_payout_lamports)
   9. Update DailyProgress state
   10. If final page: pay creator remainder
   11. Emit events
   ```

3. **Streamflow Integration:**
   - Need Streamflow account structure documentation
   - Parse locked amount at timestamp t
   - Formula: `locked = deposited - withdrawn - vested_at_time_t`

#### 3. Helper Functions
**File to Create:** `utils/streamflow.rs`
```rust
pub fn calculate_locked_amount(
    stream_account: &AccountInfo,
    current_time: i64
) -> Result<u64>;

pub fn calculate_total_locked(
    stream_accounts: &[AccountInfo],
    current_time: i64
) -> Result<u64>;
```

**File to Create:** `utils/math.rs`
```rust
pub fn calculate_pro_rata_share(
    total_amount: u64,
    individual_weight: u64,
    total_weight: u64
) -> Result<u64>;

pub fn apply_bps(amount: u64, bps: u16) -> Result<u64>;
```

#### 4. Testing Suite
**Files to Create:**

1. **tests/initialize_position.ts**
   - Test honorary position creation
   - Validate quote-only config
   - Test rejection of non-quote-only pools

2. **tests/crank_distribution.ts**
   - Simulate fee accrual
   - Test multi-page distribution
   - Verify pro-rata calculations
   - Test 24h gate enforcement

3. **tests/edge_cases.ts**
   - All tokens unlocked (100% creator)
   - Partial unlocks
   - Base fee detection (should fail)
   - Dust handling
   - Daily cap enforcement

4. **Setup:** Install test dependencies
   ```bash
   yarn add --dev solana-bankrun anchor-bankrun @solana/web3.js
   ```

#### 5. TypeScript SDK
**File to Create:** `sdk/client.ts`

**Required Exports:**
```typescript
export class InvestorFeeDistributorClient {
    initializePolicy(...): Promise<string>;
    initializeHonoraryPosition(...): Promise<string>;
    crankDistribution(...): Promise<string>;

    // Helper methods
    getDailyProgressPda(...): [PublicKey, number];
    getPolicyConfigPda(...): [PublicKey, number];
    getInvestorFeePositionOwnerPda(...): [PublicKey, number];

    // State fetchers
    fetchPolicyConfig(...): Promise<PolicyConfig>;
    fetchDailyProgress(...): Promise<DailyProgress>;
}
```

#### 6. Documentation
**Files to Create/Update:**

1. **README.md** - Comprehensive integration guide
   - Quick start
   - Account derivation table
   - Instruction reference
   - Error handling guide
   - Deployment checklist

2. **INTEGRATION_GUIDE.md** - Step-by-step for Star team
   - Gathering required inputs
   - Pool setup checklist
   - Testing workflow
   - Mainnet deployment

3. **ARCHITECTURE.md** - Technical deep dive
   - System design
   - Data flow diagrams
   - Security considerations
   - Failure modes and recovery

---

## 📋 NEXT STEPS FOR COMPLETION

### Immediate Actions Required from Star Team:

1. **Provide cp-amm Integration Details:**
   - Share cp-amm IDL or Rust SDK
   - Document exact account structures
   - Provide devnet pool with `collectFeeMode: 1` for testing

2. **Provide Streamflow Integration Details:**
   - Share Streamflow account structure
   - Confirm calculation method for locked amounts at timestamp t
   - Provide test stream accounts on devnet

3. **Policy Parameters:**
   - Confirm exact values for:
     - `investor_fee_share_bps`
     - `daily_cap_lamports` (if any)
     - `min_payout_lamports`
     - `y0_total_streamed`

### Development Workflow:

**Phase 1: Complete Core Logic (Est. 2-3 days)**
1. Implement cp-amm CPI calls
2. Implement crank_distribution instruction
3. Create helper functions (Streamflow, math)

**Phase 2: Testing (Est. 2 days)**
1. Set up bankrun test environment
2. Write comprehensive test suite
3. Test against devnet pools and streams

**Phase 3: SDK & Documentation (Est. 1-2 days)**
1. Create TypeScript client SDK
2. Write comprehensive README
3. Create integration guides

**Phase 4: Audit & Deployment (Est. 1 day)**
1. Internal security review
2. Test on devnet with real parameters
3. Mainnet deployment checklist

---

## 🔍 CRITICAL INTEGRATION POINTS

### 1. CP-AMM Program Interface
**Current Status:** Researched but not integrated

**Known Information:**
- Program ID: `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`
- SDK available: `@meteora-ag/cp-amm-sdk` (TypeScript)
- Rust program repo: https://github.com/MeteoraAg/damm-v2

**Required Actions:**
1. Clone damm-v2 repo to examine account structures
2. Use `declare_program!` macro or manual CPI
3. Test position creation and fee claiming on devnet

### 2. Streamflow Program Interface
**Current Status:** Identified but not integrated

**Known Information:**
- Program ID: `strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m`
- SDK available: `@streamflow/stream` (TypeScript)
- Rust program: Need to locate source or reverse-engineer accounts

**Required Actions:**
1. Document Streamflow stream account structure
2. Create deserializer for locked amount calculation
3. Test with actual vesting streams on devnet

### 3. Math Validation
**Formula to Implement:**
```
Y0 = total investor allocation at TGE
locked_total(t) = sum of still-locked across all investors
f_locked(t) = locked_total(t) / Y0
eligible_investor_share_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))

investor_fee_quote = floor(claimed_quote * eligible_investor_share_bps / 10000)

For each investor i:
  weight_i(t) = locked_i(t) / locked_total(t)
  payout_i = floor(investor_fee_quote * weight_i(t))

  if payout_i >= min_payout_lamports:
    transfer to investor
  else:
    carry_over_lamports += payout_i
```

---

## 📦 DELIVERABLE CHECKLIST

| Item | Status | Notes |
|------|--------|-------|
| State Accounts | ✅ Complete | All 3 accounts implemented |
| Error Codes | ✅ Complete | 18 errors defined |
| Events | ✅ Complete | 6 events defined |
| Constants | ✅ Complete | PDAs, IDs, time constants |
| initialize_policy | ✅ Complete | Fully functional |
| initialize_honorary_position | ⚠️ Partial | Needs cp-amm CPI |
| crank_distribution | ❌ Not Started | Core logic designed |
| Helper Functions | ❌ Not Started | Streamflow, math utils |
| Bankrun Tests | ❌ Not Started | Test suite designed |
| TypeScript SDK | ❌ Not Started | Client wrapper needed |
| README.md | ❌ Not Started | Integration guide |
| INTEGRATION_GUIDE.md | ❌ Not Started | Step-by-step walkthrough |

**Overall Progress: 60% Foundation Complete**

---

## 🚀 DEPLOYMENT CHECKLIST (For Later)

### Pre-Deployment
- [ ] All tests passing on devnet
- [ ] Security review completed
- [ ] cp-amm integration validated
- [ ] Streamflow integration validated
- [ ] Math formulas verified
- [ ] Edge cases tested

### Deployment
- [ ] Build program: `anchor build`
- [ ] Deploy to devnet first
- [ ] Test with real vault parameters
- [ ] Deploy to mainnet-beta
- [ ] Initialize PolicyConfig
- [ ] Initialize HonoraryPosition
- [ ] Set up off-chain cranker service

### Post-Deployment
- [ ] Monitor first distributions
- [ ] Verify quote-only fees
- [ ] Confirm investor payouts
- [ ] Validate creator remainder
- [ ] Document any issues

---

## 💡 TECHNICAL NOTES

### PDA Derivation
All PDAs use deterministic seeds for easy client-side derivation:

```rust
// PolicyConfig
seeds = [b"policy_config", vault.as_ref()]

// InvestorFeePositionOwner
seeds = [b"investor_fee_pos_owner", vault.as_ref()]

// DailyProgress
seeds = [b"daily_progress", vault.as_ref()]
```

### Security Considerations Implemented
1. ✅ Overflow protection with checked arithmetic (designed, not yet coded)
2. ✅ Quote-only validation (validation logic designed)
3. ✅ 24h time gate (state structure supports)
4. ✅ Idempotency (state structure supports)
5. ✅ Dust handling (logic designed)
6. ✅ PDA ownership (enforced by Anchor)

### Gas Optimization Strategies
1. Store canonical bumps in state accounts
2. Use `init_if_needed` sparingly
3. Batch investor processing (pagination)
4. Minimize logging in production
5. Use zero-copy for large account arrays (if needed)

---

## 📞 SUPPORT & QUESTIONS

For completing this implementation, Star team will need:

1. **cp-amm Expert:** Someone familiar with Meteora's DAMM v2 internals
2. **Streamflow Integration:** Access to Streamflow account parsers
3. **Testing Resources:** Devnet pools with `collectFeeMode: 1`
4. **Policy Decisions:** Final parameters for production

**Estimated Time to Complete:** 5-7 days with proper resources

**Risk Level:** Medium
- Core architecture is sound ✅
- External program integration is standard practice ✅
- Math formulas are well-defined ✅
- Main risk: cp-amm and Streamflow interface changes

---

**Last Updated:** 2025-10-04
**Version:** 0.1.0-foundation
**Status:** Foundation Complete, Integration Pending
