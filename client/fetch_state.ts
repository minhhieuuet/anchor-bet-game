import { PublicKey, ComputeBudgetProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

(async () => {
  const wallet = pg.wallet;
  const program = pg.program;
  const ROUND_INDEX = 0;
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
  const { totalRound } = globalState;
  console.log(`Total round: ${totalRound.toString()}`);
  const { creator, joiner, creatorScore, joinerScore, winner, isFinished } =
    roundState;
  console.log(`Creator:`, creator.toBase58());
  console.log(`Joiner:`, joiner.toBase58());
  console.log(`Creator score:`, creatorScore.toString());
  console.log(`Joiner score: `, joinerScore.toString());
  console.log(`Winner: ${winner.toBase58()}`);
  console.log("Is finished:", isFinished);
})();
