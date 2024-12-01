import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, ComputeBudgetProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import type { HelloAnchor } from "../target/types/hello_anchor";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.HelloAnchor as anchor.Program<HelloAnchor>;


(async () => {
  const wallet = pg.wallet;
  const program = program;
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
