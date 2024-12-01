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

  // // Add your test here.
  const OWNER_NUM: number = 102322;
  // const hasedNum = keccak256(new anchor.BN(OWNER_NUM).toBuffer("le", 4));
  const hashedNum = new Uint8Array([
    156, 172, 236, 78, 218, 107, 122, 189, 56, 169, 17, 167, 1, 185, 250, 21,
    57, 42, 8, 163, 193, 60, 25, 37, 228, 4, 41, 30, 107, 153, 133, 84,
  ]); // hashed keccak256 of owner_num
  const tx = await program.methods
    .createRound(ROUND_INDEX, Array.from(hashedNum))
    .accounts({
      user: wallet.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      roundState: roundStatePDA,
      vault: vaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log(`Tx: https://https://solscan.io/tx/${tx}`);
  console.log(`Round index ${ROUND_INDEX} created successfully`);
})();
