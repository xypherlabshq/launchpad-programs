const anchor = require("@project-serum/anchor");
const assert = require("assert");

const {
    TOKEN_PROGRAM_ID,
    sleep,
    getTokenAccount,
    createMint,
    createTokenAccount,
    mintToAccount,
} = require("./utils");

describe("ico-platform", () => {
    const provider = anchor.Provider.local();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.IcoPlatform;

    const nativeIcoAmount = new anchor.BN(5000000);

    let usdcMint = null;
    let nativeMint = null;
    let creatorUsdc = null;
    let creatorNative = null;

    it("Initializes the state-of-the-world", async () => {
        usdcMint = await createMint(provider);
        nativeMint = await createMint(provider);
        creatorUsdc = await createTokenAccount(
            provider,
            usdcMint,
            provider.wallet.publicKey
        );
        creatorNative = await createTokenAccount(
            provider,
            nativeMint,
            provider.wallet.publicKey
        );
        await mintToAccount(
            provider,
            nativeMint,
            creatorNative,
            nativeIcoAmount,
            provider.wallet.publicKey
        );
        creator_native_account = await getTokenAccount(provider, creatorNative);
        assert.ok(creator_native_account.amount.eq(nativeIcoAmount));
    });

    let poolSigner = null;
    let redeemableMint = null;
    let poolNative = null;
    let poolUsdc = null;
    let poolAccount = null;

    let startIcoTs = null;
    let endIcoTs = null;

    it("Initializes the ICO Pool", async () => {
        const [_poolSigner, nonce] =
            await anchor.web3.PublicKey.findProgramAddress(
                [nativeMint.toBuffer()],
                program.programId
            );
        poolSigner = _poolSigner;
        // console.log(poolSigner.toString());

        redeemableMint = await createMint(provider, poolSigner);
        poolNative = await createTokenAccount(provider, nativeMint, poolSigner);
        poolUsdc = await createTokenAccount(provider, usdcMint, poolSigner);

        poolAccount = anchor.web3.Keypair.generate();
        const nowBn = new anchor.BN(Date.now() / 1000);
        startIcoTs = nowBn.add(new anchor.BN(5));
        endIcoTs = nowBn.add(new anchor.BN(15));
        withDrawTs = nowBn.add(new anchor.BN(19));

        await program.rpc.initializePool(
            nativeIcoAmount,
            nonce,
            startIcoTs,
            endIcoTs,
            withDrawTs,
            {
                accounts: {
                    poolAccount: poolAccount.publicKey,
                    poolSigner,
                    distributionAuthority: provider.wallet.publicKey,
                    payer: provider.wallet.publicKey,
                    creatorNative,
                    redeemableMint,
                    usdcMint,
                    nativeMint,
                    poolNative,
                    poolUsdc,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [poolAccount],
            }
        );
        creator_native_account = await getTokenAccount(provider, creatorNative);
        assert.ok(creator_native_account.amount.eq(new anchor.BN(0)));
    });

    let userUsdc = null;
    let userRedeemable = null;

    const firstDeposit = new anchor.BN(10);

    it("Exchanges user USDC for redeemable tokens", async () => {
        if (Date.now() < startIcoTs.toNumber() * 1000) {
            await sleep(startIcoTs.toNumber() * 1000 - Date.now() + 1000);
        }

        userUsdc = await createTokenAccount(
            provider,
            usdcMint,
            provider.wallet.publicKey
        );
        await mintToAccount(
            provider,
            usdcMint,
            userUsdc,
            firstDeposit,
            provider.wallet.publicKey
        );
        userRedeemable = await createTokenAccount(
            provider,
            redeemableMint,
            provider.wallet.publicKey
        );

        try {
            const tx = await program.rpc.exchangeUsdcForRedeemable(
                firstDeposit,
                {
                    accounts: {
                        poolAccount: poolAccount.publicKey,
                        poolSigner,
                        redeemableMint,
                        poolUsdc,
                        userAuthority: provider.wallet.publicKey,
                        userUsdc,
                        userRedeemable,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                    },
                }
            );
        } catch (err) {
            console.log("This is the error message", err.toString());
        }
        poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
        assert.ok(poolUsdcAccount.amount.eq(firstDeposit));
        userRedeemableAccount = await getTokenAccount(provider, userRedeemable);
        assert.ok(userRedeemableAccount.amount.eq(firstDeposit));
    });

    const secondDeposit = new anchor.BN(23);
    let totalPoolUsdc = null;

    it("Exchanges a second users USDC for redeemable tokens", async () => {
        secondUserUsdc = await createTokenAccount(
            provider,
            usdcMint,
            provider.wallet.publicKey
        );
        await mintToAccount(
            provider,
            usdcMint,
            secondUserUsdc,
            secondDeposit,
            provider.wallet.publicKey
        );
        secondUserRedeemable = await createTokenAccount(
            provider,
            redeemableMint,
            provider.wallet.publicKey
        );

        await program.rpc.exchangeUsdcForRedeemable(secondDeposit, {
            accounts: {
                poolAccount: poolAccount.publicKey,
                poolSigner,
                redeemableMint,
                poolUsdc,
                userAuthority: provider.wallet.publicKey,
                userUsdc: secondUserUsdc,
                userRedeemable: secondUserRedeemable,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
        });

        totalPoolUsdc = firstDeposit.add(secondDeposit);
        poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
        assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
        secondUserRedeemableAccount = await getTokenAccount(
            provider,
            secondUserRedeemable
        );
        assert.ok(secondUserRedeemableAccount.amount.eq(secondDeposit));
    });

    it("Exchanges user Redeemable tokens for native", async () => {
        if (Date.now() < withDrawTs.toNumber() * 1000) {
            await sleep(withDrawTs.toNumber() * 1000 - Date.now() + 2000);
        }

        userNative = await createTokenAccount(
            provider,
            nativeMint,
            provider.wallet.publicKey
        );

        await program.rpc.exchangeRedeemableForNative(firstDeposit, {
            accounts: {
                poolAccount: poolAccount.publicKey,
                poolSigner,
                redeemableMint,
                poolNative,
                userAuthority: provider.wallet.publicKey,
                userNative,
                userRedeemable,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
        });

        poolNativeAccount = await getTokenAccount(provider, poolNative);
        let redeemedNative = firstDeposit
            .mul(nativeIcoAmount)
            .div(totalPoolUsdc);
        let remainingNative = nativeIcoAmount.sub(redeemedNative);
        assert.ok(poolNativeAccount.amount.eq(remainingNative));
        userNativeAccount = await getTokenAccount(provider, userNative);
        assert.ok(userNativeAccount.amount.eq(redeemedNative));
    });

    it("Exchanges second users Redeemable tokens for native", async () => {
        secondUserNative = await createTokenAccount(
            provider,
            nativeMint,
            provider.wallet.publicKey
        );

        await program.rpc.exchangeRedeemableForNative(secondDeposit, {
            accounts: {
                poolAccount: poolAccount.publicKey,
                poolSigner,
                redeemableMint,
                poolNative,
                userAuthority: provider.wallet.publicKey,
                userNative: secondUserNative,
                userRedeemable: secondUserRedeemable,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
        });

        poolNativeAccount = await getTokenAccount(provider, poolNative);
        assert.ok(poolNativeAccount.amount.eq(new anchor.BN(0)));
        secondUserNativeAccount = await getTokenAccount(
            provider,
            secondUserNative
        );
    });

    it("Withdraws total USDC from pool account", async () => {
        const acc = await getTokenAccount(provider, poolUsdc);
        await program.rpc.withdrawPoolUsdc(new anchor.BN(acc.amount), {
            accounts: {
                poolAccount: poolAccount.publicKey,
                poolSigner,
                distributionAuthority: provider.wallet.publicKey,
                creatorUsdc,
                poolUsdc,
                payer: provider.wallet.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
        });

        poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
        assert.ok(poolUsdcAccount.amount.eq(new anchor.BN(0)));
        creatorUsdcAccount = await getTokenAccount(provider, creatorUsdc);
        assert.ok(creatorUsdcAccount.amount.eq(totalPoolUsdc));
    });
});
