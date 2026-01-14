
# Nexus Solana

<div align="center">

![Rust](https://img.shields.io/badge/Rust-1.75+-orange)
![Anchor](https://img.shields.io/badge/Anchor-0.30.1-blue)
![Solana](https://img.shields.io/badge/Solana-1.18+-green)
![License](https://img.shields.io/badge/License-MIT-yellow)

Solana programs for **Nexus** - A trustless cross-chain bridge between Ethereum and Solana.

[Documentation](#documentation) • [Features](#features) • [Setup](#setup) • [Testing](#testing) • [Deployment](#deployment)

</div>

---

## Overview

Nexus Solana handles the Solana side of the cross-chain bridge, allowing users to:
- Lock SPL tokens on Solana
- Receive wrapped tokens on Ethereum
- Unlock tokens when bridging back from Ethereum

## Architecture
```
┌─────────────────┐
│  User Wallet    │
│   (Phantom)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌──────────────────┐
│ Nexus Program   │◄────►│  Nexus Relayer   │
│   (On-chain)    │      │   (Off-chain)    │
└─────────────────┘      └──────────────────┘
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌──────────────────┐
│ Locked Tokens   │      │ Ethereum Smart   │
│   (Token Acct)  │      │    Contracts     │
└─────────────────┘      └──────────────────┘
```

## Features

✅ **SPL Token Locking** - Secure token custody using PDAs  
✅ **Event Emission** - Cross-chain message passing via accounts  
✅ **Replay Protection** - Transaction hash tracking prevents double-spending  
✅ **Authority Control** - Only authorized validators can unlock tokens  
✅ **Gas Efficient** - Optimized for Solana's compute units  
✅ **Anchor Framework** - Type-safe Rust development  

## Program Accounts

### BridgeState
Main bridge configuration:
- Authority (admin pubkey)
- Total locked/unlocked amounts
- Nonce for unique event IDs

### LockEvent
Records each token lock:
- User wallet
- Amount locked
- Target Ethereum address
- Timestamp and processing status

### ProcessedTransaction
Prevents replay attacks:
- Ethereum transaction hash
- Processing timestamp
- Amount unlocked

## Setup

### Prerequisites

- Rust 1.75+
- Solana CLI 1.18+
- Anchor CLI 0.30.1+
- Node.js 18+

### Installation
```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/nexus-solana.git
cd nexus-solana

# Install dependencies
npm install

# Build the program
anchor build

# Generate program keypair (first time only)
anchor keys list
# Copy the program ID and update in:
# - Anchor.toml (programs sections)
# - lib.rs (declare_id! macro)

# Rebuild after updating program ID
anchor build
```

### Local Development
```bash
# Start local validator
solana-test-validator

# In another terminal, run tests
anchor test --skip-local-validator
```

## Testing
```bash
# Run all tests
anchor test

# Run specific test
anchor test --skip-deploy -- --grep "Initializes"

# Watch mode (requires cargo-watch)
cargo watch -x 'test'

# Check program size
solana program dump BRGx8VkKcPqsQNH5jHJdWL8nfvUaS9HvqCjt7z8XYPWE dump.so
ls -lh dump.so
```

## Deployment

### Devnet Deployment
```bash
# Configure CLI for devnet
solana config set --url devnet

# Airdrop SOL for deployment fees
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show <PROGRAM_ID>
```

### Mainnet Deployment
```bash
# Configure for mainnet
solana config set --url mainnet-beta

# Deploy (requires sufficient SOL for deployment)
anchor deploy --provider.cluster mainnet

# ALWAYS verify the deployment
anchor verify <PROGRAM_ID>
```

## Usage

### Initialize Bridge
```typescript
import * as anchor from "@coral-xyz/anchor";

const [bridgeState] = await PublicKey.findProgramAddress(
  [Buffer.from("bridge")],
  program.programId
);

await program.methods
  .initialize()
  .accounts({
    bridgeState,
    authority: wallet.publicKey,
  })
  .rpc();
```

### Lock Tokens
```typescript
await program.methods
  .lockTokens(
    new anchor.BN(1000000), // amount in smallest units
    "0x742d35Cc6634C0532925a3b844Bc454e4438f44e" // Ethereum address
  )
  .accounts({
    bridgeState,
    lockEvent,
    user: wallet.publicKey,
    userTokenAccount,
    bridgeTokenAccount,
  })
  .rpc();
```

### Unlock Tokens
```typescript
// Only callable by bridge authority
await program.methods
  .unlockTokens(
    new anchor.BN(1000000),
    "0x123...abc" // Ethereum tx hash
  )
  .accounts({
    bridgeState,
    processedTx,
    authority: authorityWallet.publicKey,
    recipient: recipientWallet.publicKey,
    recipientTokenAccount,
    bridgeTokenAccount,
  })
  .rpc();
```

## Security

- ✅ PDAs for secure token custody
- ✅ Authority checks on sensitive operations
- ✅ Replay attack prevention
- ✅ Integer overflow protection
- ✅ Comprehensive input validation
- ⚠️ **Experimental software - audit before mainnet use**

## Program Structure
```
nexus-solana/
├── programs/
│   └── nexus-bridge/
│       └── src/
│           ├── lib.rs              # Program entry point
│           ├── state.rs            # Account structures
│           ├── errors.rs           # Custom errors
│           └── instructions/       # Instruction handlers
│               ├── initialize.rs
│               ├── lock_tokens.rs
│               └── unlock_tokens.rs
├── tests/
│   └── nexus-bridge.ts            # Integration tests
└── Anchor.toml                    # Anchor configuration
```

## Compute Budget

Estimated compute units per instruction:
- `initialize`: ~5,000 CU
- `lock_tokens`: ~20,000 CU
- `unlock_tokens`: ~25,000 CU

## Related Repositories

- [nexus-ethereum](https://github.com/YOUR_USERNAME/nexus-ethereum) - Ethereum smart contracts
- [nexus-relayer](https://github.com/YOUR_USERNAME/nexus-relayer) - Bridge relayer service
- [nexus-frontend](https://github.com/YOUR_USERNAME/nexus-frontend) - Web interface

## Common Issues

**Program ID mismatch:**
```bash
anchor keys sync
anchor build
```

**Compute budget exceeded:**
Add compute budget instruction before transaction

**Account already in use:**
Program is already deployed - use `anchor upgrade` instead

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Write tests for new features
4. Commit changes (`git commit -m 'Add amazing feature'`)
5. Push to branch (`git push origin feature/amazing`)
6. Open Pull Request

## Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [SPL Token Program](https://spl.solana.com/token)

## License

MIT License - see [LICENSE](LICENSE) file

---

<div align="center">
  
Built with 🦀 Rust and ⚓ Anchor</div>
