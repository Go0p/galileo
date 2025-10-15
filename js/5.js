"use strict";
(self.webpackChunk_N_E = self.webpackChunk_N_E || []).push([[3659], {
    998: (e, t, n) => {
        n.d(t, {
            e: () => p,
            q: () => f
        });
        var r = n(48876)
          , a = n(76013)
          , i = n(26432)
          , s = n(55436)
          , o = n(82945)
          , l = n(93284)
          , c = n(72873)
          , u = n(93739)
          , d = n(59271);
        let m = (0,
        i.createContext)(void 0)
          , p = e => {
            let {children: t} = e
              , {isConnected: n, walletAddress: p} = (0,
            l.j)()
              , {appConfig: f} = (0,
            u.A)()
              , g = (0,
            a.jE)()
              , h = (0,
            i.useMemo)( () => {
                let e = (null == f ? void 0 : f.TX_HISTORY_WSS_URL) || "";
                return "".concat(e, "?wallet=").concat(p)
            }
            , [f, p]);
            (0,
            i.useEffect)( () => {
                try {
                    c.N.initialize(h, g)
                } catch (e) {
                    d.R.warn("[TxHistoryProvider] Initialize error", e)
                }
            }
            , [g, h, p]);
            let y = function() {
                let {isConnected: e, walletAddress: t} = (0,
                l.j)()
                  , n = {
                    swaps: [],
                    summary: {
                        totalRequested: 0,
                        totalProcessed: 0,
                        swapsFound: 0,
                        validSwaps: 0,
                        deepSearch: !1,
                        hasMore: !1,
                        oldestTimestamp: 0,
                        oldestDate: ""
                    },
                    hasMore: !1,
                    count: 0
                };
                return (0,
                s.I)({
                    queryKey: t ? o.l.txHistory.user(t) : o.l.txHistory.user(""),
                    queryFn: async () => n,
                    enabled: e && !!t,
                    staleTime: 1 / 0,
                    gcTime: 3e5,
                    refetchOnMount: !1,
                    refetchOnWindowFocus: !1,
                    refetchOnReconnect: !1
                })
            }();
            (0,
            i.useEffect)( () => {
                if (n && p)
                    return c.N.start(p),
                    () => {
                        c.N.stop()
                    }
                    ;
                c.N.stop()
            }
            , [n, p]);
            let w = (0,
            i.useMemo)( () => ({
                history: y.data,
                isLoading: y.isLoading,
                isError: y.isError,
                error: y.error || null,
                refetch: y.refetch
            }), [y.data, y.isLoading, y.isError, y.error, y.refetch]);
            return (0,
            r.jsx)(m.Provider, {
                value: w,
                children: t
            })
        }
          , f = () => {
            let e = (0,
            i.useContext)(m);
            if (!e)
                throw Error("useTransactionHistory must be used within TransactionHistoryProvider");
            return e
        }
    }
    ,
    1187: (e, t, n) => {
        n.d(t, {
            A: () => s
        });
        var r = n(76013)
          , a = n(82945)
          , i = n(25465);
        let s = () => {
            let e = (0,
            r.jE)()
              , t = t => {
                let n = a.l.prices.single(t)
                  , r = e.getQueryData(n);
                return null != r ? r : 0
            }
              , n = async t => {
                if (t && t.length > 0) {
                    let n = a.l.prices.multiple(t);
                    await e.invalidateQueries({
                        queryKey: n
                    })
                } else
                    await e.invalidateQueries({
                        queryKey: a.l.prices.all
                    })
            }
              , s = async () => {
                await n(i.Tt)
            }
            ;
            return {
                getTokenPrice: t,
                getTokenPrices: e => {
                    let n = {};
                    return e.forEach(e => {
                        n[e] = t(e)
                    }
                    ),
                    n
                }
                ,
                refreshPrices: n,
                refreshCommonPrices: s
            }
        }
    }
    ,
    2642: (e, t, n) => {
        n.d(t, {
            Y: () => s
        });
        var r = n(55436)
          , a = n(82945)
          , i = n(3913);
        let s = function(e) {
            var t;
            let n = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {enabled: s=!0, staleTime: o=3e4, gcTime: l=3e5, refetchInterval: c=!1} = n
              , u = (0,
            r.I)({
                queryKey: a.l.prices.single(e),
                queryFn: () => (0,
                i.lg)(e),
                enabled: s && !!e,
                staleTime: o,
                gcTime: l,
                refetchInterval: c,
                refetchIntervalInBackground: !1,
                refetchOnWindowFocus: !0,
                refetchOnMount: !0,
                refetchOnReconnect: !0,
                retry: (e, t) => !(e >= 2),
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4),
                placeholderData: 0
            })
              , d = null != (t = u.data) ? t : 0
              , m = u.dataUpdatedAt ? new Date(u.dataUpdatedAt) : null;
            return {
                price: d,
                isLoading: u.isLoading || u.isFetching,
                isError: u.isError,
                lastUpdated: m,
                hasPrice: d > 0
            }
        }
    }
    ,
    3242: (e, t, n) => {
        n.d(t, {
            s: () => s
        });
        var r = n(55436)
          , a = n(82945);
        async function i() {
            let e = await fetch("/api/geo-blocking/status", {
                method: "GET",
                headers: {
                    "Content-Type": "application/json"
                },
                cache: "no-cache"
            });
            if (!e.ok)
                throw Error((await e.json().catch( () => ({}))).error || "Failed to fetch geo-blocking status");
            return e.json()
        }
        function s() {
            return (0,
            r.I)({
                queryKey: a.l.geoBlocking.status(),
                queryFn: i,
                staleTime: 0,
                gcTime: 0,
                refetchOnWindowFocus: !1,
                refetchOnReconnect: !1,
                retry: 1,
                refetchInterval: !1
            })
        }
    }
    ,
    3913: (e, t, n) => {
        n.d(t, {
            ib: () => a,
            lg: () => i
        });
        var r = n(59271);
        async function a(e) {
            try {
                if (!e || 0 === e.length)
                    return {};
                let t = await fetch("/api/prices", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        addresses: e
                    })
                });
                if (!t.ok)
                    return r.R.warn("Token prices request failed: ".concat(t.status)),
                    s(e);
                let n = await t.json()
                  , a = {};
                return e.forEach(e => {
                    let t = n[e];
                    t && !isNaN(parseFloat(t)) ? a[e] = parseFloat(t) : a[e] = 0
                }
                ),
                a
            } catch (t) {
                return r.R.warn("Failed to fetch token prices:", t),
                s(e)
            }
        }
        async function i(e) {
            try {
                var t;
                return null != (t = (await a([e]))[e]) ? t : 0
            } catch (t) {
                return r.R.warn("Failed to fetch price for token ".concat(e, ":"), t),
                0
            }
        }
        function s(e) {
            let t = {};
            return e.forEach(e => {
                t[e] = 0
            }
            ),
            t
        }
    }
    ,
    5464: (e, t, n) => {
        n.d(t, {
            v: () => Y
        });
        var r = n(40476)
          , a = n(76535)
          , i = n(18586)
          , s = n(40359)
          , o = n.n(s)
          , l = n(26432)
          , c = n(37998)
          , u = n(88368)
          , d = n(50501)
          , m = n(55436)
          , p = n(82945)
          , f = n(43141)
          , g = n(41313)
          , h = n(63257)
          , y = n(55210);
        let w = () => {
            let {settings: e} = (0,
            g.t0)()
              , {sellAmount: t, receiveAmount: n} = (0,
            y.j)()
              , r = (0,
            l.useMemo)( () => {
                if (e)
                    if (e.isPrimeMode || "mev-protect" === e.txFeeSettings.broadcastMode)
                        return "mev-protect";
                    else
                        return "priority-fee"
            }
            , [e])
              , a = (0,
            l.useMemo)( () => {
                var t, n;
                if (e) {
                    if ((null == e || null == (t = e.txFeeSettings) ? void 0 : t.feeMode) === "auto" && "priority-fee" === r)
                        return f.tS[e.txFeeSettings.priorityFee];
                    if (e.isPrimeMode)
                        return null == e || null == (n = e.primeFeeData) ? void 0 : n.percentile;
                    if ("mev-protect" === e.txFeeSettings.broadcastMode)
                        return e.txFeeSettings.mevTipPercentile
                }
            }
            , [r, e]);
            return (0,
            m.I)({
                queryKey: p.l.fees.percentile("mev-protect" === r ? "jito" : "priority", a),
                queryFn: async () => {
                    if (!a)
                        throw Error("Missing percentile");
                    if (!r)
                        throw Error("Missing provider");
                    return (0,
                    h.h4)("mev-protect" === r ? "jito" : "priority", a)
                }
                ,
                enabled: !!(t && a && r && n),
                refetchInterval: 1e4,
                refetchIntervalInBackground: !0,
                staleTime: 5e3,
                gcTime: 3e5,
                refetchOnWindowFocus: !0,
                refetchOnMount: !0,
                refetchOnReconnect: !0,
                retry: (e, t) => !(e >= 2),
                retryDelay: e => Math.min(1e3 * 2 ** e, 1e4),
                placeholderData: null
            }).data
        }
        ;
        var v = n(47828)
          , b = n(1187)
          , S = n(62500)
          , k = n(20956)
          , x = n(30369)
          , T = n(67725)
          , R = n(59271);
        async function E(e) {
            try {
                let t = await M(e);
                R.R.debug("Analytics data:", t);
                let n = await fetch("/api/analytics", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify(t),
                    signal: AbortSignal.timeout(1e4)
                });
                if (!n.ok) {
                    R.R.warn("Analytics request failed: ".concat(n.status, ", sending fallback data")),
                    await A(e, "HTTP_".concat(n.status, "_").concat(n.statusText));
                    return
                }
                let r = await n.json();
                r.success ? R.R.info("Analytics data sent successfully") : (R.R.warn("Analytics processing failed: ".concat(r.error, ", sending fallback data")),
                await A(e, "PROCESSING_ERROR: ".concat(r.error)))
            } catch (t) {
                t instanceof Error && t.message.includes("Missing") ? (R.R.warn("Analytics parsing failed: ".concat(t.message, ", sending fallback data")),
                await A(e, "PARSING_ERROR: ".concat(t.message))) : (R.R.warn("Failed to send analytics data: ".concat(t, ", sending fallback data")),
                await A(e, "NETWORK_ERROR: ".concat(t instanceof Error ? t.message : "Unknown error")))
            }
        }
        async function A(e, t) {
            try {
                let n = function(e, t) {
                    let {signature: n, userPublicKey: r, quoteParams: a, txError: i} = e;
                    return {
                        fromToken: a ? {
                            address: a.inputMint,
                            symbol: "UNKNOWN",
                            name: "Unknown Token",
                            decimals: 0,
                            verified: !1
                        } : null,
                        fromTokenUSD: null,
                        toToken: a ? {
                            address: a.outputMint,
                            symbol: "UNKNOWN",
                            name: "Unknown Token",
                            decimals: 0,
                            verified: !1
                        } : null,
                        toTokenUSD: null,
                        signedTransaction: null,
                        transactionResponse: null,
                        user: r || null,
                        quoteId: "fallback_".concat(Date.now()),
                        isPrime: (null == a ? void 0 : a.isPrimeMode) || !1,
                        hasExcludedAmms: e.hasExcludedAmms || null,
                        txError: "ANALYTICS_ERROR: ".concat(t),
                        quoteRequest: null,
                        simulationResult: null,
                        appVersion: "3.5.12"
                    }
                }(e, t);
                R.R.info("Sending fallback analytics data:", {
                    signature: e.signature,
                    userPublicKey: e.userPublicKey,
                    error: t
                });
                let r = await fetch("/api/analytics", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify(n),
                    signal: AbortSignal.timeout(5e3)
                });
                if (!r.ok)
                    return void R.R.error("Fallback analytics also failed: ".concat(r.status));
                let a = await r.json();
                a.success ? R.R.info("Fallback analytics data sent successfully") : R.R.error("Fallback analytics processing failed: ".concat(a.error))
            } catch (e) {
                R.R.error("Fallback analytics completely failed:", e)
            }
        }
        async function M(e) {
            var t;
            let n, a, {signature: i, transactionResponse: s, signedTransaction: o, serializedSignedTransaction: l, quoteParams: c, buildResult: u, userPublicKey: d, txError: m, quotes: p, fromTokenPrice: f, toTokenPrice: g, hasExcludedAmms: h} = e;
            try {
                let e = await (0,
                T.p4)([c.inputMint, c.outputMint]);
                n = e[c.outputMint],
                a = e[c.inputMint]
            } catch (e) {
                R.R.warn("Failed to fetch toToken metadata:", e)
            }
            let y = p.id
              , w = function(e, t, n, r, a) {
                let i = Object.entries(n.quotes).reduce( (t, n) => {
                    var a, i, s, o, l, c;
                    let[u,d] = n;
                    return t[u.toLowerCase()] = {
                        user: r,
                        request: {
                            amount: (null == e ? void 0 : e.amount) || 0,
                            swapMode: String(null == e ? void 0 : e.swapMode),
                            inputMint: (null == e || null == (a = e.inputMint) ? void 0 : a.toString()) || "",
                            outputMint: (null == e || null == (i = e.outputMint) ? void 0 : i.toString()) || "",
                            slippageBps: null != (s = d.slippageBps) ? s : -100,
                            excludeDexes: null != (o = null == e ? void 0 : e.excludeDexes) ? o : [],
                            includeDexes: null != (l = null == e ? void 0 : e.includeDexes) ? l : [],
                            onlyDirectRoutes: null != (c = null == e ? void 0 : e.onlyDirectRoutes) && c
                        }
                    },
                    t
                }
                , {});
                return {
                    quoteId: a,
                    timestamp: Date.now(),
                    aggregators: i
                }
            }(c, null == u || u.feeParams, p, d, y);
            R.R.debug("Analytics quoteRequest", w);
            let v = (t = p) ? Object.entries(t.quotes).reduce( (e, n) => {
                var a, i, s;
                let[o,l] = n
                  , c = l.steps.map(e => {
                    var t;
                    return {
                        percent: null !== e.allocPpb ? new x.A(e.allocPpb).dividedBy(1e9).times(100).toNumber() : null,
                        swapInfo: {
                            label: null != (t = e.label) ? t : null,
                            ammKey: e.ammKey ? new r.PublicKey(e.ammKey).toString() : null,
                            inAmount: e.inAmount ? e.inAmount.toString() : null,
                            inputMint: e.inputMint ? new r.PublicKey(e.inputMint).toString() : null,
                            outAmount: e.outAmount ? e.outAmount.toString() : null,
                            outputMint: e.outputMint ? new r.PublicKey(e.outputMint).toString() : null,
                            feeMint: e.feeMint ? new r.PublicKey(e.feeMint).toString() : null,
                            feeAmount: e.feeAmount ? e.feeAmount.toString() : null
                        }
                    }
                }
                )
                  , u = {
                    inAmount: l.inAmount,
                    swapMode: String(t.swapMode),
                    inputMint: new r.PublicKey(t.inputMint).toBase58(),
                    outAmount: l.outAmount,
                    routePlan: c,
                    timeTaken: l.timeTakenNs ? new x.A(l.timeTakenNs).dividedBy(1e6).toNumber() : 0,
                    outputMint: new r.PublicKey(t.outputMint).toBase58(),
                    contextSlot: l.contextSlot,
                    platformFee: null != (i = null == (a = l.platformFee) ? void 0 : a.amount) ? i : null,
                    slippageBps: l.slippageBps,
                    priceImpactPct: "0",
                    otherAmountThreshold: "0"
                };
                return e[o.toLowerCase()] = {
                    quote: u,
                    exchange: o.toLowerCase(),
                    outAmount: l.outAmount,
                    simulationResult: {
                        quote: u,
                        exchange: o.toLowerCase(),
                        outAmount: l.outAmount,
                        logs: [],
                        computeUnits: null != (s = l.computeUnits) ? s : 0,
                        outputAmount: l.outAmount,
                        instructionIndex: 0,
                        error: null
                    }
                },
                e
            }
            , {}) : {};
            return {
                fromToken: a,
                fromTokenUSD: f,
                toToken: n,
                toTokenUSD: g,
                signedTransaction: function(e, t, n, r) {
                    let a = {
                        signature: t || "",
                        signedTransaction: (null == e ? void 0 : e.transaction) || null,
                        lastValidBlockHeight: (null == e ? void 0 : e.lastValidBlockHeight) || null,
                        serializedTransaction: r || null
                    };
                    return a.signedTransaction && (a.signedTransaction.signatures = (null == n ? void 0 : n.signatures) || []),
                    a
                }(u, i, o, l),
                transactionResponse: s || null,
                user: d || null,
                quoteId: y,
                isPrime: c.isPrimeMode,
                hasExcludedAmms: h,
                txError: m,
                quoteRequest: w,
                simulationResult: v,
                appVersion: "3.5.12"
            }
        }
        var C = n(73656)
          , N = n(47337)
          , P = n(31001)
          , I = n(15334)
          , O = n(96055)
          , F = n(73861);
        let j = [...h.ED, ...h.Q2].reduce( (e, t) => ({
            ...e,
            [t]: !0
        }), {})
          , L = "So11111111111111111111111111111111111111112"
          , q = "CRhtqXk98ATqo1R8gLg7qcpEMuvoPzqD5GNicPPqLMD"
          , B = "sponsorKDrY6B1TXJQ5GKUdvGNSbSKRsW8UxGp82Q5Q"
          , D = e => (e[0] | e[1] << 8 | e[2] << 16 | e[3] << 24) >>> 0
          , U = e => BigInt(e[0]) | BigInt(e[1]) << BigInt(8) | BigInt(e[2]) << BigInt(16) | BigInt(e[3]) << BigInt(24) | BigInt(e[4]) << BigInt(32) | BigInt(e[5]) << BigInt(40) | BigInt(e[6]) << BigInt(48) | BigInt(e[7]) << BigInt(56);
        function z(e, t) {
            let n = new x.A(String(e))
              , r = x.A.pow(10, -t);
            return n.mul(r).toNumber()
        }
        async function _(e) {
            let {allTokens: t, setAllTokens: n} = F.A.getState()
              , r = t.find(t => t.address === e);
            if (r)
                return r;
            let a = await (0,
            T.p4)([e]);
            if (1 == Object.values(a).length) {
                let e = Object.values(a)[0];
                return n([...t, e]),
                e
            }
        }
        let K = (e, t, n) => {
            let i = 0
              , s = e.transaction.message.compiledInstructions.some(e => n[e.programIdIndex].toString() === q)
              , o = e.transaction.message.compiledInstructions.some(e => "T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT" === n[e.programIdIndex].toString());
            return s || o ? (e.transaction.message.compiledInstructions.forEach( (s, o) => {
                let l = n[s.programIdIndex].toString();
                if (l === q || "T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT" === l) {
                    var c, u;
                    let s = null == (u = e.meta) || null == (c = u.innerInstructions) ? void 0 : c.find(e => e.index === o);
                    if (s) {
                        let o = []
                          , l = [];
                        s.instructions.forEach( (i, s) => {
                            if (n[i.programIdIndex].equals(r.SystemProgram.programId))
                                try {
                                    let r = a.default.decode(i.data)
                                      , y = new DataView(r.buffer).getUint32(0, !0);
                                    if ((0 === y || 2 === y) && i.accounts.length >= 2) {
                                        var c, u, d, m, p, f, g, h;
                                        let a = n[i.accounts[0]]
                                          , w = n[i.accounts[1]];
                                        if (0 === y) {
                                            if (a.equals(t)) {
                                                let t = Number(new DataView(r.buffer).getBigUint64(4, !0))
                                                  , n = i.accounts[1]
                                                  , a = (null == (u = e.meta) || null == (c = u.preBalances) ? void 0 : c[n]) || 0
                                                  , l = (null == (m = e.meta) || null == (d = m.postBalances) ? void 0 : d[n]) || 0;
                                                0 === a && l > 0 && o.push({
                                                    account: w.toString(),
                                                    cost: t,
                                                    innerIndex: s
                                                })
                                            }
                                        } else if (2 === y) {
                                            let n = Number(new DataView(r.buffer).getBigUint64(4, !0));
                                            if (a.equals(t)) {
                                                let t = i.accounts[1]
                                                  , r = (null == (f = e.meta) || null == (p = f.preBalances) ? void 0 : p[t]) || 0
                                                  , a = (null == (h = e.meta) || null == (g = h.postBalances) ? void 0 : g[t]) || 0;
                                                0 === r && a > 0 && o.push({
                                                    account: w.toString(),
                                                    cost: n,
                                                    innerIndex: s
                                                })
                                            } else
                                                w.equals(t) && l.push({
                                                    amount: n,
                                                    source: a.toString(),
                                                    innerIndex: s
                                                })
                                        }
                                    }
                                } catch (e) {
                                    console.warn("Failed to decode inner instruction data:", e)
                                }
                        }
                        ),
                        o.forEach(e => {
                            l.find(t => t.amount === e.cost) && (i += e.cost)
                        }
                        )
                    }
                }
            }
            ),
            i) : 0
        }
          , W = (e, t, n) => {
            let r = 0;
            return e.transaction.message.compiledInstructions.some(e => n[e.programIdIndex].toString() === B) ? (e.transaction.message.compiledInstructions.forEach( (a, i) => {
                if (n[a.programIdIndex].toString() === B && a.accountKeyIndexes.length >= 2) {
                    let i = a.accountKeyIndexes[0]
                      , u = a.accountKeyIndexes[1];
                    if (n[i].equals(t)) {
                        var s, o, l, c;
                        let t = (null == (o = e.meta) || null == (s = o.preBalances) ? void 0 : s[u]) || 0
                          , n = ((null == (c = e.meta) || null == (l = c.postBalances) ? void 0 : l[u]) || 0) - t;
                        if (n < 0) {
                            let e = Math.abs(n);
                            R.R.debug("Sponsored transaction detected: ".concat(e, " lamports reimbursed")),
                            r += e
                        }
                    }
                }
            }
            ),
            r > 0 && R.R.debug("Total sponsored transaction reimbursements: ".concat(r, " lamports")),
            r) : 0
        }
          , V = (e, t) => {
            var n, i, s, o, l, c, u, d, m, p, f, g, h, y, w, v, b, S, k, x, T, R, E, A, M, C, N, P, I, F, q, B, z, _, V, H, G, Q, J, Y;
            let X, Z, $ = new r.PublicKey(t.user), ee = null != (m = null == e || null == (n = e.meta) ? void 0 : n.fee) ? m : 0, et = [...null != (p = null == (i = e.transaction) ? void 0 : i.message.staticAccountKeys) ? p : [], ...null != (f = null == e || null == (o = e.meta) || null == (s = o.loadedAddresses) ? void 0 : s.writable) ? f : [], ...null != (g = null == e || null == (c = e.meta) || null == (l = c.loadedAddresses) ? void 0 : l.readonly) ? g : []], en = K(e, $, et), er = W(e, $, et), ea = null != (y = null != (h = t.tipAmountLamports) ? h : et.map( (t, n) => {
                if (j[t.toString()]) {
                    var r, a, i, s;
                    let t = null != (i = null == e || null == (r = e.meta) ? void 0 : r.preBalances[n]) ? i : 0;
                    return (null != (s = null == e || null == (a = e.meta) ? void 0 : a.postBalances[n]) ? s : 0) - t
                }
                return 0
            }
            ).reduce( (e, t) => e + t, 0)) ? y : 0, ei = {}, es = 0;
            null == e || e.transaction.message.compiledInstructions.forEach( (t, n) => {
                var i, s;
                let o = et[t.programIdIndex]
                  , l = t.data
                  , c = t.accountKeyIndexes.map(e => et[e])
                  , u = null == e || null == (s = e.meta) || null == (i = s.innerInstructions) ? void 0 : i.find(e => e.index === n);
                if (o.equals(r.SystemProgram.programId)) {
                    let e = D([...l.slice(0, 4)])
                      , t = c.at(1);
                    if (0 === e && t) {
                        let e = U([...l.slice(4, 12)]);
                        ei[t.toString()] = Number(e)
                    }
                } else if ((o.equals(O.x5) || o.equals(O.sy)) && 9 === l.at(0)) {
                    let e = c[0];
                    c[1].equals($) && (ei[e.toString()] ? delete ei[e.toString()] : es -= 2039280)
                } else
                    null == u || u.instructions.filter(e => {
                        if (e.accounts.length < 2)
                            return !1;
                        let t = et[e.accounts[0]];
                        return et[e.programIdIndex].equals(r.SystemProgram.programId) && t.equals($)
                    }
                    ).forEach(e => {
                        let t = a.default.decode(e.data)
                          , n = D([...t.slice(0, 4)])
                          , r = et.at(e.accounts[1]);
                        if (0 === n && r) {
                            let e = Number(U([...t.slice(4, 12)]));
                            ei[r.toString()] = e
                        }
                    }
                    ),
                    null == u || u.instructions.filter(e => {
                        if (e.accounts.length < 3)
                            return !1;
                        let t = et[e.accounts[1]]
                          , n = et[e.programIdIndex];
                        return (n.equals(O.x5) || n.equals(O.sy)) && t.equals($)
                    }
                    ).forEach(e => {
                        let t = a.default.decode(e.data)
                          , n = et.at(e.accounts[0]);
                        9 === t.at(0) && n && (ei[n.toString()] ? delete ei[n.toString()] : es += 2039280)
                    }
                    )
            }
            ),
            es += Object.values(ei).reduce( (e, t) => e + t, 0);
            let eo = null != (w = null == e || null == (u = e.meta) ? void 0 : u.preBalances[0]) ? w : 0
              , el = null != (v = null == e || null == (d = e.meta) ? void 0 : d.postBalances[0]) ? v : 0
              , ec = 0
              , eu = 0
              , ed = t.sellToken;
            if (t.inputMint === L)
                ec = 9,
                X = ((eo - el - ee - ea - es + en + er) / r.LAMPORTS_PER_SOL).toFixed(9);
            else {
                let n = e => e.owner === $.toString() && e.mint === t.inputMint
                  , r = null == e || null == (S = e.meta) || null == (b = S.preTokenBalances) ? void 0 : b.find(n);
                ec = null != (C = null == r || null == (k = r.uiTokenAmount) ? void 0 : k.decimals) ? C : 0;
                let a = null != (N = null == e || null == (R = e.meta) || null == (T = R.preTokenBalances) || null == (x = T.find(n)) ? void 0 : x.uiTokenAmount.uiAmount) ? N : 0
                  , i = null != (P = null == e || null == (M = e.meta) || null == (A = M.postTokenBalances) || null == (E = A.find(n)) ? void 0 : E.uiTokenAmount.uiAmount) ? P : 0;
                X = (Number(a) - Number(i)).toFixed(ec)
            }
            if (t.outputMint === L)
                eu = 9,
                Z = ((el - eo + ee + ea + es - en - er) / r.LAMPORTS_PER_SOL).toFixed(9);
            else {
                let n = e => e.owner === $.toString() && e.mint === t.outputMint
                  , r = null == (F = e.meta) || null == (I = F.preTokenBalances) ? void 0 : I.find(n);
                eu = null != (Q = null == r || null == (q = r.uiTokenAmount) ? void 0 : q.decimals) ? Q : null == ed ? void 0 : ed.decimals;
                let a = null != (J = null == e || null == (_ = e.meta) || null == (z = _.preTokenBalances) || null == (B = z.find(n)) ? void 0 : B.uiTokenAmount.uiAmount) ? J : 0;
                Z = (Number(null != (Y = null == e || null == (G = e.meta) || null == (H = G.postTokenBalances) || null == (V = H.find(n)) ? void 0 : V.uiTokenAmount.uiAmount) ? Y : 0) - Number(a)).toFixed(eu)
            }
            return {
                inputAmount: X,
                outputAmount: Z,
                inputDecimals: ec,
                outputDecimals: eu,
                addedTip: ea,
                priorityFees: ee
            }
        }
          , H = (e, t) => V(e, t);
        var G = n(91015).Buffer;
        let Q = async function(e, t) {
            let n = arguments.length > 2 && void 0 !== arguments[2] ? arguments[2] : 20
              , r = arguments.length > 3 && void 0 !== arguments[3] ? arguments[3] : 1e3
              , a = arguments.length > 4 && void 0 !== arguments[4] ? arguments[4] : 4e3;
            return await o()(async n => {
                let r = await e.getTransaction(t, {
                    commitment: "confirmed",
                    maxSupportedTransactionVersion: 1
                });
                return r || n(Error("Transaction not yet available")),
                r
            }
            , {
                retries: n,
                minTimeout: r,
                maxTimeout: a
            })
        }
          , J = async e => {
            let {connection: t, txid: n, timeoutMs: r=3e4, pollIntervalMs: a=1e3, commitment: i="confirmed"} = e
              , s = Date.now();
            for (; Date.now() - s < r; ) {
                let e = await t.getSignatureStatuses([n], {
                    searchTransactionHistory: !0
                });
                R.R.info("Transaction signature status:", e);
                let r = e.value[0];
                if (r) {
                    if (r.err) {
                        let e = JSON.stringify(r.err);
                        throw R.R.error("Transaction confirmation error:", r.err),
                        Error("Transaction failed: ".concat(e))
                    }
                    if (r.confirmationStatus === i || "finalized" === r.confirmationStatus)
                        return void R.R.info("Transaction confirmed successfully:", {
                            signature: n,
                            confirmationStatus: r.confirmationStatus,
                            slot: r.slot
                        })
                }
                await new Promise(e => setTimeout(e, a))
            }
            throw Error("Confirmation timeout after ".concat(r, "ms for tx ").concat(n))
        }
          , Y = () => {
            let {connected: e, publicKey: t, disconnect: n, signTransaction: s, connection: o} = (0,
            d.z)();
            (0,
            i.useRouter)();
            let {bestQuote: m, buildTransaction: p, isConnected: f, params: h, quotes: y} = (0,
            k.FF)()
              , {isPrimeMode: x, transactionConfig: T} = (0,
            v.n)()
              , {tokenBalances: A} = (0,
            I.A)()
              , {settings: M} = (0,
            g.t0)()
              , O = w()
              , {refetchWalletStats: F, refetchSponsoredTransactions: j} = (0,
            S.A)()
              , {isTradeGeoBlocked: L} = (0,
            P.A)()
              , {excludedAmmIds: q} = (0,
            N.A)()
              , {getTokenPrice: B} = (0,
            b.A)()
              , D = (0,
            l.useRef)(y)
              , U = (0,
            l.useRef)(h);
            (0,
            l.useEffect)( () => {
                D.current = y
            }
            , [y]),
            (0,
            l.useEffect)( () => {
                U.current = h
            }
            , [h]);
            let[K,W] = (0,
            l.useState)("idle")
              , V = (0,
            l.useMemo)( () => {
                if (!h.inputMint || !h.amount || !A)
                    return R.R.debug("[useSwapExecution] hasSufficientBalance: false - missing params", {
                        inputMint: h.inputMint,
                        amount: h.amount,
                        tokenBalancesLength: null == A ? void 0 : A.length
                    }),
                    !1;
                let e = A.find(e => e.token.address === h.inputMint);
                if (!e)
                    return R.R.debug("[useSwapExecution] hasSufficientBalance: false - input balance not found", {
                        inputMint: h.inputMint,
                        availableTokens: A.map(e => e.token.address)
                    }),
                    !1;
                let t = h.amount / Math.pow(10, e.token.decimals)
                  , n = e.balance >= t;
                return R.R.debug("[useSwapExecution] hasSufficientBalance calculation:", {
                    inputMint: h.inputMint,
                    rawAmount: h.amount,
                    decimals: e.token.decimals,
                    requiredAmount: t,
                    currentBalance: e.balance,
                    hasBalance: n
                }),
                n
            }
            , [h.inputMint, h.amount, A])
              , Y = (0,
            l.useMemo)( () => e && t && f && m && V && "idle" === K, [e, t, f, m, V, K])
              , X = (0,
            l.useCallback)(async () => {
                try {
                    await n()
                } catch (e) {}
            }
            , [n])
              , Z = (0,
            l.useCallback)(async (e, n, i, l, u) => {
                let d = null
                  , m = null
                  , p = n.route.referenceId
                  , f = null
                  , g = null
                  , h = null;
                W("building");
                try {
                    var y;
                    g = r.VersionedTransaction.deserialize(e),
                    R.R.info("Versioned pyth transaction: ", g),
                    W("signing"),
                    d = await s(g),
                    R.R.info("Signed pyth transaction: ", d),
                    W("sending"),
                    h = G.from(d.serialize()).toString("base64");
                    let w = (e, t) => {
                        let n = "signature"in e ? e.signature : e.signatures[null != t ? t : 0];
                        if (!n)
                            throw Error("Missing transaction signature, the transaction was not signed by the fee payer");
                        return a.default.encode(n)
                    }
                      , v = w(d, 1);
                    R.R.info("User signature: ", v);
                    let b = {
                        referenceId: p,
                        userSignature: v
                    };
                    R.R.info("Sending transaction to Pyth endpoint:", b),
                    R.R.debug("signedTransactionEncoded", h);
                    let S = await fetch("/api/pyth/submit", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json"
                        },
                        body: JSON.stringify(b)
                    });
                    if (!S.ok)
                        throw Error("Failed to submit transaction to Pyth endpoint");
                    let k = await S.json();
                    R.R.info("Response from Pyth endpoint:", k),
                    f = w(d, 0),
                    W("confirming"),
                    await J({
                        connection: o,
                        txid: f,
                        timeoutMs: 3e4,
                        pollIntervalMs: 1e3,
                        commitment: "confirmed"
                    }),
                    R.R.info("Transaction confirmation completed:", {
                        signature: f
                    }),
                    m = await Q(o, f, 5, 1e3, 2e3),
                    E({
                        signedTransaction: d,
                        signature: f,
                        transactionResponse: m,
                        serializedSignedTransaction: h,
                        quoteParams: l,
                        userPublicKey: t.toBase58(),
                        quotes: i,
                        toTokenPrice: B(l.outputMint),
                        fromTokenPrice: B(l.inputMint),
                        hasExcludedAmms: u
                    });
                    let x = n.route.steps[0].inAmount
                      , T = n.route.steps[0].outAmount
                      , A = n.route.steps[0].inputMint
                      , M = n.route.steps[0].outputMint
                      , C = new r.PublicKey(A).toBase58()
                      , N = new r.PublicKey(M).toBase58();
                    if (R.R.info("Swap transaction confirmed (pyth):", {
                        signature: f,
                        provider: n.provider,
                        inAmount: x,
                        outAmount: T,
                        inputMint: C,
                        outputMint: N
                    }),
                    null == (y = m.meta) ? void 0 : y.err) {
                        let e = m.meta.err
                          , t = Object.keys(e)[0]
                          , n = e[t]
                          , r = "".concat(t, ": ").concat(JSON.stringify(n));
                        (0,
                        c.o)({
                            title: "Transaction Failed",
                            description: r,
                            variant: "alert",
                            buttons: [{
                                children: "View Transaction",
                                onClick: () => window.open("https://solscan.io/tx/".concat(f), "_blank"),
                                variant: "secondary"
                            }],
                            duration: 5e3
                        })
                    } else {
                        let e = await _(C)
                          , t = await _(N)
                          , n = z(x, null == e ? void 0 : e.decimals).toString()
                          , r = z(T, null == t ? void 0 : t.decimals).toString();
                        (0,
                        c.o)({
                            title: "Swap successful!",
                            description: "Swapped ".concat(n, " ").concat((null == e ? void 0 : e.symbol) || "", " \n          for ").concat(r, " ").concat((null == t ? void 0 : t.symbol) || ""),
                            variant: "success",
                            buttons: [{
                                children: "View Transaction",
                                onClick: () => window.open("https://solscan.io/tx/".concat(f), "_blank"),
                                variant: "secondary"
                            }]
                        })
                    }
                    return W("idle"),
                    {
                        success: !0,
                        signature: f
                    }
                } catch (a) {
                    W("idle");
                    let e = a instanceof Error ? a.message : "Unknown error";
                    a instanceof Error && (e = "".concat(e, "\n").concat(a.stack));
                    let r = {
                        transaction: g,
                        inAmount: n.route.steps[0].inAmount,
                        outAmount: n.route.steps[0].outAmount,
                        provider: n.provider,
                        steps: n.route.steps
                    };
                    if (E({
                        signedTransaction: d || null,
                        signature: f || "",
                        quoteParams: l || null,
                        transactionResponse: m || null,
                        serializedSignedTransaction: h || null,
                        userPublicKey: (null == t ? void 0 : t.toBase58()) || "",
                        quotes: i,
                        txError: e,
                        buildResult: r || null,
                        toTokenPrice: B(l.outputMint),
                        fromTokenPrice: B(l.inputMint)
                    }),
                    R.R.error("PythExpressRelay execution failed:", e),
                    e.includes("User rejected"))
                        return (0,
                        c.o)({
                            title: "Swap failed",
                            description: "User rejected the request",
                            variant: "alert",
                            duration: 5e3
                        }),
                        {
                            success: !1,
                            error: "User rejected transaction"
                        };
                    return e.includes("slippage") ? (0,
                    c.o)({
                        title: "Slippage exceeded",
                        description: "Try increasing your slippage tolerance in settings.",
                        variant: "alert",
                        duration: 5e3
                    }) : e.includes("insufficient") ? (0,
                    c.o)({
                        title: "Insufficient funds",
                        description: "You don't have enough tokens for this swap.",
                        variant: "alert",
                        duration: 5e3
                    }) : e.includes("No wallet found") ? (0,
                    c.o)({
                        title: "Swap failed",
                        description: "User wallet not found. Please reconnect your wallet and try again.",
                        variant: "alert",
                        duration: 5e3,
                        buttons: [{
                            children: "Disconnect",
                            onClick: async () => {
                                X()
                            }
                            ,
                            variant: "secondary"
                        }]
                    }) : e.includes("WalletSignTransactionError") ? (0,
                    c.o)({
                        title: "Swap failed",
                        description: "Connected wallet failed to sign the transaction.",
                        buttons: [{
                            children: "Disconnect",
                            onClick: async () => {
                                X()
                            }
                            ,
                            variant: "secondary"
                        }],
                        variant: "alert",
                        duration: 5e3
                    }) : (0,
                    c.o)({
                        title: "Swap failed",
                        description: "Error",
                        variant: "alert",
                        duration: 5e3
                    }),
                    {
                        success: !1,
                        error: e
                    }
                } finally {
                    F(),
                    j()
                }
            }
            , [o, B, s, F, j, t, X]);
            return {
                executeSwap: async () => {
                    if (L)
                        return {
                            success: !1,
                            error: "Trade geo blocked"
                        };
                    let e = D.current
                      , n = U.current
                      , r = structuredClone(e)
                      , a = structuredClone(n)
                      , i = (0,
                    u.x_)(r);
                    if (!i)
                        return {
                            success: !1,
                            error: "No valid quotes available"
                        };
                    let l = B(a.outputMint)
                      , d = B(a.inputMint);
                    if (!Y || !t || !s)
                        return {
                            success: !1,
                            error: "Cannot execute swap"
                        };
                    let m = null
                      , f = ""
                      , g = null
                      , h = null
                      , y = null;
                    if ((null == i ? void 0 : i.provider) === "Pyth") {
                        R.R.info("PythExpressRelay transaction detected"),
                        R.R.info("Frozen best quote:", i);
                        let e = null == i ? void 0 : i.route.transaction;
                        return await Z(e, i, r, a, q.length > 0)
                    }
                    try {
                        var w;
                        if (W("building"),
                        R.R.debug("frozenBestQuote", i),
                        !(g = await p(o, i.provider, i.route, O)))
                            throw Error("Failed to build transaction");
                        W("signing"),
                        m = await s(g.transaction),
                        R.R.debug("signedTransaction", m);
                        let e = M.isPrimeMode ? "mev-protect" : M.txFeeSettings.broadcastMode
                          , n = (0,
                        C.Z)(e, g.tipBroadcaster || "none")
                          , u = {
                            sendJitoAsTransaction: n,
                            tipBroadcaster: g.tipBroadcaster
                        };
                        W("sending"),
                        R.R.info("Sending transaction via gateway worker:", {
                            provider: g.provider,
                            broadcastMode: e,
                            isJitoBundle: n,
                            tipBroadcaster: g.tipBroadcaster,
                            transactionSize: g.transactionSize,
                            instructionCount: g.instructionCount
                        }),
                        y = G.from(m.serialize()).toString("base64"),
                        R.R.debug("signedTransactionEncoded", y);
                        let v = await (0,
                        C.Y)({
                            transaction: y,
                            lastValidBlockHeight: g.lastValidBlockHeight || 0,
                            recentBlockhash: g.recentBlockhash || "",
                            broadcastMode: e,
                            isJitoBundle: n,
                            options: u
                        });
                        if (f = v[0],
                        R.R.info("Gateway worker transaction sent:", {
                            signature: f,
                            hashCount: v.length,
                            provider: g.provider
                        }),
                        W("confirming"),
                        R.R.info("Confirming transaction:", {
                            signature: f,
                            lastValidBlockHeight: g.lastValidBlockHeight
                        }),
                        await J({
                            connection: o,
                            txid: f,
                            timeoutMs: 3e4,
                            pollIntervalMs: 1e3,
                            commitment: "confirmed"
                        }),
                        R.R.info("Transaction confirmation completed:", {
                            signature: f
                        }),
                        h = await Q(o, f, 20, 1e3, 4e3),
                        E({
                            signedTransaction: m,
                            signature: f,
                            transactionResponse: h,
                            serializedSignedTransaction: y,
                            quoteParams: a,
                            bestQuote: i,
                            buildResult: g,
                            userPublicKey: t.toBase58(),
                            quotes: r,
                            toTokenPrice: l,
                            fromTokenPrice: d,
                            hasExcludedAmms: q.length > 0
                        }),
                        R.R.info("Swap transaction confirmed:", {
                            signature: f,
                            provider: g.provider,
                            broadcastMode: e,
                            transactionResponse: h
                        }),
                        null == (w = h.meta) ? void 0 : w.err) {
                            let e = h.meta.err
                              , t = Object.keys(e)[0]
                              , n = e[t]
                              , r = "".concat(t, ": ").concat(JSON.stringify(n));
                            (0,
                            c.o)({
                                title: "Transaction Failed",
                                description: r,
                                variant: "alert",
                                buttons: [{
                                    children: "View Transaction",
                                    onClick: () => window.open("https://solscan.io/tx/".concat(f), "_blank"),
                                    variant: "secondary"
                                }],
                                duration: 5e3
                            })
                        } else {
                            let e = await _(a.inputMint)
                              , n = await _(a.outputMint)
                              , {outputAmount: r, inputDecimals: i} = H(h, {
                                inputMint: a.inputMint,
                                outputMint: a.outputMint,
                                user: t.toBase58(),
                                tipAmountLamports: g.tipAmountLamports,
                                sellToken: e
                            })
                              , s = z(g.inAmount, i).toString()
                              , o = Number(r).toString();
                            (0,
                            c.o)({
                                title: "Swap successful!",
                                description: "Swapped ".concat(s, " ").concat((null == e ? void 0 : e.symbol) || "", " \n                          for ").concat(o, " ").concat((null == n ? void 0 : n.symbol) || ""),
                                variant: "success",
                                buttons: [{
                                    children: "View Transaction",
                                    onClick: () => window.open("https://solscan.io/tx/".concat(f), "_blank"),
                                    variant: "secondary"
                                }]
                            })
                        }
                        return W("idle"),
                        {
                            success: !0,
                            signature: f
                        }
                    } catch (n) {
                        W("idle");
                        let e = n instanceof Error ? n.message : "Unknown error";
                        if (n instanceof Error && (e = "".concat(e, "\n").concat(n.stack)),
                        E({
                            signedTransaction: m || null,
                            signature: f || "",
                            quoteParams: a || null,
                            transactionResponse: h || null,
                            serializedSignedTransaction: y || null,
                            userPublicKey: (null == t ? void 0 : t.toBase58()) || "",
                            quotes: r,
                            txError: e,
                            buildResult: g || null,
                            toTokenPrice: B(a.outputMint),
                            fromTokenPrice: B(a.inputMint)
                        }),
                        R.R.error("Swap execution failed:", {
                            error: e,
                            provider: null == i ? void 0 : i.provider,
                            broadcastMode: M.isPrimeMode ? "mev-protect" : M.txFeeSettings.broadcastMode,
                            executionState: K,
                            stack: n instanceof Error ? n.stack : void 0
                        }),
                        e.includes("User rejected"))
                            return (0,
                            c.o)({
                                title: "Swap failed",
                                description: "User rejected the request",
                                variant: "alert",
                                duration: 5e3
                            }),
                            {
                                success: !1,
                                error: "User rejected transaction"
                            };
                        return e.includes("slippage") ? (0,
                        c.o)({
                            title: "Slippage exceeded",
                            description: "Try increasing your slippage tolerance in settings.",
                            variant: "alert",
                            duration: 5e3
                        }) : e.includes("insufficient") ? (0,
                        c.o)({
                            title: "Insufficient funds",
                            description: "You don't have enough tokens for this swap.",
                            variant: "alert",
                            duration: 5e3
                        }) : e.includes("No wallet found") ? (0,
                        c.o)({
                            title: "Swap failed",
                            description: "User wallet not found. Please reconnect your wallet and try again.",
                            variant: "alert",
                            duration: 5e3,
                            buttons: [{
                                children: "Disconnect",
                                onClick: async () => {
                                    X()
                                }
                                ,
                                variant: "secondary"
                            }]
                        }) : e.includes("WalletSignTransactionError") ? (0,
                        c.o)({
                            title: "Swap failed",
                            description: "Connected wallet failed to sign the transaction.",
                            variant: "alert",
                            duration: 5e3,
                            buttons: [{
                                children: "Disconnect",
                                onClick: async () => {
                                    X()
                                }
                                ,
                                variant: "secondary"
                            }]
                        }) : (0,
                        c.o)({
                            title: "Swap failed",
                            description: "Error",
                            variant: "alert",
                            duration: 5e3
                        }),
                        {
                            success: !1,
                            error: e
                        }
                    } finally {
                        F(),
                        j()
                    }
                }
                ,
                isExecuting: "idle" !== K,
                canExecute: Y,
                executionState: K,
                resetExecution: (0,
                l.useCallback)( () => {
                    W("idle")
                }
                , []),
                hasSufficientBalance: V,
                isConnected: e && f
            }
        }
    }
    ,
    6008: (e, t, n) => {
        n.d(t, {
            lG: () => l,
            R4: () => y,
            HM: () => d,
            Cf: () => p,
            rr: () => w,
            Es: () => g,
            c7: () => f,
            L3: () => h,
            zM: () => c
        });
        var r = n(48876)
          , a = n(77947);
        n(26432);
        var i = n(93749)
          , s = n(8626)
          , o = n(52630);
        function l(e) {
            let {...t} = e;
            return (0,
            r.jsx)(a.bL, {
                "data-slot": "dialog",
                ...t
            })
        }
        function c(e) {
            let {...t} = e;
            return (0,
            r.jsx)(a.l9, {
                "data-slot": "dialog-trigger",
                ...t
            })
        }
        function u(e) {
            let {...t} = e;
            return (0,
            r.jsx)(a.ZL, {
                "data-slot": "dialog-portal",
                ...t
            })
        }
        function d(e) {
            let {...t} = e;
            return (0,
            r.jsx)(a.bm, {
                "data-slot": "dialog-close",
                ...t
            })
        }
        function m(e) {
            let {className: t, ...n} = e;
            return (0,
            r.jsx)(a.hJ, {
                className: (0,
                o.cn)("data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 z-dialog-overlay fixed inset-0 bg-black/50 backdrop-blur-sm", t),
                "data-slot": "dialog-overlay",
                ...n
            })
        }
        function p(e) {
            let {className: t, children: n, preventDefaultDomBehavior: i=!1, ...s} = e
              , l = e => {
                e.preventDefault()
            }
            ;
            return (0,
            r.jsxs)(u, {
                "data-slot": "dialog-portal",
                children: [(0,
                r.jsx)(m, {}), (0,
                r.jsx)(a.UC, {
                    className: (0,
                    o.cn)("focus-ring scrollbar thin-scrollbar bg-bg-low-em data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 border-border-lowest z-dialog fixed top-[50%] left-[50%] grid max-h-[calc(100dvh-6rem)] w-full max-w-[calc(100%-1rem)] translate-x-[-50%] translate-y-[-50%] overflow-y-auto rounded-lg border shadow-lg duration-200 sm:max-w-[33.75rem]", "[&::-webkit-scrollbar-track]:bg-bg-low-em", "[&::-webkit-scrollbar-corner]:bg-bg-low-em", "[&::-webkit-scrollbar-thumb]:border-bg-low-em", "[scrollbar-color:var(--scrollbar-thumb)_var(--bg-bg-low-em)]", t),
                    "data-slot": "dialog-content",
                    ...i && {
                        onEscapeKeyDown: l,
                        onInteractOutside: l,
                        onPointerDownOutside: l
                    },
                    ...s,
                    children: n
                })]
            })
        }
        function f(e) {
            let {className: t, showCloseButton: n=!0, closeBtnProps: l, children: c, ...u} = e;
            return (0,
            r.jsxs)("div", {
                className: (0,
                o.cn)("bg-bg-low-em z-dialog-header sticky top-0 flex flex-col gap-2 border-none p-4 pb-3 sm:p-6 sm:pb-3", t),
                "data-slot": "dialog-header",
                ...u,
                children: [n && (0,
                r.jsx)(a.bm, {
                    asChild: !0,
                    "data-slot": "dialog-close",
                    children: (0,
                    r.jsx)(i.$, {
                        "aria-label": "Close modal",
                        icon: (0,
                        r.jsx)(s.uv, {}),
                        size: "md",
                        variant: "ghost",
                        ...l,
                        className: (0,
                        o.cn)("absolute top-2.5 right-2.5 sm:top-4.5 sm:right-3.5", null == l ? void 0 : l.className)
                    })
                }), c]
            })
        }
        function g(e) {
            let {className: t, ...n} = e;
            return (0,
            r.jsx)("div", {
                className: (0,
                o.cn)("bg-bg-low-em z-dialog-footer sticky bottom-0 flex flex-col-reverse gap-2 p-4 pt-3 sm:flex-row sm:justify-end sm:p-6 sm:pt-3", t),
                "data-slot": "dialog-footer",
                ...n
            })
        }
        function h(e) {
            let {className: t, ...n} = e;
            return (0,
            r.jsx)(a.hE, {
                className: (0,
                o.cn)("sm:text-heading-m font-brand text-[1.625rem] leading-none", t),
                "data-slot": "dialog-title",
                ...n
            })
        }
        let y = e => {
            let {className: t, ...n} = e;
            return (0,
            r.jsx)("div", {
                className: (0,
                o.cn)("flex flex-col gap-2 px-4 py-2 sm:px-6 sm:py-3", t),
                ...n
            })
        }
        ;
        function w(e) {
            let {className: t, ...n} = e;
            return (0,
            r.jsx)(a.VY, {
                className: (0,
                o.cn)("text-muted-foreground text-sm", t),
                "data-slot": "dialog-description",
                ...n
            })
        }
    }
    ,
    10985: (e, t, n) => {
        n.d(t, {
            w9: () => h,
            Hb: () => y
        });
        var r = n(37358)
          , a = n(40476)
          , i = n(55436)
          , s = n(26432)
          , o = n(30369)
          , l = n(50273)
          , c = n(59271);
        let u = new a.PublicKey("TitanLozLMhczcwrioEguG2aAmiATAPXdYpBg3DbeKK");
        async function d(e, t, n, r) {
            let a = new o.A(e.priceBase.toString()).div(new o.A(10).pow(e.priceExponent))
              , {slot: i, timestamp: s} = await (0,
            l.G_)(n, "confirmed")
              , c = Number(e.creationSlot) - i
              , u = Number(e.expirationSlot) - i
              , d = s + c / 2
              , m = s + u / 2;
            return console.log("\uD83D\uDD50 Slot to Timestamp Conversion:", {
                currentSlot: i,
                currentTimestamp: s,
                creationSlot: e.creationSlot.toString(),
                expirationSlot: e.expirationSlot.toString(),
                creationSlotDiff: c,
                expirationSlotDiff: u,
                createdAt: d,
                expiresAt: m,
                createdAtDate: new Date(1e3 * d),
                expiresAtDate: new Date(1e3 * m)
            }),
            {
                address: t,
                maker: e.maker.toBase58(),
                inputMint: e.inputMint.toBase58(),
                outputMint: e.outputMint.toBase58(),
                createdAt: d,
                expiresAt: m,
                amount: e.amount.toString(),
                amountFilled: e.amountFilled.toString(),
                outAmountFilled: e.outAmountFilled.toString(),
                outAmountWithdrawn: e.outAmountWithdrawn.toString(),
                feesPaid: e.feesPaid.toString(),
                limitPrice: a.toNumber(),
                status: e.status,
                id: e.id,
                timeInForce: e.timeInForce,
                feeUnits: e.feeUnits,
                signature: r
            }
        }
        async function m(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {};
            try {
                var n, r, i;
                let s = [{
                    dataSize: 168
                }];
                t.owner && s.push({
                    memcmp: {
                        offset: 0,
                        encoding: "base58",
                        bytes: t.owner.toBase58()
                    }
                }),
                t.inputMint && s.push({
                    memcmp: {
                        offset: 32,
                        encoding: "base58",
                        bytes: t.inputMint.toBase58()
                    }
                }),
                t.outputMint && s.push({
                    memcmp: {
                        offset: 64,
                        encoding: "base58",
                        bytes: t.outputMint.toBase58()
                    }
                });
                let o = await e.getProgramAccounts(u, {
                    commitment: "confirmed",
                    filters: s
                });
                c.R.info("Found ".concat(o.length, " limit order accounts"));
                let l = [];
                for (let {pubkey: n, account: r} of o)
                    try {
                        let i = function(e) {
                            let t = new a.PublicKey(e.subarray(0, 32))
                              , n = new a.PublicKey(e.subarray(32, 64))
                              , r = new a.PublicKey(e.subarray(64, 96))
                              , i = e.readBigUInt64LE(96)
                              , s = e.readBigUInt64LE(104)
                              , o = e.readBigUInt64LE(112)
                              , l = e.readBigUInt64LE(120)
                              , c = e.readBigUInt64LE(128)
                              , u = e.readBigUInt64LE(136)
                              , d = e.readBigUInt64LE(144)
                              , m = e.readBigUInt64LE(152)
                              , p = e.readUInt8(160)
                              , f = e.readUInt8(161)
                              , g = e.readUInt8(162)
                              , h = e.readUInt8(163)
                              , y = e.readUInt8(164)
                              , w = e.readUInt8(165);
                            return {
                                maker: t,
                                inputMint: n,
                                outputMint: r,
                                creationSlot: i,
                                expirationSlot: s,
                                amount: o,
                                amountFilled: l,
                                outAmountFilled: c,
                                outAmountWithdrawn: u,
                                feesPaid: d,
                                priceBase: m,
                                priceExponent: p,
                                status: f,
                                bump: g,
                                id: h,
                                inputMintVaultBump: y,
                                outputMintVaultBump: w,
                                timeInForce: e.readUInt8(166),
                                feeUnits: e.readUInt8(167)
                            }
                        }(r.data);
                        if (c.R.debug("\uD83D\uDD0D Processing account", {
                            address: n.toBase58(),
                            rawStatus: i.status,
                            requestedStatus: t.status,
                            maker: i.maker.toBase58(),
                            inputMint: i.inputMint.toBase58(),
                            outputMint: i.outputMint.toBase58(),
                            amount: i.amount.toString(),
                            priceBase: i.priceBase.toString(),
                            priceExponent: i.priceExponent
                        }),
                        void 0 !== t.status && i.status !== t.status) {
                            c.R.debug("⏭️ Skipping account due to status filter", {
                                address: n.toBase58(),
                                rawStatus: i.status,
                                requestedStatus: t.status
                            });
                            continue
                        }
                        let s = await d(i, n.toBase58(), e);
                        c.R.debug("✅ Successfully processed order", {
                            address: s.address,
                            limitPrice: s.limitPrice,
                            amount: s.amount,
                            amountFilled: s.amountFilled,
                            status: s.status
                        }),
                        l.push(s)
                    } catch (e) {
                        c.R.warn("Failed to deserialize limit order account", {
                            address: n.toBase58(),
                            error: e instanceof Error ? e.message : "Unknown error"
                        })
                    }
                l.sort( (e, t) => t.createdAt - e.createdAt);
                let m = {
                    orders: l,
                    totalCount: l.length,
                    hasMore: !1
                };
                return c.R.info("\uD83D\uDCCB fetchLimitOrders: Final result summary", {
                    totalAccountsFound: o.length,
                    ordersProcessed: l.length,
                    filtersApplied: {
                        owner: null == (n = t.owner) ? void 0 : n.toBase58(),
                        inputMint: null == (r = t.inputMint) ? void 0 : r.toBase58(),
                        outputMint: null == (i = t.outputMint) ? void 0 : i.toBase58(),
                        status: t.status
                    },
                    result: {
                        totalCount: m.totalCount,
                        hasMore: m.hasMore,
                        orderCount: m.orders.length
                    }
                }),
                m
            } catch (e) {
                throw c.R.error("Failed to fetch limit orders:", e),
                Error("Failed to fetch limit orders: ".concat(e instanceof Error ? e.message : "Unknown error"))
            }
        }
        async function p(e, t, n) {
            c.R.info("\uD83D\uDD0D fetchUserLimitOrders: Starting fetch with filters", {
                userAddress: t.toBase58(),
                status: n
            });
            let r = await m(e, {
                owner: t,
                status: n
            });
            return c.R.info("\uD83D\uDCCA fetchUserLimitOrders: Raw response from fetchLimitOrders", {
                userAddress: t.toBase58(),
                status: n,
                totalCount: r.totalCount,
                hasMore: r.hasMore,
                orderCount: r.orders.length
            }),
            r.orders
        }
        async function f(e, t) {
            c.R.info("\uD83D\uDD0D fetchUserOpenLimitOrders: Starting fetch for user", {
                userAddress: t.toBase58(),
                timestamp: new Date().toISOString()
            });
            let n = await p(e, t, 0);
            return c.R.info("✅ fetchUserOpenLimitOrders: Returning open orders", {
                userAddress: t.toBase58(),
                orderCount: n.length,
                timestamp: new Date().toISOString(),
                orders: n.map(e => ({
                    address: e.address,
                    inputMint: e.inputMint,
                    outputMint: e.outputMint,
                    limitPrice: e.limitPrice,
                    amount: e.amount,
                    amountFilled: e.amountFilled,
                    status: e.status,
                    createdAt: e.createdAt,
                    expiresAt: e.expiresAt
                }))
            }),
            n
        }
        var g = n(93355);
        let h = {
            all: ["limit-orders"],
            lists: () => [...h.all, "list"],
            list: e => [...h.lists(), e],
            details: () => [...h.all, "detail"],
            detail: e => [...h.details(), e],
            userOrders: e => [...h.all, "user", e],
            userOpenOrders: e => [...h.all, "user-open", e]
        };
        function y(e) {
            let {connection: t} = (0,
            r.w)()
              , {setOpenOrders: n, setExpiredOrders: o, setIsLoadingOpenOrder: l, setErrorFetchingOpenOrders: u} = (0,
            g.A)()
              , d = !!t && !!e
              , m = (0,
            i.I)({
                queryKey: h.userOpenOrders(e),
                queryFn: async () => {
                    c.R.info("\uD83D\uDE80 useUserOpenLimitOrdersQuery: Starting query", {
                        userAddress: e,
                        hasConnection: !!t
                    });
                    let n = await f(t, new a.PublicKey(e));
                    return c.R.info("\uD83C\uDFAF useUserOpenLimitOrdersQuery: Query completed", {
                        userAddress: e,
                        resultCount: n.length,
                        result: n.map(e => ({
                            order: e
                        }))
                    }),
                    n
                }
                ,
                enabled: d,
                staleTime: 3e4,
                refetchInterval: 6e4,
                retry: 3,
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4)
            });
            return (0,
            s.useEffect)( () => {
                if (m.data) {
                    let e = []
                      , t = [];
                    m.data.map(n => (n.expiresAt <= new Date().getTime() / 1e3 ? e.push(n) : t.push(n),
                    n.expiresAt <= new Date().getTime() / 1e3)),
                    n(t || []),
                    o(e || [])
                }
            }
            , [m.data, n, o]),
            (0,
            s.useEffect)( () => {
                l(m.isLoading)
            }
            , [m.isLoading, l]),
            (0,
            s.useEffect)( () => {
                m.error && u(m.error)
            }
            , [m.error, u]),
            m
        }
    }
    ,
    15334: (e, t, n) => {
        n.d(t, {
            A: () => s
        });
        var r = n(38915)
          , a = n(188)
          , i = n(59271);
        let s = (0,
        r.h)()( (e, t, n) => ({
            tokenBalances: [],
            setTokenBalances: t => {
                i.R.info("[useWalletBalanceStore] setTokenBalances called:", {
                    tokenBalancesLength: t.length,
                    sampleBalances: t.slice(0, 3).map(e => ({
                        mint: e.token.address,
                        balance: e.balance,
                        symbol: e.token.symbol
                    }))
                }),
                e({
                    tokenBalances: t
                })
            }
            ,
            totalUsdValue: 0,
            setTotalUsdValue: t => e({
                totalUsdValue: t
            }),
            balanceLoading: !1,
            setBalanceLoading: t => e({
                balanceLoading: t
            }),
            balanceStale: !1,
            setBalanceStale: t => e({
                balanceStale: t
            }),
            balanceError: null,
            setBalanceError: t => e({
                balanceError: t
            }),
            reset: () => {
                e(n.getInitialState())
            }
        }), a.x)
    }
    ,
    15867: (e, t, n) => {
        n.d(t, {
            A: () => i
        });
        var r = n(38915)
          , a = n(188);
        let i = (0,
        r.h)()( (e, t, n) => ({
            portfolioMetrics: {
                totalUsdValue: 0,
                totalSolValue: 0
            },
            setPortfolioMetrics: t => e({
                portfolioMetrics: t
            }),
            portfolioLoading: !1,
            setPortfolioLoading: t => e({
                portfolioLoading: t
            }),
            errorLoadingBalance: !1,
            setErrorLoadingBalance: t => e({
                errorLoadingBalance: t
            }),
            reset: () => {
                e(n.getInitialState())
            }
        }), a.x)
    }
    ,
    20956: (e, t, n) => {
        n.d(t, {
            QuoteStreamProvider: () => G,
            O$: () => J,
            FF: () => Q
        });
        var r = n(48876)
          , a = n(40476)
          , i = n(19995)
          , s = n(30369)
          , o = n(82554)
          , l = n(26432)
          , c = n(5379)
          , u = n(36795)
          , d = n(88368)
          , m = n(43141);
        let p = {
            TesseraV: "Tessera",
            "Raydium AMM": "Raydium",
            "Raydium CLMM": "Raydium",
            Orca: "Orca",
            Whirlpool: "Whirlpool",
            Jupiter: "Jupiter",
            SolFi: "SolFi",
            Meteora: "Meteora",
            Phoenix: "Phoenix",
            OpenBook: "OpenBook",
            Serum: "Serum",
            Lifinity: "Lifinity",
            Aldrin: "Aldrin",
            Crema: "Crema",
            Cropper: "Cropper",
            Saber: "Saber",
            Stepn: "Stepn",
            Penguin: "Penguin",
            Saros: "Saros",
            Invariant: "Invariant",
            Dradex: "Dradex",
            GooseFX: "GooseFX"
        }
          , f = e => !e || !e.length || e.some(e => !e.label);
        function g(e, t, n) {
            if (["Okx", "Hashflow", "PythExpressRelay", "Pyth"].includes(t) && f(e.steps)) {
                let e = "Okx" === t ? "OKX" : "PythExpressRelay" === t || "Pyth" === t ? "Pyth Express Relay" : t;
                return {
                    venues: ["".concat(e)],
                    formatted: e,
                    count: 1,
                    allocations: []
                }
            }
            if (!e.steps || 0 === e.steps.length)
                return {
                    venues: [],
                    formatted: "No routes",
                    count: 0,
                    allocations: []
                };
            let r = e.steps.map(e => {
                var t;
                return {
                    venue: p[t = e.label] || t,
                    percentage: e.allocPpb / 1e9 * 100,
                    inAmount: e.inAmount,
                    outAmount: e.outAmount
                }
            }
            )
              , a = [...new Set(e.steps.map(e => {
                var t;
                return p[t = e.label] || t
            }
            ))]
              , i = function(e, t) {
                if (0 === e.length)
                    return "No routes";
                if (1 === e.length)
                    return e[0];
                if (2 === e.length)
                    return e.join(", ");
                let n = e.slice(0, 2).join(", ")
                  , r = e.length - 2;
                if (t && n.length >= 16) {
                    let t = e.slice(0, 1)
                      , n = e.length - 1;
                    return "".concat(t, " & ").concat(n, " more")
                }
                return "".concat(n, " & ").concat(r, " more")
            }(a, n);
            return {
                venues: a,
                formatted: i,
                count: a.length,
                allocations: r
            }
        }
        function h(e) {
            return new a.PublicKey(e)
        }
        var y = n(59271)
          , w = n(91015).Buffer
          , v = n(91015).Buffer;
        let b = new a.PublicKey("sponsorKDrY6B1TXJQ5GKUdvGNSbSKRsW8UxGp82Q5Q");
        var S = n(90529)
          , k = n(47828)
          , x = n(55436)
          , T = n(76013)
          , R = n(76535)
          , E = n(82945)
          , A = n(71463)
          , M = n(90178)
          , C = n(67047);
        class N {
            static setToken(e, t, n) {
                this.token = e,
                this.expiresAt = t,
                this.timeUntilExpiry = n
            }
            static clearToken() {
                this.token = null,
                this.expiresAt = null,
                this.timeUntilExpiry = null
            }
            static setIsRefreshing(e) {
                this.isRefreshing = e
            }
            static isTokenValid() {
                return !!this.token && !!this.expiresAt && Date.now() < this.expiresAt
            }
            static isTokenExpiringSoon() {
                let e = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : 6e4;
                return !this.expiresAt || Date.now() + e >= this.expiresAt
            }
            static getToken() {
                return this.isTokenValid() ? this.token : null
            }
            static getIsRefreshing() {
                return this.isRefreshing
            }
            static getRefreshPromise() {
                return this.refreshPromise
            }
            static setRefreshPromise(e) {
                this.refreshPromise = e
            }
        }
        N.token = null,
        N.expiresAt = null,
        N.timeUntilExpiry = null,
        N.isRefreshing = !1,
        N.refreshPromise = null;
        class P {
            static getInstance() {
                return P.instance || (P.instance = new P),
                P.instance
            }
            async getValidToken(e) {
                return N.isTokenValid() && !N.isTokenExpiringSoon() ? (y.R.info("Using existing valid JWT token"),
                N.getToken()) : (N.isTokenExpiringSoon() || !N.isTokenValid() ? y.R.info("JWT token expired or expiring soon, refreshing...") : y.R.info("No valid JWT token found, getting new one..."),
                this.refreshToken(e))
            }
            async refreshToken(e) {
                if (N.getRefreshPromise())
                    return y.R.info("Token refresh already in progress, waiting..."),
                    N.getRefreshPromise();
                N.setIsRefreshing(!0);
                try {
                    let t = this.performTokenRefresh(e);
                    N.setRefreshPromise(t);
                    let n = await t;
                    return y.R.info("JWT token refreshed successfully"),
                    n
                } finally {
                    N.setRefreshPromise(null),
                    N.setIsRefreshing(!1)
                }
            }
            async performTokenRefresh(e) {
                try {
                    let t = await O(e);
                    return N.setToken(t.token, t.expires_at, t.time_until_expiry),
                    y.R.info("JWT token stored successfully", {
                        expiresAt: new Date(t.expires_at).toISOString(),
                        timeUntilExpiry: t.time_until_expiry
                    }),
                    t.token
                } catch (e) {
                    throw y.R.error("Failed to refresh JWT token:", e),
                    N.clearToken(),
                    Error("Failed to refresh JWT token: ".concat(e instanceof Error ? e.message : String(e)))
                }
            }
            clearToken() {
                N.clearToken(),
                y.R.info("JWT token cleared")
            }
            isTokenValid() {
                return N.isTokenValid()
            }
            isTokenExpiringSoon() {
                let e = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : 6e4;
                return N.isTokenExpiringSoon(e)
            }
            constructor() {}
        }
        let I = P.getInstance();
        async function O(e) {
            let t = await fetch("/api/apollo-jwt?address=".concat(e), {
                method: "GET",
                headers: {
                    "Content-Type": "application/json"
                }
            });
            if (!t.ok)
                throw Error((await t.json().catch( () => ({}))).error || "An unknown error occurred");
            return await t.json()
        }
        async function F(e) {
            let t = await fetch("/api/titan-fee", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(e)
            });
            if (!t.ok)
                throw Error((await t.json().catch( () => ({
                    error: "Failed to fetch titan fee"
                }))).error || "An unknown error occurred");
            return await t.json()
        }
        class j {
            static getInstance() {
                return j.instance || (j.instance = new j),
                j.instance
            }
            setEndpoint(e) {
                this.endpoint = e
            }
            async connect(e, t) {
                if (this.endpoint = t,
                this.isConnecting && this.connectionPromise)
                    return y.R.info("Connection already in progress, reusing existing promise"),
                    this.connectionPromise;
                if (this.socket && this.socket.readyState === WebSocket.OPEN)
                    return y.R.info("WebSocket already connected"),
                    Promise.resolve();
                this.cleanup();
                let n = t;
                if (e)
                    try {
                        let r = await I.getValidToken(e);
                        n = "".concat(t, "?auth=").concat(r)
                    } catch (e) {
                        throw y.R.error("Failed to get JWT token for WebSocket connection:", e),
                        e
                    }
                return this.isConnecting = !0,
                this.connectionPromise = new Promise( (t, r) => {
                    this.connectionResolver = t,
                    this.connectionRejector = r;
                    try {
                        let t = ["".concat(d.s8.SUB_PROTOCOL, "+gzip"), d.s8.SUB_PROTOCOL];
                        y.R.info("Trying protocols: ".concat(t.join(", "))),
                        this.socket = new WebSocket(n,t),
                        this.socket.binaryType = "arraybuffer",
                        this.socket.onopen = e => {
                            var t;
                            y.R.info("WebSocket connection opened successfully"),
                            y.R.info("Selected protocol:", null == (t = this.socket) ? void 0 : t.protocol),
                            this.handleOpen()
                        }
                        ,
                        this.socket.onmessage = e => {
                            this.handleMessage(e),
                            this.timeoutId && (clearTimeout(this.timeoutId),
                            this.timeoutId = null)
                        }
                        ,
                        this.socket.onclose = t => {
                            y.R.info("WebSocket connection closed with code ".concat(t.code, ", reason: ").concat(t.reason || "No reason provided", ", wasClean: ").concat(t.wasClean)),
                            this.handleClose(t, e)
                        }
                        ,
                        this.socket.onerror = e => {
                            var t;
                            y.R.error("WebSocket error occurred"),
                            y.R.error("Socket readyState:", null == (t = this.socket) ? void 0 : t.readyState),
                            this.handleError(e)
                        }
                        ;
                        let r = setTimeout( () => {
                            if (this.connectionRejector) {
                                let e = Error("Connection timeout after ".concat(d.s8.CONNECTION_TIMEOUT_MS, "ms"));
                                y.R.error(e.message),
                                this.connectionRejector(e)
                            }
                            this.cleanup(),
                            this.reconnect(e)
                        }
                        , d.s8.CONNECTION_TIMEOUT_MS);
                        this.reconnectTimeout = r
                    } catch (t) {
                        y.R.error("Error initializing WebSocket:", t),
                        this.connectionRejector && this.connectionRejector(t),
                        this.cleanup(),
                        this.reconnect(e)
                    }
                }
                ),
                this.connectionPromise
            }
            disconnect() {
                y.R.info("Disconnecting from WebSocket"),
                this.cleanup()
            }
            async getServerInfo(e, t) {
                y.R.info("Getting server info"),
                await this.connect(e, t);
                let n = {
                    id: this.getNextRequestId(),
                    data: {
                        GetInfo: {}
                    }
                };
                return this.sendRequest(n, e, t)
            }
            async requestQuoteStream(e, t, n) {
                var r, a;
                let i;
                if (await this.connect(e.userPublicKey, t),
                null !== this.currentStreamId) {
                    y.R.info("Stopping existing stream before creating new one:", this.currentStreamId);
                    try {
                        await this.stopStream(this.currentStreamId, e.userPublicKey, t),
                        await new Promise(e => setTimeout(e, 50))
                    } catch (e) {
                        y.R.error("Error stopping existing stream:", this.currentStreamId, e)
                    }
                }
                if (e.amount <= 0)
                    return y.R.info("Amount is 0, stream stopped - not creating new stream"),
                    {
                        NewSwapQuoteStream: {
                            intervalMs: 0
                        }
                    };
                y.R.info("Requesting quote stream with params:", {
                    inputMint: e.inputMint.slice(0, 8) + "...",
                    outputMint: e.outputMint.slice(0, 8) + "...",
                    userPublicKey: e.userPublicKey.slice(0, 8) + "...",
                    amount: e.amount,
                    swapMode: e.swapMode,
                    slippageBps: e.slippageBps
                });
                let s = this.publicKeyToUint8Array(e.inputMint)
                  , o = this.publicKeyToUint8Array(e.outputMint)
                  , l = this.publicKeyToUint8Array(e.userPublicKey)
                  , c = {
                    inputMint: s,
                    outputMint: o,
                    amount: e.amount
                };
                if (void 0 !== e.swapMode && (c.swapMode = e.swapMode),
                void 0 !== e.slippageBps && (c.slippageBps = e.slippageBps),
                void 0 !== e.includeDexes && (c.dexes = e.includeDexes),
                void 0 !== e.onlyDirectRoutes && (c.onlyDirectRoutes = e.onlyDirectRoutes),
                void 0 !== e.addSizeConstraint && (c.addSizeConstraint = e.addSizeConstraint),
                e.isPrimeMode)
                    try {
                        y.R.info("Prime mode enabled - fetching Titan fees...");
                        let t = null != (a = e.usdValue) ? a : 1;
                        y.R.info("USD value being used in titan fee: ", t);
                        let n = await F({
                            fromMint: e.inputMint,
                            toMint: e.outputMint,
                            usdValue: t,
                            lockedWritableAccounts: [e.userPublicKey]
                        });
                        if (y.R.info("Titan fee response:", {
                            slippageBps: n.slippageBps,
                            microLamports: n.microLamports,
                            percentile: n.percentile
                        }),
                        c.slippageBps = n.slippageBps,
                        e.onPrimeFeeData) {
                            let t = {
                                slippageBps: n.slippageBps,
                                microLamports: n.microLamports,
                                percentile: n.percentile,
                                fromMint: e.inputMint,
                                toMint: e.outputMint,
                                timestamp: Date.now()
                            };
                            e.onPrimeFeeData(t),
                            y.R.info("Stored Prime fee data in settings:", {
                                slippageBps: t.slippageBps,
                                microLamports: t.microLamports,
                                percentile: t.percentile
                            })
                        }
                    } catch (e) {
                        y.R.error("Failed to fetch Titan fees in Prime mode:", e),
                        y.R.info("Falling back to user slippage settings")
                    }
                let u = {
                    userPublicKey: l
                };
                void 0 !== e.closeInputTokenAccount && (u.closeInputTokenAccount = e.closeInputTokenAccount),
                void 0 !== e.createOutputTokenAccount && (u.createOutputTokenAccount = e.createOutputTokenAccount),
                (null == (r = e.update) ? void 0 : r.intervalMs) !== void 0 && (i = {
                    intervalMs: e.update.intervalMs
                });
                let d = {
                    swap: c,
                    transaction: u,
                    update: i
                };
                y.R.info("Full request params:", JSON.stringify(d, (e, t) => t instanceof Uint8Array ? R.default.encode(t) : t, 2));
                let m = {
                    id: this.getNextRequestId(),
                    data: {
                        NewSwapQuoteStream: d
                    }
                };
                return this.messageHandlers.set(m.id, {
                    onResponse: e => {
                        y.R.info("Stream response received:", JSON.stringify(e, null, 2)),
                        e.stream && (y.R.info("Stream created with ID:", e.stream.id),
                        this.currentStreamId = e.stream.id,
                        this.streamHandlers.set(e.stream.id, {
                            onData: n.onData,
                            onEnd: e => {
                                this.currentStreamId === e.id && (this.currentStreamId = null),
                                this.activeStreams.delete(e.id),
                                n.onEnd(e)
                            }
                        }),
                        this.activeStreams.add(e.stream.id))
                    }
                }),
                this.sendRequest(m, e.userPublicKey, t)
            }
            async stopStream(e, t, n) {
                if (await this.connect(t, n),
                y.R.info("Stopping stream:", e),
                !this.activeStreams.has(e))
                    return y.R.info("Skip StopStream; stream not active:", e),
                    this.currentStreamId === e && (this.currentStreamId = null),
                    Promise.resolve({});
                let r = {
                    id: this.getNextRequestId(),
                    data: {
                        StopStream: {
                            id: e
                        }
                    }
                };
                return this.currentStreamId === e && (this.currentStreamId = null),
                this.activeStreams.delete(e),
                this.sendRequest(r, t, n)
            }
            extractTransactionComponents(e) {
                var t;
                return {
                    instructions: e.instructions,
                    addressLookupTables: e.addressLookupTables,
                    computeUnits: null != (t = e.computeUnits) ? t : 0
                }
            }
            async sendRequest(e, t, n) {
                try {
                    return await this.connect(t, n),
                    new Promise( (t, n) => {
                        try {
                            this.requestHandlers.set(e.id, {
                                resolve: t,
                                reject: n
                            }),
                            this.sendMessage(e),
                            this.timeoutId = setTimeout( () => {
                                y.R.error("Request timed out"),
                                n(d.W2)
                            }
                            , 1e4),
                            y.R.info("Request ".concat(e.id, " sent successfully"))
                        } catch (t) {
                            this.requestHandlers.delete(e.id),
                            y.R.error("Error sending request ".concat(e.id, ":"), t),
                            n(t)
                        }
                    }
                    )
                } catch (e) {
                    throw y.R.error("Connection failed in sendRequest:", e),
                    Error("WebSocket connection failed: ".concat(e instanceof Error ? e.message : String(e)))
                }
            }
            isGzipEnabled() {
                var e, t;
                return null != (t = null == (e = this.socket) ? void 0 : e.protocol.includes("gzip")) && t
            }
            sendMessage(e) {
                if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
                    var t;
                    let e = Error("WebSocket is not connected (readyState: ".concat(null == (t = this.socket) ? void 0 : t.readyState, ")"));
                    throw y.R.error(e.message),
                    e
                }
                try {
                    let t = (0,
                    A.l)(e);
                    if (this.isGzipEnabled()) {
                        let e = C.Ay.gzip(t);
                        this.socket.send(e)
                    } else
                        this.socket.send(t);
                    y.R.info("Sending message, size: ".concat(t.byteLength, " bytes"))
                } catch (e) {
                    throw y.R.error("Error encoding or sending message:", e),
                    Error("Failed to send message: " + (e instanceof Error ? e.message : String(e)))
                }
            }
            handleOpen() {
                y.R.info("WebSocket connection opened"),
                this.isConnecting = !1,
                this.reconnectAttempts = 0,
                this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null),
                this.connectionResolver && (this.connectionResolver(),
                this.connectionResolver = null,
                this.connectionRejector = null)
            }
            handleMessage(e) {
                try {
                    let t;
                    if (!(e.data instanceof ArrayBuffer))
                        return void y.R.warn("Received non-binary message, ignoring");
                    if (e.data,
                    this.isGzipEnabled()) {
                        let n = C.Ay.ungzip(e.data);
                        t = (0,
                        M.D)(n)
                    } else
                        t = (0,
                        M.D)(e.data);
                    "Response"in t ? (y.R.info("Received response message:", JSON.stringify(t.Response, (e, t) => t instanceof Uint8Array ? "<binary:".concat(t.byteLength, " bytes>") : t, 2)),
                    this.handleResponseMessage(t.Response)) : "Error"in t ? (y.R.error("Received error message:", t.Error),
                    this.handleErrorMessage(t.Error)) : "StreamData"in t ? this.handleStreamDataMessage(t.StreamData) : "StreamEnd"in t ? (y.R.info("Received stream end for stream ID:", t.StreamEnd.id),
                    this.handleStreamEndMessage(t.StreamEnd)) : y.R.warn("Received unknown message type:", t)
                } catch (e) {
                    y.R.error("Error handling message:", e)
                }
            }
            handleClose(e, t) {
                y.R.info("WebSocket connection closed:", e.code, e.reason),
                this.cleanup(),
                1e3 !== e.code && this.reconnect(t)
            }
            handleError(e) {
                y.R.error("WebSocket error:", e),
                this.isConnecting && this.connectionRejector && (this.connectionRejector(Error("WebSocket connection error")),
                this.connectionResolver = null,
                this.connectionRejector = null)
            }
            handleResponseMessage(e) {
                let t = this.requestHandlers.get(e.requestId);
                if (t) {
                    let n = this.messageHandlers.get(e.requestId);
                    n && n.onResponse && n.onResponse(e),
                    t.resolve(e.data),
                    this.requestHandlers.delete(e.requestId),
                    e.stream || this.messageHandlers.delete(e.requestId)
                } else
                    y.R.warn("Received response for unknown request:", e.requestId)
            }
            handleErrorMessage(e) {
                let t = this.requestHandlers.get(e.requestId);
                if (t) {
                    let n = this.messageHandlers.get(e.requestId);
                    n && n.onError && n.onError(e);
                    let r = Error(e.message);
                    r.code = e.code,
                    t.reject(r),
                    this.requestHandlers.delete(e.requestId),
                    this.messageHandlers.delete(e.requestId)
                } else
                    y.R.warn("Received error for unknown request:", e.requestId)
            }
            handleStreamDataMessage(e) {
                let t = this.streamHandlers.get(e.id);
                t ? t.onData(e) : y.R.warn("Received data for unknown stream:", e.id)
            }
            handleStreamEndMessage(e) {
                let t = this.streamHandlers.get(e.id);
                t ? (t.onEnd(e),
                this.streamHandlers.delete(e.id),
                this.activeStreams.delete(e.id)) : y.R.warn("Received end for unknown stream:", e.id)
            }
            getNextRequestId() {
                let e = this.nextRequestId;
                return this.nextRequestId >= d.A2.MAX ? this.nextRequestId = d.A2.INITIAL : this.nextRequestId += 1,
                e
            }
            reconnect(e) {
                if (!this.endpoint)
                    return void y.R.error("Cannot reconnect: no endpoint set");
                if (this.reconnectAttempts >= d.s8.MAX_RECONNECTION_ATTEMPTS) {
                    y.R.error("Maximum reconnection attempts (".concat(d.s8.MAX_RECONNECTION_ATTEMPTS, ") reached")),
                    this.reconnectAttempts = 0,
                    this.isConnecting = !1;
                    return
                }
                let t = Math.min(d.s8.INITIAL_RECONNECTION_DELAY_MS * Math.pow(d.s8.RECONNECTION_BACKOFF_FACTOR, this.reconnectAttempts), d.s8.MAX_RECONNECTION_DELAY_MS);
                y.R.info("Reconnecting in ".concat(t, "ms (attempt ").concat(this.reconnectAttempts + 1, "/").concat(d.s8.MAX_RECONNECTION_ATTEMPTS, ")")),
                this.reconnectTimeout && clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = setTimeout( () => {
                    this.reconnectAttempts++,
                    this.isConnecting = !1,
                    this.connectionPromise = null,
                    this.connect(e, this.endpoint).catch(e => {
                        y.R.error("Reconnection attempt failed:", e)
                    }
                    )
                }
                , t)
            }
            cleanup() {
                if (this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null),
                this.socket) {
                    if (this.socket.onopen = null,
                    this.socket.onmessage = null,
                    this.socket.onclose = null,
                    this.socket.onerror = null,
                    this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING) {
                        y.R.info("Closing WebSocket connection");
                        try {
                            this.socket.close()
                        } catch (e) {
                            y.R.error("Error closing WebSocket:", e)
                        }
                    }
                    this.socket = null
                }
                this.isConnecting = !1,
                this.currentStreamId = null,
                this.activeStreams.clear(),
                this.connectionResolver = null,
                this.connectionRejector = null
            }
            publicKeyToUint8Array(e) {
                return "string" == typeof e ? R.default.decode(e) : e.toBytes()
            }
            constructor() {
                this.socket = null,
                this.isConnecting = !1,
                this.connectionPromise = null,
                this.connectionResolver = null,
                this.connectionRejector = null,
                this.reconnectAttempts = 0,
                this.reconnectTimeout = null,
                this.activeStreams = new Set,
                this.currentStreamId = null,
                this.endpoint = null,
                this.nextRequestId = d.A2.INITIAL,
                this.requestHandlers = new Map,
                this.messageHandlers = new Map,
                this.streamHandlers = new Map,
                this.timeoutId = null
            }
        }
        let L = j.getInstance();
        function q(e) {
            return R.default.encode(e)
        }
        var B = n(47337)
          , D = n(93739)
          , U = n(31001)
          , z = n(41043)
          , _ = n(41313)
          , K = n(63257)
          , W = n(80032)
          , V = n(91015).Buffer;
        let H = (0,
        l.createContext)(void 0);
        function G(e) {
            let {children: t} = e
              , {publicKey: n, walletAddress: p, connected: f} = (0,
            S.z)()
              , {quoteStreamId: A} = (0,
            U.A)()
              , {appConfig: M} = (0,
            D.A)()
              , C = (0,
            i.Ub)("(max-width: 600px)")
              , {settings: N, setPrimeFeeData: P} = (0,
            _.t0)()
              , {isTradeGeoBlocked: O, excludedQuoteProviders: F} = (0,
            U.A)()
              , {priorityLaneEnabled: j, sponsoredTransactionStatus: G, walletVipStatus: Q} = (0,
            W.Ay)()
              , {data: J, error: Y} = function() {
                let {publicKey: e} = (0,
                S.z)()
                  , {appConfig: t} = (0,
                D.A)()
                  , n = !!e
                  , [r,a] = (0,
                l.useState)(!1);
                return (0,
                x.I)({
                    queryKey: E.l.quoteStream.serverInfo(),
                    queryFn: async () => {
                        if (y.R.info("Fetching server info, wallet connected:", n),
                        !n)
                            throw y.R.info("Wallet not connected, skipping server info fetch"),
                            Error("Wallet not connected");
                        if (!(null == t ? void 0 : t.QUOTE_STREAM_ENDPOINT))
                            throw Error("Quote stream endpoint not configured");
                        try {
                            a(!0),
                            y.R.info("Connecting to WebSocket for server info...");
                            let n = await L.getServerInfo(null == e ? void 0 : e.toBase58(), t.QUOTE_STREAM_ENDPOINT);
                            return y.R.info("Server info received successfully"),
                            n.GetInfo
                        } catch (e) {
                            throw y.R.debug("Error fetching server info:", e),
                            e
                        }
                    }
                    ,
                    staleTime: 3e5,
                    gcTime: 6e5,
                    enabled: n && !!(null == t ? void 0 : t.QUOTE_STREAM_ENDPOINT),
                    retry: +!r,
                    retryDelay: 5e3
                })
            }()
              , {slippageBps: X, excludeDexes: Z, includeDexes: $, isPrimeMode: ee} = (0,
            k.n)()
              , [et,en] = (0,
            l.useState)({
                inputMint: null,
                outputMint: null,
                amount: 0,
                swapMode: z.z.ExactIn,
                slippageBps: X,
                excludeDexes: Z,
                includeDexes: $,
                isPrimeMode: ee
            });
            (0,
            l.useEffect)( () => {
                en(e => ({
                    ...e,
                    slippageBps: X,
                    excludeDexes: Z,
                    includeDexes: $,
                    isPrimeMode: ee
                }))
            }
            , [X, Z, $, ee]),
            (0,
            l.useEffect)( () => {
                I.clearToken(),
                L.disconnect()
            }
            , [p]);
            let {quotes: er, bestQuote: ea, isLoading: ei, isStreaming: es, hasNoRoutes: eo, error: el, streamError: ec, refetch: eu} = function(e) {
                let {setQuoteStreamId: t} = (0,
                U.A)()
                  , {allAmmList: n} = (0,
                B.A)()
                  , {appConfig: r} = (0,
                D.A)()
                  , {walletAddress: a, connected: i} = (0,
                S.z)()
                  , s = (0,
                T.jE)()
                  , {inputMint: o, outputMint: u, amount: m, usdValue: p, swapMode: f=z.z.ExactIn, slippageBps: g=d.t7.SLIPPAGE_BPS, enabled: h=!0, dexes: w, excludeDexes: v, includeDexes: b, onlyDirectRoutes: k, addSizeConstraint: A, closeInputTokenAccount: M, createOutputTokenAccount: C, updateIntervalMs: N, isPrimeMode: P, onPrimeFeeData: I} = e
                  , [O,F] = (0,
                l.useState)(!1)
                  , [j,q] = (0,
                l.useState)(null)
                  , [_,K] = (0,
                l.useState)(null)
                  , [W,V] = (0,
                l.useState)(!1)
                  , [H,G] = (0,
                l.useState)(d.kB)
                  , Q = (0,
                l.useRef)(0)
                  , J = (0,
                l.useRef)(0)
                  , Y = (0,
                l.useRef)(!1)
                  , X = (0,
                l.useRef)(0)
                  , Z = (0,
                l.useRef)(0)
                  , $ = (0,
                l.useMemo)( () => a || c.y4, [a])
                  , ee = (0,
                l.useMemo)( () => !!$ && !!o && !!u && o !== u && m > 0 && Object.keys(n || {}).length > 0, [$, o, u, m, n]);
                (0,
                l.useEffect)( () => {
                    y.R.info("Quote stream params changed:", {
                        wallet: $.slice(0, 8) + "...",
                        inputMint: o.slice(0, 8) + "...",
                        outputMint: u.slice(0, 8) + "...",
                        amount: m,
                        hasRequiredParams: ee,
                        enabled: h,
                        prevAmount: Q.current
                    }),
                    Q.current = m
                }
                , [$, o, u, m, ee, h]);
                let et = (0,
                l.useMemo)( () => n && 0 !== Object.keys(n).length ? Object.values(n).map(e => e.label) : [], [n])
                  , en = (0,
                l.useMemo)( () => i ? w : et, [i, w, et])
                  , er = (0,
                l.useMemo)( () => {
                    try {
                        return JSON.stringify({
                            slippageBps: g,
                            dexes: (null != en ? en : []).slice().sort(),
                            excludeDexes: (null != v ? v : []).slice().sort(),
                            includeDexes: (null != b ? b : []).slice().sort(),
                            onlyDirectRoutes: !!k,
                            addSizeConstraint: !!A,
                            closeInputTokenAccount: !!M,
                            createOutputTokenAccount: !!C,
                            updateIntervalMs: null != N ? N : null
                        })
                    } catch (e) {
                        return ""
                    }
                }
                , [g, en, v, b, k, A, M, C, N])
                  , ea = E.l.quoteStream.quotes(o, u, m.toString(), f, !!P, er)
                  , ei = (0,
                l.useCallback)(async () => {
                    if (y.R.info("Starting quote stream request with params:", {
                        inputMint: o,
                        outputMint: u,
                        amount: m,
                        usdValue: p,
                        swapMode: f,
                        slippageBps: g
                    }),
                    !$)
                        throw y.R.error("Cannot request quotes: Wallet not connected"),
                        Error("Wallet not connected");
                    if (!o || !u)
                        throw y.R.error("Cannot request quotes: Invalid swap parameters"),
                        Error("Invalid swap parameters");
                    if ("" === o.trim() || "" === u.trim())
                        throw y.R.error("Cannot request quotes: Empty mint addresses"),
                        Error("Empty mint addresses");
                    if (m <= 0)
                        throw y.R.error("Cannot request quotes: Invalid amount"),
                        Error("Invalid amount");
                    try {
                        return q(null),
                        Z.current = 0,
                        V(!1),
                        y.R.info("Requesting quote stream..."),
                        await s.setQueryData(ea, H || d.kB),
                        await L.requestQuoteStream({
                            inputMint: o,
                            outputMint: u,
                            userPublicKey: $,
                            amount: m,
                            usdValue: p,
                            swapMode: f,
                            slippageBps: g,
                            dexes: en,
                            excludeDexes: v,
                            includeDexes: i ? b : et,
                            onlyDirectRoutes: k,
                            addSizeConstraint: A,
                            closeInputTokenAccount: M,
                            createOutputTokenAccount: C,
                            update: N ? {
                                intervalMs: N
                            } : void 0,
                            isPrimeMode: P,
                            onPrimeFeeData: I
                        }, (null == r ? void 0 : r.QUOTE_STREAM_ENDPOINT) || "wss://fra.api.titan-sol.tech/api/v1/ws", {
                            onData: e => {
                                if (J.current = Date.now(),
                                null !== e.id && (K(e.id),
                                t(e.id)),
                                "SwapQuotes"in e.payload) {
                                    let t = e.payload.SwapQuotes;
                                    G(e.payload.SwapQuotes),
                                    0 === Object.keys(t.quotes).length ? (Z.current += 1,
                                    Z.current > 5 && Z.current <= 10 ? V(!0) : Z.current > 10 && (Z.current = 0,
                                    V(!1))) : V(!1),
                                    s.setQueryData(ea, t)
                                }
                            }
                            ,
                            onEnd: e => {
                                if (y.R.info("Stream ended:", e),
                                F(!1),
                                K(null),
                                t(null),
                                G(d.kB),
                                Z.current = 0,
                                V(!1),
                                e.errorCode && e.errorMessage) {
                                    let t = Error(e.errorMessage);
                                    t.code = e.errorCode,
                                    q(t),
                                    y.R.error("Stream error:", t)
                                }
                            }
                        }),
                        {
                            ...d.kB,
                            inputMint: R.default.decode(o),
                            outputMint: R.default.decode(u),
                            swapMode: f,
                            amount: m
                        }
                    } catch (e) {
                        throw y.R.error("Error requesting quote stream:", e),
                        F(!1),
                        K(null),
                        t(null),
                        String(e) === d.W2 ? (Z.current = 6,
                        V(!0)) : (Z.current = 0,
                        V(!1)),
                        e
                    }
                }
                , [$, o, u, m, p, f, g, en, v, b, et, i, H, k, A, M, C, N, s, ea, P, I, t, null == r ? void 0 : r.QUOTE_STREAM_ENDPOINT])
                  , {data: es=d.kB, isLoading: eo, isFetching: el, error: ec, refetch: eu} = (0,
                x.I)({
                    queryKey: ea,
                    queryFn: ei,
                    enabled: h && ee,
                    refetchOnWindowFocus: !1,
                    refetchOnReconnect: !1,
                    refetchOnMount: !0,
                    staleTime: 0,
                    gcTime: 0,
                    retry: 1,
                    retryDelay: 3e3
                })
                  , ed = (0,
                l.useCallback)( () => {
                    let e = Date.now();
                    if (!Y.current && !(e - X.current < 3e3)) {
                        Y.current = !0,
                        X.current = e;
                        try {
                            L.disconnect()
                        } catch (e) {}
                        Promise.resolve(eu()).finally( () => {
                            Y.current = !1
                        }
                        )
                    }
                }
                , [eu])
                  , em = (0,
                l.useMemo)( () => (0,
                d.x_)(es), [es]);
                return (0,
                l.useEffect)( () => () => {
                    if (null !== _) {
                        y.R.info("Cleaning up stream on unmount:", _);
                        let e = (null == r ? void 0 : r.QUOTE_STREAM_ENDPOINT) || "wss://fra.api.titan-sol.tech/api/v1/ws";
                        L.stopStream(_, $, e).catch(y.R.error)
                    }
                    Z.current = 0,
                    V(!1)
                }
                , [_, $, null == r ? void 0 : r.QUOTE_STREAM_ENDPOINT]),
                (0,
                l.useEffect)( () => {
                    let e = () => {
                        if ("visible" === document.visibilityState && Object.keys(null == H ? void 0 : H.quotes).length > 0 && ee) {
                            let e = J.current || 0
                              , t = Date.now();
                            e > 0 && t - e > 5e3 && setTimeout( () => ed(), 500)
                        }
                    }
                    ;
                    return document.addEventListener("visibilitychange", e),
                    () => {
                        document.removeEventListener("visibilitychange", e)
                    }
                }
                , [ed, H, ee]),
                (0,
                l.useEffect)( () => {
                    if (!h || !ee)
                        return;
                    let e = setInterval( () => {
                        let e = J.current || 0
                          , t = Date.now();
                        e > 0 && t - e > 5e3 && (y.R.info("Quotes appear stale; restarting quote stream"),
                        ed())
                    }
                    , 2e3);
                    return () => clearInterval(e)
                }
                , [h, ee, ed]),
                {
                    quotes: es,
                    bestQuote: em,
                    isLoading: eo || el,
                    isStreaming: O,
                    hasNoRoutes: W,
                    error: ec,
                    streamError: j,
                    refetch: eu
                }
            }({
                ...et,
                inputMint: et.inputMint || "",
                outputMint: et.outputMint || "",
                enabled: !!et.inputMint && !!et.outputMint && !O,
                onPrimeFeeData: P
            })
              , ed = (0,
            l.useMemo)( () => el || ec || Y || null, [el, ec, Y])
              , em = (0,
            l.useMemo)( () => {
                if (u.rL)
                    return er;
                if (!(null == er ? void 0 : er.quotes) || !F || 0 === F.length)
                    return er;
                let e = new Set(F.map(e => e.toLowerCase()))
                  , t = {};
                for (let[n,r] of Object.entries(er.quotes))
                    e.has(n.toLowerCase()) || (t[n] = r);
                return {
                    ...er,
                    quotes: t
                }
            }
            , [er, F])
              , ep = (0,
            l.useMemo)( () => u.rL ? ea : F && F.length > 0 ? (0,
            d.x_)(em) : ea, [em, ea, F])
              , ef = (0,
            l.useMemo)( () => p || c.y4, [p])
              , eg = !!ed
              , eh = (0,
            l.useCallback)( () => {}
            , [])
              , ey = (0,
            l.useCallback)( () => {
                if ((0,
                o.A)(A))
                    return;
                let e = (null == M ? void 0 : M.QUOTE_STREAM_ENDPOINT) || "wss://fra.api.titan-sol.tech/api/v1/ws";
                L.stopStream(A, ef, e).catch(y.R.error)
            }
            , [A, ef, null == M ? void 0 : M.QUOTE_STREAM_ENDPOINT])
              , ew = (0,
            l.useCallback)(e => {
                en(t => ({
                    ...t,
                    ...e
                }))
            }
            , [])
              , ev = function() {
                let {publicKey: e} = (0,
                S.z)();
                return (0,
                l.useCallback)(t => {
                    if (!e || !t)
                        return null;
                    let {instructions: n, addressLookupTables: r, computeUnits: a} = L.extractTransactionComponents(t);
                    return {
                        instructions: n,
                        addressLookupTables: r,
                        computeUnits: a,
                        inputMint: q(t.steps[0].inputMint),
                        outputMint: q(t.steps[t.steps.length - 1].outputMint),
                        inAmount: t.inAmount,
                        outAmount: t.outAmount,
                        steps: t.steps
                    }
                }
                , [e])
            }()
              , eb = (0,
            l.useCallback)(async (e, t, r, i) => {
                let o;
                if (!n)
                    return null;
                let l = t || "";
                if (!r)
                    return y.R.error("No route provided to buildTransaction - cannot proceed", {
                        providedRoute: !!r,
                        providedProvider: t
                    }),
                    null;
                let c = ev(r);
                if (!c)
                    return null;
                let {instructions: u, addressLookupTables: d, computeUnits: p} = c;
                y.R.info("Starting transaction build - Components:", c);
                let f = "priority-fee" == (o = N.isPrimeMode || "mev-protect" === N.txFeeSettings.broadcastMode ? "mev-protect" : "priority-fee")
                  , g = m.zc
                  , S = null
                  , k = async () => {
                    let e;
                    y.R.info("Generating fee instructions");
                    let t = []
                      , r = "none";
                    p > 0 && (g = s.A.ceil(1.25 * p).toNumber(),
                    g = s.A.min(g, m.zc).toNumber(),
                    y.R.info("Setting compute unit limit to ".concat(g, " (original: ").concat(p, ")")));
                    let l = new TextDecoder;
                    u.some(e => l.decode(e.p) === m.wy) && (g = 14e4,
                    y.R.info("Hashflow detected, setting compute unit limit to ".concat(g)));
                    let c = a.ComputeBudgetProgram.setComputeUnitLimit({
                        units: g
                    });
                    t.push(c);
                    let d = 0;
                    if ("priority-fee" === o)
                        if (y.R.info("Priority fee mode detected"),
                        "auto" === N.txFeeSettings.feeMode)
                            try {
                                y.R.info("Auto priority fee mode detected");
                                let e = m.tS[N.txFeeSettings.priorityFee];
                                y.R.info("Priority fee percentile: ".concat(e)),
                                y.R.info("Priority fee returned by gateway: ".concat(i));
                                let t = N.txFeeSettings.maxCapFee;
                                if (y.R.info("Checking against max cap fee of ".concat(t)),
                                t && i && i > t) {
                                    y.R.info("Priority fee ".concat(i, " is greater than max cap fee ").concat(t));
                                    let e = s.A.floor(1e6 * t / g);
                                    d = s.A.min(s.A.floor(i), e).toNumber()
                                }
                                t && i && !(i <= 0) || (y.R.warn("Either no max cap fee or no priority fee, setting to 10000"),
                                d = 1e4)
                            } catch (e) {
                                d = 1e4,
                                y.R.error("Error getting priority fee from API: ".concat(e, ". Using fallback value: ").concat(d))
                            }
                        else {
                            y.R.info("".concat(N.txFeeSettings.feeMode, " priority fee mode detected"));
                            let e = N.txFeeSettings.priorityExactFee;
                            !e || e <= 0 ? (y.R.warn("Invalid exact fee, setting to 10000"),
                            e = 1e4) : (e = s.A.max(e, 5e3).toNumber(),
                            y.R.info("Custom priority fee set by user: ".concat(e, " - 5000 is min value"))),
                            d = s.A.floor(1e6 * e / g).toNumber()
                        }
                    if (t.push(a.ComputeBudgetProgram.setComputeUnitPrice({
                        microLamports: d
                    })),
                    N.isPrimeMode && j && (null == Q ? void 0 : Q.isVip) && G.exists && G.count > 0) {
                        y.R.info("Sponsored transaction status detected");
                        let e = function(e, t) {
                            let[n] = a.PublicKey.findProgramAddressSync([v.from("sponsor_vault"), v.from([0])], b)
                              , [r] = a.PublicKey.findProgramAddressSync([v.from("sponsorship_tracker"), e.toBuffer()], b)
                              , i = [{
                                pubkey: e,
                                isSigner: !0,
                                isWritable: !0
                            }, {
                                pubkey: n,
                                isSigner: !1,
                                isWritable: !0
                            }, {
                                pubkey: r,
                                isSigner: !1,
                                isWritable: !0
                            }, {
                                pubkey: a.SYSVAR_INSTRUCTIONS_PUBKEY,
                                isSigner: !1,
                                isWritable: !1
                            }]
                              , s = v.from([3]);
                            return new a.TransactionInstruction({
                                keys: i,
                                programId: b,
                                data: s
                            })
                        }(n, 0);
                        t.push(e)
                    }
                    if (N.isPrimeMode || "mev-protect" === o) {
                        y.R.info("Currently mev-protect broadcast mode - either manual or prime");
                        let l = 0
                          , c = "50";
                        N.isPrimeMode ? (y.R.info("Prime mode detected"),
                        l = i,
                        j ? (y.R.info("Priority lane enabled"),
                        l = s.A.max(1e6, l).toNumber(),
                        y.R.info("Tip amount for priority lane is ".concat(l))) : (y.R.info("Priority lane not enabled"),
                        c = N.primeFeeData.percentile,
                        y.R.info("Tip amount for percentile ".concat(c, " is ").concat(l, " (from gateway call)")))) : "mev-protect" === o && ("auto" === N.txFeeSettings.feeMode ? (c = N.txFeeSettings.mevTipPercentile,
                        y.R.info("Mev auto mode detected and percentile set to ".concat(c)),
                        l = i,
                        y.R.info("Tip amount for percentile ".concat(c, " is ").concat(l, " (from gateway call)"))) : "custom" === N.txFeeSettings.feeMode ? (l = N.txFeeSettings.mevTipLamports,
                        y.R.info("Custom mev-protect mode detected and tip set to ".concat(l))) : (y.R.warn("Invalid mev-protect fee mode: ".concat(N.txFeeSettings.feeMode, ", setting to 10000")),
                        l = 1e4)),
                        l = s.A.max(l, 0).toNumber(),
                        l = s.A.floor(l).toNumber(),
                        y.R.info("Final tip amount as integer in lamports: ".concat(l));
                        let u = s.A.div(l, 1e9).toNumber()
                          , d = (i, s) => {
                            y.R.info("Using ".concat(s, " tip accounts")),
                            S = new a.PublicKey(i[Math.floor(Math.random() * i.length)]),
                            t.push(a.SystemProgram.transfer({
                                fromPubkey: n,
                                toPubkey: S,
                                lamports: l
                            })),
                            e = l,
                            r = s
                        }
                        ;
                        u >= m.$w ? (y.R.info("Tip amount ".concat(u, " SOL is greater than or equal to fee threshold ").concat(m.$w, " SOL")),
                        d(K.cR, "quark"),
                        y.R.info("Fee ".concat(u, " SOL >= ").concat(m.$w, " SOL, added Quark tip of ").concat(l, " lamports to ").concat(S.toString()))) : u >= m.LJ ? (y.R.info("Tip amount ".concat(u, " SOL is greater than or equal to fee threshold ").concat(m.LJ, " SOL")),
                        d(K.Q2, "nozomi"),
                        y.R.info("Fee ".concat(u, " SOL >= ").concat(m.LJ, " SOL, added Nozomi tip of ").concat(l, " lamports to ").concat(S.toString()))) : (y.R.info("Tip amount ".concat(u, " SOL is less than fee threshold ").concat(m.LJ, " SOL")),
                        d(K.ED, "jito"),
                        y.R.info("Fee ".concat(u, " SOL < ").concat(m.LJ, " SOL, added Jito tip of ").concat(l, " lamports to ").concat(S.toString())))
                    }
                    return {
                        feeInstructions: t,
                        computeUnitPrice: d,
                        tipAmountLamports: e,
                        tipBroadcaster: r
                    }
                }
                  , x = async () => {
                    y.R.info("Fetching and parsing address lookup table accounts");
                    let t = d.map(e => new a.PublicKey(e));
                    f && t.push(m.Xk),
                    y.R.info("Fetching ".concat(t.length, " accounts. Accounts to fetch: ").concat(t));
                    let n = await e.getMultipleAccountsInfo(t)
                      , r = [];
                    return n.forEach( (e, n) => {
                        if (null === e)
                            return;
                        let i = t[n]
                          , s = V.from(e.data)
                          , l = a.AddressLookupTableAccount.deserialize(s);
                        if (S && l.addresses.some(e => e.equals(S))) {
                            y.R.info("Masking selected tip account ".concat(S.toString(), " in ALT for broadcast mode: ").concat(o));
                            let e = {
                                ...l,
                                addresses: l.addresses.map(e => {
                                    if (!e.equals(S))
                                        return e;
                                    {
                                        let t = a.Keypair.generate().publicKey;
                                        return y.R.info("Masked tip account ".concat(e.toString(), " with ").concat(t.toString())),
                                        t
                                    }
                                }
                                )
                            };
                            r.push(new a.AddressLookupTableAccount({
                                key: i,
                                state: e
                            }))
                        } else
                            r.push(new a.AddressLookupTableAccount({
                                key: i,
                                state: l
                            }))
                    }
                    ),
                    r
                }
                ;
                try {
                    let {feeInstructions: t, computeUnitPrice: i, tipAmountLamports: s, tipBroadcaster: c} = await k()
                      , [p,{blockhash: f, lastValidBlockHeight: g}] = await Promise.all([x(), e.getLatestBlockhash()]);
                    y.R.info("Loaded ".concat(p.length, " lookup tables out of ").concat(d.length, " addresses")),
                    y.R.info("Processing and validating instructions: ", u);
                    let v = function(e) {
                        try {
                            let t, n;
                            return t = 0,
                            n = !1,
                            {
                                swapInstructions: e.filter(e => !h(e.p).equals(a.ComputeBudgetProgram.programId)).map(e => {
                                    try {
                                        var r;
                                        let a, i = h(e.p), s = e.a.map(e => {
                                            try {
                                                return {
                                                    pubkey: h(e.p),
                                                    isSigner: e.s,
                                                    isWritable: e.w
                                                }
                                            } catch (e) {
                                                throw Error("Failed to convert account public key: ".concat(e instanceof Error ? e.message : "Unknown error"))
                                            }
                                        }
                                        ), o = (r = i.toString(),
                                        Object.values(m.DB).includes(r));
                                        o && t++,
                                        o && !n && (n = !0);
                                        try {
                                            a = w.from(e.d)
                                        } catch (e) {
                                            throw Error("Failed to convert instruction data: ".concat(e instanceof Error ? e.message : "Unknown error"))
                                        }
                                        return {
                                            programId: i,
                                            keys: s,
                                            data: a
                                        }
                                    } catch (e) {
                                        throw Error("Failed to process instruction: ".concat(e instanceof Error ? e.message : "Unknown error"))
                                    }
                                }
                                ),
                                swapInstructionCount: t,
                                mevProtectionAdded: n
                            }
                        } catch (e) {
                            throw Error("Instruction processing failed: ".concat(e instanceof Error ? e.message : "Unknown error"))
                        }
                    }(u)
                      , b = v.swapInstructions;
                    y.R.info("Instruction processing completed:", {
                        totalInstructions: b.length,
                        swapInstructionCount: v.swapInstructionCount,
                        mevProtectionAdded: v.mevProtectionAdded,
                        provider: l
                    });
                    let S = function(e, t) {
                        y.R.info("Starting transaction assembly:", {
                            feeInstructionCount: e.feeInstructions.length,
                            swapInstructionCount: e.swapInstructions.length,
                            lookupTableCount: e.addressLookupTableAccounts.length,
                            provider: t
                        });
                        try {
                            let n = function(e) {
                                let {feeInstructions: t, swapInstructions: n, walletPublicKey: r, recentBlockhash: i, addressLookupTableAccounts: s} = e;
                                var o = e;
                                let {feeInstructions: l, swapInstructions: c, walletPublicKey: u, recentBlockhash: d, addressLookupTableAccounts: m} = o;
                                if (!u)
                                    throw Error("Wallet public key is required");
                                if (!d || "string" != typeof d)
                                    throw Error("Valid recent blockhash is required");
                                if (!Array.isArray(l))
                                    throw Error("Fee instructions must be an array");
                                if (!Array.isArray(c))
                                    throw Error("Swap instructions must be an array");
                                if (0 === c.length)
                                    throw Error("At least one swap instruction is required");
                                let p = l.length + c.length;
                                if (p > 1e3 && y.R.warn("High instruction count: ".concat(p, ". May exceed Solana limits.")),
                                !Array.isArray(m))
                                    throw Error("Address lookup table accounts must be an array");
                                [...l, ...c].forEach( (e, t) => {
                                    if (!e.programId)
                                        throw Error("Instruction ".concat(t, " missing program ID"));
                                    if (!e.keys || !Array.isArray(e.keys))
                                        throw Error("Instruction ".concat(t, " missing or invalid keys"));
                                    if (!e.data)
                                        throw Error("Instruction ".concat(t, " missing data"))
                                }
                                );
                                let f = [...t, ...n]
                                  , g = new a.TransactionMessage({
                                    payerKey: r,
                                    recentBlockhash: i,
                                    instructions: f
                                }).compileToV0Message(s)
                                  , h = new a.VersionedTransaction(g)
                                  , w = h.serialize().length;
                                return function(e) {
                                    if (!e)
                                        throw Error("Failed to create transaction");
                                    if (!e.message)
                                        throw Error("Transaction missing message");
                                    let t = e.serialize();
                                    t.length > 1200 && y.R.warn("Large transaction size: ".concat(t.length, " bytes. May exceed network limits."));
                                    let n = e.message;
                                    if (!n.header)
                                        throw Error("Transaction message missing header");
                                    if (!n.staticAccountKeys || 0 === n.staticAccountKeys.length)
                                        throw Error("Transaction message missing account keys");
                                    if (!n.compiledInstructions || 0 === n.compiledInstructions.length)
                                        throw Error("Transaction message missing compiled instructions")
                                }(h),
                                {
                                    transaction: h,
                                    instructionCount: {
                                        fee: t.length,
                                        swap: n.length,
                                        total: f.length
                                    },
                                    transactionSize: w,
                                    usesLookupTables: s.length > 0
                                }
                            }(e);
                            return y.R.info("Transaction assembly completed:", {
                                totalInstructions: n.instructionCount.total,
                                transactionSize: n.transactionSize,
                                usesLookupTables: n.usesLookupTables,
                                provider: t
                            }),
                            n
                        } catch (n) {
                            throw y.R.error("Transaction assembly failed:", {
                                error: n instanceof Error ? n.message : "Unknown error",
                                provider: t,
                                inputValid: !!e.walletPublicKey && !!e.recentBlockhash
                            }),
                            n
                        }
                    }({
                        feeInstructions: t,
                        swapInstructions: b,
                        walletPublicKey: n,
                        recentBlockhash: f,
                        addressLookupTableAccounts: p
                    }, l);
                    return y.R.info("Transaction assembly summary:", {
                        provider: l,
                        broadcastMode: o,
                        tipBroadcaster: c,
                        tipAmountLamports: s,
                        computeUnitPrice: i,
                        instructionBreakdown: S.instructionCount,
                        transactionSize: S.transactionSize,
                        usesLookupTables: S.usesLookupTables
                    }),
                    {
                        transaction: S.transaction,
                        inAmount: r.inAmount,
                        outAmount: r.outAmount,
                        provider: l,
                        steps: r.steps.map(e => ({
                            label: e.label,
                            inAmount: e.inAmount,
                            outAmount: e.outAmount
                        })),
                        feeParams: {},
                        tipAmountLamports: s,
                        tipBroadcaster: c,
                        lastValidBlockHeight: g,
                        recentBlockhash: f,
                        transactionSize: S.transactionSize,
                        instructionCount: S.instructionCount
                    }
                } catch (e) {
                    return y.R.error("Error building transaction:", {
                        error: e instanceof Error ? e.message : "Unknown error",
                        provider: l,
                        stack: e instanceof Error ? e.stack : void 0
                    }),
                    null
                }
            }
            , [n, ev, N.txFeeSettings, N.isPrimeMode, N.primeFeeData, j, Q, G])
              , eS = (0,
            l.useMemo)( () => (null == ep ? void 0 : ep.route) ? g(ep.route, ep.provider, C) : null, [ep, C])
              , ek = (0,
            l.useCallback)(e => {
                var t;
                return e && (null == (t = em.quotes) ? void 0 : t[e]) ? g(em.quotes[e], e) : eS
            }
            , [em.quotes, eS])
              , ex = (0,
            l.useCallback)( () => {
                eu()
            }
            , [eu])
              , eT = (0,
            l.useMemo)( () => {
                var e, t;
                return {
                    isConnected: f,
                    walletAddress: p,
                    params: et,
                    setParams: ew,
                    quotes: em,
                    bestQuote: ep,
                    venueInfo: eS,
                    getVenuesForQuote: ek,
                    isLoading: ei,
                    isStreaming: es,
                    hasNoRoutes: eo,
                    hasError: eg,
                    error: ed,
                    serverInfo: {
                        protocolVersion: null == J ? void 0 : J.protocolVersion,
                        settings: {
                            swap: {
                                slippageBps: null == J || null == (t = J.settings) || null == (e = t.swap) ? void 0 : e.slippageBps
                            }
                        }
                    },
                    refreshQuotes: ex,
                    stopStream: ey,
                    resetErrors: eh,
                    buildTransaction: eb
                }
            }
            , [f, p, et, ew, em, ep, eS, ek, ei, es, eo, eg, ed, J, ex, ey, eh, eb]);
            return (0,
            r.jsx)(H.Provider, {
                value: eT,
                children: t
            })
        }
        function Q() {
            let e = (0,
            l.useContext)(H);
            if (void 0 === e)
                throw Error("useQuoteStreamContext must be used within a QuoteStreamProvider");
            return e
        }
        function J() {
            let {params: e, setParams: t, isStreaming: n, hasNoRoutes: r} = Q();
            return {
                params: e,
                setParams: t,
                isStreaming: n,
                hasNoRoutes: r,
                inputMint: e.inputMint,
                outputMint: e.outputMint,
                amount: e.amount,
                swapMode: e.swapMode || z.z.ExactIn,
                slippageBps: e.slippageBps
            }
        }
    }
    ,
    25465: (e, t, n) => {
        n.d(t, {
            BX: () => c,
            Es: () => i,
            Tt: () => l,
            cG: () => s,
            iU: () => o,
            wV: () => a,
            xc: () => u
        });
        var r = n(30369);
        let a = "So11111111111111111111111111111111111111112"
          , i = {
            address: a,
            symbol: "SOL",
            name: "Solana",
            decimals: 9,
            verified: !0,
            logoURI: "/images/tokens/sol.png"
        }
          , s = e => e === a
          , o = e => (0,
        r.A)(e).div(new r.A(10).pow(i.decimals)).toNumber()
          , l = ["So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj"]
          , c = ["So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"]
          , u = ["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo", "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA", "susdabGDNbhrnCa6ncrYo81u4s9GM8ecK2UwMyZiq4X", "DEkqHyPN7GMRJ5cArtQFAWefqbZb33Hyf6s5iCwjEonT"]
    }
    ,
    27550: (e, t, n) => {
        n.r(t),
        n.d(t, {
            Sonner: () => h,
            toast: () => p
        });
        var r = n(48876)
          , a = n(19995)
          , i = n(26432)
          , s = n(23145)
          , o = n(93749)
          , l = n(35958)
          , c = n(8626);
        let u = e => {
            let {...t} = e;
            return (0,
            r.jsx)("svg", {
                fill: "none",
                height: "28",
                viewBox: "0 0 28 28",
                width: "28",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                r.jsx)("path", {
                    d: "M14 10.6667V14M14 17.3333H14.0083M10.5 21.5H17.5C18.9001 21.5 19.6002 21.5 20.135 21.2275C20.6054 20.9878 20.9878 20.6054 21.2275 20.135C21.5 19.6002 21.5 18.9001 21.5 17.5V10.5C21.5 9.09987 21.5 8.3998 21.2275 7.86502C20.9878 7.39462 20.6054 7.01217 20.135 6.77248C19.6002 6.5 18.9001 6.5 17.5 6.5H10.5C9.09987 6.5 8.3998 6.5 7.86502 6.77248C7.39462 7.01217 7.01217 7.39462 6.77248 7.86502C6.5 8.3998 6.5 9.09987 6.5 10.5V17.5C6.5 18.9001 6.5 19.6002 6.77248 20.135C7.01217 20.6054 7.39462 20.9878 7.86502 21.2275C8.3998 21.5 9.09987 21.5 10.5 21.5Z",
                    stroke: "currentColor",
                    strokeLinecap: "round",
                    strokeLinejoin: "round",
                    strokeWidth: "1.6"
                })
            })
        }
          , d = e => {
            let {...t} = e;
            return (0,
            r.jsx)("svg", {
                fill: "none",
                height: "24",
                viewBox: "0 0 24 24",
                width: "24",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                r.jsx)("path", {
                    d: "M7.5 12L10.5 15L16.5 9M7.8 21H16.2C17.8802 21 18.7202 21 19.362 20.673C19.9265 20.3854 20.3854 19.9265 20.673 19.362C21 18.7202 21 17.8802 21 16.2V7.8C21 6.11984 21 5.27976 20.673 4.63803C20.3854 4.07354 19.9265 3.6146 19.362 3.32698C18.7202 3 17.8802 3 16.2 3H7.8C6.11984 3 5.27976 3 4.63803 3.32698C4.07354 3.6146 3.6146 4.07354 3.32698 4.63803C3 5.27976 3 6.11984 3 7.8V16.2C3 17.8802 3 18.7202 3.32698 19.362C3.6146 19.9265 4.07354 20.3854 4.63803 20.673C5.27976 21 6.11984 21 7.8 21Z",
                    stroke: "currentColor",
                    strokeLinecap: "round",
                    strokeLinejoin: "round",
                    strokeWidth: "2"
                })
            })
        }
        ;
        var m = n(52630);
        function p(e, t) {
            var n;
            return s.oR.custom(t => (0,
            r.jsx)(f, {
                id: t,
                ...e
            }), {
                ...t,
                duration: null != (n = e.duration) ? n : 4e3
            })
        }
        function f(e) {
            let {title: t, description: n, buttons: a, type: i="square", id: l, variant: p="default", closeBtnProps: f, hideCloseBtn: g} = e
              , h = "simple" === i ? (0,
            r.jsx)(c.g_, {
                className: "text-success size-5"
            }) : (0,
            r.jsx)(d, {
                className: "text-success size-5"
            });
            return "alert" === p && (h = "simple" === i ? (0,
            r.jsx)(c.qh, {
                className: "text-alert size-5"
            }) : (0,
            r.jsx)(u, {
                className: "text-alert size-5"
            })),
            (0,
            r.jsx)("div", {
                className: (0,
                m.cn)("bg-bg-low-em border-border-lowest pointer-events-auto w-full border font-sans shadow-lg ring-1 ring-black/5", "simple" === i ? "simple rounded-full p-3 pr-5 [@media(min-width:600px)]:max-w-fit [@media(min-width:600px)]:min-w-fit" : "complex rounded-xl p-3 pb-5 [@media(min-width:600px)]:max-w-[22.875rem] [@media(min-width:600px)]:min-w-[22.875rem]"),
                children: (0,
                r.jsxs)("div", {
                    className: (0,
                    m.cn)("flex justify-between gap-3", "simple" === i ? "items-center" : "items-start"),
                    children: [(0,
                    r.jsx)("span", {
                        className: (0,
                        m.cn)("simple" === i ? "mt-0" : "mt-1"),
                        children: h
                    }), (0,
                    r.jsxs)("div", {
                        className: "flex-1",
                        children: [(0,
                        r.jsx)("h4", {
                            className: (0,
                            m.cn)("simple" === i ? "text-base text-neutral-50" : "text-heading-xs font-brand text-neutral-50"),
                            children: t
                        }), n && (0,
                        r.jsx)("div", {
                            className: "text-body-s mt-1 text-neutral-50",
                            children: n
                        }), a && (0,
                        r.jsx)("div", {
                            className: "mt-4 flex gap-2",
                            children: null == a ? void 0 : a.map( (e, t) => (0,
                            r.jsx)(o.$, {
                                size: "sm",
                                ...e
                            }, "".concat(l, "-").concat(e.children, "-").concat(t)))
                        })]
                    }), !g && (0,
                    r.jsx)("button", {
                        ...f,
                        className: (0,
                        m.cn)("grid size-8 place-items-center rounded-full transition-all duration-200 hover:bg-neutral-800 active:scale-90", null == f ? void 0 : f.className),
                        onClick: e => {
                            var t;
                            s.oR.dismiss(l),
                            null == f || null == (t = f.onClick) || t.call(f, e)
                        }
                        ,
                        children: (0,
                        r.jsx)(c.uv, {
                            className: "text-icons size-5"
                        })
                    })]
                })
            })
        }
        let g = () => {
            let[e,t] = (0,
            i.useState)("top-right")
              , n = (0,
            a.Ub)("(max-width: 600px)");
            return (0,
            i.useEffect)( () => {
                n ? t("bottom-center") : t("top-right")
            }
            , [n]),
            (0,
            r.jsx)(s.l$, {
                className: (0,
                m.cn)("flex items-center justify-center", "z-toast [&:has(.simple)_li]:max-w-fit", '[&:has(.simple)_li[data-x-position="center"]]:mx-auto'),
                position: e
            })
        }
          , h = () => (0,
        r.jsx)(l.Y, {
            children: (0,
            r.jsx)(g, {})
        })
    }
    ,
    28764: (e, t, n) => {
        n.d(t, {
            S: () => c
        });
        var r = n(55436)
          , a = n(26432)
          , i = n(82945)
          , s = n(59271);
        async function o() {
            try {
                let e = await fetch("/api/vip-nft-availability", {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!e.ok) {
                    let t = await e.json().catch( () => ({
                        error: "Unknown error occurred"
                    }));
                    throw s.R.error("VIP NFT availability fetch error:", t),
                    Error("Unknown error fetching VIP NFT availability")
                }
                let t = await e.json();
                if (!t.hasOwnProperty("success") || !t.hasOwnProperty("smbAvailable") || !t.hasOwnProperty("madLadsAvailable"))
                    throw Error("Invalid response format from VIP NFT availability service");
                return t
            } catch (e) {
                throw s.R.error("VIP NFT availability fetch error:", e),
                Error("Unknown error fetching VIP NFT availability")
            }
        }
        var l = n(60811);
        function c() {
            let {setVipNftAvailability: e, setVipNftAvailabilityLoading: t, setVipNftAvailabilityError: n} = (0,
            l.Ay)()
              , s = (0,
            r.I)({
                queryKey: i.l.vip.nftAvailability(),
                queryFn: async () => {
                    let t = await o();
                    return e(t),
                    t
                }
                ,
                staleTime: 6e5,
                gcTime: 9e5,
                retry: (e, t) => !(t.message.includes("400") || t.message.includes("403")) && e < 3,
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4)
            });
            return (0,
            a.useEffect)( () => {
                t(s.isLoading)
            }
            , [s.isLoading, t]),
            (0,
            a.useEffect)( () => {
                s.error ? n(s.error) : n(null)
            }
            , [s.error, n]),
            s
        }
    }
    ,
    31001: (e, t, n) => {
        n.d(t, {
            A: () => r
        });
        let r = (0,
        n(90385).v)()(e => ({
            isTradeGeoBlocked: !1,
            setIsTradeGeoBlocked: t => e({
                isTradeGeoBlocked: t
            }),
            isExecuting: !1,
            setIsExecuting: t => e({
                isExecuting: t
            }),
            excludedQuoteProviders: [],
            setExcludedQuoteProviders: t => e({
                excludedQuoteProviders: t
            }),
            selectedSellToken: void 0,
            setSelectedSellToken: t => e({
                selectedSellToken: t
            }),
            selectedBuyToken: void 0,
            setSelectedBuyToken: t => e({
                selectedBuyToken: t
            }),
            quoteStreamId: null,
            setQuoteStreamId: t => e({
                quoteStreamId: t
            }),
            sellInputRef: null,
            setSellInputRef: t => e({
                sellInputRef: t
            })
        }))
    }
    ,
    31283: (e, t, n) => {
        n.d(t, {
            A: () => d
        });
        var r = n(48876)
          , a = n(26432)
          , i = n(8626)
          , s = n(52630)
          , o = n(51092);
        let l = e => {
            let {value: t, min: n=1e-4, precision: i=4, className: l, subProps: c={}, ...u} = e
              , {className: d, ...m} = c
              , {initial: p, leadingZerosCount: f, remainingDigits: g} = function(e) {
                let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : 3
                  , n = arguments.length > 2 && void 0 !== arguments[2] ? arguments[2] : 1e-5;
                if (parseFloat(e) >= n) {
                    let[n,r] = e.split(".");
                    return {
                        initial: r ? "".concat(n, ".").concat(r.slice(0, t)) : n
                    }
                }
                let r = e.match(/^0\.(0+)/);
                return r ? {
                    initial: "0.0",
                    leadingZerosCount: r[1].length,
                    remainingDigits: e.slice(r[0].length).slice(0, t)
                } : {
                    initial: e
                }
            }(t, i, n);
            return t ? (0,
            r.jsx)("span", {
                className: (0,
                s.cn)("relative uppercase", "text-body-xs", l),
                ...u,
                children: parseFloat(t) >= n ? (0,
                r.jsxs)(r.Fragment, {
                    children: [(0,
                    o.A)({
                        number: Number(t),
                        options: {
                            maximumFractionDigits: i,
                            style: "decimal"
                        }
                    }), " "]
                }) : (0,
                r.jsxs)(r.Fragment, {
                    children: [p, f && (0,
                    r.jsx)("sub", {
                        className: (0,
                        s.cn)("", d),
                        ...m,
                        children: f
                    }), g]
                })
            }) : (0,
            r.jsx)(a.Fragment, {})
        }
        ;
        var c = n(25465)
          , u = n(19646);
        let d = e => {
            let {className: t, inTokenAddress: n, outTokenAddress: d, inTokenSymbol: m="", outTokenSymbol: p="", rate: f, isFlipped: g=!1, usingCustomFormat: h=!1, showPlaceholder: y=!1, ...w} = e
              , [v,b,S] = (0,
            a.useMemo)( () => {
                let e = n === c.wV
                  , t = d === c.wV
                  , r = c.xc.includes(n || "")
                  , a = c.xc.includes(d || "")
                  , i = m || ""
                  , s = p || ""
                  , o = null === f || 0 === f ? 0 : 1 / f;
                return e ? [f, i, s] : t ? [o, s, i] : r && !t ? [f, i, s] : a && !e ? [o, s, i] : [f, m, p]
            }
            , [n, d, f, p, m])
              , k = null === v || 0 === v ? 0 : 1 / v;
            return (0,
            r.jsxs)("div", {
                className: (0,
                s.cn)("flex items-center gap-1 font-medium", t),
                ...w,
                children: [(0,
                r.jsxs)("span", {
                    className: "flex items-center gap-0.5",
                    children: [(0,
                    r.jsx)("span", {
                        children: "1"
                    }), (0,
                    r.jsx)("span", {
                        children: g ? S : b
                    })]
                }), (0,
                r.jsx)("span", {
                    children: "≈"
                }), (0,
                r.jsxs)("span", {
                    className: "flex items-center gap-x-0.5",
                    children: [y ? "..." : h ? (0,
                    r.jsx)(l, {
                        value: g ? (0,
                        u.Df)(k) : (0,
                        u.Df)(v)
                    }) : (0,
                    r.jsx)("span", {
                        children: (0,
                        o.A)({
                            number: g ? k : v
                        })
                    }), (0,
                    r.jsx)("span", {
                        children: g ? b : S
                    })]
                }), (0,
                r.jsx)("span", {
                    children: (0,
                    r.jsx)(i.Oy, {})
                })]
            })
        }
    }
    ,
    32641: (e, t, n) => {
        n.d(t, {
            AC: () => f.hf,
            Mk: () => o,
            NI: () => h.N,
            ns: () => m,
            gK: () => c,
            YQ: () => u.Y,
            d2: () => f.d2,
            Hb: () => g.Hb,
            Qi: () => p
        });
        var r = n(55436)
          , a = n(82945)
          , i = n(59271);
        async function s() {
            try {
                let e = await fetch("/api/jito-status", {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!e.ok)
                    return i.R.warn("Jito status request failed: ".concat(e.status)),
                    "unknown";
                let t = await e.json();
                if (!t.status || !["active", "inactive", "unknown"].includes(t.status))
                    return i.R.warn("Invalid Jito status response:", t),
                    "unknown";
                return t.status
            } catch (e) {
                return i.R.warn("Failed to fetch Jito status:", e),
                "unknown"
            }
        }
        let o = function() {
            let e = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : {}
              , {enabled: t=!0, polling: n=!0} = e;
            return (0,
            r.I)({
                queryKey: a.l.alerts.jitoStatus(),
                queryFn: s,
                enabled: t,
                refetchInterval: !!n && 3e4,
                refetchIntervalInBackground: !0,
                staleTime: 25e3,
                gcTime: 3e5,
                refetchOnWindowFocus: !0,
                refetchOnMount: !0,
                refetchOnReconnect: !0,
                retry: (e, t) => !(e >= 2),
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4),
                placeholderData: "unknown"
            })
        };
        var l = n(3913);
        let c = function(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {enabled: n=!0, staleTime: i=3e4, gcTime: s=3e5} = t;
            return (0,
            r.I)({
                queryKey: a.l.prices.multiple(e),
                queryFn: () => (0,
                l.ib)(e),
                enabled: n && e.length > 0,
                staleTime: i,
                gcTime: s,
                refetchInterval: !1,
                refetchIntervalInBackground: !1,
                refetchOnWindowFocus: !0,
                refetchOnMount: !0,
                refetchOnReconnect: !0,
                retry: (e, t) => !(e >= 2),
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4),
                placeholderData: {}
            })
        };
        var u = n(2642)
          , d = n(67725);
        function m() {
            return (0,
            r.I)({
                queryKey: a.l.tokens.lst(),
                queryFn: d.Qt,
                staleTime: 9e5,
                gcTime: 18e5,
                refetchOnWindowFocus: !1,
                refetchOnReconnect: !0,
                enabled: !0
            })
        }
        function p() {
            return (0,
            r.I)({
                queryKey: a.l.tokens.verified(),
                queryFn: d.vh,
                staleTime: 18e5,
                gcTime: 36e5,
                refetchOnWindowFocus: !1,
                refetchOnReconnect: !0,
                enabled: !0
            })
        }
        n(58019),
        n(96853);
        var f = n(73971);
        n(28764),
        n(60811);
        var g = n(10985)
          , h = n(92191);
        n(72938),
        n(92631),
        n(34563),
        n(47828),
        n(5464),
        n(998)
    }
    ,
    33194: (e, t, n) => {
        n.d(t, {
            A: () => s
        });
        var r = n(38915)
          , a = n(188);
        let i = {
            campaigns: {
                success: !1,
                banner_campaigns: [],
                toast_campaigns: [],
                modal_campaigns: []
            },
            isLoading: !1,
            error: null
        }
          , s = (0,
        r.h)()( (e, t, n) => ({
            ...i,
            setCampaigns: t => e({
                campaigns: t
            }),
            setIsLoading: t => e({
                isLoading: t
            }),
            setError: t => e({
                error: t
            }),
            reset: () => {
                let t = n.getState()
                  , r = {
                    banner_campaigns: t.campaigns.banner_campaigns.filter(e => {
                        var t;
                        return null == (t = e.triggers) ? void 0 : t.includes("none")
                    }
                    ),
                    toast_campaigns: t.campaigns.toast_campaigns.filter(e => {
                        var t;
                        return null == (t = e.triggers) ? void 0 : t.includes("none")
                    }
                    ),
                    modal_campaigns: t.campaigns.modal_campaigns.filter(e => {
                        var t;
                        return null == (t = e.triggers) ? void 0 : t.includes("none")
                    }
                    )
                };
                e({
                    ...n.getInitialState(),
                    campaigns: {
                        ...n.getInitialState().campaigns,
                        ...r
                    }
                })
            }
        }), a.x)
    }
    ,
    34563: (e, t, n) => {
        n.d(t, {
            TM: () => u
        });
        var r = n(55436);
        let a = {
            staleTime: 9e5,
            gcTime: 18e5,
            retry: 3,
            retryDelay: e => Math.min(1e3 * 2 ** e, 3e4)
        }
          , i = {
            MIN_LABEL_LENGTH: 1,
            MAX_LABEL_LENGTH: 100,
            MIN_PROGRAM_ID_LENGTH: 32,
            MAX_PROGRAM_ID_LENGTH: 50
        };
        function s(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] && arguments[1];
            return {
                label: e.trim(),
                isExcluded: t
            }
        }
        var o = n(82945)
          , l = n(59271);
        async function c() {
            try {
                let e = await fetch("/api/amm/list", {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!e.ok) {
                    let t = await e.json().catch( () => ({}));
                    throw Error("Failed to fetch AMM list: ".concat(e.status, " - ").concat(t.error || e.statusText))
                }
                let t = await e.json();
                if (!t.success || !Array.isArray(t.data))
                    throw Error("Invalid AMM list response format");
                return {
                    ammMap: function(e) {
                        let t = {}
                          , n = {
                            total: 0,
                            valid: 0,
                            invalid: 0
                        };
                        for (let c of e) {
                            var r, a, o;
                            if (n.total++,
                            !c.program_id || !c.label) {
                                n.invalid++;
                                continue
                            }
                            if (!("string" == typeof (r = c.program_id) && r.length >= i.MIN_PROGRAM_ID_LENGTH && r.length <= i.MAX_PROGRAM_ID_LENGTH && /^[1-9A-HJ-NP-Za-km-z]+$/i.test(r))) {
                                l.R.debug("AMM validation failed for ".concat(c.label, ": program_id=").concat(c.program_id, ", length=").concat(null == (a = c.program_id) ? void 0 : a.length)),
                                n.invalid++;
                                continue
                            }
                            if (!("string" == typeof (o = c.label) && o.length >= i.MIN_LABEL_LENGTH && o.length <= i.MAX_LABEL_LENGTH && o.trim().length > 0)) {
                                n.invalid++;
                                continue
                            }
                            t[c.program_id] = s(c.label, !1),
                            n.valid++
                        }
                        return n.invalid > 0 && l.R.info("AMM validation summary: ".concat(n.valid, "/").concat(n.total, " valid, ").concat(n.invalid, " invalid/skipped")),
                        Object.entries(t).sort( (e, t) => {
                            let[,n] = e
                              , [,r] = t;
                            return n.label.localeCompare(r.label)
                        }
                        ).reduce( (e, t) => {
                            let[n,r] = t;
                            return e[n] = r,
                            e
                        }
                        , {})
                    }(t.data)
                }
            } catch (e) {
                throw l.R.error("AMM list fetch error:", e),
                e instanceof Error ? e : Error("Unknown error fetching AMM list")
            }
        }
        function u() {
            return (0,
            r.I)({
                queryKey: o.l.amm.list(),
                queryFn: c,
                staleTime: a.staleTime,
                gcTime: a.gcTime,
                retry: a.retry,
                retryDelay: a.retryDelay,
                refetchOnWindowFocus: !0,
                refetchOnReconnect: !0,
                placeholderData: {
                    ammMap: {
                        JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: s("Jupiter", !1),
                        "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP": s("Orca V2", !1)
                    }
                }
            })
        }
    }
    ,
    35849: (e, t, n) => {
        n.d(t, {
            A: () => u
        });
        var r = n(47337)
          , a = n(15867)
          , i = n(33194)
          , s = n(77729)
          , o = n(15334)
          , l = n(94187)
          , c = n(80032);
        let u = () => {
            let {reset: e} = (0,
            c.Ay)()
              , {reset: t} = (0,
            a.A)()
              , {reset: n} = (0,
            l.Ay)()
              , {reset: u} = (0,
            o.A)()
              , {reset: d} = (0,
            s.Ay)()
              , {reset: m} = (0,
            i.A)()
              , {setIncludedAmmIds: p, setExcludedAmmIds: f} = (0,
            r.A)();
            return {
                handleResetStore: () => {
                    e(),
                    t(),
                    n(),
                    d(),
                    m(),
                    u(),
                    p([]),
                    f([])
                }
            }
        }
    }
    ,
    37998: (e, t, n) => {
        n.d(t, {
            o: () => r.toast
        });
        var r = n(27550)
    }
    ,
    38079: (e, t, n) => {
        n.d(t, {
            tU: () => m,
            av: () => g,
            j7: () => p,
            Xi: () => f
        });
        var r = n(48876)
          , a = n(78365)
          , i = n(26432);
        let s = (0,
        n(86741).tv)({
            slots: {
                base: ["flex", "flex-col"],
                list: ["flex", "items-center"],
                trigger: ["focus-ring", "cursor-pointer", "flex", "items-center", "rounded-full", "text-text-mid-em", "enabled:data-[state=active]:text-text-high-em", "enabled:data-[state=active]:bg-bg-mid-em", "enabled:data-[state=inactive]:hover:bg-transparent", "enabled:data-[state=inactive]:hover:text-text-high-em", "font-medium", "disabled:cursor-not-allowed", "disabled:text-text-disabled", "disabled:bg-transparent", "disabled:pointer-events-none"],
                content: ["flex-1", "outline-none"],
                icon: ["shrink-0", "text-current"]
            },
            variants: {
                size: {
                    md: {
                        base: ["gap-5"],
                        list: ["gap-1"],
                        trigger: ["gap-0.5", '[&_[data-slot="tabs-trigger-text"]]:px-1', "py-2", "px-3", "text-body-m"],
                        content: "",
                        icon: "h-4 w-4"
                    },
                    sm: {
                        base: ["gap-3"],
                        list: ["gap-0.5"],
                        trigger: ["gap-0.5", '[&_[data-slot="tabs-trigger-text"]]:px-1', "py-1.5", "px-2.5", "text-body-s"],
                        content: "",
                        icon: "h-3.5 w-3.5"
                    }
                }
            },
            defaultVariants: {
                size: "md"
            }
        })
          , o = e => e
          , l = (0,
        i.createContext)(void 0)
          , c = () => {
            let e = (0,
            i.useContext)(l);
            if (void 0 === e)
                throw Error("useTabs was used outside of its Provider");
            return e
        }
          , u = e => {
            let {children: t, ...n} = e
              , a = o(n);
            return (0,
            r.jsx)(l.Provider, {
                value: a,
                children: t
            })
        }
        ;
        var d = n(54094);
        function m(e) {
            let {className: t, ...n} = e
              , {base: i} = s({
                size: n.size
            });
            return (0,
            r.jsx)(u, {
                ...n,
                children: (0,
                r.jsx)(a.bL, {
                    className: i({
                        className: t
                    }),
                    ...n
                })
            })
        }
        function p(e) {
            let {className: t, ...n} = e
              , {size: i} = c()
              , {list: o} = s({
                size: i
            });
            return (0,
            r.jsx)(a.B8, {
                className: o({
                    className: t
                }),
                "data-slot": "tabs-list",
                ...n
            })
        }
        function f(e) {
            let {className: t, children: n, iconLeft: i, iconRight: o, ...l} = e
              , {size: u} = c()
              , {trigger: m, icon: p} = s({
                size: u
            });
            return (0,
            r.jsxs)(a.l9, {
                className: m({
                    className: t
                }),
                "data-slot": "tabs-trigger",
                ...l,
                children: [i && (0,
                r.jsx)(e => {
                    let {icon: t} = e;
                    return (0,
                    d.O)({
                        element: t,
                        themeStyle: p,
                        "data-slot": "tabs-trigger-icon"
                    })
                }
                , {
                    icon: i
                }), (0,
                r.jsx)("span", {
                    "data-slot": "tabs-trigger-text",
                    children: n
                }), o && (0,
                r.jsx)(e => {
                    let {icon: t} = e;
                    return (0,
                    d.O)({
                        element: t,
                        themeStyle: p,
                        "data-slot": "tabs-trigger-icon"
                    })
                }
                , {
                    icon: o
                })]
            })
        }
        function g(e) {
            let {className: t, ...n} = e
              , {size: i} = c()
              , {content: o} = s({
                size: i
            });
            return (0,
            r.jsx)(a.UC, {
                className: o({
                    className: t
                }),
                "data-slot": "tabs-content",
                ...n
            })
        }
    }
    ,
    41043: (e, t, n) => {
        n.d(t, {
            z: () => r
        });
        var r = function(e) {
            return e.ExactIn = "ExactIn",
            e.ExactOut = "ExactOut",
            e
        }({})
    }
    ,
    41313: (e, t, n) => {
        n.d(t, {
            og: () => d,
            Z6: () => f,
            t0: () => g
        });
        var r = n(48876)
          , a = n(85120)
          , i = n.n(a)
          , s = n(26432)
          , o = n(93284);
        async function l(e) {
            let t = await fetch("/api/settings?walletAddress=".concat(e));
            if (!t.ok)
                throw Error((await t.json().catch( () => ({
                    error: "Failed to fetch settings"
                }))).error || "An unknown error occurred");
            return (await t.json()).settings
        }
        async function c(e, t) {
            let n = await fetch("/api/settings", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    wallet_address: e,
                    settings: t
                })
            });
            if (!n.ok)
                throw Error((await n.json().catch( () => ({
                    error: "Failed to save settings"
                }))).error || "An unknown error occurred while saving")
        }
        var u = n(47337);
        let d = {
            baseToken: {
                bpsStr: "1.5%",
                bps: 150
            },
            stableToken: {
                bpsStr: "0.1%",
                bps: 10
            }
        }
          , m = {
            txFeeSettings: {
                broadcastMode: "mev-protect",
                feeMode: "auto",
                priorityFee: "faster",
                prioritySpeed: 8200,
                tipSetting: "med",
                mevTipPercentile: "75",
                mevTipLamports: 0,
                nozomiMinFee: 1e-4,
                maxCapFee: 1e6
            },
            slippageSettings: d,
            ammSettings: {
                ammMap: {}
            },
            isPrimeMode: !0
        }
          , p = (0,
        s.createContext)(void 0)
          , f = e => {
            let {children: t} = e
              , {isConnected: n, walletAddress: a} = (0,
            o.j)()
              , {setExcludedAmmIds: d, setIncludedAmmIds: f} = (0,
            u.A)()
              , [g,h] = (0,
            s.useState)(m)
              , [y,w] = (0,
            s.useState)(!1)
              , [v,b] = (0,
            s.useState)(null)
              , S = (0,
            s.useRef)(null)
              , k = (0,
            s.useMemo)( () => i()(async (e, t) => {
                try {
                    let n = u.A.getState()
                      , r = n.ammMap
                      , a = n.excludedAmmIds
                      , i = Object.keys(r).reduce( (e, t) => (e[t] = {
                        ...r[t],
                        isExcluded: a.includes(t)
                    },
                    e), {})
                      , s = {
                        ...t,
                        ammSettings: {
                            ammMap: i
                        }
                    };
                    await c(e, s)
                } catch (e) {
                    b(e instanceof Error ? e.message : "Failed to save settings")
                }
            }
            , 300), []);
            (0,
            s.useEffect)( () => {
                if (!n || !a) {
                    w(!1),
                    h(m),
                    S.current = null;
                    return
                }
                S.current !== a && (async () => {
                    try {
                        var e, t;
                        w(!0),
                        b(null);
                        let n = await l(a)
                          , r = {
                            ...n,
                            isPrimeMode: null == (t = n.isPrimeMode) || t
                        }
                          , i = (null == n || null == (e = n.ammSettings) ? void 0 : e.ammMap) || {};
                        if (Object.keys(i).length > 0) {
                            let e = []
                              , t = [];
                            Object.keys(i).forEach(n => {
                                let {isExcluded: r} = i[n];
                                r ? e.push(n) : t.push(n)
                            }
                            ),
                            d(e),
                            f(t)
                        }
                        h(r),
                        S.current = a
                    } catch (t) {
                        b(t instanceof Error ? t.message : "Failed to load settings"),
                        h(m);
                        let e = u.A.getState();
                        e.ammMap && Object.keys(e.ammMap).length > 0 && e.setIncludedAmmIds(Object.keys(e.ammMap)),
                        S.current = null
                    } finally {
                        w(!1)
                    }
                }
                )()
            }
            , [n, a, d, f]),
            (0,
            s.useEffect)( () => () => {
                k.cancel(),
                S.current = null
            }
            , [k]);
            let x = (0,
            s.useCallback)(e => {
                n && a && h(t => {
                    let n = e(t);
                    return k(a, n),
                    n
                }
                )
            }
            , [n, a, k])
              , T = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        broadcastMode: e
                    }
                }))
            }
            , [x])
              , R = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        feeMode: e
                    }
                }))
            }
            , [x])
              , E = (0,
            s.useCallback)( (e, t) => {
                x(n => ({
                    ...n,
                    slippageSettings: {
                        ...n.slippageSettings,
                        ["base" === e ? "baseToken" : "stableToken"]: t
                    }
                }))
            }
            , [x])
              , A = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        maxCapFee: e
                    }
                }))
            }
            , [x])
              , M = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        mevTipLamports: e
                    }
                }))
            }
            , [x])
              , C = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        priorityFee: e
                    }
                }))
            }
            , [x])
              , N = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        priorityExactFee: e
                    }
                }))
            }
            , [x])
              , P = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    isPrimeMode: e
                }))
            }
            , [x])
              , I = (0,
            s.useCallback)(e => {
                x(t => ({
                    ...t,
                    txFeeSettings: {
                        ...t.txFeeSettings,
                        mevTipPercentile: e
                    }
                }))
            }
            , [x])
              , O = (0,
            s.useCallback)(e => {
                h(t => ({
                    ...t,
                    primeFeeData: e
                }))
            }
            , [])
              , F = (0,
            s.useCallback)( () => {
                x(e => ({
                    ...m,
                    primeFeeData: e.primeFeeData
                }))
            }
            , [x])
              , j = (0,
            s.useCallback)( () => {
                n && a && k(a, g)
            }
            , [n, a, k, g])
              , L = (0,
            s.useMemo)( () => ({
                settings: g,
                loading: y,
                error: v,
                updateBroadcastMode: T,
                updateFeeMode: R,
                updateSlippage: E,
                updateMaxCapFee: A,
                updateMevTip: M,
                updatePriorityFee: C,
                updatePriorityExactFee: N,
                updateIsPrimeMode: P,
                setPrimeFeeData: O,
                updateMevTipPercentile: I,
                resetToDefaults: F,
                triggerSave: j
            }), [g, y, v, T, R, E, A, M, C, N, P, O, I, F, j]);
            return (0,
            r.jsx)(p.Provider, {
                value: L,
                children: t
            })
        }
          , g = () => {
            let e = (0,
            s.useContext)(p);
            if (void 0 === e)
                throw Error("useSettings must be used within a SettingsProvider");
            return e
        }
    }
    ,
    43106: (e, t, n) => {
        n.d(t, {
            M5: () => a,
            PI: () => r
        });
        var r = function(e) {
            return e.Custom = "Custom",
            e["7D"] = "7D",
            e["1D"] = "1D",
            e["1H"] = "1H",
            e.Never = "Never",
            e
        }({});
        let a = ["Custom", "7D", "1D", "1H", "Never"]
    }
    ,
    43141: (e, t, n) => {
        n.d(t, {
            $w: () => c,
            DB: () => u,
            LJ: () => l,
            Wd: () => s,
            Xk: () => a,
            tS: () => i,
            wy: () => d,
            zc: () => o
        });
        var r = n(40476);
        let a = new r.PublicKey("Fy699f3oM3c4N2Ye8PWS5fR5Vc2jGBNaDG8WaHEmMqVQ")
          , i = {
            fast: "50",
            faster: "75",
            fastest: "95"
        }
          , s = {
            fast: "50",
            faster: "75",
            fastest: "95"
        };
        new r.PublicKey("jitodontfronttitans111111111111111111111111");
        let o = 14e5
          , l = 1e-4
          , c = .001
          , u = {
            TITAN: "T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT",
            JUPITER: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            DFLOW_0: "DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH",
            DFLOW_1: "DF1ow3DqMj3HvTj8i8J9yM2hE9hCrLLXpdbaKZu4ZPnz",
            OKX: "6m2CDdhRgxpH4WjvdzxAYbGxwdGUz5MziiL5jek2kBma"
        }
          , d = "CRhtqXk98ATqo1R8gLg7qcpEMuvoPzqD5GNicPPqLMD"
    }
    ,
    47337: (e, t, n) => {
        n.d(t, {
            A: () => i
        });
        var r = n(19618)
          , a = n(188);
        let i = (0,
        n(38915).h)()((0,
        r.Zr)(e => ({
            ammMap: {},
            setAmmMap: t => e({
                ammMap: t
            }),
            allAmmList: {},
            setAllAmmList: t => e({
                allAmmList: t
            }),
            excludedAmmIds: [],
            setExcludedAmmIds: t => e({
                excludedAmmIds: t
            }),
            includedAmmIds: [],
            setIncludedAmmIds: t => e({
                includedAmmIds: t
            }),
            isLoading: !1,
            setIsLoading: t => e({
                isLoading: t
            }),
            isError: !1,
            setIsError: t => e({
                isError: t
            }),
            error: null,
            setError: t => e({
                error: t
            }),
            lastUpdated: null,
            setLastUpdated: t => e({
                lastUpdated: t
            })
        }), {
            name: "amm-store",
            partialize: e => ({
                excludedAmmIds: e.excludedAmmIds,
                includedAmmIds: e.includedAmmIds
            })
        }), a.x)
    }
    ,
    47828: (e, t, n) => {
        n.d(t, {
            n: () => u
        });
        var r = n(30369)
          , a = n(26432)
          , i = n(25465)
          , s = n(47337)
          , o = n(31001)
          , l = n(73861)
          , c = n(53833);
        let u = () => {
            let {baseTokenSlippage: e, stableTokenSlippage: t, isPrimeMode: n, broadCastMode: u, feeMode: d, priorityFee: m, maxCap: p, mevTip: f, priorityExactFee: g} = (0,
            c.iD)()
              , {excludedAmmIds: h, includedAmmIds: y, ammMap: w} = (0,
            s.A)()
              , v = (0,
            a.useMemo)( () => h.map(e => {
                var t;
                return null == (t = w[e]) ? void 0 : t.label
            }
            ).filter(e => !!e), [h, w])
              , b = (0,
            a.useMemo)( () => y.map(e => {
                var t;
                return null == (t = w[e]) ? void 0 : t.label
            }
            ).filter(e => !!e), [y, w])
              , {lstTokens: S} = (0,
            l.A)()
              , {selectedSellToken: k, selectedBuyToken: x} = (0,
            o.A)()
              , T = (0,
            a.useMemo)( () => {
                let e = i.xc.includes(null == k ? void 0 : k.address) && i.xc.includes(null == x ? void 0 : x.address)
                  , t = (null == k ? void 0 : k.address) === i.wV && S.some(e => e.address === (null == x ? void 0 : x.address))
                  , n = S.some(e => e.address === (null == k ? void 0 : k.address)) && (null == x ? void 0 : x.address) === i.wV
                  , r = S.some(e => e.address === (null == k ? void 0 : k.address)) && S.some(e => e.address === (null == x ? void 0 : x.address));
                return e || t || n || r
            }
            , [x, S, k])
              , R = (0,
            a.useMemo)( () => {
                let n = parseFloat(e)
                  , r = isNaN(n) ? 50 : Math.round(100 * n)
                  , a = parseFloat(t)
                  , i = isNaN(a) ? 50 : Math.round(100 * a);
                return T ? i : r
            }
            , [e, T, t])
              , E = (0,
            a.useMemo)( () => ({
                slippageBps: R,
                excludeDexes: v,
                includeDexes: [],
                broadcastMode: "priority-fee" === u ? "priority-fee" : "mev-shield",
                feeMode: d,
                priorityFeeConfig: "priority-fee" === u ? {
                    speed: m,
                    maxCapLamports: new r.A(p).mul(1e9).toNumber(),
                    exactFeeLamports: g ? new r.A(g).mul(1e9).toNumber() : void 0
                } : void 0,
                mevConfig: "mev-shield" === u ? {
                    tipPercentile: "75",
                    tipLamports: f ? new r.A(f).mul(1e9).toNumber() : void 0
                } : void 0
            }), [R, v, u, d, m, p, g, f]);
            return {
                slippageBps: R,
                excludeDexes: v,
                includeDexes: b,
                isPrimeMode: n,
                transactionConfig: E
            }
        }
    }
    ,
    50273: (e, t, n) => {
        n.d(t, {
            G_: () => s,
            kZ: () => i
        });
        var r = n(59271);
        let a = new Map;
        async function i(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : "confirmed"
              , n = "".concat(e.rpcEndpoint, "-").concat(t)
              , i = Date.now()
              , s = a.get(n);
            if (s && i < s.expiresAt)
                return r.R.debug("[SlotCache] Using cached slot", {
                    slot: s.slot,
                    age: i - s.timestamp,
                    commitment: t
                }),
                s.slot;
            r.R.debug("[SlotCache] Fetching new slot", {
                commitment: t
            });
            let o = await e.getSlot(t);
            return a.set(n, {
                slot: o,
                timestamp: i,
                expiresAt: i + 2e3
            }),
            r.R.debug("[SlotCache] Cached new slot", {
                slot: o,
                commitment: t,
                expiresAt: new Date(i + 2e3).toISOString()
            }),
            o
        }
        async function s(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : "confirmed";
            return {
                slot: await i(e, t),
                timestamp: Math.floor(Date.now() / 1e3)
            }
        }
    }
    ,
    50769: (e, t, n) => {
        n.d(t, {
            A: () => r
        });
        function r(e, t) {
            let n = Number(e)
              , r = Number(t);
            return n && r && 0 !== n ? r / n : 0
        }
    }
    ,
    51491: (e, t, n) => {
        n.d(t, {
            _: () => a
        });
        var r = n(37998);
        async function a(e) {
            let {successMessage: t="Address copied to clipboard", errorMessage: n="Failed to copy address", durationMs: a=1e3} = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {};
            try {
                await navigator.clipboard.writeText(e),
                (0,
                r.o)({
                    title: t,
                    type: "simple",
                    hideCloseBtn: !0,
                    duration: a
                })
            } catch (e) {
                (0,
                r.o)({
                    title: n,
                    variant: "alert",
                    type: "simple",
                    hideCloseBtn: !0
                }, {
                    duration: Math.max(a, 1400)
                })
            }
        }
    }
    ,
    52905: (e, t, n) => {
        n.d(t, {
            A: () => o
        });
        var r = n(48876)
          , a = n(8626)
          , i = n(55796)
          , s = n(52630);
        let o = e => {
            let {sellTokenLogoURI: t, sellTokenSymbol: n, buyTokenLogoURI: o, buyTokenSymbol: l, size: c=16, className: u, logoClassName: d, symbolClassName: m, ...p} = e;
            return (0,
            r.jsxs)("div", {
                className: (0,
                s.cn)("flex items-center gap-x-0.5", "text-text-high-em font-medium", u),
                ...p,
                children: [(0,
                r.jsx)(i.H, {
                    className: (0,
                    s.cn)("rounded-full", d),
                    logoURI: t,
                    size: c,
                    symbol: n
                }), (0,
                r.jsx)("p", {
                    className: (0,
                    s.cn)("ml-0.5", m),
                    children: n
                }), (0,
                r.jsx)(a.HK, {
                    className: "text-text-lowest-em size-3 -rotate-90"
                }), (0,
                r.jsx)(i.H, {
                    className: (0,
                    s.cn)("rounded-full", d),
                    logoURI: o,
                    size: c,
                    symbol: l
                }), (0,
                r.jsx)("p", {
                    className: (0,
                    s.cn)("ml-0.5", m),
                    children: l
                })]
            })
        }
    }
    ,
    53833: (e, t, n) => {
        n.d(t, {
            default: () => y,
            fk: () => c,
            iD: () => h,
            pj: () => o,
            wn: () => l
        });
        var r = n(48876)
          , a = n(26432)
          , i = n(43141)
          , s = n(41313);
        let o = [{
            label: "Priority fee",
            value: "priority-fee"
        }, {
            label: "MEV Shield",
            value: "mev-shield"
        }]
          , l = [{
            label: "Auto",
            value: "auto"
        }, {
            label: "Custom",
            value: "custom"
        }]
          , c = [{
            label: "Fast",
            value: "fast"
        }, {
            label: "Faster",
            value: "faster"
        }, {
            label: "Fastest",
            value: "fastest"
        }]
          , u = e => {
            switch (e) {
            case "priority-fee":
            default:
                return "priority-fee";
            case "mev-protect":
                return "mev-shield"
            }
        }
          , d = e => {
            switch (e) {
            case "priority-fee":
            default:
                return "priority-fee";
            case "mev-shield":
                return "mev-protect"
            }
        }
          , m = e => e
          , p = e => e
          , f = () => {
            var e, t, n, r, o, l, c, f, g;
            let h = (0,
            s.t0)()
              , [y,w] = (0,
            a.useState)(!1)
              , v = h.settings.isPrimeMode
              , b = (null == (n = h.settings) || null == (t = n.slippageSettings) || null == (e = t.baseToken) ? void 0 : e.bpsStr) || s.og.baseToken.bpsStr
              , S = (null == (l = h.settings) || null == (o = l.slippageSettings) || null == (r = o.stableToken) ? void 0 : r.bpsStr) || s.og.stableToken.bpsStr
              , k = b.replace("%", "")
              , x = S.replace("%", "")
              , T = h.settings.txFeeSettings.maxCapFee ? (h.settings.txFeeSettings.maxCapFee / 1e9).toString() : "0.001"
              , R = h.settings.txFeeSettings.mevTipLamports ? (h.settings.txFeeSettings.mevTipLamports / 1e9).toString() : "0"
              , E = h.settings.txFeeSettings.priorityExactFee ? (h.settings.txFeeSettings.priorityExactFee / 1e9).toString() : "0"
              , A = u(h.settings.txFeeSettings.broadcastMode)
              , M = m(h.settings.txFeeSettings.feeMode)
              , C = h.settings.txFeeSettings.priorityFee
              , N = h.settings.txFeeSettings.mevTipPercentile
              , P = (0,
            a.useCallback)(e => {
                let t = p(e);
                h.updateFeeMode(t)
            }
            , [h])
              , I = (0,
            a.useCallback)(e => {
                let t = Math.round(100 * parseFloat(e));
                h.updateSlippage("base", {
                    bpsStr: "".concat(e, "%"),
                    bps: t
                })
            }
            , [h])
              , O = (0,
            a.useCallback)(e => {
                let t = d(e);
                h.updateBroadcastMode(t)
            }
            , [h])
              , F = (0,
            a.useCallback)(e => {
                let t = Math.round(100 * parseFloat(e));
                h.updateSlippage("stable", {
                    bpsStr: "".concat(e, "%"),
                    bps: t
                })
            }
            , [h])
              , j = (0,
            a.useCallback)(e => {
                let t = Math.round(1e9 * parseFloat(e));
                h.updateMaxCapFee(t)
            }
            , [h])
              , L = (0,
            a.useCallback)(e => {
                let t = Math.round(1e9 * parseFloat(e));
                h.updateMevTip(t)
            }
            , [h])
              , q = (0,
            a.useCallback)(e => {
                let t = Math.round(1e9 * parseFloat(e));
                h.updatePriorityExactFee(t)
            }
            , [h])
              , B = (0,
            a.useCallback)(e => {
                h.updatePriorityFee(e)
            }
            , [h])
              , D = (0,
            a.useCallback)(e => {
                let t = i.Wd[e];
                h.updateMevTipPercentile(t)
            }
            , [h])
              , U = (0,
            a.useCallback)(e => {
                h.updateIsPrimeMode(e)
            }
            , [h])
              , z = (0,
            a.useCallback)( () => {
                h.resetToDefaults()
            }
            , [h])
              , _ = (null == (g = h.settings) || null == (f = g.slippageSettings) || null == (c = f.baseToken) ? void 0 : c.bpsStr) || s.og.baseToken.bpsStr
              , K = (0,
            a.useMemo)( () => {
                let {broadcastMode: e} = h.settings.txFeeSettings;
                return "priority-fee" === e ? "Priority Fee" : "MEV Shield"
            }
            , [h.settings.txFeeSettings])
              , W = (0,
            a.useMemo)( () => {
                let {broadcastMode: e, priorityFee: t, feeMode: n, mevTipPercentile: r} = h.settings.txFeeSettings;
                if ("custom" === n)
                    return "Custom";
                if ("mev-protect" === e) {
                    let e = {
                        50: "fast",
                        75: "faster",
                        95: "fastest",
                        99: "fastest"
                    }[r] || "faster";
                    return e.charAt(0).toUpperCase() + e.slice(1)
                }
                return "priority-fee" === e ? t.charAt(0).toUpperCase() + t.slice(1) : "Priority Fee"
            }
            , [h.settings.txFeeSettings]);
            return (0,
            a.useMemo)( () => ({
                baseTokenSlippage: k,
                onBaseTokenSlippageChange: I,
                broadCastMode: A,
                onBroadCastModeChange: O,
                feeMode: M,
                onFeeModeChange: P,
                resetAll: z,
                isSettingsModalOpen: y,
                setIsSettingsModalOpen: w,
                stableTokenSlippage: x,
                onStableTokenSlippageChange: F,
                maxCap: T,
                onMaxCapChange: j,
                mevTip: R,
                onMevTipChange: L,
                priorityExactFee: E,
                onPriorityExactFeeChange: q,
                onPriorityFeeChange: B,
                onMevSpeedChange: D,
                loading: h.loading,
                error: h.error,
                displaySlippage: _,
                displayMevShield: W,
                displayMevShieldLabel: K,
                priorityFee: C,
                mevTipPercentile: N,
                setIsPrimeMode: U,
                isPrimeMode: v
            }), [k, I, A, O, M, P, z, y, w, x, F, T, j, R, L, E, q, B, D, h.loading, h.error, _, W, K, C, N, U, v])
        }
          , g = (0,
        a.createContext)(void 0)
          , h = () => {
            let e = (0,
            a.useContext)(g);
            if (void 0 === e)
                throw Error("useSwapSettings was used outside of its Provider");
            return e
        }
          , y = e => {
            let {children: t} = e
              , n = f();
            return (0,
            r.jsx)(g.Provider, {
                value: n,
                children: t
            })
        }
    }
    ,
    55210: (e, t, n) => {
        n.d(t, {
            default: () => k,
            j: () => S
        });
        var r = n(48876)
          , a = n(19995)
          , i = n(30369)
          , s = n(26432)
          , o = n(88368)
          , l = n(51092)
          , c = n(60647)
          , u = n(99188)
          , d = n(3242)
          , m = n(1187)
          , p = n(20956)
          , f = n(31001)
          , g = n(73861)
          , h = n(41043)
          , y = n(19646)
          , w = n(59271);
        let v = () => {
            let e = (0,
            a.Ub)("(hover: hover)")
              , {allTokens: t, isLoadingToken: n} = (0,
            g.A)()
              , {updateUrl: r, parseTokensFromSearchParams: v} = (0,
            u.A)()
              , {bestQuote: b, error: S, isLoading: k, hasNoRoutes: x, isStreaming: T, refreshQuotes: R} = (0,
            p.FF)()
              , {isTradeGeoBlocked: E, setSelectedBuyToken: A, setSelectedSellToken: M, setSellInputRef: C} = (0,
            f.A)()
              , {setParams: N} = (0,
            p.O$)()
              , {refetch: P} = (0,
            d.s)()
              , I = (0,
            s.useRef)(null)
              , O = (0,
            s.useRef)(null)
              , [F,j] = (0,
            s.useState)(null)
              , [L,q] = (0,
            s.useState)(null)
              , [B,D] = (0,
            s.useState)("")
              , [U,z] = (0,
            s.useState)(!1)
              , [_,K] = (0,
            s.useState)("")
              , W = (0,
            s.useRef)(null)
              , {getTokenPrice: V} = (0,
            m.A)()
              , H = (0,
            s.useMemo)( () => {
                if (!(null == F ? void 0 : F.address) || !_)
                    return 0;
                try {
                    let e = V(F.address)
                      , t = parseFloat((0,
                    c.x)(_)) || 0;
                    return e * t
                } catch (e) {
                    return w.R.warn("Error calculating USD value:", e),
                    0
                }
            }
            , [null == F ? void 0 : F.address, _, V]);
            (0,
            s.useEffect)( () => {
                N({
                    usdValue: H
                })
            }
            , [H, N]);
            let G = (0,
            s.useCallback)(e => t.find(t => t.address === e) || null, [t]);
            (0,
            s.useEffect)( () => {
                (String(S) === o.W2 || x) && (D(""),
                z(!1))
            }
            , [S, x]),
            (0,
            s.useEffect)( () => {
                if (0 === t.length || n)
                    return;
                let e = null
                  , r = null
                  , {sellTokenAddress: a, receiveTokenAddress: i} = v();
                a && (e = G(a)),
                i && (r = G(i)),
                !e && t.length > 0 && (e = G("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")),
                !r && t.length > 0 && (r = G("So11111111111111111111111111111111111111112")),
                F || L || (j(e),
                q(r))
            }
            , [t, v, G, F, L, n]),
            (0,
            s.useEffect)( () => {
                !I.current && (null == F ? void 0 : F.address) && (I.current = F.address),
                !O.current && (null == L ? void 0 : L.address) && (O.current = L.address)
            }
            , [null == F ? void 0 : F.address, null == L ? void 0 : L.address]),
            (0,
            s.useEffect)( () => {
                F && L && r(F.address, L.address)
            }
            , [F, L, r]),
            (0,
            s.useEffect)( () => {
                if (F && L) {
                    let e = (0,
                    c.x)(_)
                      , t = e ? parseFloat(e) : 0;
                    if (t > 0) {
                        let e = setTimeout( () => {
                            N({
                                inputMint: F.address,
                                outputMint: L.address,
                                amount: (0,
                                i.A)(t).mul(new i.A(10).pow(F.decimals)).toNumber(),
                                swapMode: h.z.ExactIn
                            })
                        }
                        , 750);
                        return () => clearTimeout(e)
                    }
                    N({
                        inputMint: F.address,
                        outputMint: L.address,
                        amount: 0
                    })
                }
            }
            , [F, L, _, N]);
            let Q = (0,
            s.useCallback)( () => {
                var t;
                if (j(L),
                q(F),
                L && F && r(L.address, F.address),
                L && F) {
                    let e = (0,
                    c.x)(B)
                      , t = e ? parseFloat(e) : 0;
                    N({
                        inputMint: L.address,
                        outputMint: F.address,
                        amount: new i.A(t).mul(new i.A(10).pow(L.decimals)).toNumber()
                    }),
                    K(B)
                }
                e && (null == (t = W.current) || t.focus())
            }
            , [F, L, B, e, r, N, K])
              , J = (0,
            s.useMemo)( () => {
                if ("" === _)
                    return !0;
                let e = (0,
                c.x)(_);
                return 0 === (e ? parseFloat(e) : 0)
            }
            , [_])
              , Y = (0,
            s.useCallback)(async () => {
                R()
            }
            , [R])
              , X = (0,
            s.useCallback)( (e, t) => {
                let n = F
                  , a = L
                  , s = (0,
                y.Kl)(_);
                "sell" === e && t.decimals < F.decimals && s > t.decimals && K(""),
                "sell" === e ? (j(t),
                n = t) : (q(t),
                a = t),
                (t.address === (null == L ? void 0 : L.address) || t.address === (null == F ? void 0 : F.address)) && (q(F),
                a = F,
                j(L),
                n = L),
                N({
                    inputMint: L.address,
                    outputMint: F.address,
                    amount: new i.A(0).mul(new i.A(10).pow(L.decimals)).toNumber()
                }),
                n && a && r(n.address, a.address);
                let o = I.current
                  , l = O.current;
                (null == t ? void 0 : t.address) && t.address !== o && t.address !== l && P()
            }
            , [j, q, F, L, _, r, P, I, O, N])
              , Z = (0,
            s.useMemo)( () => (0,
            c.x)(_), [_])
              , $ = (0,
            s.useCallback)( (e, t) => {
                if (!t || !e || "" === e)
                    return 0;
                let n = (0,
                c.x)(e)
                  , r = n ? parseFloat(n) : 0;
                return 0 === r ? 0 : new i.A(r).mul(new i.A(10).pow(t.decimals)).toNumber()
            }
            , []);
            return (0,
            s.useEffect)( () => {
                if (E)
                    return;
                if (!b || J)
                    return void (J || x ? (D(""),
                    z(!1)) : z(!0));
                let e = $(_, F);
                if (e > 0 && b.route.inAmount !== e)
                    return void z(!0);
                if (L) {
                    let e = b.route.outAmount
                      , t = L.decimals;
                    D((0,
                    l.A)({
                        number: new i.A(e).div(new i.A(10).pow(t)).toNumber(),
                        fixedDecimals: t
                    })),
                    z(!1)
                }
            }
            , [b, J, L, _, F, E, x, $]),
            (0,
            s.useEffect)( () => {
                (k || T) && z(!0)
            }
            , [T, b, J, k]),
            (0,
            s.useEffect)( () => {
                A(L),
                M(F)
            }
            , [F, L, A, M]),
            (0,
            s.useEffect)( () => {
                C(W)
            }
            , [C]),
            (0,
            s.useMemo)( () => ({
                sanitizeSellAmount: Z,
                sellAmount: _,
                isRefreshing: k || T,
                hasNoRoutes: x,
                sellToken: F,
                receiveToken: L,
                receiveAmount: B,
                receiveAmountLoading: U,
                onSwitchTokens: Q,
                handleRefresh: Y,
                isEmpty: J,
                setSellAmount: K,
                setReceiveAmount: D,
                sellInputRef: W,
                onTokenSelect: X,
                isLoadingToken: n,
                bestQuote: b
            }), [Z, _, k, T, x, F, L, B, U, Q, Y, J, K, D, W, X, n, b])
        }
          , b = (0,
        s.createContext)(void 0)
          , S = () => {
            let e = (0,
            s.useContext)(b);
            if (void 0 === e)
                throw Error("useSwap was used outside of its Provider");
            return e
        }
          , k = e => {
            let {children: t} = e
              , n = v();
            return (0,
            r.jsx)(b.Provider, {
                value: n,
                children: t
            })
        }
    }
    ,
    55796: (e, t, n) => {
        n.d(t, {
            H: () => c
        });
        var r = n(48876)
          , a = n(5657)
          , i = n(26432)
          , s = n(52630)
          , o = n(65314)
          , l = n(19646);
        let c = (0,
        i.memo)(e => {
            let {logoURI: t, symbol: n, size: c=32, className: u} = e
              , [d,m] = (0,
            i.useState)({})
              , [p,f] = (0,
            i.useState)(new Set)
              , g = t && p.has(t) || !t ? "/images/tokens/default.webp" : t.includes("ipfs.nftstorage.link") ? (0,
            l.ox)(t) || "/images/tokens/default.webp" : t
              , h = !d[g] || "loading" === d[g];
            return (0,
            r.jsxs)("div", {
                className: (0,
                s.cn)("relative flex shrink-0 items-center justify-center overflow-hidden rounded-full", u),
                style: {
                    width: c,
                    height: c
                },
                children: [h && (0,
                r.jsx)("div", {
                    className: "absolute inset-0 animate-pulse rounded-full bg-gray-700 will-change-auto"
                }), (0,
                r.jsx)(a.default, {
                    alt: n || "Not Found",
                    className: "rounded-full bg-neutral-800 object-contain",
                    height: c,
                    loader: o.f,
                    loading: "lazy",
                    src: g.trimEnd(),
                    width: c,
                    onError: () => {
                        t && !p.has(t) && f(e => new Set(e).add(t)),
                        m(e => ({
                            ...e,
                            [g]: "error"
                        }))
                    }
                    ,
                    onLoadingComplete: () => {
                        m(e => ({
                            ...e,
                            [g]: "loaded"
                        }))
                    }
                }, g)]
            })
        }
        , (e, t) => e.logoURI === t.logoURI && e.symbol === t.symbol && e.size === t.size && e.className === t.className);
        c.displayName = "TokenImage"
    }
    ,
    58019: (e, t, n) => {
        n.d(t, {
            h: () => o
        });
        var r = n(55436)
          , a = n(26432)
          , i = n(82945)
          , s = n(67725);
        function o(e) {
            let t = !(arguments.length > 1) || void 0 === arguments[1] || arguments[1]
              , n = (0,
            a.useRef)(null);
            return (0,
            r.I)({
                queryKey: i.l.tokens.search(e),
                queryFn: async () => (n.current = new AbortController,
                (0,
                s.ag)(e, n.current.signal)),
                enabled: t && e.trim().length >= 2,
                staleTime: 3e5,
                gcTime: 6e5,
                refetchOnWindowFocus: !1,
                refetchOnReconnect: !1,
                retry: (e, t) => (null == t ? void 0 : t.name) !== "AbortError" && e < 2
            })
        }
    }
    ,
    59183: (e, t, n) => {
        n.d(t, {
            A: () => s
        });
        var r = n(48876);
        let a = e => {
            let {...t} = e;
            return (0,
            r.jsx)("svg", {
                fill: "none",
                height: "12",
                viewBox: "0 0 12 12",
                width: "12",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                r.jsx)("path", {
                    d: "M7.12476 1H4.24698C4.15724 1 4.11237 1 4.07275 1.01366C4.03772 1.02575 4.00582 1.04547 3.97934 1.0714C3.9494 1.10072 3.92934 1.14085 3.88921 1.22111L1.78921 5.42111C1.69337 5.61279 1.64545 5.70863 1.65696 5.78654C1.66701 5.85457 1.70464 5.91545 1.76099 5.95486C1.82552 6 1.93267 6 2.14698 6H5.24976L3.74976 11L9.84631 4.67765C10.052 4.46435 10.1548 4.3577 10.1609 4.26644C10.1661 4.18723 10.1334 4.11024 10.0727 4.05902C10.0028 4 9.85469 4 9.55837 4H5.99976L7.12476 1Z",
                    fill: "currentColor",
                    stroke: "currentColor",
                    strokeLinecap: "round",
                    strokeLinejoin: "round"
                })
            })
        }
        ;
        var i = n(52630);
        let s = e => {
            let {className: t, ...n} = e;
            return (0,
            r.jsxs)("div", {
                className: (0,
                i.cn)("text-brand h-5 w-fit", "relative overflow-hidden", "bg-brand-soft gap-x-1 px-1", "flex items-center justify-center", "border-brand/10 rounded-full border", t),
                ...n,
                children: [(0,
                r.jsx)("div", {
                    className: (0,
                    i.cn)("bg-brand-opacity-30 blur-[4px]", "h-20 w-2.5 -rotate-[30deg]", "animate-shimmer", "absolute -left-3")
                }), (0,
                r.jsx)(a, {}), (0,
                r.jsx)("p", {
                    className: "text-body-xs",
                    children: "VIP"
                })]
            })
        }
    }
    ,
    60647: (e, t, n) => {
        n.d(t, {
            x: () => r
        });
        let r = e => e.replace(/[^0-9.]/g, "") || ""
    }
    ,
    60811: (e, t, n) => {
        n.d(t, {
            Ay: () => l,
            ml: () => o
        });
        var r = n(38915)
          , a = n(188);
        let i = {
            vipNftAvailability: null,
            vipNftAvailabilityLoading: !1,
            vipNftAvailabilityError: null
        }
          , s = (0,
        r.h)()( (e, t) => ({
            ...i,
            setVipNftAvailability: t => e({
                vipNftAvailability: t
            }),
            setVipNftAvailabilityLoading: t => e({
                vipNftAvailabilityLoading: t
            }),
            setVipNftAvailabilityError: t => e({
                vipNftAvailabilityError: t
            })
        }), a.x)
          , o = () => {
            var e, t, n, r;
            let a = s(e => e.vipNftAvailability);
            return {
                smbAvailable: null != (e = null == a ? void 0 : a.smbAvailable) && e,
                madLadsAvailable: null != (t = null == a ? void 0 : a.madLadsAvailable) && t,
                anyAvailable: null != (n = null == a ? void 0 : a.smbAvailable) && n || null != (r = null == a ? void 0 : a.madLadsAvailable) && r
            }
        }
          , l = s
    }
    ,
    62500: (e, t, n) => {
        n.d(t, {
            A: () => c
        });
        var r = n(76013)
          , a = n(90272)
          , i = n(82945)
          , s = n(90529)
          , o = n(46750)
          , l = n(80032);
        let c = () => {
            let e = (0,
            r.jE)()
              , {walletAddress: t} = (0,
            s.z)()
              , {walletVipStatus: n} = (0,
            l.Ay)()
              , c = (0,
            a.n)({
                mutationFn: async e => {
                    let {walletAddress: t, badgeType: n} = e;
                    return (0,
                    o.Jx)(t, n)
                }
                ,
                onSuccess: () => {
                    t && e.invalidateQueries({
                        queryKey: i.l.walletStats.badges(t)
                    })
                }
            })
              , u = async () => {
                let n = i.l.walletStats.stats(t || "");
                e.invalidateQueries({
                    queryKey: n
                })
            }
              , d = async () => {
                let n = i.l.walletStats.referralStats(t || "");
                e.invalidateQueries({
                    queryKey: n
                })
            }
              , m = async () => {
                let n = i.l.walletStats.badges(t || "");
                e.invalidateQueries({
                    queryKey: n
                })
            }
              , p = async () => {
                let r = i.l.walletStats.sponsoredTransactions(t ? "".concat(n, "-").concat(n.isVip) : "");
                e.invalidateQueries({
                    queryKey: r
                })
            }
              , f = async () => {
                u(),
                d(),
                m(),
                p()
            }
            ;
            return {
                notifyBadge: e => {
                    if (!t)
                        throw Error("Wallet address is required");
                    return c.mutateAsync({
                        walletAddress: t,
                        badgeType: e
                    })
                }
                ,
                refetchWalletStats: u,
                refetchReferralStats: d,
                refetchBadges: m,
                refetchSponsoredTransactions: p,
                refreshAll: f
            }
        }
    }
    ,
    63025: (e, t, n) => {
        n.d(t, {
            p: () => d
        });
        var r = n(48876)
          , a = n(26432)
          , i = n(93749)
          , s = n(37998)
          , o = n(90529)
          , l = n(35849)
          , c = n(59271);
        let u = e => {
            let {...t} = e
              , {handleResetStore: n} = (0,
            l.A)()
              , {connected: u, connect: d, connecting: m, ready: p} = (0,
            o.z)();
            (0,
            a.useEffect)( () => {
                u && (0,
                s.o)({
                    title: "Wallet connected!",
                    variant: "success",
                    type: "simple"
                }, {
                    position: "bottom-center",
                    duration: 1e3
                })
            }
            , [u]);
            let f = async () => {
                try {
                    c.R.info("Wallet connect button clicked, setting user initiated flag"),
                    n(),
                    d()
                } catch (e) {
                    c.R.error("[HybridWalletList] Failed to start Privy flow:", e),
                    (0,
                    s.o)({
                        title: "Failed to connect wallet",
                        description: e instanceof Error ? e.message : "Please try again",
                        variant: "alert",
                        type: "simple"
                    }, {
                        position: "bottom-center",
                        duration: 3e3
                    })
                }
            }
            ;
            return (0,
            r.jsx)(i.$, {
                className: "px-6",
                disabled: u || m || !p,
                variant: "primary",
                onClick: f,
                ...t,
                children: u ? "Wallet Connected" : "Connect wallet"
            })
        }
          , d = e => {
            let {children: t, ...n} = e;
            return (0,
            r.jsx)(a.Suspense, {
                children: (0,
                r.jsx)(u, {
                    ...n,
                    children: t
                })
            })
        }
    }
    ,
    63257: (e, t, n) => {
        n.d(t, {
            ED: () => a,
            Q2: () => i,
            cR: () => s,
            h4: () => u
        });
        var r = n(59271);
        let a = ["96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5", "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe", "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY", "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49", "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh", "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt", "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL", "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT"]
          , i = ["TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq", "noz3jAjPiHuBPqiSPkkugaJDkJscPuRhYnSpbi8UvC4", "noz3str9KXfpKknefHji8L1mPgimezaiUyCHYMDv1GE", "noz6uoYCDijhu1V7cutCpwxNiSovEwLdRHPwmgCGDNo", "noz9EPNcT7WH6Sou3sr3GGjHQYVkN3DNirpbvDkv9YJ", "nozc5yT15LazbLTFVZzoNZCwjh3yUtW86LoUyqsBu4L", "nozFrhfnNGoyqwVuwPAW4aaGqempx4PU6g6D9CJMv7Z", "nozievPk7HyK1Rqy1MPJwVQ7qQg2QoJGyP71oeDwbsu", "noznbgwYnBLDHu8wcQVCEw6kDrXkPdKkydGJGNXGvL7", "nozNVWs5N8mgzuD3qigrCG2UoKxZttxzZ85pvAQVrbP", "nozpEGbwx4BcGp6pvEdAh1JoC2CQGZdU6HbNP1v2p6P", "nozrhjhkCr3zXT3BiT4WCodYCUFeQvcdUkM7MqhKqge", "nozrwQtWhEdrA6W8dkbt9gnUaMs52PdAv5byipnadq3", "nozUacTVWub3cL4mJmGCYjKZTnE9RbdY5AP46iQgbPJ", "nozWCyTPppJjRuw2fpzDhhWbW355fzosWSzrrMYB1Qk", "nozWNju6dY353eMkMqURqwQEoM3SFgEKC6psLCSfUne", "nozxNBgWohjR75vdspfxR5H9ceC7XXH99xpxhVGt3Bb"]
          , s = ["TJzUUoKA3ngRDkgf6g42r2tv4uUNnf2S2mngawi61fi"]
          , o = new Map
          , l = {
            fast: 1e3,
            faster: 1e4,
            fastest: 1e5
        };
        async function c(e, t) {
            try {
                let n = new URLSearchParams({
                    provider: e,
                    percentile: t
                })
                  , r = await fetch("/api/fees?".concat(n.toString()), {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!r.ok) {
                    let e = await r.json().catch( () => ({
                        error: "Unknown error"
                    }));
                    throw Error(e.error || "HTTP error! status: ".concat(r.status))
                }
                let a = await r.json();
                return a.fee || a
            } catch (e) {
                return r.R.error("Failed to fetch percentile value:", e),
                null
            }
        }
        async function u(e, t) {
            let n = "".concat(e, "-").concat(t)
              , a = o.get(n);
            if (a && Date.now() - a.timestamp < 3e4)
                return a.value;
            let i = await c(e, t);
            if (null !== i && i > 0)
                return o.set(n, {
                    value: i,
                    timestamp: Date.now()
                }),
                i;
            if ("priority" === e) {
                let e = l[t] || 1e4;
                return r.R.warn("Priority fee API failed for ".concat(t, ", using fallback: ").concat(e, " microlamports")),
                e
            }
            return null
        }
    }
    ,
    65993: (e, t, n) => {
        n.d(t, {
            A: () => l
        });
        var r = n(76013)
          , a = n(26432)
          , i = n(90529)
          , s = n(92191)
          , o = n(72938);
        let l = () => {
            let {walletAddress: e} = (0,
            i.z)()
              , t = (0,
            r.jE)()
              , n = (0,
            o.hf)()
              , {executeLimitOrderTransaction: l} = (0,
            s.N)()
              , [c,u] = (0,
            a.useState)(!1)
              , [d,m] = (0,
            a.useState)("")
              , p = async () => {
                if (d && e) {
                    u(!0);
                    try {
                        let r = await n.mutateAsync({
                            params: {
                                limitOrder: d
                            },
                            payer: e,
                            feeParams: {
                                microLamports: "5000"
                            }
                        });
                        (await l(r, {
                            successToast: {
                                title: "Limit Order Deleted",
                                description: "Your limit order has been successfully deleted."
                            },
                            errorToast: {
                                title: "Delete Failed",
                                description: "Failed to delete the limit order. Please try again."
                            }
                        })).success && setTimeout( () => {
                            t.invalidateQueries({
                                queryKey: ["limit-orders", "user-open", e]
                            })
                        }
                        , 3e3)
                    } catch (e) {} finally {
                        u(!1),
                        m("")
                    }
                }
            }
            ;
            return {
                isDeleting: c || n.isPending,
                handleDeleteOrder: p,
                selectedDeletedOrder: d,
                setSelectedDeletedOrder: m
            }
        }
    }
    ,
    67725: (e, t, n) => {
        n.d(t, {
            Qt: () => s,
            ag: () => l,
            p4: () => o,
            vh: () => i
        });
        var r = n(59271);
        function a(e) {
            return "So11111111111111111111111111111111111111112" === e.address ? {
                ...e,
                name: "SOL"
            } : e
        }
        async function i() {
            try {
                let e = await fetch("/api/tokens/verified", {
                    method: "GET",
                    headers: {
                        Accept: "application/json"
                    }
                });
                if (!e.ok)
                    throw Error("Failed to fetch verified tokens: ".concat(e.status));
                let t = await e.json();
                if (!t.success)
                    throw Error("API returned unsuccessful response");
                return t.results.map(a)
            } catch (e) {
                return r.R.error("Error fetching verified tokens:", e),
                []
            }
        }
        async function s() {
            try {
                let e = await fetch("/api/tokens/lst", {
                    method: "GET",
                    headers: {
                        Accept: "application/json"
                    }
                });
                if (!e.ok)
                    throw Error("Failed to fetch LST tokens: ".concat(e.status));
                let t = await e.json();
                if (!t.success)
                    throw Error("API returned unsuccessful response");
                return t.results.map(a)
            } catch (e) {
                return r.R.error("Error fetching LST tokens:", e),
                []
            }
        }
        async function o(e) {
            try {
                let t = await fetch("/api/tokens/multiple", {
                    method: "POST",
                    headers: {
                        Accept: "application/json",
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        addresses: e
                    })
                });
                if (!t.ok)
                    throw Error("Failed to fetch multiple tokens: ".concat(t.status));
                let n = await t.json();
                if (!n.success)
                    throw Error("API returned unsuccessful response");
                let r = {};
                return n.results.forEach(e => {
                    let t = a(e);
                    r[t.address] = t
                }
                ),
                r
            } catch (e) {
                return r.R.error("Error fetching multiple tokens:", e),
                {}
            }
        }
        async function l(e, t) {
            try {
                let n = await fetch("/api/tokens/search", {
                    method: "POST",
                    headers: {
                        Accept: "application/json",
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        query: e
                    }),
                    signal: t
                });
                if (!n.ok)
                    throw Error("Failed to search tokens: ".concat(n.status));
                let r = await n.json();
                if (!r.success)
                    throw Error("API returned unsuccessful response");
                return r.results.map(a)
            } catch (e) {
                if (e instanceof Error && "AbortError" === e.name)
                    return [];
                return r.R.error("Error searching tokens:", e),
                []
            }
        }
    }
    ,
    70734: (e, t, n) => {
        n.d(t, {
            i: () => u
        });
        var r = n(48876)
          , a = n(27442)
          , i = n(51816)
          , s = n(26432)
          , o = n(8626)
          , l = n(93052)
          , c = n(52630);
        let u = e => {
            let {copyContent: t, className: n, copyIconClassName: u, copiedIconClassName: d, iconClassName: m} = e
              , [p,f] = (0,
            s.useState)(!1);
            return (0,
            s.useEffect)( () => {
                p && setTimeout( () => {
                    f(!1)
                }
                , 1500)
            }
            , [p]),
            (0,
            r.jsx)(a.P.button, {
                className: (0,
                c.cn)("text-brand grid place-items-center", n),
                initial: {
                    scale: 1
                },
                whileTap: {
                    scale: .95
                },
                onClick: () => {
                    navigator.clipboard.writeText(t),
                    f(!0)
                }
                ,
                children: (0,
                r.jsx)("div", {
                    className: "z-elevated relative",
                    children: (0,
                    r.jsx)(i.N, {
                        mode: "popLayout",
                        children: p ? (0,
                        r.jsx)(a.P.span, {
                            animate: {
                                scale: 1
                            },
                            className: "block",
                            exit: {
                                transition: {
                                    duration: .1
                                },
                                opacity: 0,
                                scale: 0
                            },
                            initial: {
                                scale: .2
                            },
                            transition: {
                                type: "spring",
                                stiffness: 200,
                                damping: 18
                            },
                            children: (0,
                            r.jsx)(l.S, {
                                className: (0,
                                c.cn)("z-elevated relative size-3", m, d)
                            })
                        }, "copied") : (0,
                        r.jsx)(a.P.span, {
                            animate: {
                                scale: 1
                            },
                            className: "block",
                            exit: {
                                transition: {
                                    duration: .1
                                },
                                opacity: 0,
                                scale: 0
                            },
                            initial: {
                                scale: .2
                            },
                            transition: {
                                type: "spring",
                                stiffness: 200,
                                damping: 18
                            },
                            children: (0,
                            r.jsx)(o.Td, {
                                className: (0,
                                c.cn)("z-elevated relative size-3", m, u)
                            })
                        }, "clipboard")
                    })
                })
            })
        }
    }
    ,
    72873: (e, t, n) => {
        n.d(t, {
            N: () => s
        });
        var r = n(82945)
          , a = n(59271);
        class i {
            initialize(e, t) {
                this.wsUrl = e,
                this.queryClient = t
            }
            start(e) {
                if (this.walletAddress = e,
                this.reconnectAttempts = 0,
                !this.wsUrl)
                    return void a.R.info("[TxHistoryWS] No WS URL configured; using local ingestion only");
                this.connect()
            }
            stop() {
                this.cleanup(),
                this.walletAddress = null,
                this.reconnectAttempts = 0
            }
            async sendMessage(e) {
                if (!this.socket || this.socket.readyState !== WebSocket.OPEN)
                    return a.R.warn("[TxHistoryWS] Cannot send message: WebSocket not connected"),
                    !1;
                try {
                    let t = JSON.stringify(e);
                    return this.socket.send(t),
                    a.R.debug("[TxHistoryWS] Message sent:", e),
                    !0
                } catch (e) {
                    return a.R.error("[TxHistoryWS] Failed to send message:", e),
                    !1
                }
            }
            ingestSwapHistory(e) {
                var t;
                if (!this.walletAddress || !this.queryClient)
                    return;
                let n = r.l.txHistory.user(this.walletAddress)
                  , a = [...e.swaps].sort( (e, t) => t.timestamp - e.timestamp)
                  , i = this.buildSummaryFromSwaps(a)
                  , s = {
                    swaps: a,
                    summary: i,
                    hasMore: !e.endOfHistory,
                    count: null != (t = e.count) ? t : a.length
                };
                this.queryClient.setQueryData(n, s)
            }
            ingestSwapHistoryExtended(e) {
                if (!this.walletAddress || !this.queryClient)
                    return;
                let t = r.l.txHistory.user(this.walletAddress);
                this.queryClient.setQueryData(t, t => {
                    let n = (null == t ? void 0 : t.swaps) || []
                      , r = e.swaps
                      , i = new Map;
                    n.forEach(e => {
                        i.set(e.signature, e)
                    }
                    ),
                    r.forEach(e => {
                        if (i.has(e.signature)) {
                            var t;
                            (null == (t = i.get(e.signature)) ? void 0 : t.trading_edge) || i.set(e.signature, e),
                            a.R.debug("[TxHistoryWS] Duplicate transaction detected:", e.signature)
                        } else
                            i.set(e.signature, e)
                    }
                    );
                    let s = Array.from(i.values()).sort( (e, t) => t.timestamp - e.timestamp)
                      , o = this.buildSummaryFromSwaps(s);
                    return {
                        ...t,
                        swaps: s,
                        summary: o,
                        hasMore: !e.endOfHistory,
                        count: s.length
                    }
                }
                )
            }
            ingestNewSwap(e) {
                if (!this.walletAddress || !this.queryClient)
                    return;
                let t = r.l.txHistory.user(this.walletAddress);
                this.queryClient.setQueryData(t, t => this.addOrMergeTx(t, e.swap))
            }
            ingestSwapEnriched(e) {
                if (!this.walletAddress || !this.queryClient)
                    return;
                let t = r.l.txHistory.user(this.walletAddress);
                this.queryClient.setQueryData(t, t => this.addOrMergeTx(t, e.swap))
            }
            ingestEndOfHistory() {
                if (!this.walletAddress || !this.queryClient)
                    return;
                let e = r.l.txHistory.user(this.walletAddress);
                this.queryClient.setQueryData(e, e => ({
                    ...e,
                    hasMore: !1
                }))
            }
            connect() {
                if (this.wsUrl && this.walletAddress && !this.isConnecting && (!this.socket || this.socket.readyState !== WebSocket.OPEN))
                    try {
                        this.isConnecting = !0;
                        let e = new URL(this.wsUrl);
                        this.socket = new WebSocket(e.toString()),
                        this.socket.onopen = () => {
                            this.isConnecting = !1,
                            this.reconnectAttempts = 0,
                            a.R.info("[TxHistoryWS] Connected")
                        }
                        ,
                        this.socket.onmessage = e => {
                            try {
                                let t = JSON.parse(e.data);
                                this.handleMessage(t)
                            } catch (e) {
                                a.R.warn("[TxHistoryWS] Failed to parse message", e)
                            }
                        }
                        ,
                        this.socket.onclose = () => {
                            this.isConnecting = !1,
                            this.handleReconnect()
                        }
                        ,
                        this.socket.onerror = () => {}
                    } catch (e) {
                        this.isConnecting = !1,
                        a.R.error("[TxHistoryWS] Failed to open socket", e),
                        this.handleReconnect()
                    }
            }
            handleMessage(e) {
                switch (e.type) {
                case "swap_history":
                    this.ingestSwapHistory(e);
                    break;
                case "swap_history_extended":
                    this.ingestSwapHistoryExtended(e);
                    break;
                case "new_swap":
                    this.ingestNewSwap(e);
                    break;
                case "swap_enriched":
                    this.ingestSwapEnriched(e);
                    break;
                case "end_of_history":
                    this.ingestEndOfHistory()
                }
            }
            addOrMergeTx(e, t) {
                let n, r = e || {
                    swaps: [],
                    summary: {
                        totalRequested: 0,
                        totalProcessed: 0,
                        swapsFound: 0,
                        validSwaps: 0,
                        deepSearch: !1,
                        hasMore: !0,
                        oldestTimestamp: 0,
                        oldestDate: ""
                    },
                    hasMore: !1,
                    count: 0
                }, a = r.swaps.findIndex(e => e.signature === t.signature);
                if (a >= 0) {
                    let e = {
                        ...r.swaps[a],
                        ...t
                    };
                    (n = [...r.swaps])[a] = e
                } else
                    n = [t, ...r.swaps];
                return {
                    ...r,
                    swaps: n,
                    count: n.length
                }
            }
            handleReconnect() {
                if (this.cleanupSocketOnly(),
                !this.wsUrl || !this.walletAddress || this.reconnectAttempts >= 5)
                    return;
                this.reconnectAttempts += 1;
                let e = Math.min(500 * Math.pow(1.5, this.reconnectAttempts), 1e4);
                this.reconnectTimeout = setTimeout( () => this.connect(), e)
            }
            buildSummaryFromSwaps(e) {
                let t = e.length
                  , n = e.map(e => e.timestamp)
                  , r = n.length ? Math.min(...n) : 0;
                return {
                    totalRequested: t,
                    totalProcessed: t,
                    swapsFound: t,
                    validSwaps: t,
                    deepSearch: !1,
                    hasMore: !0,
                    oldestTimestamp: r,
                    oldestDate: r ? new Date(1e3 * r).toISOString() : ""
                }
            }
            cleanupSocketOnly() {
                if (this.socket) {
                    try {
                        this.socket.onopen = null,
                        this.socket.onmessage = null,
                        this.socket.onclose = null,
                        this.socket.onerror = null,
                        (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING) && this.socket.close()
                    } catch (e) {}
                    this.socket = null
                }
                this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null),
                this.isConnecting = !1
            }
            cleanup() {
                this.cleanupSocketOnly()
            }
            constructor() {
                this.socket = null,
                this.isConnecting = !1,
                this.reconnectAttempts = 0,
                this.reconnectTimeout = null,
                this.wsUrl = null,
                this.walletAddress = null,
                this.queryClient = null
            }
        }
        let s = new i
    }
    ,
    72938: (e, t, n) => {
        n.d(t, {
            hf: () => c,
            h8: () => l
        });
        var r = n(76013)
          , a = n(90272)
          , i = n(59271);
        async function s(e, t, n) {
            try {
                var r;
                i.R.info("Creating limit order:", {
                    params: e
                });
                let a = await fetch("/api/limit-orders/create", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        params: e,
                        payer: t,
                        feeParams: n
                    })
                });
                if (!a.ok) {
                    let e = await a.json().catch( () => ({}));
                    throw Error("Failed to create limit order: ".concat(a.status, " - ").concat(e.error || a.statusText))
                }
                let s = await a.json();
                if (!s.transaction || !(null == (r = s.details) ? void 0 : r.limitOrder))
                    throw Error("Invalid limit order response format");
                return i.R.info("Limit order created successfully:", {
                    limitOrder: s.details.limitOrder
                }),
                s
            } catch (e) {
                throw i.R.error("Limit order creation error:", e),
                e instanceof Error ? e : Error("Unknown error creating limit order")
            }
        }
        async function o(e, t, n) {
            try {
                var r;
                i.R.info("Canceling limit order:", {
                    limitOrder: e.limitOrder,
                    payer: t
                });
                let a = await fetch("/api/limit-orders/cancel", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        params: e,
                        payer: t,
                        feeParams: n
                    })
                });
                if (!a.ok) {
                    let e = await a.json().catch( () => ({}));
                    throw Error("Failed to cancel limit order: ".concat(a.status, " - ").concat(e.error || a.statusText))
                }
                let s = await a.json();
                if (!s.transaction || !(null == (r = s.details) ? void 0 : r.limitOrder))
                    throw Error("Invalid limit order response format");
                return i.R.info("Limit order canceled successfully:", {
                    limitOrder: s.details.limitOrder
                }),
                s
            } catch (e) {
                throw i.R.error("Limit order cancellation error:", e),
                e instanceof Error ? e : Error("Unknown error canceling limit order")
            }
        }
        function l() {
            let e = (0,
            r.jE)();
            return (0,
            a.n)({
                mutationFn: async e => {
                    let {params: t, payer: n, feeParams: r} = e;
                    i.R.info("useCreateLimitOrder: Starting limit order creation", {
                        params: t
                    });
                    let a = await s(t, n, r);
                    return i.R.info("useCreateLimitOrder: Limit order creation completed", {
                        limitOrder: a.details.limitOrder,
                        hasTransaction: !!a.transaction
                    }),
                    a
                }
                ,
                onSuccess: t => {
                    i.R.info("Limit order created successfully:", t),
                    console.log("\uD83C\uDF89 Limit Order Created Successfully:", {
                        limitOrder: t.details.limitOrder,
                        transaction: t.transaction,
                        timestamp: new Date().toISOString()
                    }),
                    e.invalidateQueries({
                        queryKey: ["limit-orders"]
                    })
                }
                ,
                onError: e => {
                    i.R.error("Limit order creation failed:", e),
                    console.error("❌ Limit Order Creation Failed:", {
                        error: e.message,
                        timestamp: new Date().toISOString()
                    })
                }
            })
        }
        function c() {
            return (0,
            a.n)({
                mutationFn: async e => {
                    let {params: t, payer: n, feeParams: r} = e;
                    i.R.info("useCancelLimitOrder: Starting limit order cancellation", {
                        limitOrder: t.limitOrder,
                        payer: n
                    });
                    let a = await o(t, n, r);
                    return i.R.info("useCancelLimitOrder: Limit order cancellation completed", {
                        limitOrder: a.details.limitOrder,
                        hasTransaction: !!a.transaction
                    }),
                    a
                }
                ,
                onSuccess: e => {
                    i.R.info("Limit order canceled successfully:", e),
                    console.log("\uD83D\uDDD1️ Limit Order Canceled Successfully:", {
                        limitOrder: e.details.limitOrder,
                        transaction: e.transaction,
                        timestamp: new Date().toISOString()
                    })
                }
                ,
                onError: e => {
                    i.R.error("Limit order cancellation failed:", e),
                    console.error("❌ Limit Order Cancellation Failed:", {
                        error: e.message,
                        timestamp: new Date().toISOString()
                    })
                }
            })
        }
    }
    ,
    73656: (e, t, n) => {
        n.d(t, {
            Y: () => a,
            Z: () => i
        });
        var r = n(59271);
        async function a(e) {
            var t;
            let n = {
                transactionData: {
                    transaction: e.transaction,
                    lastValidBlockHeight: e.lastValidBlockHeight,
                    recentBlockhash: e.recentBlockhash,
                    is_jito_bundle: null != (t = e.isJitoBundle) && t,
                    broadcastMode: e.broadcastMode,
                    options: e.options
                }
            };
            r.R.info("Sending transaction to gateway worker:", {
                broadcastMode: e.broadcastMode,
                isJitoBundle: e.isJitoBundle,
                hasOptions: !!e.options,
                lastValidBlockHeight: e.lastValidBlockHeight
            });
            let a = await fetch("/api/transaction", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(n)
            });
            if (!a.ok)
                throw Error((await a.json().catch( () => ({
                    success: !1,
                    error: "HTTP ".concat(a.status, ": ").concat(a.statusText)
                }))).error || "Gateway worker request failed: ".concat(a.status));
            let i = await a.json();
            if (!i.success || !i.transactionHashes)
                throw Error(i.error || "Gateway worker returned no transaction hashes");
            return r.R.info("Gateway worker response:", {
                success: i.success,
                hashCount: i.transactionHashes.length,
                hashes: i.transactionHashes
            }),
            i.transactionHashes
        }
        function i(e, t) {
            return "mev-protect" === e && "jito" === t
        }
    }
    ,
    73861: (e, t, n) => {
        n.d(t, {
            A: () => i
        });
        var r = n(19618)
          , a = n(188);
        let i = (0,
        n(38915).h)()((0,
        r.Zr)(e => ({
            isLoadingToken: !1,
            setIsLoadingToken: t => e({
                isLoadingToken: t
            }),
            allTokens: [],
            setAllTokens: t => e({
                allTokens: t
            }),
            lstTokens: [],
            setLstTokens: t => e({
                lstTokens: t
            }),
            popularTokens: [],
            setPopularTokens: t => e({
                popularTokens: t
            }),
            verifiedTokens: [],
            setVerifiedTokens: t => e({
                verifiedTokens: t
            })
        }), {
            name: "token-store",
            partialize: e => ({
                allTokens: e.allTokens,
                lstTokens: e.lstTokens,
                popularTokens: e.popularTokens,
                verifiedTokens: e.verifiedTokens
            })
        }), a.x)
    }
    ,
    73971: (e, t, n) => {
        n.d(t, {
            hf: () => x,
            v4: () => S,
            d2: () => k
        });
        var r = n(55436)
          , a = n(76013)
          , i = n(90272)
          , s = n(26432)
          , o = n(82945)
          , l = n(90529)
          , c = n(92631)
          , u = n(59271)
          , d = n(40476)
          , m = n(91015).Buffer;
        async function p(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : "profile update";
            try {
                let n = await fetch("/api/profile/get-vm", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        wallet_address: e,
                        purpose: t
                    })
                });
                if (!n.ok)
                    throw Error("Failed to get verification message: ".concat(n.status));
                return await n.json()
            } catch (e) {
                throw u.R.error("Failed to get verification message:", e),
                e instanceof Error ? e : Error("Unknown error getting verification message")
            }
        }
        async function f(e, t, r, a) {
            if (!e)
                return {
                    success: !1,
                    error: "Wallet does not support message signing"
                };
            try {
                u.R.info("\uD83D\uDD10 Requesting wallet signature for message");
                let i = new d.PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")
                  , s = await r.getLatestBlockhash({
                    commitment: "max"
                }).then(e => e.blockhash)
                  , o = new d.TransactionInstruction({
                    programId: i,
                    keys: [],
                    data: m.from(a, "utf8")
                })
                  , l = new d.TransactionMessage({
                    payerKey: t,
                    recentBlockhash: s,
                    instructions: [o]
                }).compileToV0Message()
                  , c = new d.VersionedTransaction(l)
                  , p = await e(c)
                  , f = (await Promise.resolve().then(n.bind(n, 76535))).default.encode(p.signatures[0])
                  , g = m.from(p.serialize()).toString("base64");
                return u.R.info("✅ Message signed successfully"),
                {
                    success: !0,
                    signature: f,
                    message: a,
                    serialized_tx: g
                }
            } catch (e) {
                if (u.R.warn("⚠️ Signature failed:", e),
                4001 === e.code)
                    return {
                        success: !1,
                        error: "User cancelled signature request"
                    };
                return {
                    success: !1,
                    error: e.message || "Failed to sign message"
                }
            }
        }
        async function g(e, t) {
            if (!e)
                return {
                    success: !1,
                    error: "Wallet does not support message signing"
                };
            try {
                u.R.info("\uD83D\uDD10 Requesting wallet signature for message");
                let r = new TextEncoder().encode(t)
                  , a = await e(r)
                  , i = (await Promise.resolve().then(n.bind(n, 76535))).default.encode(a);
                return u.R.info("✅ Message signed successfully"),
                {
                    success: !0,
                    signature: i,
                    message: t
                }
            } catch (e) {
                if (u.R.warn("⚠️ Signature failed:", e),
                4001 === e.code)
                    return {
                        success: !1,
                        error: "User cancelled signature request"
                    };
                return {
                    success: !1,
                    error: e.message || "Failed to sign message"
                }
            }
        }
        async function h(e, t, n, r, a) {
            let i = arguments.length > 5 && void 0 !== arguments[5] ? arguments[5] : "profile update"
              , s = arguments.length > 6 ? arguments[6] : void 0;
            try {
                let o, l = await p(e, i);
                return s ? await f(t, r, a, l.message) : await g(n, l.message)
            } catch (e) {
                return u.R.warn("Signature verification flow failed:", e),
                {
                    success: !1,
                    error: e.message || "Signature verification failed"
                }
            }
        }
        async function y(e, t) {
            try {
                let n = new URL("/api/profile/ensure-profile",window.location.origin);
                n.searchParams.append("privy_id", e || ""),
                n.searchParams.append("wallet_address", t);
                let r = await fetch(n.toString(), {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!r.ok) {
                    let e = await r.json().catch( () => ({
                        error: "Unknown error occurred"
                    }))
                      , t = e.details ? "".concat(e.error, ": ").concat(e.details) : e.error || "HTTP error! status: ".concat(r.status);
                    throw Error(t)
                }
                let a = await r.json();
                if (u.R.info("\uD83D\uDD35 Profile data:", a),
                !a.hasOwnProperty("exists"))
                    throw Error("Invalid response format from profile service");
                return a
            } catch (e) {
                throw u.R.error("Profile fetch error:", e),
                e instanceof Error ? e : Error("Unknown error fetching profile")
            }
        }
        async function w(e) {
            try {
                let t = new URL("/api/profile/check-username",window.location.origin);
                t.searchParams.append("username", e);
                let n = await fetch(t.toString(), {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json"
                    }
                });
                if (!n.ok) {
                    let e = await n.json().catch( () => ({
                        error: "Unknown error occurred"
                    }))
                      , t = e.details ? "".concat(e.error, ": ").concat(e.details) : e.error || "HTTP error! status: ".concat(n.status);
                    throw Error(t)
                }
                let r = await n.json();
                if (!r.hasOwnProperty("available") || !r.username)
                    throw Error("Invalid response format from username check service");
                return r
            } catch (e) {
                throw u.R.error("Username check error:", e),
                e instanceof Error ? e : Error("Unknown error checking username")
            }
        }
        async function v(e, t) {
            try {
                if (!(null == t ? void 0 : t.walletSignTransaction) || !(null == t ? void 0 : t.publicKey) || !(null == t ? void 0 : t.connection))
                    throw Error("Wallet signing capability is required for profile updates");
                if (null == t ? void 0 : t.skipSignature)
                    throw Error("Signature verification cannot be skipped");
                u.R.info("\uD83D\uDD10 Starting mandatory signature verification for profile update");
                let n = await h(e.wallet_address, t.walletSignTransaction, t.walletSignMessage, t.publicKey, t.connection, "profile update", t.isLedger);
                if (u.R.info("\uD83D\uDD10 Signature result:", n),
                !n.success)
                    throw Error("Signature verification failed: ".concat(n.error || "Unknown error"));
                let r = {
                    ...e,
                    signature: n.signature,
                    message: n.message
                };
                t.isLedger && (r.serialized_tx = n.serialized_tx),
                u.R.info("✅ Signature verification successful, proceeding with profile update");
                let a = await fetch("/api/profile/set", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify(r)
                });
                if (!a.ok) {
                    let e = await a.json().catch( () => ({
                        error: "Unknown error occurred"
                    }))
                      , t = e.details ? "".concat(e.error, ": ").concat(e.details) : e.error || "HTTP error! status: ".concat(a.status);
                    throw Error(t)
                }
                let i = await a.json();
                if (!i.hasOwnProperty("success"))
                    throw Error("Invalid response format from profile update service");
                return i
            } catch (e) {
                throw u.R.error("Profile update error:", e),
                e instanceof Error ? e : Error("Unknown error updating profile")
            }
        }
        var b = n(77813);
        function S(e, t) {
            return (0,
            r.I)({
                queryKey: o.l.profile.user(null, t || ""),
                queryFn: async () => {
                    if (!t)
                        throw Error("wallet_address is required");
                    return y(null, t)
                }
                ,
                enabled: !!t,
                staleTime: 3e5,
                gcTime: 9e5
            })
        }
        function k() {
            let e = (0,
            a.jE)()
              , {signTransaction: t, signMessage: n, connected: r, publicKey: s, connection: c} = (0,
            l.z)();
            return (0,
            i.n)({
                mutationFn: e => {
                    if (!r)
                        throw Error("Wallet must be connected to update profile");
                    if (!t || !n)
                        throw Error("Wallet does not support message signing. Please use a compatible wallet.");
                    return v(e, {
                        walletSignTransaction: t,
                        walletSignMessage: n,
                        skipSignature: !1,
                        publicKey: s,
                        connection: c,
                        isLedger: e.is_ledger_wallet || !1
                    })
                }
                ,
                retry: !1,
                onSuccess: (t, n) => {
                    e.invalidateQueries({
                        queryKey: o.l.profile.user(null, n.wallet_address)
                    }),
                    n.username && e.removeQueries({
                        queryKey: o.l.profile.usernameCheck(n.username)
                    })
                }
            })
        }
        function x(e) {
            var t, n, a, i, l, u, d;
            let m = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : 500
              , p = (0,
            c.d)(e, m)
              , f = (0,
            r.I)({
                queryKey: o.l.profile.usernameCheck(p || ""),
                queryFn: async () => {
                    if (!p)
                        throw Error("Username is required");
                    return w(p)
                }
                ,
                enabled: !!p && p.length >= 3,
                staleTime: 12e4,
                gcTime: 6e5
            })
              , g = (0,
            s.useMemo)( () => e && 0 !== e.length ? /^[a-zA-Z0-9_]+$/.test(e) ? e.length < 4 || e.length > 20 ? {
                valid: !1,
                error: "Username must be 4-20 characters"
            } : (0,
            b.y)(e) ? {
                valid: !1,
                error: "Username is not valid"
            } : {
                valid: !0
            } : {
                valid: !1,
                error: "Username can only contain letters, numbers, and underscores"
            } : null, [e])
              , h = e !== p && !!e;
            return {
                isValid: null != (l = null == g ? void 0 : g.valid) && l,
                isAvailable: null == f || null == (t = f.data) ? void 0 : t.available,
                canUseUsername: null != (u = null == g ? void 0 : g.valid) && u && null != (d = null == (n = f.data) ? void 0 : n.available) && d,
                isCheckingAvailability: (null == g ? void 0 : g.valid) && f.isFetching,
                isValidating: h || (null == g ? void 0 : g.valid) && f.isFetching,
                validationError: null == g ? void 0 : g.error,
                availabilityError: (null == (a = f.data) ? void 0 : a.error) || (null == (i = f.error) ? void 0 : i.message),
                availabilityData: f.data,
                availabilityQuery: f
            }
        }
    }
    ,
    77813: (e, t, n) => {
        n.d(t, {
            K: () => a,
            y: () => i
        });
        let r = new (n(38871)).d;
        r.addWords("titan", "titanexchange", "titan_exchange", "titandex");
        let a = e => e.length < 4 || e.length > 20 ? {
            valid: !1,
            error: "Username must be 4-20 characters"
        } : /^[a-zA-Z0-9_]+$/.test(e) ? {
            valid: !0
        } : {
            valid: !1,
            error: "Username can only contain letters, numbers, and underscores"
        }
          , i = e => !!e.trim() && r.isProfane(e)
    }
    ,
    80032: (e, t, n) => {
        n.d(t, {
            Ay: () => o
        });
        var r = n(38915)
          , a = n(188);
        let i = {
            isVip: !1,
            vipType: void 0,
            reason: void 0,
            notified: void 0,
            holdingAssets: []
        }
          , s = {
            exists: !1,
            count: null,
            isLoading: !0,
            error: null
        }
          , o = (0,
        r.h)()( (e, t, n) => ({
            walletVipStatus: i,
            setWalletVipStatus: t => e({
                walletVipStatus: t || i
            }),
            vipStatusLoading: !1,
            setVipStatusLoading: t => e({
                vipStatusLoading: t
            }),
            sponsoredTransactionStatus: s,
            setSponsoredTransactionStatus: t => e({
                sponsoredTransactionStatus: t
            }),
            priorityLaneEnabled: !1,
            setPriorityLaneEnabled: t => e({
                priorityLaneEnabled: t
            }),
            reset: () => {
                e(n.getInitialState())
            }
        }), a.x)
    }
    ,
    86698: (e, t, n) => {
        n.d(t, {
            s: () => r
        });
        let r = e => {
            switch (e) {
            case "jupiter":
            case "metis":
                return "/images/sources/jupiter.png";
            case "hashflow":
                return "/images/sources/hashflow.png";
            case "pyth":
                return "/images/sources/per.png";
            case "okx":
                return "/images/sources/okx.svg";
            case "dflow":
                return "/images/sources/dflow.svg";
            default:
                return "/images/sources/titan.png"
            }
        }
    }
    ,
    88368: (e, t, n) => {
        n.d(t, {
            A2: () => o,
            W2: () => c,
            kB: () => s,
            s8: () => a,
            t7: () => i,
            x_: () => l
        });
        var r = n(41043);
        let a = {
            SUB_PROTOCOL: "v1.api.titan.ag",
            CONNECTION_TIMEOUT_MS: 1e4,
            INITIAL_RECONNECTION_DELAY_MS: 500,
            RECONNECTION_BACKOFF_FACTOR: 1.5,
            MAX_RECONNECTION_DELAY_MS: 1e4,
            MAX_RECONNECTION_ATTEMPTS: 5
        }
          , i = {
            SLIPPAGE_BPS: 50,
            ONLY_DIRECT_ROUTES: !1,
            ADD_SIZE_CONSTRAINT: !0
        }
          , s = {
            id: null,
            inputMint: new Uint8Array,
            outputMint: new Uint8Array,
            swapMode: r.z.ExactIn,
            amount: 0,
            quotes: {}
        }
          , o = {
            INITIAL: 1,
            MAX: 999999
        };
        function l(e) {
            let t;
            if (!e || 0 === Object.keys(e).length)
                return;
            let n = ""
              , r = 0;
            return Object.entries(e.quotes).forEach(a => {
                let[i,s] = a;
                0 === s.steps.length && (s.steps[0] = {
                    allocPpb: null,
                    ammKey: null,
                    label: null,
                    inAmount: s.inAmount,
                    outAmount: s.outAmount,
                    inputMint: e.inputMint,
                    outputMint: e.outputMint
                }),
                s.outAmount > r ? (r = s.outAmount,
                n = i,
                t = s) : s.outAmount === r && r > 0 && ("Titan" === i && "Titan" !== n ? (n = i,
                t = s) : "Titan" !== n && "Titan" !== i && ("" === n || 0 > i.localeCompare(n)) && (n = i,
                t = s))
            }
            ),
            t ? {
                provider: n,
                route: t
            } : void 0
        }
        let c = "Timeout - No Routes Found"
    }
    ,
    91975: (e, t, n) => {
        n.d(t, {
            A: () => o
        });
        var r = n(48876)
          , a = n(2392)
          , i = n(52630);
        let s = (0,
        n(73296).tv)({
            slots: {
                base: ["flex", "w-fit", "justify-between", "items-start", "group", "gap-2"],
                label: ["text-text-high-em", "text-body-m", "select-none", "font-medium", "data-[disabled]:text-text-disabled"],
                description: ["text-text-mid-em", "text-body-m", "block", "data-[disabled]:text-text-disabled"],
                root: ["cursor-pointer", "p-0.5", "min-w-[2.5rem]", "max-w-[2.875rem]", "h-6", "rounded-full", "relative", "shadow", "tap-highlight-transparent", "transition-colors", "duration-300", "[&[data-state=checked][data-disabled]]:bg-brand", "[&[data-state=unchecked][data-disabled]]:bg-bg-high-em", "data-[state=checked]:bg-brand", "data-[state=unchecked]:bg-bg-high-em", "group-hover:data-[state=unchecked]:bg-brand/10", "focus-ring"],
                thumb: ["data-[state=checked]:translate-x-[16px]", "block rounded-full w-5 h-5", "flex items-center justify-center", "data-[state=checked]:bg-bg-low-em", "data-[state=unchecked]:bg-text-high-em", "[&[data-state=unchecked][data-disabled]]:bg-text-disabled", "[&[data-state=checked][data-disabled]]:bg-[hsla(47,63%,20%,1)]", "transition-all duration-300"]
            }
        })
          , o = e => {
            let {className: t, id: n, label: o, disabled: l, labelProps: c, description: u, thumbContent: d, customBackgroundContent: m, descriptionProps: p, wrapperProps: f, thumbProps: g, ...h} = e
              , {base: y, label: w, description: v, root: b, thumb: S} = s();
            return (0,
            r.jsx)("div", {
                className: y({
                    className: t
                }),
                ...f,
                children: (0,
                r.jsxs)(a.bL, {
                    disabled: l,
                    id: n,
                    ...h,
                    className: b({
                        className: t
                    }),
                    children: [(0,
                    r.jsx)("div", {
                        className: "absolute top-0 left-0 h-full w-full",
                        children: m
                    }), (0,
                    r.jsx)(a.zi, {
                        ...g,
                        className: S({
                            className: (0,
                            i.cn)(null == g ? void 0 : g.className)
                        }),
                        children: d
                    }), (o || u) && (0,
                    r.jsx)("div", {
                        className: "flex flex-1 flex-col gap-1",
                        children: o && (0,
                        r.jsxs)("label", {
                            "data-disabled": l ? "" : void 0,
                            htmlFor: n,
                            ...c,
                            className: w({
                                className: null == c ? void 0 : c.className
                            }),
                            children: [o, (0,
                            r.jsx)("span", {
                                ...p,
                                className: v({
                                    className: null == p ? void 0 : p.className
                                }),
                                "data-disabled": l ? "" : void 0,
                                children: u
                            })]
                        })
                    })]
                })
            })
        }
    }
    ,
    92191: (e, t, n) => {
        n.d(t, {
            N: () => m
        });
        var r = n(40476)
          , a = n(26432)
          , i = n(37998)
          , s = n(90529)
          , o = n(72938)
          , l = n(41313)
          , c = n(73656)
          , u = n(59271)
          , d = n(91015).Buffer;
        let m = () => {
            let {connected: e, signTransaction: t, connection: n, disconnect: m} = (0,
            s.z)()
              , {settings: p} = (0,
            l.t0)()
              , f = (0,
            o.h8)()
              , [g,h] = (0,
            a.useState)("idle")
              , y = "signing" === g || "sending" === g || f.isPending
              , w = (0,
            a.useMemo)( () => e && !!t && !!n && "idle" === g, [e, t, n, g])
              , v = (0,
            a.useCallback)(e => {
                try {
                    let t = d.from(e, "base64");
                    return r.VersionedTransaction.deserialize(t)
                } catch (e) {
                    throw u.R.error("Failed to deserialize transaction:", e),
                    Error("Invalid transaction format")
                }
            }
            , [])
              , b = (0,
            a.useCallback)(async (e, r) => {
                if (!t || !n)
                    return {
                        success: !1,
                        error: "Wallet not connected"
                    };
                try {
                    h("signing"),
                    u.R.info("Starting limit order transaction execution", {
                        limitOrder: e.details.limitOrder
                    });
                    let a = v(e.transaction);
                    u.R.info("Transaction deserialized successfully", {
                        hasSignatures: a.signatures.length > 0,
                        signatureCount: a.signatures.length
                    });
                    let s = await t(a);
                    u.R.info("Transaction signed successfully", {
                        signatureCount: s.signatures.length
                    }),
                    h("sending");
                    let {blockhash: o, lastValidBlockHeight: l} = await n.getLatestBlockhash("confirmed")
                      , m = d.from(s.serialize()).toString("base64")
                      , f = await (0,
                    c.Y)({
                        transaction: m,
                        recentBlockhash: o,
                        lastValidBlockHeight: l,
                        broadcastMode: p.txFeeSettings.broadcastMode,
                        isJitoBundle: "mev-protect" === p.txFeeSettings.broadcastMode
                    });
                    if (0 === f.length)
                        throw Error("No transaction hashes returned from gateway");
                    let g = f[0];
                    u.R.info("Limit order transaction sent successfully", {
                        signature: g,
                        limitOrder: e.details.limitOrder
                    }),
                    h("idle");
                    let y = (null == r ? void 0 : r.successToast) || {
                        title: "Limit Order Created",
                        description: "Your limit order has been created successfully."
                    };
                    return (0,
                    i.o)({
                        title: y.title,
                        description: y.description,
                        variant: "success",
                        duration: 5e3
                    }),
                    {
                        success: !0,
                        signature: g
                    }
                } catch (t) {
                    h("idle");
                    let e = t instanceof Error ? t.message : "Unknown error";
                    if (u.R.error("Limit order execution failed:", t),
                    e.includes("User rejected")) {
                        let e = (null == r ? void 0 : r.errorToast) || {
                            title: "Limit Order Failed",
                            description: "User rejected the transaction"
                        };
                        return (0,
                        i.o)({
                            title: e.title,
                            description: e.description,
                            variant: "alert",
                            duration: 5e3
                        }),
                        {
                            success: !1,
                            error: "User rejected transaction"
                        }
                    }
                    if (e.includes("insufficient")) {
                        let e = (null == r ? void 0 : r.errorToast) || {
                            title: "Insufficient Funds",
                            description: "You don't have enough tokens for this limit order."
                        };
                        (0,
                        i.o)({
                            title: e.title,
                            description: e.description,
                            variant: "alert",
                            duration: 5e3
                        })
                    } else if (e.includes("No wallet found")) {
                        let e = (null == r ? void 0 : r.errorToast) || {
                            title: "Limit Order Failed",
                            description: "User wallet not found. Please reconnect your wallet and try again."
                        };
                        (0,
                        i.o)({
                            title: e.title,
                            description: e.description,
                            variant: "alert",
                            duration: 5e3,
                            buttons: [{
                                children: "Disconnect",
                                onClick: async () => {
                                    m()
                                }
                                ,
                                variant: "secondary"
                            }]
                        })
                    } else if (e.includes("WalletSignTransactionError")) {
                        let e = (null == r ? void 0 : r.errorToast) || {
                            title: "Limit Order Failed",
                            description: "Connected wallet failed to sign the transaction."
                        };
                        (0,
                        i.o)({
                            title: e.title,
                            description: e.description,
                            buttons: [{
                                children: "Disconnect",
                                onClick: async () => {
                                    m()
                                }
                                ,
                                variant: "secondary"
                            }]
                        })
                    } else {
                        let e = (null == r ? void 0 : r.errorToast) || {
                            title: "Limit Order Failed",
                            description: "An unexpected error occurred. Please try again."
                        };
                        (0,
                        i.o)({
                            title: e.title,
                            description: e.description,
                            variant: "alert",
                            duration: 5e3
                        })
                    }
                    return {
                        success: !1,
                        error: e
                    }
                }
            }
            , [t, n, v, p.txFeeSettings.broadcastMode, m]);
            return {
                executeLimitOrder: (0,
                a.useCallback)(async e => {
                    if (!w)
                        return {
                            success: !1,
                            error: "Cannot execute limit order"
                        };
                    try {
                        let t = await f.mutateAsync(e);
                        return await b(t)
                    } catch (t) {
                        let e = t instanceof Error ? t.message : "Unknown error";
                        return u.R.error("Complete limit order execution failed:", t),
                        {
                            success: !1,
                            error: e
                        }
                    }
                }
                , [w, f, b]),
                executeLimitOrderTransaction: b,
                isExecuting: y,
                canExecute: w,
                executionState: g
            }
        }
    }
    ,
    92631: (e, t, n) => {
        n.d(t, {
            d: () => a
        });
        var r = n(26432);
        function a(e, t) {
            let[n,a] = (0,
            r.useState)(e);
            return (0,
            r.useEffect)( () => {
                let n = setTimeout( () => {
                    a(e)
                }
                , t);
                return () => {
                    clearTimeout(n)
                }
            }
            , [e, t]),
            n
        }
    }
    ,
    93355: (e, t, n) => {
        n.d(t, {
            A: () => s
        });
        var r = n(38915)
          , a = n(188)
          , i = n(43106);
        let s = (0,
        r.h)()(e => ({
            sellToken: void 0,
            setSellToken: t => e({
                sellToken: t
            }),
            buyToken: void 0,
            setBuyToken: t => e({
                buyToken: t
            }),
            expireValue: i.PI.Never,
            setExpireValue: t => e({
                expireValue: t
            }),
            sellValue: "",
            setSellValue: t => e({
                sellValue: t
            }),
            buyValue: "",
            setBuyValue: t => e({
                buyValue: t
            }),
            rateValue: "",
            setRateValue: t => e({
                rateValue: t
            }),
            openOrders: [],
            setOpenOrders: t => e({
                openOrders: t
            }),
            expiredOrders: [],
            setExpiredOrders: t => e({
                expiredOrders: t
            }),
            isLoadingOpenOrder: !1,
            setIsLoadingOpenOrder: t => e({
                isLoadingOpenOrder: t
            }),
            errorFetchingOpenOrders: null,
            setErrorFetchingOpenOrders: t => e({
                errorFetchingOpenOrders: t
            })
        }), a.x)
    }
    ,
    96853: (e, t, n) => {
        n.d(t, {
            X2: () => y
        });
        var r = n(55436)
          , a = n(26432)
          , i = n(82945)
          , s = n(40476)
          , o = n(30369)
          , l = n(25465)
          , c = n(59271);
        let u = new s.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
          , d = new s.PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
          , m = new Map
          , p = async function(e, t) {
            let n = !(arguments.length > 2) || void 0 === arguments[2] || arguments[2]
              , r = "".concat(t, "-").concat(n)
              , a = Date.now()
              , i = m.get(r);
            if (i && a - i.timestamp < 1e3)
                return c.R.debug("[getWalletBalances] Returning cached result for wallet:", t),
                i.promise;
            let p = (async () => {
                try {
                    c.R.debug("[getWalletBalances] Called for wallet:", t, {
                        timestamp: new Date().toISOString(),
                        includeMetadata: n
                    });
                    let r = new s.Connection(e,"processed")
                      , a = new s.PublicKey(t)
                      , [i,m,p] = await Promise.allSettled([r.getBalance(a, "processed"), r.getParsedTokenAccountsByOwner(a, {
                        programId: u
                    }, "processed"), r.getParsedTokenAccountsByOwner(a, {
                        programId: d
                    }, "processed")]);
                    if ("rejected" === i.status || "rejected" === m.status || "rejected" === p.status)
                        throw c.R.error("Wallet balances fetch due to rpc issue:", {
                            solBalance: "rejected" === i.status ? i.reason : i.status,
                            splTokensResult: "rejected" === m.status ? m.reason : m.status,
                            token2022Result: "rejected" === p.status ? p.reason : p.status
                        }),
                        Error("Wallet balances fetch due to rpc issue");
                    let f = "fulfilled" === i.status ? i.value : 0
                      , g = "fulfilled" === m.status ? m.value.value.map(e => {
                        let t = e.account.data.parsed.info
                          , n = Number(t.tokenAmount.amount)
                          , r = t.tokenAmount.decimals
                          , a = new o.A(n).div(new o.A(10).pow(r)).toNumber();
                        return {
                            mint: t.mint,
                            owner: t.owner,
                            rawAmount: n,
                            amount: a,
                            decimals: r,
                            tokenAccount: e.pubkey.toString(),
                            programId: u.toString()
                        }
                    }
                    ) : []
                      , h = "fulfilled" === p.status ? p.value.value.map(e => {
                        let t = e.account.data.parsed.info
                          , n = Number(t.tokenAmount.amount)
                          , r = t.tokenAmount.decimals
                          , a = new o.A(n).div(new o.A(10).pow(r)).toNumber();
                        return {
                            mint: t.mint,
                            owner: t.owner,
                            rawAmount: n,
                            amount: a,
                            decimals: r,
                            tokenAccount: e.pubkey.toString(),
                            programId: d.toString()
                        }
                    }
                    ) : []
                      , y = [...g, ...h].filter(e => e.amount > 0 && 0 != e.decimals)
                      , w = [...new Set(y.map(e => e.mint))]
                      , v = [l.wV, ...w]
                      , b = {}
                      , S = {}
                      , k = {
                        method: "POST",
                        headers: {
                            Accept: "application/json",
                            "Content-Type": "application/json"
                        }
                    };
                    if (n) {
                        let[e,t] = await Promise.allSettled([fetch("/api/tokens/multiple", {
                            ...k,
                            body: JSON.stringify({
                                addresses: v
                            })
                        }), fetch("/api/prices", {
                            ...k,
                            body: JSON.stringify({
                                addresses: v
                            })
                        })]);
                        if ("fulfilled" === e.status) {
                            let t = await e.value.json();
                            t && "object" == typeof t && "success"in t && t.success && "results"in t && Array.isArray(t.results) ? (b = {},
                            t.results.forEach(e => {
                                e && "object" == typeof e && "address"in e && (b[e.address] = e)
                            }
                            )) : t && "object" == typeof t && (b = t)
                        } else
                            c.R.error("Token metadata fetch failed:", e.reason);
                        if ("fulfilled" === t.status) {
                            let e = await t.value.json();
                            e && "object" == typeof e && (S = e)
                        } else
                            c.R.error("Token prices fetch failed:", t.reason)
                    }
                    let x = {
                        token: {
                            ...l.Es,
                            address: l.wV,
                            decimals: l.Es.decimals
                        },
                        balance: (0,
                        l.iU)(f),
                        rawBalance: f,
                        price: parseFloat(S[l.wV]) || 0,
                        usdValue: (0,
                        l.iU)(f) * parseFloat(S[l.wV]) || 0,
                        accountAddress: t,
                        isStale: !1
                    }
                      , T = y.map(e => {
                        let t = b[e.mint]
                          , n = {
                            address: e.mint,
                            decimals: e.decimals,
                            symbol: "UNKNOWN",
                            name: "Unknown Token",
                            verified: !1,
                            logoURI: void 0
                        };
                        t && "object" == typeof t && ("string" == typeof t.symbol && (n.symbol = t.symbol),
                        "string" == typeof t.name && (n.name = t.name),
                        "boolean" == typeof t.verified && (n.verified = t.verified),
                        "string" == typeof t.logoURI && (n.logoURI = t.logoURI));
                        let r = parseFloat(S[e.mint]) || 0
                          , a = e.amount * r;
                        return {
                            token: n,
                            balance: e.amount,
                            rawBalance: e.rawAmount,
                            usdValue: a,
                            price: r,
                            accountAddress: e.tokenAccount,
                            isStale: !1
                        }
                    }
                    )
                      , R = [x, ...T].filter(e => e.balance > 0 || e.token.address === l.wV);
                    R.sort( (e, t) => e.token.verified !== t.token.verified ? t.token.verified ? 1 : -1 : t.usdValue !== e.usdValue ? t.usdValue - e.usdValue : t.balance - e.balance);
                    let E = R.reduce( (e, t) => e + t.usdValue, 0);
                    return {
                        walletAddress: t,
                        tokenBalances: R,
                        totalUsdValue: E,
                        lastUpdated: Date.now()
                    }
                } catch (r) {
                    if (r instanceof Error)
                        throw c.R.error("Wallet balances fetch failed:", {
                            walletAddress: t,
                            includeMetadata: n,
                            error: r.message,
                            stack: r.stack
                        }),
                        r;
                    let e = Error("Failed to fetch wallet balances for wallet ".concat(t, ": ").concat(String(r)));
                    throw c.R.error("Wallet balances fetch failed:", {
                        walletAddress: t,
                        includeMetadata: n,
                        error: e.message
                    }),
                    e
                }
            }
            )();
            if (m.set(r, {
                timestamp: a,
                promise: p
            }),
            m.size > 10) {
                let e = a - 1e3;
                for (let[t,n] of m.entries())
                    n.timestamp < e && m.delete(t)
            }
            return p
        };
        var f = n(93739)
          , g = n(15334);
        let h = function(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {enabled: n=!0, refetchInterval: a=!1, staleTime: s=1e4, gcTime: o=12e4} = t
              , {appConfig: l} = (0,
            f.A)();
            return (0,
            r.I)({
                queryKey: i.l.wallet.balances(e || ""),
                queryFn: () => p(l.RPC_NODE_URL, e || ""),
                enabled: n && !!e && !!l.RPC_NODE_URL,
                staleTime: s,
                gcTime: o,
                refetchInterval: a,
                refetchOnWindowFocus: !1,
                retry: (e, t) => e < 3,
                retryDelay: e => Math.min(1e3 * 2 ** e, 3e4)
            })
        }
          , y = e => {
            var t, n;
            let r = h(e)
              , {setBalanceError: i, setBalanceLoading: s, setBalanceStale: o, setTokenBalances: l, setTotalUsdValue: u} = (0,
            g.A)()
              , d = (0,
            a.useMemo)( () => {
                var e, t, n, a;
                return (null == (e = r.data) ? void 0 : e.totalUsdValue) && (null == (t = r.data) ? void 0 : t.tokenBalances.length) ? null == (a = r.data) || null == (n = a.tokenBalances) ? void 0 : n.filter(e => !0 === e.token.verified).reduce( (e, t) => e + t.usdValue, 0) : 0
            }
            , [r.data]);
            return (0,
            a.useEffect)( () => {
                u(d)
            }
            , [d, u]),
            (0,
            a.useEffect)( () => {
                var e;
                let t = (null == (e = r.data) ? void 0 : e.tokenBalances) || [];
                c.R.debug("[useWalletBalance] Updating tokenBalances in store:", {
                    tokenBalancesLength: t.length,
                    sampleBalances: t.slice(0, 3).map(e => ({
                        mint: e.token.address,
                        balance: e.balance,
                        symbol: e.token.symbol
                    })),
                    queryDataExists: !!r.data,
                    queryIsLoading: r.isLoading,
                    queryIsFetching: r.isFetching
                }),
                l(t)
            }
            , [l, null == (t = r.data) ? void 0 : t.tokenBalances, r.data, r.isLoading, r.isFetching]),
            (0,
            a.useEffect)( () => {
                s(r.isLoading || r.isFetching)
            }
            , [s, r.isLoading, r.isFetching]),
            (0,
            a.useEffect)( () => {
                var e;
                o((null == (e = r.data) ? void 0 : e.isStale) || !1)
            }
            , [o, null == (n = r.data) ? void 0 : n.isStale]),
            (0,
            a.useEffect)( () => {
                i(r.isError ? r.error : null)
            }
            , [i, r.isError, r.error]),
            r
        }
    }
    ,
    99188: (e, t, n) => {
        n.d(t, {
            A: () => i
        });
        var r = n(18586)
          , a = n(26432);
        let i = () => {
            let e = (0,
            r.useRouter)()
              , t = (0,
            r.usePathname)()
              , n = (0,
            r.useSearchParams)();
            return {
                updateUrl: (0,
                a.useCallback)( (n, r) => {
                    if (!n || !r)
                        return void e.replace(t, {
                            scroll: !1
                        });
                    let a = "".concat(n, "-").concat(r)
                      , i = "".concat(t, "?").concat(a);
                    e.replace(i, {
                        scroll: !1
                    })
                }
                , [e, t]),
                parseTokensFromSearchParams: (0,
                a.useCallback)( () => {
                    let e = Array.from(n.keys())[0];
                    if (!e || !e.includes("-"))
                        return {
                            sellTokenAddress: void 0,
                            receiveTokenAddress: void 0
                        };
                    let[t,r] = e.split("-");
                    return {
                        sellTokenAddress: t || void 0,
                        receiveTokenAddress: r || void 0
                    }
                }
                , [n])
            }
        }
    }
    ,
    99633: (e, t, n) => {
        n.d(t, {
            A: () => a
        });
        var r = n(48876);
        let a = e => {
            let {label: t, value: n} = e;
            return (0,
            r.jsxs)("div", {
                className: "text-body-xs text-text-low-em flex items-center justify-between gap-1",
                children: [(0,
                r.jsx)("span", {
                    children: t
                }), (0,
                r.jsx)("span", {
                    className: "text-text-high-em font-medium",
                    children: n
                })]
            })
        }
    }
}]);
