### 🧩 **Step 1 — Prime Generation**

#### Goal:

Generate two large primes ( p ) and ( q ) such that ( \gcd(pq, (p-1)(q-1)) = 1 ).

#### Math:

[
\begin{aligned}
p &\leftarrow \text{PrimeGen}(k/2) \
q &\leftarrow \text{PrimeGen}(k/2) \
N &= p \cdot q \
\lambda &= \text{lcm}(p-1, q-1)
\end{aligned}
]
Where ( k ) = security parameter (e.g., 2048 bits).

#### Verification:

Use probabilistic primality tests:
[
\text{Miller-Rabin}(p) = \text{true}, \quad \text{Miller-Rabin}(q) = \text{true}
]

---

### 🧩 **Step 2 — Polynomial Creation (Shamir Secret Sharing)**

Each party ( P*i ) generates a private polynomial:
[
f_i(x) = a*{i0} + a*{i1}x + a*{i2}x^2 + \dots + a\_{i,t-1}x^{t-1}
]
where:

-   ( a\_{i0} ) is the **private secret share** for that party (randomly chosen mod ( N )).
-   ( t ) is the **threshold**.

---

### 🧩 **Step 3 — Pedersen Commitment**

Each coefficient is committed to using a **Pedersen commitment** for verifiability:

[
C_{ij} = g^{a_{ij}} \cdot h^{r_{ij}} \pmod{N^2}
]

Where:

-   ( g, h ) are public generators of the group ( \mathbb{Z}\_{N^2}^\* ).
-   ( r\_{ij} ) is random blinding.

---

### 🧩 **Step 4 — Share Distribution**

Each party ( P*i ) computes **shares** for all other parties ( P_j ):
[
s*{ij} = f*i(j) \pmod{N}
]
Then securely sends ( s*{ij} ) to ( P_j ).

Each party ( P*j ) receives shares ( s*{ij} ) from all ( P_i ).

---

### 🧩 **Step 5 — Share Verification**

Each party verifies the received shares using commitments:

[
g^{s_{ij}} \stackrel{?}{=} \prod_{k=0}^{t-1} (C_{ik})^{j^k} \pmod{N^2}
]

If equality holds, the share is valid.

---

### 🧩 **Step 6 — Schnorr Proof (for honesty)**

Each ( P*i ) must prove knowledge of their secret ( a*{i0} ) without revealing it.

#### Math:

-   Choose random ( r )
-   Compute commitment ( t = g^r )
-   Compute challenge ( c = H(g, g^{a\_{i0}}, t) )
-   Compute response ( s = r + c \cdot a\_{i0} )
-   Verify:
    [
    g^s \stackrel{?}{=} t \cdot (g^{a_{i0}})^c
    ]

---

### 🧩 **Step 7 — Secret Combination**

After verification, each party’s local secret is:
[
s_j = \sum_{i=1}^{n} s_{ij} \pmod{N}
]

No single party knows the full secret, only their **share**.

---

### 🧩 **Step 8 — Public Key Construction**

Public key:
[
pk = (N, g)
]
Private key (shared form):
[
sk_i = s_i
]
Reconstruction (when ≥ t parties cooperate):
[
S = \sum_{i=1}^{t} \lambda_i \cdot s_i \pmod{N}
]
where ( \lambda*i ) are **Lagrange coefficients**:
[
\lambda_i = \prod*{j \ne i} \frac{j}{j - i} \pmod{N}
]

---

### 🧩 **Step 9 — Paillier Encryption**

Encryption of message ( m ):
[
c = g^m \cdot r^N \pmod{N^2}
]
Decryption (threshold reconstruction):
Each party produces partial decryption share:
[
d_i = c^{2\Delta s_i} \pmod{N^2}
]
Combine with Lagrange interpolation to recover ( m ).

---

### 🧩 **Step 10 — Zero-Knowledge Consistency Proofs**

To ensure:

-   The commitments are correct (Pedersen + Schnorr)
-   The shares are consistent
-   No one faked polynomials

---

✅ **Final Output**
All parties hold:

-   A **public key** ( (N, g) )
-   A **private share** ( s_i )
    They can jointly decrypt or sign **without revealing secrets**.

---
