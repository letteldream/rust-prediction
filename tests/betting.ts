import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Betting } from '../target/types/betting';
import { TOKEN_PROGRAM_ID, Token, ASSOCIATED_TOKEN_PROGRAM_ID, } from '@solana/spl-token';
import { assert } from "chai";
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { loadWalletKey } from './utils';
import { u64 } from "@saberhq/token-utils";

describe('betting', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Betting as Program<Betting>;

  const adminAccount = loadWalletKey("./wallet/admin.json");
  const user1Account = loadWalletKey("./wallet/user1.json");
  const user2Account = loadWalletKey("./wallet/user2.json");
  const user3Account = loadWalletKey("./wallet/user3.json");
  const treasuryAccount = new PublicKey("HeGTsuhcCpuHnia7tNyq8JZKZ4CTQ71w9WcGf23h9kMG");

  const pyth_account1 = new PublicKey("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J"); // BTC/USD
  const locked_price1 = 3900000000000;// UP

  const pyth_account2 = new PublicKey("4dqq5VBpN4EwYb7wyywjjfknvMKu7m78j9mKZRXTj462"); // DOT/USD
  const locked_price2 = 3000000000; // DOWN

  const pyth_account3 = new PublicKey("3gnSbT7bhoTdGkFVZc1dW1PvjreWzpUNUD5ppXwv1N59"); // NEAR/USD
  const locked_price3 = 800000000; // UP

  const bet_id: u64 = new u64(32132132);
  const bet_amount = 1000000000; // 1SOL
  const betting_period = 1; // 1 min

  let bet_detail_account_pda = null;
  let bet_detail_account_bump = null;
  let escrow_pda = null;
  let escrow_bump = null;
  let user1_bet_detail_account_pda = null;
  let user2_bet_detail_account_pda = null;
  let user3_bet_detail_account_pda = null;
  let bump1 = null;
  let bump2 = null;
  let bump3 = null;

  console.log(adminAccount.publicKey.toString())

  it('Get PDAS', async () => {
    // Airdrop 2 SOL to payer
    // await provider.connection.confirmTransaction(
    //   await provider.connection.requestAirdrop(adminAccount.publicKey, 2000000000),
    //   "confirmed"
    // );

    // await provider.send(
    //   (() => {
    //     const tx = new Transaction();
    //     tx.add(
    //       SystemProgram.transfer({
    //         fromPubkey: adminAccount.publicKey,
    //         toPubkey: user3Account.publicKey,
    //         lamports: 1500000000,
    //       }),
    //     );
    //     return tx;
    //   })(),
    //   [adminAccount]
    // );

    [bet_detail_account_pda, bet_detail_account_bump] = await PublicKey.findProgramAddress([
      Buffer.from("bet-detail"),
      bet_id.toArrayLike(Buffer, "le", 8),
      adminAccount.publicKey.toBuffer(),
    ], program.programId);
    console.log("bet detail account: ", bet_detail_account_pda.toString());

    [escrow_pda, escrow_bump] = await PublicKey.findProgramAddress([
      Buffer.from("hypothese-escrow"),
      bet_id.toArrayLike(Buffer, "le", 8),
    ], program.programId);
    console.log("escrow account: ", escrow_pda.toString());
    
    [user1_bet_detail_account_pda, bump1] = await PublicKey.findProgramAddress([
      Buffer.from("user-bet"),
      bet_id.toArrayLike(Buffer, "le", 8),
      pyth_account1.toBuffer(),
      user1Account.publicKey.toBuffer(),
    ], program.programId);

    [user2_bet_detail_account_pda, bump2] = await PublicKey.findProgramAddress([
      Buffer.from("user-bet"),
      bet_id.toArrayLike(Buffer, "le", 8),
      pyth_account2.toBuffer(),
      user2Account.publicKey.toBuffer(),
    ], program.programId);

    [user3_bet_detail_account_pda, bump3] = await PublicKey.findProgramAddress([
      Buffer.from("user-bet"),
      bet_id.toArrayLike(Buffer, "le", 8),
      pyth_account3.toBuffer(),
      user3Account.publicKey.toBuffer(),
    ], program.programId);

  })

  // it('Initialize Bet!', async () => {
    
  //   await program.rpc.initializeBet(
  //     new anchor.BN(bet_id),
  //     new anchor.BN(bet_amount),
  //     betting_period,
  //     {
  //       accounts: {
  //         adminAccount: adminAccount.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [adminAccount]
  //     }
  //   );
  // });

  // it('User Bet!', async () => {

  //   await program.rpc.userBet(
  //     new anchor.BN(bet_id),
  //     true, // true: UP, false: DOWN
  //     new anchor.BN(locked_price1),
  //     {
  //       accounts: {
  //         userAccount: user1Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         escrowAccount: escrow_pda,
  //         userBetDetailAccount: user1_bet_detail_account_pda,
  //         betPythAccount: pyth_account1,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [user1Account]
  //     }
  //   );

  //   await program.rpc.userBet(
  //     new anchor.BN(bet_id),
  //     true, // true: UP, false: DOWN
  //     new anchor.BN(locked_price2),
  //     {
  //       accounts: {
  //         userAccount: user2Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         escrowAccount: escrow_pda,
  //         userBetDetailAccount: user2_bet_detail_account_pda,
  //         betPythAccount: pyth_account2,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [user2Account]
  //     }
  //   );

  //   await program.rpc.userBet(
  //     new anchor.BN(bet_id),
  //     true, // true: UP, false: DOWN
  //     new anchor.BN(locked_price3),
  //     {
  //       accounts: {
  //         userAccount: user3Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         escrowAccount: escrow_pda,
  //         userBetDetailAccount: user3_bet_detail_account_pda,
  //         betPythAccount: pyth_account3,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [user3Account]
  //     }
  //   );

  // });

  // it('Start Bet!', async () => {

  //   await program.rpc.startBet(
  //     new anchor.BN(bet_id),
  //     {
  //       accounts: {
  //         adminAccount: adminAccount.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //       },
  //       signers: [adminAccount]
  //     }
  //   );

  // });

  // it('Complete Bet!', async () => {

  //   await program.rpc.completeBet(
  //     new anchor.BN(bet_id),
  //     {
  //       accounts: {
  //         adminAccount: adminAccount.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //       },
  //       signers: [adminAccount]
  //     }
  //   );
  // });

  // it('Determine Bet Result', async () => {

  //   await program.rpc.deterimineBetResult(
  //     new anchor.BN(bet_id),
  //     {
  //       accounts: {
  //         userAccount: user1Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         userBetDetailAccount: user1_bet_detail_account_pda,
  //         betPythAccount: pyth_account1,
  //       },
  //       signers: [user1Account]
  //     }
  //   );

  //   await program.rpc.deterimineBetResult(
  //     new anchor.BN(bet_id),
  //     {
  //       accounts: {
  //         userAccount: user2Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         userBetDetailAccount: user2_bet_detail_account_pda,
  //         betPythAccount: pyth_account2,
  //       },
  //       signers: [user2Account]
  //     }
  //   );

  //   await program.rpc.deterimineBetResult(
  //     new anchor.BN(bet_id),
  //     {
  //       accounts: {
  //         userAccount: user3Account.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         userBetDetailAccount: user3_bet_detail_account_pda,
  //         betPythAccount: pyth_account3,
  //       },
  //       signers: [user3Account]
  //     }
  //   );

  //   let temp = await program.provider.connection.getAccountInfo(bet_detail_account_pda);
  //   console.log(temp);

  //   let user1 = await program.provider.connection.getAccountInfo(user1_bet_detail_account_pda);
  //   console.log(user1);

  //   let user2 = await program.provider.connection.getAccountInfo(user2_bet_detail_account_pda);
  //   console.log(user2);

  //   let user3 = await program.provider.connection.getAccountInfo(user3_bet_detail_account_pda);
  //   console.log(user3);

  // });

  // it('Reward admin', async () => {
  //   let temp = await program.provider.connection.getAccountInfo(treasuryAccount);
  //   console.log(temp);

  //   await program.rpc.rewardAdmin(
  //     new anchor.BN(bet_id),
  //     escrow_bump,
  //     {
  //       accounts: {
  //         adminAccount: adminAccount.publicKey,
  //         betDetailAccount: bet_detail_account_pda,
  //         escrowAccount: escrow_pda,
  //         treasuryAccount: treasuryAccount,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [adminAccount]
  //     }
  //   );

  //   temp = await program.provider.connection.getAccountInfo(treasuryAccount);
  //   console.log(temp.lamports.toString());

  // });

  it('Distribute Prize', async () => {

    // await program.rpc.distributePrize(
    //   new anchor.BN(bet_id),
    //   escrow_bump,
    //   {
    //     accounts: {
    //       userAccount: user1Account.publicKey,
    //       betDetailAccount: bet_detail_account_pda,
    //       escrowAccount: escrow_pda,
    //       userBetDetailAccount: user1_bet_detail_account_pda,
    //       betPythAccount: pyth_account1,
    //       systemProgram: anchor.web3.SystemProgram.programId,
    //     },
    //     signers: [user1Account]
    //   }
    // );

    // await program.rpc.distributePrize(
    //   new anchor.BN(bet_id),
    //   escrow_bump,
    //   {
    //     accounts: {
    //       userAccount: user2Account.publicKey,
    //       betDetailAccount: bet_detail_account_pda,
    //       escrowAccount: escrow_pda,
    //       userBetDetailAccount: user2_bet_detail_account_pda,
    //       betPythAccount: pyth_account2,
    //       systemProgram: anchor.web3.SystemProgram.programId,
    //     },
    //     signers: [user2Account]
    //   }
    // );

    // await program.rpc.distributePrize(
    //   new anchor.BN(bet_id),
    //   escrow_bump,
    //   {
    //     accounts: {
    //       userAccount: user3Account.publicKey,
    //       betDetailAccount: bet_detail_account_pda,
    //       escrowAccount: escrow_pda,
    //       userBetDetailAccount: user3_bet_detail_account_pda,
    //       betPythAccount: pyth_account3,
    //       systemProgram: anchor.web3.SystemProgram.programId,
    //     },
    //     signers: [user3Account]
    //   }
    // );

    // let user1 = await program.account.userBetDetails.fetchNullable(user1Account.publicKey);
    // console.log(user1);

    // let user2 = await program.provider.connection.getAccountInfo(user2Account.publicKey);
    // console.log(user2.data);

    // user3 = await program.provider.connection.getAccountInfo(user3Account.publicKey);
    // console.log(user3);

  });
});
