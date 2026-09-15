## Stage 1: Mid

- Invariants: none
- Target:
  - 1x3x3x2 block of M in M (`[0,-1,-1,-1]..=[0,1,1,0]`)
  - R/L ridges oriented
- Moves: all
- Representation: `u64`
  - for each ridge: 11, 01, or 10 (48 bits)
    - 11 if belongs in M
    - 01 if belongs in R/L + good orientation
    - 10 if belongs in R/L + bad orientation
  - for each edge: 1 if belongs in M, 0 otherwise (32 bits)

## Stage 2: Left

- Invariants: R/L ridge orientation, mid block preserved
- Target: 1x3x3x2 block of R/L in L
  - `[-1,-1,-1,-1]..=[-1,1,1,0]`
- Moves: R*, L*, I*, OR*, UO2, DO2, FO2, BO2
- Representation: `(u16, u64, u32)`
  - for each ridge in R/I/L: 1 if belongs in R/L, 0 otherwise (16 bits)
  - for each edge in R/I/L: 00, 01, or 10 (56 bits)
    - 00 if belongs in M
    - 11 if belongs in R/L + good orientation
    - 01 if belongs in R/L + bad orientation 1
    - 10 if belongs in R/L + bad orientation 2
  - for each corner: 2 bits indicating the axis containing its R/L sticker (32 bits)
    - possibly use 4 bits instead? uses more space, but can prune

## Stage 3: Numbers

- Invariants: R/L ridge orientation, mid & left blocks preserved
- Target:
  - M ridges: 10 in M, 2 in R/L
  - R/L ridges: 2 in M, 10 oriented in R/L
  - M edges: 4 in M, 4 in R/L
  - R/L edges: 4 in M, 4 misoriented in R/L, 4 oriented in R/L
  - corners: 8 misoriented, 8 oriented
- Moves: R*, LO*, I*, OR*
- Representation: (95 bits)
  - for each ridge in R/I: 1 if belongs in R/L, 0 otherwise (11 bits)
  - for each edge sticker in R/I: 3 bits (60 bits)
  - for each corner in R/I: 2 bits indicating the axis containing its R/L sticker (24 bits)

## Stage 4: Pre-domino

- Invariants: numbers from stage 3
- Target: 1 move from solved domino
- Moves: domino
- Representation: `(u32, u128)`
  - for each of the 2 R/L ridges in M: 6 bits indicating attitude (12 bits)
    - 3 bits indicating primary facet
    - 3 bits indicating secondary facet
  - for each of the 2 M ridges in R/L: 4 bits indicating location (8 bits)
    - 1 bit indicating sign of primary facet (R/L)
    - 3 bits indicating secondary facet
  - for each edge sticker location on the puzzle: 1 bit indicating whether it is R/L (96 bits)
  - for each edge on the puzzle: 2 bits (64 bits)
    - 00 if belongs in M
    - 11 if belongs in R/L + good orientation
    - 01 if belongs in R/L + bad orientation 1
    - 10 if belongs in R/L + bad orientation 2
  - for each corner: 2 bits indicating the axis containing its R/L sticker (32 bits)

## Stage 5: Domino

1 move to solve domino

## Stage 6: Separate

- Invariants: domino
- Target: R/L separated + R/L parities resolved (1/12 chance)
- Moves: domino
- Representation: `u64`
  - 1 bit indicating permutation parity (1 bit)
  - for each sticker on R/L: 1 bit indicating sign (52 bits)
  <!-- - for each edge in R/L: 1 bit indicating 3D orientation (24 bits) -->
  <!-- - for each corner: 2 bits indicating 3D orientation (32 bits) -->
