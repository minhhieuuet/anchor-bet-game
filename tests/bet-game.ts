import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BetGame } from "../target/types/bet_game";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { keccak256 } from "ethereum-cryptography/keccak.js";
import bs58  from "bs58";

describe("bet-game", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BetGame;
  return;
  let payer = anchor.web3.Keypair.fromSecretKey(
     new Uint8Array([73,70,80,94,245,200,222,222,16,76,165,146,67,8,157,155,17,47,176,79,142,127,1,98,143,113,70,188,217,126,81,135,8,167,152,92,252,174,189,208,88,186,215,231,182,230,127,239,94,212,178,171,77,69,236,56,132,138,168,90,54,121,95,31])
  );
  let player = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array([58,118,10,126,113,154,2,34,17,196,130,88,252,76,190,25,20,194,239,172,50,138,44,250,124,213,55,173,147,118,121,223,8,155,169,74,145,204,104,110,166,126,167,183,201,187,229,229,242,196,1,232,169,75,119,14,115,131,31,27,253,199,179,128])
  );

  // Fund the payer.
  // request airdrop from the local cluster
  before(async () => {
    // Create and fund person
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        payer.publicKey,
        50 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "processed"
    );
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        player.publicKey,
        50 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "processed"
    );
  });

  const ROUND_INDEX = 1;
  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const ROUND_STATE_SEED = "ROUND-STATE-SEED";
  const VAULT_SEED = "VAULT-SEED";

  const [globalStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(GLOBAL_STATE_SEED)],
    program.programId
  );
  const [roundStatePDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ROUND_STATE_SEED), new anchor.BN(ROUND_INDEX).toBuffer("le", 4)],
    program.programId
  );

  const [vaultPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED)],
    program.programId
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().accounts({
      systemProgram: anchor.web3.SystemProgram.programId,
      //@ts-ignore
      globalState: globalStatePDA,
    }).rpc();
  });

  it("Can create a new game", async () => {
    // Add your test here.
    const OWNER_NUM: number = 102322;
    const hasedNum = keccak256(new anchor.BN(OWNER_NUM).toBuffer("le", 4));
    console.log(hasedNum);
    const tx = await program.methods.createRound(
      ROUND_INDEX,
      Array.from(hasedNum),
    ).accounts({
      user: payer.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      roundState: roundStatePDA,
      vault: vaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([payer]).rpc();
    console.log(tx);
    console.log("Round created successfully");
  });

  it("Can join a game", async () => {
    const JOINER_NUM: number = 1022;
    const tx = await program.methods.joinRound(
      ROUND_INDEX,
      JOINER_NUM
    ).accounts({
      user: player.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      roundState: roundStatePDA,
      vault: vaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([player]).rpc();
    console.log(tx);
    console.log("Round joined successfully");
  })

  it("Can reveal number", async () => {
    const REVEAL_NUM: number = 102322;
    const tx = await program.methods.reveal(
      ROUND_INDEX,
      REVEAL_NUM
    ).accounts({
      user: payer.publicKey,
      //@ts-ignore
      globalState: globalStatePDA,
      roundState: roundStatePDA,
      vault: vaultPDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([payer]).rpc();
    console.log(tx);
    console.log("Number revealed successfully");
    const roundStateData = await program.account.roundState.fetch(roundStatePDA);
    const {
      winner
    } = roundStateData;
    if(winner.toBase58() === payer.publicKey.toBase58()) {
      console.log("Creator won the game");
    } else {
      console.log("Player won the game");
    }
  })

});
