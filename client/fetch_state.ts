import { PublicKey, ComputeBudgetProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

(async () => {
  const wallet = pg.wallet;
  const program = pg.program;
  const ROUND_INDEX = 1;
  const ROUND_STATE_SEED = "ROUND-STATE-SEED";
  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";

  const roundIndex = new anchor.BN(ROUND_INDEX);
  const [roundStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_STATE_SEED), roundIndex.toArrayLike(Buffer, "le", 4)],
    program.programId
  );
  const [globalStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_STATE_SEED)],
    program.programId
  );
  const globalState = await program.account.globalState.fetch(globalStatePDA);
  const roundState = await program.account.roundState.fetch(roundStatePDA);
  console.log(`Global state:`, globalState);
  console.log(`Round state:`, roundState);
})();
