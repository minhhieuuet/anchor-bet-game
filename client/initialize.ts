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
  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const ROUND_STATE_SEED = "ROUND-STATE-SEED";
  const VAULT_SEED = "VAULT_SEED";

  const [globalStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_STATE_SEED)],
    program.programId
  );
  const roundIndex = new anchor.BN(ROUND_INDEX);
  console.log(roundIndex.toArrayLike(Buffer, "le", 4).toString("hex"));
  console.log(program.programId.toBase58());
  const [roundStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_STATE_SEED), roundIndex.toArrayLike(Buffer, "le", 4)],
    program.programId
  );

  const [vaultPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED)],
    program.programId
  );

  const tx = await program.methods
    .initialize()
    .accounts({
      user: wallet.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      vault: vaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log(`Tx: https://https://solscan.io/tx/${tx}`);
  console.log(`Init success`);
})();
