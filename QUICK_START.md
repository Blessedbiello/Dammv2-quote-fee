# Quick Start Guide

Get up and running with the Investor Fee Distributor in 5 minutes.

## 📁 What's in This Repository

```
investor-fee-distributor/
├── README.md                      ← Start here for full documentation
├── IMPLEMENTATION_STATUS.md       ← See what's done and what remains
├── DELIVERY_SUMMARY.md            ← Executive summary of delivery
├── QUICK_START.md                 ← You are here!
└── programs/investor-fee-distributor/
    └── src/
        ├── state/                 ← 3 state accounts (100% complete)
        ├── instructions/          ← 2/3 instructions (66% complete)
        ├── error.rs               ← 18 error codes (100% complete)
        └── events.rs              ← 6 events (100% complete)
```

## 🎯 Current Status

**✅ 60% Complete - Foundation Ready**
- All state accounts implemented
- Policy initialization works
- Honorary position structure complete (CPI integration pending)
- Comprehensive documentation

**⚠️ 40% Remaining - Integration Needed**
- CP-AMM CPI calls (need Meteora interface)
- Crank distribution instruction (logic designed, not coded)
- Test suite (plan ready)
- TypeScript SDK (straightforward generation)

## 🚀 How to Use This Delivery

### Option 1: Review & Understand (5 minutes)

1. Read [README.md](README.md) - See what the program does
2. Read [DELIVERY_SUMMARY.md](DELIVERY_SUMMARY.md) - Understand what's delivered
3. Check `programs/investor-fee-distributor/src/` - Review the code

### Option 2: Complete the Implementation (5-7 days)

