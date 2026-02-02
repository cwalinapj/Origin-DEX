# Origin DEX — Whitepaper v0.1
**Status:** Public draft (v0.1)  
**Network:** Solana  
**Launch posture:** V1 ships what must work. Everything else is roadmap.

Optional LP NFT Staking + Accounting Model (V1)

Two modes
	•	Unstaked (default): user keeps custody of the LP NFT/position. UI computes “value now” on-demand (same idea as Orca/Raydium: read position + pool/oracle state → compute). No on-chain writes needed.
	•	Staked (optional): user deposits the LP NFT/position into an escrow vault controlled by our program. This mode enables immutable deposit/withdraw receipts + vault-level accounting for reviewers.

What we record on-chain (minimal + immutable)

Per stake (StakeRecord)
	•	staker_pubkey
	•	position_pubkey (or NFT mint / position account)
	•	deposit_slot/time
	•	function_params_hash (or full params if small)
	•	deposit_value_quote (e.g., USDC microunits or SOL lamports)
	•	(optional) deposit_amounts_x/y

Per unstake (UnstakeRecord)
	•	withdraw_slot/time
	•	withdraw_value_quote
	•	(optional) withdraw_amounts_x/y
	•	Duration is derived: withdraw_time - deposit_time

Per epoch (EpochVaultSummary) — ONE write per epoch
	•	epoch_number
	•	snapshot_slot/time
	•	vault_total_value_quote (sum of staked vault positions only)
	•	position_count
	•	(optional) safe aggregates like median, p95, top10_share

✅ We do not store per-position values each epoch (avoids making an easy indexed leaderboard + keeps fees tiny).

Off-chain jobs (keeper)
	•	Each epoch, a keeper:
	1.	loads all currently staked positions
	2.	computes each position value from public on-chain state (amounts/liquidity/fees) + price source. For V1, the price source is the pool spot mid price at the snapshot slot recorded in EpochVaultSummary. The spot mid price is the midpoint price implied by the active bin.
	3.	sums to vault_total_value_quote
	4.	submits one tx: EpochVaultSummary(...)
	•	UI can still compute per-position value “now” on demand for any wallet/position.

UI expectations
	•	My Positions: unstaked positions (value live), button: “Stake to Vault” (optional)
	•	My Vault: staked positions, shows deposit/withdraw receipts + duration
	•	Vault Accounting: chart/table from EpochVaultSummary, plus Print/Export receipts

⸻


Curve Presets (GUI) + Param API (Origin OS)

The DEX exposes a single deterministic distribution engine that converts deposits into per-bin liquidity allocations. Humans use picture-based presets in the GUI (V/U/Walls/Bowl/etc). The Origin OS / LAM uses an API param object that maps to the same preset engine.

Core idea
	1.	Convert bin distance to normalized input: x = (d - 1) / (maxD - 1) where d = |binId - activeId| and d=0 (active bin) is forbidden.
	2.	Evaluate a normalized curve f(x) ∈ [0,1].
	3.	Convert to weight: weight(d) = base + amp * f(x), with weight(0)=0.
	4.	Normalize weights into exact token amounts, using a deterministic remainder rule.

Supported curve families

Each family produces f(x) in [0,1]:
	•	power: f(x)=x^p (p controls “U-ness”; larger p = heavier tails)
	•	exp: f(x)=expm1(kx)/expm1(k) (k controls “wall strength”)
	•	log: f(x)=log1p(ax)/log1p(a) (a controls “center heaviness”)
	•	sigmoid: normalized logistic curve (k=steepness, m=midpoint)

Presets

GUI presets are just named parameter sets (picture cards). Example mapping:
	•	“U / Bowl” → { type:"power", p:2.5, base:1, amp:120 }
	•	“Hard Walls” → { type:"exp", k:10, base:1, amp:200 }
	•	“Soft Bowl” → { type:"log", a:15, base:1, amp:80 }
	•	“Delayed Wall” → { type:"sigmoid", k:12, m:0.65, base:1, amp:200 }

API contract (Origin OS)

The LAM calls the distribution endpoint with params; backend returns xYAmountDistribution:


