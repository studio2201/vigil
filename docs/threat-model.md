# Threat model — vigil

## 1. Adversary
A malicious or unmaintained npm / cargo / pip registry, plus an attacker
who controls the user's lockfile at install time. The attacker can ship
a "looks-fine" package that is a backdoor, or rewrite a pinned version to
one with a known CVE.

## 2. Trust boundaries
We trust: the user's policy file (`vigil.toml`), the user's git history
of the lockfile, and the SHA-256 of the lockfile at the time of scan.
We do not trust: the registry, the contents of any package fetched at
scan time, or the network between the user's machine and the registry.

## 3. Out of scope
Vigil does not defend against: compromised developer credentials, malicious
post-install scripts that run after Vigil has approved a package, and
supply-chain attacks that exploit build-time tools the user has installed
on their own machine (e.g., a poisoned `npm` binary).

## 4. Residual risk
Vigil's classifier relies on heuristics (last-commit date, downloads,
CVE cross-reference). A targeted attack that *looks* active for the 30-day
heuristic window will pass. The buyer is accepting "best-effort flagging,
not a guarantee."