1. Read [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Get the roadmap
2. Follow "Next Steps for Completion" section
3. Search for `TODO` comments in code - These mark integration points
4. Reference README.md for examples and formulas

### Option 3: Test What Exists (1 hour)

```bash
# Install dependencies
cd investor-fee-distributor
yarn install

# Build (requires Rust 1.80+)
anchor build

# Check state accounts
cat programs/investor-fee-distributor/src/state/*.rs

# Check implemented instructions
cat programs/investor-fee-distributor/src/instructions/*.rs
```

## 📖 Documentation Guide

| Document | Read Time | Purpose |
|----------|-----------|---------|
| **QUICK_START.md** | 5 min | You are here - orientation |
| **README.md** | 20 min | Complete usage guide + examples |
| **IMPLEMENTATION_STATUS.md** | 15 min | What's done, what remains, how to finish |
| **DELIVERY_SUMMARY.md** | 10 min | Executive summary for stakeholders |

## 🔑 Key Concepts

### The Problem This Solves
Star wants to distribute DAMM v2 trading fee revenue to investors based on how many tokens they still have locked in Streamflow vesting contracts.

### The Solution (3 Parts)

1. **Honorary Position** - Program-owned LP position in DAMM v2 pool that only earns fees in quote token
2. **24h Distribution Crank** - Anyone can call once per day to distribute accumulated fees
3. **Pro-Rata to Locked** - Investors get share proportional to their still-locked amounts

### The Architecture (3 State Accounts)

```
PolicyConfig          ← Stores: investor_fee_share_bps, Y0, creator ATA, caps
      ↓
InvestorFeePositionOwner  ← Owns the honorary DAMM v2 position
      ↓
DailyProgress        ← Tracks: current day, pages processed, amounts distributed
```

### The Flow (Daily Cycle)

```
1. Trading happens → Fees accumulate in honorary position (quote token only)
2. Crank called → Claims fees, reads Streamflow locked amounts
3. Distribution → Pays investors pro-rata, sends remainder to creator
4. Repeat next day
```

## 🎓 Understanding the Code

### Start Here
1. **State Accounts:** `programs/investor-fee-distributor/src/state/`
   - Read `policy_config.rs` first - This holds all the policy parameters
   - Then `daily_progress.rs` - This tracks the 24h window state
   - Then `investor_fee_position_owner.rs` - This owns the LP position

2. **Instructions:** `programs/investor-fee-distributor/src/instructions/`
   - `initialize_policy.rs` - ✅ Complete (creates PolicyConfig)
   - `initialize_honorary_position.rs` - ⚠️ Partial (structure done, CPI pending)
   - `crank_distribution.rs` - ❌ Not created yet (but fully designed in docs)

3. **Supporting Files:**
   - `error.rs` - All possible errors
   - `events.rs` - All emitted events
   - `constants.rs` - PDA seeds and program IDs

### Key Design Patterns Used

**PDA Architecture:**
```rust
// All PDAs are deterministically derived from vault pubkey
PolicyConfig: seeds = [b"policy_config", vault]
InvestorFeePositionOwner: seeds = [b"investor_fee_pos_owner", vault]
DailyProgress: seeds = [b"daily_progress", vault]
```

**24-Hour Window:**
```rust
day_id = unix_timestamp / 86400  // Days since epoch
window_start = day_id * 86400    // Start of current 24h window
```

**Pro-Rata Distribution:**
```rust
f_locked = locked_total / Y0
investor_share_bps = min(policy_bps, floor(f_locked * 10000))
investor_fee = floor(total_fees * investor_share_bps / 10000)

for each investor:
    weight = investor_locked / locked_total
    payout = floor(investor_fee * weight)
```

## 🔧 What You Need to Complete

### Required Information

1. **From Meteora (for cp-amm integration):**
   - CP-AMM program IDL or Rust SDK
   - How to create a position via CPI
   - How to claim fees via CPI
   - Devnet pool with `collectFeeMode: 1`

2. **From Streamflow (for vesting integration):**
   - How to parse stream account data
   - How to calculate locked amount at timestamp t
   - Devnet vesting streams for testing

3. **From Star Team (for deployment):**
   - Final policy parameters:
     - `investor_fee_share_bps` (e.g., 7000 = 70%)
     - `Y0` value (total investor allocation at TGE)
     - `daily_cap_lamports` (optional)
     - `min_payout_lamports` (dust threshold)
     - Creator quote token ATA
   - List of investor Streamflow streams + ATAs
   - Vault identifier (Pubkey)

### Required Skills for Completion

- **Solana/Anchor Development:** Moderate (CPI calls, account parsing)
- **Testing:** Moderate (Bankrun test setup)
- **TypeScript:** Basic (SDK wrapper generation)

**Estimated Time:** 5-7 days for experienced Solana developer

## 🏃 Next Steps

### For Project Managers

1. **Review** [DELIVERY_SUMMARY.md](DELIVERY_SUMMARY.md) for executive overview
2. **Decide** on completion approach:
   - Complete internally with roadmap?
   - Get external program interfaces and collaborate?
   - Phase deployment (foundation first, then integration)?

### For Developers

1. **Read** [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) in detail
2. **Study** the TODO comments in:
   - `instructions/initialize_honorary_position.rs`
   - (Future) `instructions/crank_distribution.rs`
3. **Gather** required resources (cp-amm SDK, Streamflow docs)
4. **Follow** the phase-by-phase completion guide

### For Integration Engineers

1. **Check** [README.md](README.md) "External Programs" section
2. **Research** cp-amm repository: https://github.com/MeteoraAg/damm-v2
3. **Test** with devnet pools before mainnet
4. **Validate** quote-only fee collection with small amounts

## 📊 Completion Checklist

Use this to track remaining work:

### Core Implementation
- [ ] Add cp-amm dependency to Cargo.toml
- [ ] Complete `initialize_honorary_position` CPI calls
- [ ] Create `instructions/crank_distribution.rs`
- [ ] Implement Streamflow account parsing
- [ ] Add helper functions (math, utils)

### Testing
- [ ] Install test dependencies (bankrun)
- [ ] Write position initialization tests
- [ ] Write crank distribution tests
- [ ] Write edge case tests
- [ ] Test on devnet with real pool

### SDK & Docs
- [ ] Generate TypeScript SDK client
- [ ] Test SDK with examples
- [ ] Create deployment guide
- [ ] Update README with final examples

### Deployment
- [ ] Test on devnet end-to-end
- [ ] Security review
- [ ] Deploy to mainnet
- [ ] Initialize production accounts
- [ ] Set up crank service

## ❓ Common Questions

**Q: Can I deploy this to devnet as-is?**
A: No, the `initialize_honorary_position` instruction needs cp-amm CPI integration first.

**Q: What's the main blocker to completion?**
A: CP-AMM program interface details. Once you have the SDK or IDL, integration is straightforward.

**Q: How hard is the remaining work?**
A: Moderate difficulty. If you have a working cp-amm CPI example, you can complete in 5-7 days.

**Q: Is the architecture sound?**
A: Yes. State accounts, math formulas, and protocol rules are production-ready. Only external integrations remain.

**Q: Can we use this in production when complete?**
A: Yes, but conduct a security audit first. The foundation follows Anchor best practices.

**Q: Where's the test suite?**
A: Not yet written, but the test plan is fully documented in IMPLEMENTATION_STATUS.md.

## 🆘 Getting Help

### Code Questions
- Check TODO comments in code
- Read IMPLEMENTATION_STATUS.md "Critical Integration Points"
- Review README.md examples

### Math Questions
- See README.md "Fee Distribution Formula"
- Check `state/daily_progress.rs` helper methods

### Integration Questions
- DAMM v2: https://docs.meteora.ag/overview/products/damm-v2
- Streamflow: https://streamflow.finance/
- Anchor: https://www.anchor-lang.com/docs

### Architecture Questions
- Read DELIVERY_SUMMARY.md "Technical Highlights"
- Review `state/` account structures
- Check README.md "Architecture" section

## ✅ Success Criteria

You'll know you're done when:

1. **Build Success:** `anchor build` completes without errors
2. **Tests Pass:** All Bankrun tests passing on devnet
3. **Quote-Only Verified:** Base token balance always zero after claims
4. **Distribution Works:** Investors receive correct pro-rata amounts
5. **Creator Gets Remainder:** Leftover fees route to creator correctly
6. **24h Gate Works:** Cannot crank more than once per day
7. **Events Emit:** All 6 events logging correctly
8. **SDK Functions:** TypeScript client works with examples

---

**Ready to start?** Read [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for the detailed roadmap!

**Need context?** Read [README.md](README.md) for the complete guide!

**Sharing with stakeholders?** Send them [DELIVERY_SUMMARY.md](DELIVERY_SUMMARY.md)!

---

**Version:** 0.1.0-foundation
**Last Updated:** 2025-10-04
**Status:** Foundation Complete (60%), Integration Pending (40%)