root1@P-Mk-Pro meteora-devnet % node seed_liquidity.cjs
User: 14ToUhE8e88JLu8Jhdvn9iTdqwY4yW9CybQP55BgtK7q
Pair: BL9k1nsrBxYtYQMxHy6HdcbLLjHLHShrQvrr2DuWaRXZ
tokenX mint: 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
tokenY mint: So11111111111111111111111111111111111111112
activeId: 703
Detected ordering: X=USDC, Y=wSOL
Creating position: EgBuD1vGp5GnuhMe6jQkCgfWP29E6s7xrp1LKockRQtB
Range: 693 → 713
✅ LP position created + liquidity added
Signature: 4voRDD1jirycuTmdYH1Zfmfh2m49cqPEBMg9xHhZSync2s7JdXd6TbacCfSTzRhAkvY8kvGP883qXy2dmxYjGWDk
Position: EgBuD1vGp5GnuhMe6jQkCgfWP29E6s7xrp1LKockRQtB

---

## 0) Read This First
This whitepaper is published alongside the initial Origin DEX deployment. It contains:
- a **precise description of what V1 ships**
- a **roadmap of what we want to build later**

**V1 is intentionally narrow.** We will not ship optional complexity (token, incentives, margin, bridging, cross-chain routing, governance) until the base DEX is proven safe and reliable.

Nothing in this document should be interpreted as an entitlement to future distributions, rewards, or token allocations.

## Build (from this .md)
Generate an HTML artifact of this whitepaper:

```bash
make build
```

Output is written to `build/index.html`. Use `make clean` to remove it.

---

## V1 (Devnet) Quickstart

**Network:** Solana Devnet  
**V1 scope:** single pool only (wSOL / Circle devnet USDC) + function-based LP allocation

