# Physical Entropy Source for Cryptographic Systems

## Overview

This project implements a **physical entropy source** using optical phenomena and camera capture.

Continuously changing light patterns are produced by an intentionally unstable optical setup involving diffused laser light, airflow, and thermal effects. A camera captures these patterns, which are converted into a raw bitstream, conditioned using standard cryptographic hashes, and used to reseed a cryptographically secure pseudorandom number generator (CSPRNG).

The system is concerned with **entropy generation and conditioning**, not the design of cryptographic primitives.

---

## Motivation

In practice, cryptographic failures are more often caused by poor randomness, key reuse, or system-level errors than by weaknesses in encryption algorithms.

This project focuses on improving key unpredictability by:

- Sourcing entropy from physical, non-deterministic processes
- Conditioning biased physical measurements into usable entropy
- Monitoring entropy quality over time

---

## Assumptions and Scope

This project assumes:

- The use of standard cryptographic primitives (e.g., SHA/BLAKE hashes, ChaCha-based CSPRNGs)
- A local, unsynchronised entropy source
- Software-level access to camera input

This project does not attempt to:

- Design new encryption or hash algorithms
- Replace existing cryptographic systems
- Address physical tampering or hostile local access

---

## System Architecture

1. Optical setup generates unstable light patterns
2. Camera captures frames at fixed exposure and gain
3. Frames are converted into a raw bitstream
4. Temporal and spatial decorrelation reduces structure
5. Data is conditioned using a cryptographic hash
6. Output is used to reseed a CSPRNG
7. CSPRNG generates keys, nonces, and session material

---

## Optical Design Rationale

A bare laser beam is highly deterministic and unsuitable as a direct entropy source.

This system deliberately destroys laser coherence using diffusers, vibration, airflow, and thermal convection to generate **laser speckle patterns** that are highly sensitive to small environmental changes. The camera observes scattered light rather than the laser source itself.

---

## Entropy Conditioning

Raw physical measurements are biased and correlated.

Entropy extraction includes:

- Frame differencing and mixing to reduce static patterns
- Cryptographic hashing to remove bias and correlations

The output of this process is treated strictly as **entropy input**, not as a cryptographic key.

---

## Entropy Health Monitoring

Physical entropy sources can degrade without obvious failure.

This system monitors:

- Variance collapse
- Bit bias drift
- Reduced temporal unpredictability

If entropy quality falls below defined thresholds, reseeding is suspended.

---

## Limitations

- Physical systems require calibration and maintenance
- Entropy quality depends on sustained environmental instability
- Camera sensors introduce fixed-pattern and electronic noise
- This system is intended to supplement, not replace, OS entropy sources

---

## Repository Structure

/capture - Camera input and frame handling
/extraction - Bit harvesting and decorrelation
/conditioning - Hashing and CSPRNG reseeding
/analysis - Entropy testing and monitoring
/docs - Setup photos and diagrams


---

## Cryptographic Primitives

This project relies on established cryptographic primitives:

- SHA-256 / BLAKE2 / BLAKE3 for entropy conditioning
- ChaCha-based CSPRNGs for entropy expansion

These components are well-studied and widely deployed. The experimental focus is limited to the entropy source itself.

---

## Status

This project is experimental and intended for research and educational purposes.
