import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NexusBridge } from "../target/types/nexus_bridge";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("nexus-bridge", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NexusBridge as Program<NexusBridge>;

  let bridgeState: anchor.web3.PublicKey;
  let bridgeStateBump: number;
  let tokenMint: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;
  let bridgeTokenAccount: anchor.web3.PublicKey;

  // Test constants
  const VALID_ETH_ADDRESS = "0x742d35Cc6634C0532925a3b844Bc9e7595f8e123";
  const VALID_ETH_TX_HASH = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
  const MIN_BRIDGE_AMOUNT = new anchor.BN(1_000_000_000); // 1 token with 9 decimals
  const LOCK_AMOUNT = new anchor.BN(5_000_000_000); // 5 tokens

  before(async () => {
    // Derive bridge state PDA
    [bridgeState, bridgeStateBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("bridge")],
      program.programId
    );

    // Create token mint
    tokenMint = await createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      provider.wallet.publicKey,
      null,
      9 // 9 decimals
    );

    // Create user token account
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      tokenMint,
      provider.wallet.publicKey
    );

    // Mint tokens to user
    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      tokenMint,
      userTokenAccount,
      provider.wallet.publicKey,
      100_000_000_000 // 100 tokens
    );

    // Derive bridge token account (ATA)
    bridgeTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      bridgeState,
      true // allowOwnerOffCurve for PDA
    );
  });

  describe("initialize", () => {
    it("Initializes the bridge successfully", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          bridgeState,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize transaction signature:", tx);

      const account = await program.account.bridgeState.fetch(bridgeState);
      assert.ok(account.authority.equals(provider.wallet.publicKey));
      assert.equal(account.totalLocked.toNumber(), 0);
      assert.equal(account.totalUnlocked.toNumber(), 0);
      assert.equal(account.nonce.toNumber(), 0);
    });
  });

  describe("lock_tokens", () => {
    it("Locks tokens successfully", async () => {
      // Get nonce for lock event PDA
      const bridgeAccount = await program.account.bridgeState.fetch(bridgeState);
      const nonce = bridgeAccount.nonce;

      const [lockEvent] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lock_event"), nonce.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const userBalanceBefore = (await getAccount(provider.connection, userTokenAccount)).amount;

      const tx = await program.methods
        .lockTokens(LOCK_AMOUNT, VALID_ETH_ADDRESS)
        .accounts({
          bridgeState,
          lockEvent,
          user: provider.wallet.publicKey,
          tokenMint,
          userTokenAccount,
          bridgeTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Lock tokens transaction signature:", tx);

      // Verify bridge state updated
      const updatedBridge = await program.account.bridgeState.fetch(bridgeState);
      assert.equal(updatedBridge.totalLocked.toNumber(), LOCK_AMOUNT.toNumber());
      assert.equal(updatedBridge.nonce.toNumber(), 1);

      // Verify lock event created
      const lockEventAccount = await program.account.lockEvent.fetch(lockEvent);
      assert.ok(lockEventAccount.user.equals(provider.wallet.publicKey));
      assert.equal(lockEventAccount.amount.toNumber(), LOCK_AMOUNT.toNumber());
      assert.equal(lockEventAccount.ethAddress, VALID_ETH_ADDRESS);
      assert.equal(lockEventAccount.processed, false);

      // Verify token balances
      const userBalanceAfter = (await getAccount(provider.connection, userTokenAccount)).amount;
      const bridgeBalance = (await getAccount(provider.connection, bridgeTokenAccount)).amount;

      assert.equal(
        BigInt(userBalanceBefore.toString()) - BigInt(LOCK_AMOUNT.toString()),
        BigInt(userBalanceAfter.toString())
      );
      assert.equal(bridgeBalance.toString(), LOCK_AMOUNT.toString());
    });

    it("Fails with invalid Ethereum address format", async () => {
      const bridgeAccount = await program.account.bridgeState.fetch(bridgeState);
      const nonce = bridgeAccount.nonce;

      const [lockEvent] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lock_event"), nonce.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      try {
        await program.methods
          .lockTokens(LOCK_AMOUNT, "invalid-address")
          .accounts({
            bridgeState,
            lockEvent,
            user: provider.wallet.publicKey,
            tokenMint,
            userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for invalid Ethereum address");
      } catch (err: any) {
        assert.include(err.message, "InvalidEthAddress");
      }
    });

    it("Fails with invalid hex characters in Ethereum address", async () => {
      const bridgeAccount = await program.account.bridgeState.fetch(bridgeState);
      const nonce = bridgeAccount.nonce;

      const [lockEvent] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lock_event"), nonce.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      try {
        // Address with invalid characters (G, H, I are not valid hex)
        await program.methods
          .lockTokens(LOCK_AMOUNT, "0xGHI35Cc6634C0532925a3b844Bc9e7595f8e123")
          .accounts({
            bridgeState,
            lockEvent,
            user: provider.wallet.publicKey,
            tokenMint,
            userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for invalid hex in Ethereum address");
      } catch (err: any) {
        assert.include(err.message, "InvalidEthAddress");
      }
    });

    it("Fails with amount below minimum", async () => {
      const bridgeAccount = await program.account.bridgeState.fetch(bridgeState);
      const nonce = bridgeAccount.nonce;

      const [lockEvent] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lock_event"), nonce.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      try {
        await program.methods
          .lockTokens(new anchor.BN(100), VALID_ETH_ADDRESS) // Below minimum
          .accounts({
            bridgeState,
            lockEvent,
            user: provider.wallet.publicKey,
            tokenMint,
            userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for amount below minimum");
      } catch (err: any) {
        assert.include(err.message, "AmountBelowMinimum");
      }
    });

    it("Fails with zero amount", async () => {
      const bridgeAccount = await program.account.bridgeState.fetch(bridgeState);
      const nonce = bridgeAccount.nonce;

      const [lockEvent] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("lock_event"), nonce.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      try {
        await program.methods
          .lockTokens(new anchor.BN(0), VALID_ETH_ADDRESS)
          .accounts({
            bridgeState,
            lockEvent,
            user: provider.wallet.publicKey,
            tokenMint,
            userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for zero amount");
      } catch (err: any) {
        assert.include(err.message, "InvalidAmount");
      }
    });
  });

  describe("unlock_tokens", () => {
    it("Unlocks tokens successfully", async () => {
      const [processedTx] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("processed_tx"), Buffer.from(VALID_ETH_TX_HASH.slice(2, 34))],
        program.programId
      );

      const unlockAmount = new anchor.BN(2_000_000_000); // 2 tokens
      const userBalanceBefore = (await getAccount(provider.connection, userTokenAccount)).amount;

      const tx = await program.methods
        .unlockTokens(unlockAmount, VALID_ETH_TX_HASH)
        .accounts({
          bridgeState,
          processedTx,
          authority: provider.wallet.publicKey,
          recipient: provider.wallet.publicKey,
          tokenMint,
          recipientTokenAccount: userTokenAccount,
          bridgeTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Unlock tokens transaction signature:", tx);

      // Verify bridge state updated
      const updatedBridge = await program.account.bridgeState.fetch(bridgeState);
      assert.equal(updatedBridge.totalUnlocked.toNumber(), unlockAmount.toNumber());

      // Verify processed transaction recorded
      const processedTxAccount = await program.account.processedTransaction.fetch(processedTx);
      assert.equal(processedTxAccount.ethTxHash, VALID_ETH_TX_HASH);
      assert.equal(processedTxAccount.amount.toNumber(), unlockAmount.toNumber());

      // Verify token balances
      const userBalanceAfter = (await getAccount(provider.connection, userTokenAccount)).amount;
      assert.equal(
        BigInt(userBalanceAfter.toString()) - BigInt(userBalanceBefore.toString()),
        BigInt(unlockAmount.toString())
      );
    });

    it("Fails when trying to replay same transaction", async () => {
      const [processedTx] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("processed_tx"), Buffer.from(VALID_ETH_TX_HASH.slice(2, 34))],
        program.programId
      );

      try {
        await program.methods
          .unlockTokens(new anchor.BN(1_000_000_000), VALID_ETH_TX_HASH)
          .accounts({
            bridgeState,
            processedTx,
            authority: provider.wallet.publicKey,
            recipient: provider.wallet.publicKey,
            tokenMint,
            recipientTokenAccount: userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for replay attack");
      } catch (err: any) {
        // Account already exists - replay protection working
        assert.ok(err.message.includes("already in use") || err.message.includes("custom program error"));
      }
    });

    it("Fails with invalid transaction hash format", async () => {
      const invalidTxHash = "0xinvalid";
      const [processedTx] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("processed_tx"), Buffer.from(invalidTxHash)],
        program.programId
      );

      try {
        await program.methods
          .unlockTokens(MIN_BRIDGE_AMOUNT, invalidTxHash)
          .accounts({
            bridgeState,
            processedTx,
            authority: provider.wallet.publicKey,
            recipient: provider.wallet.publicKey,
            tokenMint,
            recipientTokenAccount: userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for invalid tx hash format");
      } catch (err: any) {
        assert.include(err.message, "InvalidTxHash");
      }
    });

    it("Fails with unauthorized caller", async () => {
      const newTxHash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
      const [processedTx] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("processed_tx"), Buffer.from(newTxHash)],
        program.programId
      );

      // Create a new keypair for unauthorized user
      const unauthorizedUser = anchor.web3.Keypair.generate();

      // Airdrop some SOL for fees
      const airdropSig = await provider.connection.requestAirdrop(
        unauthorizedUser.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdropSig);

      try {
        await program.methods
          .unlockTokens(MIN_BRIDGE_AMOUNT, newTxHash)
          .accounts({
            bridgeState,
            processedTx,
            authority: unauthorizedUser.publicKey,
            recipient: provider.wallet.publicKey,
            tokenMint,
            recipientTokenAccount: userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([unauthorizedUser])
          .rpc();
        assert.fail("Expected error for unauthorized caller");
      } catch (err: any) {
        assert.include(err.message, "Unauthorized");
      }
    });

    it("Fails with insufficient bridge balance", async () => {
      const newTxHash = "0x9999999999999999999999999999999999999999999999999999999999999999";
      const [processedTx] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("processed_tx"), Buffer.from(newTxHash)],
        program.programId
      );

      // Get current bridge balance
      const bridgeBalance = (await getAccount(provider.connection, bridgeTokenAccount)).amount;
      const excessiveAmount = new anchor.BN(bridgeBalance.toString()).add(new anchor.BN(1_000_000_000));

      try {
        await program.methods
          .unlockTokens(excessiveAmount, newTxHash)
          .accounts({
            bridgeState,
            processedTx,
            authority: provider.wallet.publicKey,
            recipient: provider.wallet.publicKey,
            tokenMint,
            recipientTokenAccount: userTokenAccount,
            bridgeTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        assert.fail("Expected error for insufficient bridge balance");
      } catch (err: any) {
        assert.include(err.message, "InsufficientBridgeBalance");
      }
    });
  });
});
