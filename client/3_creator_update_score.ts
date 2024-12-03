import { PublicKey, ComputeBudgetProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

(async () => {
  const wallet = pg.wallet;
  const program = pg.program;
  const ROUND_INDEX = 0;
  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const ROUND_STATE_SEED = "ROUND-STATE-SEED";
  const VAULT_SEED = "VAULT_SEED";

  const [globalStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_STATE_SEED)],
    program.programId
  );
  const roundIndex = new anchor.BN(ROUND_INDEX);
  const [roundStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_STATE_SEED), roundIndex.toArrayLike(Buffer, "le", 4)],
    program.programId
  );

  const [vaultPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED)],
    program.programId
  );
  const CREATOR_SCORE = 120;
  const roundState = await program.account.roundState.fetch(roundStatePDA);
  const tx = await program.methods
    .creatorUpdateScore(ROUND_INDEX, CREATOR_SCORE)
    .accounts({
      user: wallet.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      roundState: roundStatePDA,
      vault: vaultPDA,
      joiner: roundState.joiner,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log(`Tx: https://https://solscan.io/tx/${tx}`);
  console.log(`Updated score index ${ROUND_INDEX} successfully`);
})();
