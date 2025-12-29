import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NexusBridge } from "../target/types/nexus_bridge";
import { assert } from "chai";

describe("nexus-bridge", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NexusBridge as Program<NexusBridge>;
  
  let bridgeState: anchor.web3.PublicKey;
  let bridgeStateBump: number;

  before(async () => {
    [bridgeState, bridgeStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("bridge")],
      program.programId
    );
  });

  it("Initializes the bridge", async () => {
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
  });

  // Add more tests for lock_tokens and unlock_tokens
});
