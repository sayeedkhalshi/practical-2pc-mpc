Use this as your “checklist + recipe” to implement the Paillier + threshold setup and to plug it into the rest of the 2PC-MPC system.

# Production-ready ordered steps (high level)

1. **Entropy & CSPRNG bootstrap**
2. **Prime generation + primality testing** (MR + BPSW)
3. **Compute Paillier parameters (N, N², g, λ, μ)**
4. **Commitments & VSS (Pedersen / Feldman)**
5. **Zero-knowledge proofs (Schnorr-style / Fiat–Shamir NIZK)**
6. **Dealer vs DKG decision** (use DKG for trustless; dealer OK for dev)
7. **If DKG: distributed RSA/Paillier DKG (e.g., Tiresias / vetted lib)**
8. **Share verification / complaint handling**
9. **Publish Paillier public key, register epoch**
10. **Secure storage (HSM/KMS) of shares; attestation**
11. **Implement TDec (decryption shares) + ZK proofs for TDec correctness**
12. **Test vectors, integration, and monitoring (primality logs, audit trail)**
13. **Operational: epoch rotate (reshare) / emergency rotate procedure**

Below I expand each step and show which technology is used and why.

---

## 1 — Entropy & CSPRNG (very first)

**What:** seed secure CSPRNG and all randomness sources.
**Why:** prime generation, polynomial coefficients, ZK randomness, nonces — MUST be cryptographically secure.
**How / tech:** `OsRng` / system CSPRNG; if available, hardware TRNG or HSM RNG. Use deterministic DRBG seeded from high-entropy.
**Output:** RNG instance used by all subsequent steps.
**Notes:** log RNG health; do NOT reuse randomness across roles.

---

## 2 — Prime generation + primality testing

**What:** generate two large primes `p` and `q` to build Paillier modulus `N = p * q`.
**Why:** Paillier security relies on hardness of factoring `N`.
**How / tech:**

-   Generate candidate random odd `p` and `q` with requested bit-length (recommendation below).
-   **Miller–Rabin (MR)**: run **64 MR rounds** (for 4096-bit primes).
-   **BPSW** (Baillie–PSW) as an extra (if library supports) for additional confidence.
    **Parameters (recommended):**
-   `N` size: **4096 bits** (i.e., `p` ≈ 2048 bits, `q` ≈ 2048 bits) — production modern security.
-   MR rounds: **64** for 4096-bit.
    **Output:** confirmed primes `p`, `q`.
    **Notes:** keep prime generation inside HSM if possible (avoid raw p,q exposure); discard intermediate candidate states.

---

## 3 — Compute Paillier parameters

**What:** from `p`, `q` compute `N`, `N²`, `g` (commonly `N+1`), compute `λ = lcm(p-1,q-1)` and `μ = L(g^λ mod N²)^{-1} mod N`.
**Why:** yields Paillier `(PK, SK)` where PK = `(N,g)` and SK = `(λ, μ)` (or equivalent).
**Output:** `(PK, SK)` (SK only insight to be thresholdized).
**Notes:** never expose λ or μ in plaintext to any single non-trusted machine in the DKG path.

---

## 4 — Commitments & VSS (Pedersen / Feldman)

**What:** commit to secret material and split secrets with verifiable secret sharing.
**Why:** share security, binding/hiding commitment and verifiability so invalid shares are detected.
**How / tech:**

-   Use **Pedersen commitments** for hiding+binding (group with prime order; use curve or separate prime-order group).
-   Use **Pedersen VSS** or **Feldman VSS** to produce shares for each validator: polynomial `a(x)` with `a0 = secret` and `a_k` random; send shares `s_i = a(i)` to each party. Publish commitment vector `C_k = g^{a_k} h^{r_k}`.
    **Where used:** right after computing `λ` (if dealer) or inside each DKG participant when combining partial secrets.
    **Output:** per-validator share + global commitments (broadcast).
    **Notes:** commitments must be published on an authenticated channel (signed and optionally on-chain / epoch state).

---

## 5 — Zero-knowledge proofs (Schnorr-style / Fiat–Shamir NIZK)

**What:** prove correctness of operations without revealing secrets.
**Why:** prevents malicious participants from distributing incorrect shares or forging decryption shares.
**How / tech:**