### Circle Devnet USDC
Mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`

### 1) Create a devnet wallet + fund it (macOS)
```bash
chmod +x scripts/setup_devnet_wallet_macos_v2.sh
TARGET_SOL=2 AIRDROP_CHUNK=1 TARGET_USDC=40 ./scripts/setup_devnet_wallet_macos_v2.sh
```
`chmod` only needs to be run once per machine. This script is intended for macOS; other platforms may need a similar flow.

---

## 1) Summary
Origin DEX is a standalone Solana DEX based on **bin liquidity** and one differentiator:

> **Function-based, two-sided bin allocation at LP deposit time.**

Liquidity providers choose a **left** and **right** parameterized function that controls how their deposit is distributed across bins around the active price. The protocol converts the function output into a deterministic per-bin allocation.

The functions are used **only during allocation**. After allocation:
- liquidity does not reshape itself
- changing the distribution requires opening a new position or withdrawing/redepositing

V1 provides:
- pool creation
- swaps with bounded bin traversal
- adding/removing liquidity
- claiming fees
- function-based LP allocation (two-sided functions)

---

## 2) Motivation
Liquidity placement is one of the most important levers in AMMs. Many designs either:
- restrict placement to a few “preset” distributions, or
- expose too much expressiveness in ways that are difficult to simulate, audit, and keep safe.

Origin DEX aims for a middle path:
- **high expressiveness** (infinite shapes via parameterization)
- **bounded complexity** (hard caps on bins touched and bins traversed)
- **deterministic behavior** (audit-friendly and simulation-friendly)

---

## 3) Acknowledgements & Prior Art
Bin-style liquidity AMMs and discretized liquidity designs have been explored across DeFi.

On Solana specifically, Origin DEX acknowledges the influence of **Meteora’s DLMM** as prior art that helped validate bin-based liquidity as a practical market structure on Solana.
V1 implementation begins by forking DLMM-style components (bin-based AMM structure and fee accounting) and then extending allocation to parameterized two-sided functions.

Origin DEX is **not affiliated with Meteora** and does not claim endorsement by Meteora. References to third-party protocols and designs are for educational context and interoperability.

---

## 4) What Ships in V1
### 4.1 V1 is a standalone DEX (Solana)
V1 ships a complete bin-based spot DEX:
- Create pool
- Add liquidity
- Swap
- Remove liquidity
- Claim fees

### 4.2 V1 differentiator: function-based LP allocation
LPs choose two functions (left / right) plus parameters and ranges.

**Key rule: allocation-time only**
- the function is applied once to compute the allocation distribution
- the protocol writes liquidity into bins deterministically
- after this, the distribution is immutable unless the LP opens a new position or withdraws/redeposits

### 4.3 V1 is intentionally minimal
V1 does **not** ship:
- a token launch or emissions
- governance/DAO
- incentives/rewards programs
- margin, leverage, shorting
- cross-chain routing/bridging
- managed vaults or automated rebalancing

---

## 5) Core Concepts
### 5.1 Bins
Price space is discretized into bins. Each bin holds:
- reserves of token A and token B
- fee growth accounting for LPs
- metadata needed to traverse bins efficiently

Swaps move through bins to fill the requested size.

### 5.2 wSOL (Wrapped SOL)
wSOL is the SPL-token representation of SOL. It exists so SOL can be handled by token programs and AMMs that operate on SPL tokens. Native SOL remains in lamports; wrapping/unwrapping uses the token program’s native mint mechanics.

### 5.3 Positions
An LP position records:
- owner
- the set/range of bins touched
- per-bin shares or accounting references (implementation detail)
- fee checkpoints for accurate claiming
- a strategy record (strategy id + params hash) for reproducibility

### 5.4 Fees
Swaps pay fees which accrue to LPs.
Fee accounting must remain correct for:
- multi-bin swaps
- multiple LPs per bin
- partial fills
- repeated fee claims

---

## 6) Function-Based Allocation Design
### 6.1 Two-sided allocation model
Let:
- `b0` be the center bin (typically the active bin at deposit time)
- `d` be the absolute distance from center in bins, `d = |binId - b0|`
- left side bins have `binId < b0` (indexed by `d = 1..NL` as `b0 - d`), right side bins have `binId > b0` (indexed by `d = 1..NR` as `b0 + d`)

**V1 rule (no active-bin deposits):**
- Active bin (`d = 0`) deposits are forbidden; allocation begins at `d = 1` on the left side (`d = 1..NL`) and right side (`d = 1..NR`).

LP defines two parameterized functions:
- left side: `wL(d) = fL(d; θL)` for `d = 1..NL`
- right side: `wR(d) = fR(d; θR)` for `d = 1..NR`

The protocol:
1) computes per-bin weights from the functions  
2) normalizes weights across all bins so total weight sums to 1  
3) converts weights into exact token amounts  
4) applies deterministic rounding: floor each amount, then distribute the remainder to bins with the largest fractional weights.  
   Bin order is by increasing distance from center within each side.  
   Tie-breaker: when fractional weights tie, distribute remainder one unit at a time to tied left bins starting from the closest bin to center (`d = 1..NL`), then to tied right bins starting from the closest bin to center (`d = 1..NR`).  
5) writes liquidity to bins and updates the position

**Allocation-time only:** liquidity never reshapes after deposit. Changing distribution requires a new position or withdraw+redeposit.

### 6.2 Why parameterized functions
Parameterized functions allow:
- simulation across ranges of parameters
- predictable behavior and bounded complexity
- a small set of safe “function families” that still feels unlimited to users

### 6.3 V1 function families (initial)
V1 aligns its initial presets with the **DIM profile** shapes seen in Meteora's DLMM:
- **Spot (balanced):** flat weights across bins (`meteora_spot`)
- **Curve (bell):** center-heavy Gaussian-style weights (`meteora_curve`)
- **BidAsk (U-curve):** edge-heavy distribution (`meteora_bidask`)

Additional parameterized families remain available:
- **Exponential decay:** `w(d) = r^d`
- **Power decay:** `w(d) = 1 / (d + c)^p`
- **Wall + decay:** constant for `d <= k`, then decay

These families can express:
- tight vs wide liquidity
- skewed distributions
- wall+curve behaviors
- asymmetric left/right placement

---

## 7) V1 Safety Rails (Bounded Complexity)
These are protocol rules designed to keep execution safe and predictable.

### 7.1 Bounded deposits
- `NL + NR <= MAX_BINS_PER_DEPOSIT`
- optional minimum per-bin threshold to prevent “dust spraying”
- deterministic rounding and remainder placement (defined in 6.1)

### 7.2 Bounded swaps
- swaps are capped by `MAX_BINS_PER_SWAP`
- if a swap would exceed this bound, it fails fast (V1)

### 7.3 No reshaping rule
- allocations do not change in-place after deposit
- modifying a distribution requires opening a new position or withdrawing/redepositing

This keeps accounting simple and prevents hidden state changes.

---

## 8) “Open LP Positions Easily” (Developer + User Simplicity)
V1 is designed so anyone can open LP positions with minimal friction:
- a basic UI flow with presets and parameter sliders
- a small SDK interface to:
  - preview bins touched
  - generate deterministic allocation vectors
  - build deposit instructions

The goal is to make “opening LPs” a simple action—while preserving safety bounds.

---

## 9) Long-Term Vision (Very Wall Street)
Origin DEX is intended to evolve into market infrastructure that can support workflows traditionally associated with Wall Street: deeper market structures, sophisticated instruments, and eventually options-style exposure.

This vision is aspirational and will be pursued in stages. Each stage must be safe, auditable, and operationally supportable.

**V1 is spot-only.** Derivatives, margin, and leverage are explicitly out of scope for V1.

---

## 10) Future Modules (Roadmap, Not Shipped in V1)
This section describes things we want to build later. It is not a commitment to timeline, implementation, or final design.

### 10.1 Governance / DAO (future)
A governance system may be introduced to manage:
- protocol parameters (fees, caps, whitelists)
- strategy registry updates
- emergency procedures

### 10.2 Native token (future, optional)
A native token may be introduced to coordinate governance and ecosystem participation (contributors, community ops, aligned voters).

V1 does not include any token distribution mechanism.

### 10.3 Margin & shorting against LP positions (future, extremely conservative — V1.5+)
A later release may introduce margin primitives where LP positions can be used as collateral.

If shipped, it will be conservative by design:
- only whitelisted high-liquidity assets
- strict borrow caps
- liquidation logic driven by repay feasibility

**Depth-driven liquidation principle (planned):**
If market depth for a borrowed asset falls, risk limits tighten. If debt becomes too large relative to repay depth (within bounded slippage), forced deleveraging/liquidation can occur even if spot price is unchanged.

This module is not included in V1.

### 10.4 Emergency backstop (future)
If future modules introduce credit risk (e.g., margin lending), the protocol may use layered backstops:
- insurance fund first
- capped emergency mechanisms later

Any such system would be published with explicit constraints and audited before activation.

### 10.5 Managed strategies / vaults (future)
A managed layer may be added for users who want:
- automated placement
- rebalancing
- compounding

Not required for V1.

### 10.6 Cross-chain routing / bridging (future)
Cross-chain execution introduces additional risk and complexity.
Not required for V1.

---

## 11) Risk Disclosure
Origin DEX is experimental software. Risks include (non-exhaustive):
- smart contract vulnerabilities
- economic attacks and market manipulation
- unexpected volatility and liquidity shocks
- LP losses including impermanent loss
- chain/runtime risks inherent to Solana

V1 intentionally limits features to reduce risk and simplify verification.

---

## 12) Appendix — V1 Implementation Checklist
This appendix is a practical “ship list” for V1.

### Protocol (V1)
- [ ] Pool create + initialize
- [ ] Add liquidity (single-bin baseline)
- [ ] Add liquidity (two-sided function-based allocation)
- [ ] Swap (bounded bins traversed)
- [ ] Remove liquidity
- [ ] Claim fees
- [ ] Events for: pool create, add/remove, swap, claim

### UI (V1)
- [ ] Swap screen
- [ ] LP screen: create pool, add, remove, claim
- [ ] Presets: Tight / Wide / Wall+Curve
- [ ] Advanced params panel (still functions)
- [ ] “Bins touched” + complexity warnings

### SDK (V1)
- [ ] Allocation preview: deterministic output
- [ ] Complexity estimate: bins touched + warnings
- [ ] Instruction builder helpers

---

> End of Origin DEX — Whitepaper v0.1