-   **Schnorr-style proofs** for discrete-log relations (e.g., proof of knowledge of `x` s.t. `X = x*G`).
-   **Fiat–Shamir** to make non-interactive ZK (NIZK) in the random-oracle model.
-   For range proofs or complex relations, use **Bulletproofs** if you need no trusted setup; if you need aggregation, ensure the chosen proof supports it.
    **Where used:** verify share correctness vs commitments, proof someone computed TDec correctly, proof for Eval correctness in Paillier evaluations.
    **Output:** NIZK transcripts attached to messages.
    **Notes:** choose curves & hash functions with domain separation; store proofs for audits.

---

## 6 — Tiresias (or equivalent threshold Paillier lib)

**What:** an existing vetted implementation (like Tiresias in dWallet ecosystem) that implements threshold Paillier primitives: DKG, TDec, Rec, proofs.
**Why:** threshold Paillier & distributed RSA DKG is hard to implement correctly — prefer a tested lib.
**How / tech:** integrate the Tiresias crate or other audited implementation for:

-   DKG (distributed RSA/Paillier generation) or utilities for sharing SK,
-   TDec/Rec routines,
-   ZK proof helpers.
    **Where used:** if you want full DKG and secure threshold Paillier operations.
    **Notes:** if using Tiresias, follow its API and compliance patterns; do not re-implement from scratch unless you are crypto experts.

---

## 7 — DKG decision & execution

**What:** distributed generation of the Paillier secret key shares (no single party learns the private key).
**Why:** eliminates single trusted dealer and prevents any one node from decrypting alone.
**Two modes:**

-   **Dealer-based + VSS** (simpler): Dealer generates SK then uses VSS to distribute shares; use only if dealer is trusted for setup.
-   **Full DKG** (recommended for decentralized deployment): everyone contributes randomness to produce a shared Paillier PK and local shares of SK; requires RSA modulus DKG and additional ZK proofs.
    **Order if DKG chosen:** agreement → commitments → encrypted share exchange → verify → complaint resolution → combine → publish PK.
    **Where used:** run at epoch bootstrap (and when rotating keys).
    **Notes:** use Tiresias or other vetted DKG code; DKG for RSA/Paillier is complex (distributed prime generation or safe multiparty composing) — don't DIY in prod.

---

## 8 — Share verification & complaint handling

**What:** after shares are distributed, validators verify shares using commitments; if mismatch, run complaint resolution.
**Why:** detection of malicious shares, prevents a corrupted party from providing invalid data.
**How:** parties broadcast complaints; accused party reveals polynomial coefficients (or other evidence) or replaced; consensus on malicious set leads to abort or exclusion.
**Output:** validated set of shares or list of malicious nodes to stop.
**Notes:** ensure timeouts, authenticated messages, and logging.

---

## 9 — Publish Paillier public key & register epoch

**What:** record PK (N,g) and epoch metadata (epoch id, validator set, threshold t) in network state (optionally on-chain).
**Why:** all protocol participants need to use the same PK for encryptions and evaluation; epoch ensures rotation semantics.
**How:** broadcast signed PK object or write to a chain state with validators’ signatures.
**Output:** canonical PK for the epoch.

---

## 10 — Secure storage of shares (HSM / KMS)

**What:** each validator stores its share (`ski`) in an HSM/KMS-backed storage; HSM performs sensitive operations (TDec) inside a secure boundary.
**Why:** prevents share leakage from node compromise.
**How / tech:** HSMs, cloud KMS with hardware keys, or enclave sealing with attestation. Must support modular exponentiation / pow_mod inside device.
**Notes:** require attestation proof and auditing on startup and key use.

---

## 11 — Implement TDec (decryption share generation) + ZK correctness proofs

**What:** for decrypting a ciphertext, each validator runs `TDec(ski, ct)` to produce decryption share plus a proof that the share was computed correctly. One or more designated aggregator(s) combine shares via `Rec` to get plaintext.
**Why:** threshold decryption operation without exposing SK.
**How / tech:** follow Tiresias/Tiered Paillier TDec scheme; attach ZK proof that `d_i` is computed correctly w.r.t. validator’s committed share. Use single aggregator optionally for performance (with verifiable announcement).
**Notes:** implement fast aggregation and verify proofs before combining. Use amortized design (designate different aggregators per ciphertext to spread load).

---

## 12 — Testing, logging, and audit

**What:** tests for MR/BPSW, share split/reconstruct, DKG happy & adversarial cases, TDec fail scenarios, end-to-end signature correctness.
**Why:** ensure correctness and detect regressions / attacks.
**How:** unit tests, multi-node integration tests, fuzz, replay/protocol ABI tests. Keep audit logs for: prime generation, MR rounds, proof verification results, complaint outcomes.
**Notes:** do not log secrets.

---

## 13 — Operational: epoch rotate / resharing / emergency rotation

**What:** schedule resharing at epoch boundary to refresh shares and mitigate leaked share exposure. Optionally re-run full DKG if you need new modulus or new validator set membership.
**Why:** forward security and handling validator churn.
**How:** resharing protocol that produces fresh shares of same secret (or DKG to create new secret if requested). Use batched/parallel resharing to scale. Maintain public epoch state and orchestrator.
**Notes:** design emergency rotation path if threshold is exceeded.

---

# Compact mapping to your original list

You gave numbered items — here’s exactly where each belongs in the ordered steps:

1. **We get two primes using random generator, where random numbers are not really random.**
   → Step 1 (CSPRNG) + Step 2 (prime generation). Use CSPRNG inside HSM.

1.1 **CSPRNG**
→ Step 1. MUST be seeded from OS/HSM RNG; provide health checks.

2. **We use primality test? MR**
   → Step 2. Use **Miller–Rabin**, ~64 rounds for 4096-bit primes.

3. **We use BPSW**
   → Step 2 as extra sanity check. Use MR + BPSW if available.

4. **We use zk proof Schnorr style?**
   → Step 5. Use Schnorr-style NIZK for PoK (proof of knowledge) and for TDec proofs, and Fiat–Shamir to make them non-interactive.

5. **We use commitment and VSS and padding**
   → Step 4. Use Pedersen/Feldman VSS, commitments published for share verification. Padding: ensure Paillier encodings and evaluated results have correct upper bounds and masking (see Eval design in paper).

6. **We use Tiresias?**
   → Step 6. Prefer Tiresias (or an audited alternative) to implement threshold Paillier primitives (DKG, TDec, Rec).

7. **We use DKG?**
   → Step 7 — yes if you want no trusted dealer. DKG produces the shared Paillier key.

---

# Concrete parameters (recommended)

-   `N` (Paillier modulus): **4096 bits**
-   MR rounds: **64** (for 4096-bit) plus **BPSW** if available
-   Threshold `t`: set per your attacker model (e.g., `t = floor(n/3)` or `t = floor((n-1)/2)`)
-   Commitment group: choose a prime-order group with order >= 256-bit (or use same secp256k1 group for Pedersen commitments if safe)
-   Randomness: use OS/HSM RNG, never fallback to weaker PRNG

---

# Minimal pseudocode skeleton (very compact)

```text
// 1. RNG
rng = CSPRNG::new()

// 2. Primes -> Paillier SK/PK
p = strong_prime(bits/2, rng, MR=64, do_bpsw=true)
q = strong_prime(bits/2, rng, MR=64, do_bpsw=true)
N = p * q
g = N + 1
lambda = lcm(p-1, q-1)
mu = L(g^lambda mod N^2)^{-1} mod N

// 3. Commitment + VSS (dealer)
commitments = Pedersen.commit_poly(a0=lambda_mod_N, a1..a_{t-1})
shares = Pedersen.vss_split(lambda_mod_N, t, n)
broadcast(commitments); send shares to validators

// 4. Validators verify shares
for each validator:
    verify_share_against_commitments(share, commitments)
    store share into HSM

// 5. Publish PK + epoch
publish(PK={N,g}, epoch_id)

// 6. TDec/Rec protocol implemented (use Tiresias)
// 7. DKG version: call Tiresias::run_dkg(validators, t, bits)
```

---

## Final practical advice / pitfalls

-   **Don’t implement RSA/Paillier DKG from scratch** unless you're a specialist — use Tiresias or equivalent.
-   **HSMs** are critical for security in production. They reduce attack surface.
-   **Proofs & complaint handling** add complexity but are essential for a malicious environment.
-   **Audit trails** (record MR test metrics, ZK proof checks, complaints) are invaluable for forensics.
-   **Start with dealer + VSS** for early testing, then upgrade to DKG for production decentralization.

---
