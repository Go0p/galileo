(self.webpackChunk_N_E = self.webpackChunk_N_E || []).push([[7285], {
    297: (e, t, n) => {
        "use strict";
        n.d(t, {
            g: () => a
        });
        var r, i = n(26432), s = n(42789);
        function a() {
            let e, t = (e = "undefined" == typeof document,
            (0,
            (r || (r = n.t(i, 2))).useSyncExternalStore)( () => () => {}
            , () => !1, () => !e)), [a,o] = i.useState(s._.isHandoffComplete);
            return a && !1 === s._.isHandoffComplete && o(!1),
            i.useEffect( () => {
                !0 !== a && o(!0)
            }
            , [a]),
            i.useEffect( () => s._.handoff(), []),
            !t && a
        }
    }
    ,
    2874: (e, t, n) => {
        "use strict";
        n.d(t, {
            d: () => W
        });
        var r, i, s, a, o, l, c, d, u, h, p, f, y, m, g, w = n(50901), _ = n(72286), v = n(17972), b = n(89037), E = n(40476), I = function(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        }, k = function(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        };
        class M extends Event {
            get detail() {
                return I(this, r, "f")
            }
            get type() {
                return "wallet-standard:register-wallet"
            }
            constructor(e) {
                super("wallet-standard:register-wallet", {
                    bubbles: !1,
                    cancelable: !1,
                    composed: !1
                }),
                r.set(this, void 0),
                k(this, r, e, "f")
            }
            preventDefault() {
                throw Error("preventDefault cannot be called")
            }
            stopImmediatePropagation() {
                throw Error("stopImmediatePropagation cannot be called")
            }
            stopPropagation() {
                throw Error("stopPropagation cannot be called")
            }
        }
        r = new WeakMap;
        var S = n(83597)
          , A = n(53676)
          , T = n(21715)
          , C = n(97680)
          , O = n(34994)
          , N = n(6801)
          , L = n(23713)
          , x = function(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        }
          , j = function(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        };
        class D {
            constructor() {
                i.add(this),
                s.set(this, {}),
                a.set(this, "1.0.0"),
                o.set(this, "MetaMask"),
                l.set(this, "data:image/svg+xml;base64,PHN2ZyBmaWxsPSJub25lIiBoZWlnaHQ9IjMxIiB2aWV3Qm94PSIwIDAgMzEgMzEiIHdpZHRoPSIzMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayI+PGxpbmVhckdyYWRpZW50IGlkPSJhIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgeDE9IjIwLjI1IiB4Mj0iMjYuNTcxIiB5MT0iMjcuMTczIiB5Mj0iMTkuODU4Ij48c3RvcCBvZmZzZXQ9Ii4wOCIgc3RvcC1jb2xvcj0iIzk5NDVmZiIvPjxzdG9wIG9mZnNldD0iLjMiIHN0b3AtY29sb3I9IiM4NzUyZjMiLz48c3RvcCBvZmZzZXQ9Ii41IiBzdG9wLWNvbG9yPSIjNTQ5N2Q1Ii8+PHN0b3Agb2Zmc2V0PSIuNiIgc3RvcC1jb2xvcj0iIzQzYjRjYSIvPjxzdG9wIG9mZnNldD0iLjcyIiBzdG9wLWNvbG9yPSIjMjhlMGI5Ii8+PHN0b3Agb2Zmc2V0PSIuOTciIHN0b3AtY29sb3I9IiMxOWZiOWIiLz48L2xpbmVhckdyYWRpZW50PjxnIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iLjA5NCI+PHBhdGggZD0ibTI2LjEwOSAzLjY0My05LjM2OSA2Ljk1OSAxLjczMy00LjEwNSA3LjYzNy0yLjg1M3oiIGZpbGw9IiNlMjc2MWIiIHN0cm9rZT0iI2UyNzYxYiIvPjxnIGZpbGw9IiNlNDc2MWIiIHN0cm9rZT0iI2U0NzYxYiI+PHBhdGggZD0ibTQuNDgxIDMuNjQzIDkuMjk0IDcuMDI0LTEuNjQ4LTQuMTcxem0xOC4yNTggMTYuMTMtMi40OTUgMy44MjMgNS4zMzkgMS40NjkgMS41MzUtNS4yMDctNC4zNzgtLjA4NXptLTE5LjI0Ny4wODUgMS41MjUgNS4yMDcgNS4zMzktMS40NjktMi40OTUtMy44MjN6Ii8+PHBhdGggZD0ibTEwLjA1NSAxMy4zMTMtMS40ODggMi4yNTEgNS4zMDEuMjM1LS4xODgtNS42OTd6bTEwLjQ4IDAtMy42NzItMy4yNzctLjEyMiA1Ljc2MyA1LjI5Mi0uMjM1LTEuNDk3LTIuMjUxem0tMTAuMTc4IDEwLjI4MyAzLjE4My0xLjU1NC0yLjc0OS0yLjE0Ny0uNDMzIDMuNzAxem02LjY5NS0xLjU1NCAzLjE5MiAxLjU1NC0uNDQzLTMuNzAxeiIvPjwvZz48cGF0aCBkPSJtMjAuMjQ0IDIzLjU5Ni0zLjE5Mi0xLjU1NC4yNTQgMi4wODEtLjAyOC44NzZ6bS05Ljg4NyAwIDIuOTY2IDEuNDAzLS4wMTktLjg3Ni4yMzUtMi4wODEtMy4xODMgMS41NTR6IiBmaWxsPSIjZDdjMWIzIiBzdHJva2U9IiNkN2MxYjMiLz48cGF0aCBkPSJtMTMuMzY5IDE4LjUyMS0yLjY1NS0uNzgxIDEuODc0LS44NTd6bTMuODUxIDAgLjc4MS0xLjYzOCAxLjg4My44NTctMi42NjUuNzgxeiIgZmlsbD0iIzIzMzQ0NyIgc3Ryb2tlPSIjMjMzNDQ3Ii8+PHBhdGggZD0ibTEwLjM1NyAyMy41OTYuNDUyLTMuODIzLTIuOTQ3LjA4NXptOS40MzUtMy44MjMuNDUyIDMuODIzIDIuNDk1LTMuNzM4em0yLjI0MS00LjIwOS01LjI5Mi4yMzUuNDkgMi43MjEuNzgyLTEuNjM4IDEuODgzLjg1N3ptLTExLjMxOCAyLjE3NSAxLjg4My0uODU3Ljc3MiAxLjYzOC40OTktMi43MjEtNS4zMDEtLjIzNXoiIGZpbGw9IiNjZDYxMTYiIHN0cm9rZT0iI2NkNjExNiIvPjxwYXRoIGQ9Im04LjU2NyAxNS41NjQgMi4yMjIgNC4zMzEtLjA3NS0yLjE1NnptMTEuMzI4IDIuMTc1LS4wOTQgMi4xNTYgMi4yMzItNC4zMzEtMi4xMzcgMi4xNzV6bS02LjAyNi0xLjk0LS40OTkgMi43MjEuNjIxIDMuMjExLjE0MS00LjIyOC0uMjY0LTEuNzA0em0yLjg3MiAwLS4yNTQgMS42OTUuMTEzIDQuMjM3LjYzMS0zLjIxMXoiIGZpbGw9IiNlNDc1MWYiIHN0cm9rZT0iI2U0NzUxZiIvPjxwYXRoIGQ9Im0xNy4yMyAxOC41Mi0uNjMxIDMuMjExLjQ1Mi4zMTEgMi43NS0yLjE0Ny4wOTQtMi4xNTZ6bS02LjUxNi0uNzgxLjA3NSAyLjE1NiAyLjc1IDIuMTQ3LjQ1Mi0uMzExLS42MjItMy4yMTF6IiBmaWxsPSIjZjY4NTFiIiBzdHJva2U9IiNmNjg1MWIiLz48cGF0aCBkPSJtMTcuMjc3IDI0Ljk5OS4wMjgtLjg3Ni0uMjM1LS4yMDdoLTMuNTVsLS4yMTcuMjA3LjAxOS44NzYtMi45NjYtMS40MDMgMS4wMzYuODQ4IDIuMSAxLjQ1OWgzLjYwNmwyLjEwOS0xLjQ1OSAxLjAzNi0uODQ4eiIgZmlsbD0iI2MwYWQ5ZSIgc3Ryb2tlPSIjYzBhZDllIi8+PHBhdGggZD0ibTE3LjA1MSAyMi4wNDItLjQ1Mi0uMzExaC0yLjYwOGwtLjQ1Mi4zMTEtLjIzNSAyLjA4MS4yMTctLjIwN2gzLjU1bC4yMzUuMjA3LS4yNTQtMi4wODF6IiBmaWxsPSIjMTYxNjE2IiBzdHJva2U9IiMxNjE2MTYiLz48cGF0aCBkPSJtMjYuNTA1IDExLjA1My44LTMuODQyLTEuMTk2LTMuNTY5LTkuMDU4IDYuNzIzIDMuNDg0IDIuOTQ3IDQuOTI1IDEuNDQxIDEuMDkyLTEuMjcxLS40NzEtLjMzOS43NTMtLjY4Ny0uNTg0LS40NTIuNzUzLS41NzQtLjQ5OS0uMzc3em0tMjMuMjExLTMuODQxLjggMy44NDItLjUwOC4zNzcuNzUzLjU3NC0uNTc0LjQ1Mi43NTMuNjg3LS40NzEuMzM5IDEuMDgzIDEuMjcxIDQuOTI1LTEuNDQxIDMuNDg0LTIuOTQ3LTkuMDU5LTYuNzIzeiIgZmlsbD0iIzc2M2QxNiIgc3Ryb2tlPSIjNzYzZDE2Ii8+PHBhdGggZD0ibTI1LjQ2IDE0Ljc1NC00LjkyNS0xLjQ0MSAxLjQ5NyAyLjI1MS0yLjIzMiA0LjMzMSAyLjkzOC0uMDM4aDQuMzc4bC0xLjY1Ny01LjEwNHptLTE1LjQwNS0xLjQ0MS00LjkyNSAxLjQ0MS0xLjYzOCA1LjEwNGg0LjM2OWwyLjkyOC4wMzgtMi4yMjItNC4zMzEgMS40ODgtMi4yNTF6bTYuNjg1IDIuNDg2LjMxMS01LjQzMyAxLjQzMS0zLjg3aC02LjM1NmwxLjQxMyAzLjg3LjMyOSA1LjQzMy4xMTMgMS43MTQuMDA5IDQuMjE5aDIuNjFsLjAxOS00LjIxOS4xMjItMS43MTR6IiBmaWxsPSIjZjY4NTFiIiBzdHJva2U9IiNmNjg1MWIiLz48L2c+PGNpcmNsZSBjeD0iMjMuNSIgY3k9IjIzLjUiIGZpbGw9IiMwMDAiIHI9IjYuNSIvPjxwYXRoIGQ9Im0yNy40NzMgMjUuNTQ1LTEuMzEgMS4zNjhjLS4wMjkuMDMtLjA2My4wNTMtLjEwMS4wN2EuMzEuMzEgMCAwIDEgLS4xMjEuMDI0aC02LjIwOWMtLjAzIDAtLjA1OS0uMDA4LS4wODMtLjAyNGEuMTUuMTUgMCAwIDEgLS4wNTYtLjA2NWMtLjAxMi0uMDI2LS4wMTUtLjA1Ni0uMDEtLjA4NHMuMDE4LS4wNTUuMDM5LS4wNzZsMS4zMTEtMS4zNjhjLjAyOC0uMDMuMDYzLS4wNTMuMTAxLS4wNjlhLjMxLjMxIDAgMCAxIC4xMjEtLjAyNWg2LjIwOGMuMDMgMCAuMDU5LjAwOC4wODMuMDI0YS4xNS4xNSAwIDAgMSAuMDU2LjA2NWMuMDEyLjAyNi4wMTUuMDU2LjAxLjA4NHMtLjAxOC4wNTUtLjAzOS4wNzZ6bS0xLjMxLTIuNzU2Yy0uMDI5LS4wMy0uMDYzLS4wNTMtLjEwMS0uMDdhLjMxLjMxIDAgMCAwIC0uMTIxLS4wMjRoLTYuMjA5Yy0uMDMgMC0uMDU5LjAwOC0uMDgzLjAyNHMtLjA0NC4wMzgtLjA1Ni4wNjUtLjAxNS4wNTYtLjAxLjA4NC4wMTguMDU1LjAzOS4wNzZsMS4zMTEgMS4zNjhjLjAyOC4wMy4wNjMuMDUzLjEwMS4wNjlhLjMxLjMxIDAgMCAwIC4xMjEuMDI1aDYuMjA4Yy4wMyAwIC4wNTktLjAwOC4wODMtLjAyNGEuMTUuMTUgMCAwIDAgLjA1Ni0uMDY1Yy4wMTItLjAyNi4wMTUtLjA1Ni4wMS0uMDg0cy0uMDE4LS4wNTUtLjAzOS0uMDc2em0tNi40MzEtLjk4M2g2LjIwOWEuMzEuMzEgMCAwIDAgLjEyMS0uMDI0Yy4wMzgtLjAxNi4wNzMtLjA0LjEwMS0uMDdsMS4zMS0xLjM2OGMuMDItLjAyMS4wMzQtLjA0Ny4wMzktLjA3NnMuMDAxLS4wNTgtLjAxLS4wODRhLjE1LjE1IDAgMCAwIC0uMDU2LS4wNjVjLS4wMjUtLjAxNi0uMDU0LS4wMjQtLjA4My0uMDI0aC02LjIwOGEuMzEuMzEgMCAwIDAgLS4xMjEuMDI1Yy0uMDM4LjAxNi0uMDcyLjA0LS4xMDEuMDY5bC0xLjMxIDEuMzY4Yy0uMDIuMDIxLS4wMzQuMDQ3LS4wMzkuMDc2cy0uMDAxLjA1OC4wMS4wODQuMDMxLjA0OS4wNTYuMDY1LjA1NC4wMjQuMDgzLjAyNHoiIGZpbGw9InVybCgjYSkiLz48L3N2Zz4="),
                c.set(this, null),
                d.set(this, (e, t) => (x(this, s, "f")[e]?.push(t) || (x(this, s, "f")[e] = [t]),
                () => x(this, i, "m", h).call(this, e, t))),
                p.set(this, async () => {
                    if (!x(this, c, "f")) {
                        let e;
                        try {
                            e = (await n.e(5590).then(n.bind(n, 15590))).default
                        } catch (e) {
                            throw Error("Unable to load Solflare MetaMask SDK")
                        }
                        j(this, c, new e, "f"),
                        x(this, c, "f").on("standard_change", e => x(this, i, "m", u).call(this, "change", e))
                    }
                    return this.accounts.length || await x(this, c, "f").connect(),
                    {
                        accounts: this.accounts
                    }
                }
                ),
                f.set(this, async () => {
                    x(this, c, "f") && await x(this, c, "f").disconnect()
                }
                ),
                y.set(this, async (...e) => {
                    if (!x(this, c, "f"))
                        throw new v.kW;
                    return await x(this, c, "f").standardSignAndSendTransaction(...e)
                }
                ),
                m.set(this, async (...e) => {
                    if (!x(this, c, "f"))
                        throw new v.kW;
                    return await x(this, c, "f").standardSignTransaction(...e)
                }
                ),
                g.set(this, async (...e) => {
                    if (!x(this, c, "f"))
                        throw new v.kW;
                    return await x(this, c, "f").standardSignMessage(...e)
                }
                )
            }
            get version() {
                return x(this, a, "f")
            }
            get name() {
                return x(this, o, "f")
            }
            get icon() {
                return x(this, l, "f")
            }
            get chains() {
                return [S.CE, S.sE, S.re]
            }
            get features() {
                return {
                    [O.u]: {
                        version: "1.0.0",
                        connect: x(this, p, "f")
                    },
                    [N.w]: {
                        version: "1.0.0",
                        disconnect: x(this, f, "f")
                    },
                    [L.j]: {
                        version: "1.0.0",
                        on: x(this, d, "f")
                    },
                    [A.R]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signAndSendTransaction: x(this, y, "f")
                    },
                    [T.q]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signTransaction: x(this, m, "f")
                    },
                    [C.F]: {
                        version: "1.0.0",
                        signMessage: x(this, g, "f")
                    }
                }
            }
            get accounts() {
                return x(this, c, "f") ? x(this, c, "f").standardAccounts : []
            }
        }
        s = new WeakMap,
        a = new WeakMap,
        o = new WeakMap,
        l = new WeakMap,
        c = new WeakMap,
        d = new WeakMap,
        p = new WeakMap,
        f = new WeakMap,
        y = new WeakMap,
        m = new WeakMap,
        g = new WeakMap,
        i = new WeakSet,
        u = function(e, ...t) {
            x(this, s, "f")[e]?.forEach(e => e.apply(null, t))
        }
        ,
        h = function(e, t) {
            x(this, s, "f")[e] = x(this, s, "f")[e]?.filter(e => t !== e)
        }
        ;
        let R = !1;
        async function P() {
            let e = "solflare-detect-metamask";
            function t() {
                window.postMessage({
                    target: "metamask-contentscript",
                    data: {
                        name: "metamask-provider",
                        data: {
                            id: e,
                            jsonrpc: "2.0",
                            method: "wallet_getSnaps"
                        }
                    }
                }, window.location.origin)
            }
            function n(r) {
                let i = r.data;
                i?.target === "metamask-inpage" && i.data?.name === "metamask-provider" && (i.data.data?.id === e ? (window.removeEventListener("message", n),
                !i.data.data.error && (R || (function(e) {
                    let t = ({register: t}) => t(e);
                    try {
                        window.dispatchEvent(new M(t))
                    } catch (e) {
                        console.error("wallet-standard:register-wallet event could not be dispatched\n", e)
                    }
                    try {
                        window.addEventListener("wallet-standard:app-ready", ({detail: e}) => t(e))
                    } catch (e) {
                        console.error("wallet-standard:app-ready event listener could not be added\n", e)
                    }
                }(new D),
                R = !0))) : t())
            }
            window.addEventListener("message", n),
            window.setTimeout( () => window.removeEventListener("message", n), 5e3),
            t()
        }
        class W extends w.DE {
            constructor(e={}) {
                super(),
                this.name = "Solflare",
                this.url = "https://solflare.com",
                this.icon = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIGlkPSJTIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA1MCA1MCI+PGRlZnM+PHN0eWxlPi5jbHMtMXtmaWxsOiMwMjA1MGE7c3Ryb2tlOiNmZmVmNDY7c3Ryb2tlLW1pdGVybGltaXQ6MTA7c3Ryb2tlLXdpZHRoOi41cHg7fS5jbHMtMntmaWxsOiNmZmVmNDY7fTwvc3R5bGU+PC9kZWZzPjxyZWN0IGNsYXNzPSJjbHMtMiIgeD0iMCIgd2lkdGg9IjUwIiBoZWlnaHQ9IjUwIiByeD0iMTIiIHJ5PSIxMiIvPjxwYXRoIGNsYXNzPSJjbHMtMSIgZD0iTTI0LjIzLDI2LjQybDIuNDYtMi4zOCw0LjU5LDEuNWMzLjAxLDEsNC41MSwyLjg0LDQuNTEsNS40MywwLDEuOTYtLjc1LDMuMjYtMi4yNSw0LjkzbC0uNDYuNS4xNy0xLjE3Yy42Ny00LjI2LS41OC02LjA5LTQuNzItNy40M2wtNC4zLTEuMzhoMFpNMTguMDUsMTEuODVsMTIuNTIsNC4xNy0yLjcxLDIuNTktNi41MS0yLjE3Yy0yLjI1LS43NS0zLjAxLTEuOTYtMy4zLTQuNTF2LS4wOGgwWk0xNy4zLDMzLjA2bDIuODQtMi43MSw1LjM0LDEuNzVjMi44LjkyLDMuNzYsMi4xMywzLjQ2LDUuMThsLTExLjY1LTQuMjJoMFpNMTMuNzEsMjAuOTVjMC0uNzkuNDItMS41NCwxLjEzLTIuMTcuNzUsMS4wOSwyLjA1LDIuMDUsNC4wOSwyLjcxbDQuNDIsMS40Ni0yLjQ2LDIuMzgtNC4zNC0xLjQyYy0yLS42Ny0yLjg0LTEuNjctMi44NC0yLjk2TTI2LjgyLDQyLjg3YzkuMTgtNi4wOSwxNC4xMS0xMC4yMywxNC4xMS0xNS4zMiwwLTMuMzgtMi01LjI2LTYuNDMtNi43MmwtMy4zNC0xLjEzLDkuMTQtOC43Ny0xLjg0LTEuOTYtMi43MSwyLjM4LTEyLjgxLTQuMjJjLTMuOTcsMS4yOS04Ljk3LDUuMDktOC45Nyw4Ljg5LDAsLjQyLjA0LjgzLjE3LDEuMjktMy4zLDEuODgtNC42MywzLjYzLTQuNjMsNS44LDAsMi4wNSwxLjA5LDQuMDksNC41NSw1LjIybDIuNzUuOTItOS41Miw5LjE0LDEuODQsMS45NiwyLjk2LTIuNzEsMTQuNzMsNS4yMmgwWiIvPjwvc3ZnPg==",
                this.supportedTransactionVersions = new Set(["legacy", 0]),
                this._readyState = "undefined" == typeof window || "undefined" == typeof document ? _.Ok.Unsupported : _.Ok.Loadable,
                this._disconnected = () => {
                    let e = this._wallet;
                    e && (e.off("disconnect", this._disconnected),
                    this._wallet = null,
                    this._publicKey = null,
                    this.emit("error", new v.PQ),
                    this.emit("disconnect"))
                }
                ,
                this._accountChanged = e => {
                    if (!e)
                        return;
                    let t = this._publicKey;
                    if (t) {
                        try {
                            e = new E.PublicKey(e.toBytes())
                        } catch (e) {
                            this.emit("error", new v.Kd(e?.message,e));
                            return
                        }
                        t.equals(e) || (this._publicKey = e,
                        this.emit("connect", e))
                    }
                }
                ,
                this._connecting = !1,
                this._publicKey = null,
                this._wallet = null,
                this._config = e,
                this._readyState !== _.Ok.Unsupported && ((0,
                _.qG)( () => (!!window.solflare?.isSolflare || !!window.SolflareApp) && (this._readyState = _.Ok.Installed,
                this.emit("readyStateChange", this._readyState),
                !0)),
                P())
            }
            get publicKey() {
                return this._publicKey
            }
            get connecting() {
                return this._connecting
            }
            get connected() {
                return !!this._wallet?.connected
            }
            get readyState() {
                return this._readyState
            }
            async autoConnect() {
                this.readyState === _.Ok.Loadable && (0,
                _.Br)() || await this.connect()
            }
            async connect() {
                try {
                    let e, t, r;
                    if (this.connected || this.connecting)
                        return;
                    if (this._readyState !== _.Ok.Loadable && this._readyState !== _.Ok.Installed)
                        throw new v.AE;
                    if (this.readyState === _.Ok.Loadable && (0,
                    _.Br)()) {
                        let e = encodeURIComponent(window.location.href)
                          , t = encodeURIComponent(window.location.origin);
                        window.location.href = `https://solflare.com/ul/v1/browse/${e}?ref=${t}`;
                        return
                    }
                    try {
                        e = (await n.e(9460).then(n.bind(n, 99460))).default
                    } catch (e) {
                        throw new v.Sz(e?.message,e)
                    }
                    try {
                        t = new e({
                            network: this._config.network
                        })
                    } catch (e) {
                        throw new v.Ez(e?.message,e)
                    }
                    if (this._connecting = !0,
                    !t.connected)
                        try {
                            await t.connect()
                        } catch (e) {
                            throw new v.Y6(e?.message,e)
                        }
                    if (!t.publicKey)
                        throw new v.Y6;
                    try {
                        r = new E.PublicKey(t.publicKey.toBytes())
                    } catch (e) {
                        throw new v.Kd(e?.message,e)
                    }
                    t.on("disconnect", this._disconnected),
                    t.on("accountChanged", this._accountChanged),
                    this._wallet = t,
                    this._publicKey = r,
                    this.emit("connect", r)
                } catch (e) {
                    throw this.emit("error", e),
                    e
                } finally {
                    this._connecting = !1
                }
            }
            async disconnect() {
                let e = this._wallet;
                if (e) {
                    e.off("disconnect", this._disconnected),
                    e.off("accountChanged", this._accountChanged),
                    this._wallet = null,
                    this._publicKey = null;
                    try {
                        await e.disconnect()
                    } catch (e) {
                        this.emit("error", new v.Y8(e?.message,e))
                    }
                }
                this.emit("disconnect")
            }
            async sendTransaction(e, t, n={}) {
                try {
                    let r = this._wallet;
                    if (!r)
                        throw new v.kW;
                    try {
                        let {signers: i, ...s} = n;
                        return (0,
                        b.Y)(e) ? i?.length && e.sign(i) : (e = await this.prepareTransaction(e, t, s),
                        i?.length && e.partialSign(...i)),
                        s.preflightCommitment = s.preflightCommitment || t.commitment,
                        await r.signAndSendTransaction(e, s)
                    } catch (e) {
                        if (e instanceof v.m7)
                            throw e;
                        throw new v.UF(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signTransaction(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new v.kW;
                    try {
                        return await t.signTransaction(e) || e
                    } catch (e) {
                        throw new v.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signAllTransactions(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new v.kW;
                    try {
                        return await t.signAllTransactions(e) || e
                    } catch (e) {
                        throw new v.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signMessage(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new v.kW;
                    try {
                        return await t.signMessage(e, "utf8")
                    } catch (e) {
                        throw new v.K3(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
        }
    }
    ,
    3148: (e, t, n) => {
        "use strict";
        let r;
        n.d(t, {
            r: () => $
        });
        var i, s, a, o, l, c, d, u, h, p, f, y, m, g, w, _, v, b, E, I = n(17151), k = n(53676), M = n(21715), S = n(34994), A = n(23713);
        let T = function(e) {
            return S.u in e.features && A.j in e.features && (k.R in e.features || M.q in e.features)
        };
        var C = n(72286)
          , O = n(17972)
          , N = n(89037)
          , L = n(97680);
        let x = "solana:signIn";
        var j = n(83597);
        function D(e) {
            switch (e) {
            case "processed":
            case "confirmed":
            case "finalized":
            case void 0:
                return e;
            case "recent":
                return "processed";
            case "single":
            case "singleGossip":
                return "confirmed";
            case "max":
            case "root":
                return "finalized";
            default:
                return
            }
        }
        var R = n(40476)
          , P = n(6801);
        new WeakMap,
        new WeakMap,
        new WeakMap,
        new WeakMap,
        new WeakMap,
        new WeakMap;
        var W = n(76535)
          , U = function(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        }
          , z = function(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        };
        class q extends C.Ce {
            get name() {
                return U(this, u, "f").name
            }
            get url() {
                return "https://github.com/solana-labs/wallet-standard"
            }
            get icon() {
                return U(this, u, "f").icon
            }
            get readyState() {
                return U(this, h, "f")
            }
            get publicKey() {
                return U(this, a, "f")
            }
            get connecting() {
                return U(this, o, "f")
            }
            get supportedTransactionVersions() {
                return U(this, d, "f")
            }
            get wallet() {
                return U(this, u, "f")
            }
            get standard() {
                return !0
            }
            constructor({wallet: e}) {
                super(),
                i.add(this),
                s.set(this, void 0),
                a.set(this, void 0),
                o.set(this, void 0),
                l.set(this, void 0),
                c.set(this, void 0),
                d.set(this, void 0),
                u.set(this, void 0),
                h.set(this, "undefined" == typeof window || "undefined" == typeof document ? C.Ok.Unsupported : C.Ok.Installed),
                g.set(this, e => {
                    if ("accounts"in e) {
                        let e = U(this, u, "f").accounts[0];
                        U(this, s, "f") && !U(this, l, "f") && e !== U(this, s, "f") && (e ? U(this, i, "m", f).call(this, e) : (this.emit("error", new O.PQ),
                        U(this, i, "m", y).call(this)))
                    }
                    "features"in e && U(this, i, "m", m).call(this)
                }
                ),
                z(this, u, e, "f"),
                z(this, s, null, "f"),
                z(this, a, null, "f"),
                z(this, o, !1, "f"),
                z(this, l, !1, "f"),
                z(this, c, U(this, u, "f").features[A.j].on("change", U(this, g, "f")), "f"),
                U(this, i, "m", m).call(this)
            }
            destroy() {
                z(this, s, null, "f"),
                z(this, a, null, "f"),
                z(this, o, !1, "f"),
                z(this, l, !1, "f");
                let e = U(this, c, "f");
                e && (z(this, c, null, "f"),
                e())
            }
            async autoConnect() {
                return U(this, i, "m", p).call(this, {
                    silent: !0
                })
            }
            async connect() {
                return U(this, i, "m", p).call(this)
            }
            async disconnect() {
                if (P.w in U(this, u, "f").features)
                    try {
                        z(this, l, !0, "f"),
                        await U(this, u, "f").features[P.w].disconnect()
                    } catch (e) {
                        this.emit("error", new O.Y8(e?.message,e))
                    } finally {
                        z(this, l, !1, "f")
                    }
                U(this, i, "m", y).call(this)
            }
            async sendTransaction(e, t, n={}) {
                try {
                    var r;
                    let i, a = U(this, s, "f");
                    if (!a)
                        throw new O.kW;
                    if (k.R in U(this, u, "f").features)
                        if (a.features.includes(k.R))
                            i = k.R;
                        else if (M.q in U(this, u, "f").features && a.features.includes(M.q))
                            i = M.q;
                        else
                            throw new O.fk;
                    else if (M.q in U(this, u, "f").features) {
                        if (!a.features.includes(M.q))
                            throw new O.fk;
                        i = M.q
                    } else
                        throw new O.Ez;
                    let o = (r = t.rpcEndpoint).includes("https://api.mainnet-beta.solana.com") ? j.CE : /\bdevnet\b/i.test(r) ? j.sE : /\btestnet\b/i.test(r) ? j.re : /\blocalhost\b/i.test(r) || /\b127\.0\.0\.1\b/.test(r) ? j.g4 : j.CE;
                    if (!a.chains.includes(o))
                        throw new O.UF;
                    try {
                        let r, {signers: s, ...l} = n;
                        if ((0,
                        N.Y)(e) ? (s?.length && e.sign(s),
                        r = e.serialize()) : (e = await this.prepareTransaction(e, t, l),
                        s?.length && e.partialSign(...s),
                        r = new Uint8Array(e.serialize({
                            requireAllSignatures: !1,
                            verifySignatures: !1
                        }))),
                        i === k.R) {
                            let[e] = await U(this, u, "f").features[k.R].signAndSendTransaction({
                                account: a,
                                chain: o,
                                transaction: r,
                                options: {
                                    preflightCommitment: D(l.preflightCommitment || t.commitment),
                                    skipPreflight: l.skipPreflight,
                                    maxRetries: l.maxRetries,
                                    minContextSlot: l.minContextSlot
                                }
                            });
                            return W.default.encode(e.signature)
                        }
                        {
                            let[e] = await U(this, u, "f").features[M.q].signTransaction({
                                account: a,
                                chain: o,
                                transaction: r,
                                options: {
                                    preflightCommitment: D(l.preflightCommitment || t.commitment),
                                    minContextSlot: l.minContextSlot
                                }
                            });
                            return await t.sendRawTransaction(e.signedTransaction, {
                                ...l,
                                preflightCommitment: D(l.preflightCommitment || t.commitment)
                            })
                        }
                    } catch (e) {
                        if (e instanceof O.m7)
                            throw e;
                        throw new O.UF(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
        }
        s = new WeakMap,
        a = new WeakMap,
        o = new WeakMap,
        l = new WeakMap,
        c = new WeakMap,
        d = new WeakMap,
        u = new WeakMap,
        h = new WeakMap,
        g = new WeakMap,
        i = new WeakSet,
        p = async function(e) {
            try {
                if (this.connected || this.connecting)
                    return;
                if (U(this, h, "f") !== C.Ok.Installed)
                    throw new O.AE;
                if (z(this, o, !0, "f"),
                !U(this, u, "f").accounts.length)
                    try {
                        await U(this, u, "f").features[S.u].connect(e)
                    } catch (e) {
                        throw new O.Y6(e?.message,e)
                    }
                let t = U(this, u, "f").accounts[0];
                if (!t)
                    throw new O.fk;
                U(this, i, "m", f).call(this, t)
            } catch (e) {
                throw this.emit("error", e),
                e
            } finally {
                z(this, o, !1, "f")
            }
        }
        ,
        f = function(e) {
            let t;
            try {
                t = new R.PublicKey(e.address)
            } catch (e) {
                throw new O.Kd(e?.message,e)
            }
            z(this, s, e, "f"),
            z(this, a, t, "f"),
            U(this, i, "m", m).call(this),
            this.emit("connect", t)
        }
        ,
        y = function() {
            z(this, s, null, "f"),
            z(this, a, null, "f"),
            U(this, i, "m", m).call(this),
            this.emit("disconnect")
        }
        ,
        m = function() {
            let e = k.R in U(this, u, "f").features ? U(this, u, "f").features[k.R].supportedTransactionVersions : U(this, u, "f").features[M.q].supportedTransactionVersions;
            z(this, d, !function(e, t) {
                if (e === t)
                    return !0;
                let n = e.length;
                if (n !== t.length)
                    return !1;
                for (let r = 0; r < n; r++)
                    if (e[r] !== t[r])
                        return !1;
                return !0
            }(e, ["legacy"]) ? new Set(e) : null, "f"),
            M.q in U(this, u, "f").features && U(this, s, "f")?.features.includes(M.q) ? (this.signTransaction = U(this, i, "m", w),
            this.signAllTransactions = U(this, i, "m", _)) : (delete this.signTransaction,
            delete this.signAllTransactions),
            L.F in U(this, u, "f").features && U(this, s, "f")?.features.includes(L.F) ? this.signMessage = U(this, i, "m", v) : delete this.signMessage,
            x in U(this, u, "f").features ? this.signIn = U(this, i, "m", b) : delete this.signIn
        }
        ,
        w = async function(e) {
            try {
                let t = U(this, s, "f");
                if (!t)
                    throw new O.kW;
                if (!(M.q in U(this, u, "f").features))
                    throw new O.Ez;
                if (!t.features.includes(M.q))
                    throw new O.fk;
                try {
                    let n = (await U(this, u, "f").features[M.q].signTransaction({
                        account: t,
                        transaction: (0,
                        N.Y)(e) ? e.serialize() : new Uint8Array(e.serialize({
                            requireAllSignatures: !1,
                            verifySignatures: !1
                        }))
                    }))[0].signedTransaction;
                    return (0,
                    N.Y)(e) ? R.VersionedTransaction.deserialize(n) : R.Transaction.from(n)
                } catch (e) {
                    if (e instanceof O.m7)
                        throw e;
                    throw new O.z4(e?.message,e)
                }
            } catch (e) {
                throw this.emit("error", e),
                e
            }
        }
        ,
        _ = async function(e) {
            try {
                let t = U(this, s, "f");
                if (!t)
                    throw new O.kW;
                if (!(M.q in U(this, u, "f").features))
                    throw new O.Ez;
                if (!t.features.includes(M.q))
                    throw new O.fk;
                try {
                    let n = await U(this, u, "f").features[M.q].signTransaction(...e.map(e => ({
                        account: t,
                        transaction: (0,
                        N.Y)(e) ? e.serialize() : new Uint8Array(e.serialize({
                            requireAllSignatures: !1,
                            verifySignatures: !1
                        }))
                    })));
                    return e.map( (e, t) => {
                        let r = n[t].signedTransaction;
                        return (0,
                        N.Y)(e) ? R.VersionedTransaction.deserialize(r) : R.Transaction.from(r)
                    }
                    )
                } catch (e) {
                    throw new O.z4(e?.message,e)
                }
            } catch (e) {
                throw this.emit("error", e),
                e
            }
        }
        ,
        v = async function(e) {
            try {
                let t = U(this, s, "f");
                if (!t)
                    throw new O.kW;
                if (!(L.F in U(this, u, "f").features))
                    throw new O.Ez;
                if (!t.features.includes(L.F))
                    throw new O.fk;
                try {
                    return (await U(this, u, "f").features[L.F].signMessage({
                        account: t,
                        message: e
                    }))[0].signature
                } catch (e) {
                    throw new O.K3(e?.message,e)
                }
            } catch (e) {
                throw this.emit("error", e),
                e
            }
        }
        ,
        b = async function(e={}) {
            try {
                let t;
                if (!(x in U(this, u, "f").features))
                    throw new O.Ez;
                try {
                    [t] = await U(this, u, "f").features[x].signIn(e)
                } catch (e) {
                    throw new O.o7(e?.message,e)
                }
                if (!t)
                    throw new O.o7;
                return U(this, i, "m", f).call(this, t.account),
                t
            } catch (e) {
                throw this.emit("error", e),
                e
            }
        }
        ;
        var F = n(55664)
          , B = n(26432);
        function H(e) {
            let t = (0,
            B.useRef)(void 0);
            return void 0 === t.current && (t.current = {
                value: e()
            }),
            t.current.value
        }
        function K(e) {
            return e.filter(T).map(e => new q({
                wallet: e
            }))
        }
        !function(e) {
            e[e.DESKTOP_WEB = 0] = "DESKTOP_WEB",
            e[e.MOBILE_WEB = 1] = "MOBILE_WEB"
        }(E || (E = {}));
        var Q = n(37358);
        class V extends O.m7 {
            constructor() {
                super(...arguments),
                this.name = "WalletNotSelectedError"
            }
        }
        var Y = n(27519);
        function G({children: e, wallets: t, adapter: n, isUnloadingRef: r, onAutoConnectRequest: i, onConnectError: s, onError: a, onSelectWallet: o}) {
            let l = (0,
            B.useRef)(!1)
              , [c,d] = (0,
            B.useState)(!1)
              , u = (0,
            B.useRef)(!1)
              , [h,p] = (0,
            B.useState)(!1)
              , [f,y] = (0,
            B.useState)( () => n?.publicKey ?? null)
              , [m,g] = (0,
            B.useState)( () => n?.connected ?? !1)
              , w = (0,
            B.useRef)(a);
            (0,
            B.useEffect)( () => (w.current = a,
            () => {
                w.current = void 0
            }
            ), [a]);
            let _ = (0,
            B.useRef)( (e, t) => (!r.current && (w.current ? w.current(e, t) : (console.error(e, t),
            e instanceof O.AE && "undefined" != typeof window && t && window.open(t.url, "_blank"))),
            e))
              , [v,b] = (0,
            B.useState)( () => t.map(e => ({
                adapter: e,
                readyState: e.readyState
            })).filter( ({readyState: e}) => e !== C.Ok.Unsupported));
            (0,
            B.useEffect)( () => {
                function e(e) {
                    b(t => {
                        let n = t.findIndex( ({adapter: e}) => e === this);
                        if (-1 === n)
                            return t;
                        let {adapter: r} = t[n];
                        return [...t.slice(0, n), {
                            adapter: r,
                            readyState: e
                        }, ...t.slice(n + 1)].filter( ({readyState: e}) => e !== C.Ok.Unsupported)
                    }
                    )
                }
                return b(e => t.map( (t, n) => {
                    let r = e[n];
                    return r && r.adapter === t && r.readyState === t.readyState ? r : {
                        adapter: t,
                        readyState: t.readyState
                    }
                }
                ).filter( ({readyState: e}) => e !== C.Ok.Unsupported)),
                t.forEach(t => t.on("readyStateChange", e, t)),
                () => {
                    t.forEach(t => t.off("readyStateChange", e, t))
                }
            }
            , [n, t]);
            let E = (0,
            B.useMemo)( () => v.find(e => e.adapter === n) ?? null, [n, v]);
            (0,
            B.useEffect)( () => {
                if (!n)
                    return;
                let e = e => {
                    y(e),
                    l.current = !1,
                    d(!1),
                    g(!0),
                    u.current = !1,
                    p(!1)
                }
                  , t = () => {
                    r.current || (y(null),
                    l.current = !1,
                    d(!1),
                    g(!1),
                    u.current = !1,
                    p(!1))
                }
                  , i = e => {
                    _.current(e, n)
                }
                ;
                return n.on("connect", e),
                n.on("disconnect", t),
                n.on("error", i),
                () => {
                    n.off("connect", e),
                    n.off("disconnect", t),
                    n.off("error", i),
                    t()
                }
            }
            , [n, r]);
            let I = (0,
            B.useRef)(!1);
            (0,
            B.useEffect)( () => () => {
                I.current = !1
            }
            , [n]),
            (0,
            B.useEffect)( () => {
                I.current || l.current || m || !i || E?.readyState !== C.Ok.Installed && E?.readyState !== C.Ok.Loadable || (l.current = !0,
                d(!0),
                I.current = !0,
                async function() {
                    try {
                        await i()
                    } catch {
                        s()
                    } finally {
                        d(!1),
                        l.current = !1
                    }
                }())
            }
            , [m, i, s, E]);
            let k = (0,
            B.useCallback)(async (e, t, r) => {
                if (!n)
                    throw _.current(new V);
                if (!m)
                    throw _.current(new O.kW, n);
                return await n.sendTransaction(e, t, r)
            }
            , [n, m])
              , M = (0,
            B.useMemo)( () => n && "signTransaction"in n ? async e => {
                if (!m)
                    throw _.current(new O.kW, n);
                return await n.signTransaction(e)
            }
            : void 0, [n, m])
              , S = (0,
            B.useMemo)( () => n && "signAllTransactions"in n ? async e => {
                if (!m)
                    throw _.current(new O.kW, n);
                return await n.signAllTransactions(e)
            }
            : void 0, [n, m])
              , A = (0,
            B.useMemo)( () => n && "signMessage"in n ? async e => {
                if (!m)
                    throw _.current(new O.kW, n);
                return await n.signMessage(e)
            }
            : void 0, [n, m])
              , T = (0,
            B.useMemo)( () => n && "signIn"in n ? async e => await n.signIn(e) : void 0, [n])
              , N = (0,
            B.useCallback)(async () => {
                if (l.current || u.current || E?.adapter.connected)
                    return;
                if (!E)
                    throw _.current(new V);
                let {adapter: e, readyState: t} = E;
                if (t !== C.Ok.Installed && t !== C.Ok.Loadable)
                    throw _.current(new O.AE, e);
                l.current = !0,
                d(!0);
                try {
                    await e.connect()
                } catch (e) {
                    throw s(),
                    e
                } finally {
                    d(!1),
                    l.current = !1
                }
            }
            , [s, E])
              , L = (0,
            B.useCallback)(async () => {
                if (!u.current && n) {
                    u.current = !0,
                    p(!0);
                    try {
                        await n.disconnect()
                    } finally {
                        p(!1),
                        u.current = !1
                    }
                }
            }
            , [n]);
            return B.createElement(Y.b.Provider, {
                value: {
                    autoConnect: !!i,
                    wallets: v,
                    wallet: E,
                    publicKey: f,
                    connected: m,
                    connecting: c,
                    disconnecting: h,
                    select: o,
                    connect: N,
                    disconnect: L,
                    sendTransaction: k,
                    signTransaction: M,
                    signAllTransactions: S,
                    signMessage: A,
                    signIn: T
                }
            }, e)
        }
        function Z(e) {
            return function({adapters: e, userAgentString: t}) {
                return e.some(e => e.name !== I.fG && e.readyState === C.Ok.Installed) ? E.DESKTOP_WEB : t && /android/i.test(t) && !/(WebView|Version\/.+(Chrome)\/(\d+)\.(\d+)\.(\d+)\.(\d+)|; wv\).+(Chrome)\/(\d+)\.(\d+)\.(\d+)\.(\d+))/i.test(t) ? E.MOBILE_WEB : E.DESKTOP_WEB
            }({
                adapters: e,
                userAgentString: (void 0 === r && (r = globalThis.navigator?.userAgent ?? null),
                r)
            }) === E.MOBILE_WEB
        }
        function $({children: e, wallets: t, autoConnect: n, localStorageKey: r="walletName", onError: i}) {
            let {connection: s} = (0,
            Q.w)()
              , a = function(e) {
                let t = H( () => new Set)
                  , {get: n, on: r} = H( () => (0,
                F.N)())
                  , [i,s] = (0,
                B.useState)( () => K(n()));
                (0,
                B.useEffect)( () => {
                    let e = [r("register", (...e) => s(t => [...t, ...K(e)])), r("unregister", (...e) => s(t => t.filter(t => e.some(e => e === t.wallet))))];
                    return () => e.forEach(e => e())
                }
                , [r]);
                let a = function(e) {
                    let t = (0,
                    B.useRef)(void 0);
                    return (0,
                    B.useEffect)( () => {
                        t.current = e
                    }
                    ),
                    t.current
                }(i);
                return (0,
                B.useEffect)( () => {
                    if (!a)
                        return;
                    let e = new Set(i);
                    new Set(a.filter(t => !e.has(t))).forEach(e => e.destroy())
                }
                , [a, i]),
                (0,
                B.useEffect)( () => () => i.forEach(e => e.destroy()), []),
                (0,
                B.useMemo)( () => [...i, ...e.filter( ({name: e}) => !i.some(t => t.name === e) || (t.has(e) || (t.add(e),
                console.warn(`${e} was registered as a Standard Wallet. The Wallet Adapter for ${e} can be removed from your app.`)),
                !1))], [i, e, t])
            }(t)
              , o = (0,
            B.useMemo)( () => {
                var e;
                if (!Z(a))
                    return null;
                let t = a.find(e => e.name === I.fG);
                return t ? t : new I.Ne({
                    addressSelector: (0,
                    I.RP)(),
                    appIdentity: {
                        uri: function() {
                            let e = globalThis.location;
                            if (e)
                                return `${e.protocol}//${e.host}`
                        }()
                    },
                    authorizationResultCache: (0,
                    I.u)(),
                    cluster: (e = s?.rpcEndpoint) ? /devnet/i.test(e) ? "devnet" : /testnet/i.test(e) ? "testnet" : "mainnet-beta" : "mainnet-beta",
                    onWalletNotFound: (0,
                    I.O0)()
                })
            }
            , [a, s?.rpcEndpoint])
              , l = (0,
            B.useMemo)( () => null == o || -1 !== a.indexOf(o) ? a : [o, ...a], [a, o])
              , [c,d] = function(e, t) {
                let n = (0,
                B.useState)( () => {
                    try {
                        let t = localStorage.getItem(e);
                        if (t)
                            return JSON.parse(t)
                    } catch (e) {
                        "undefined" != typeof window && console.error(e)
                    }
                    return null
                }
                )
                  , r = n[0]
                  , i = (0,
                B.useRef)(!0);
                return (0,
                B.useEffect)( () => {
                    if (i.current) {
                        i.current = !1;
                        return
                    }
                    try {
                        null === r ? localStorage.removeItem(e) : localStorage.setItem(e, JSON.stringify(r))
                    } catch (e) {
                        "undefined" != typeof window && console.error(e)
                    }
                }
                , [r, e]),
                n
            }(r, 0)
              , u = (0,
            B.useMemo)( () => l.find(e => e.name === c) ?? null, [l, c])
              , h = (0,
            B.useCallback)(e => {
                c !== e && (u && u.name !== I.fG && u.disconnect(),
                d(e))
            }
            , [u, d, c]);
            (0,
            B.useEffect)( () => {
                if (u)
                    return u.on("disconnect", e),
                    () => {
                        u.off("disconnect", e)
                    }
                    ;
                function e() {
                    y.current || d(null)
                }
            }
            , [u, a, d, c]);
            let p = (0,
            B.useRef)(!1)
              , f = (0,
            B.useMemo)( () => {
                if (n && u)
                    return async () => {
                        (!0 === n || await n(u)) && (p.current ? await u.connect() : await u.autoConnect())
                    }
            }
            , [n, u])
              , y = (0,
            B.useRef)(!1);
            (0,
            B.useEffect)( () => {
                if (c === I.fG && Z(a)) {
                    y.current = !1;
                    return
                }
                function e() {
                    y.current = !0
                }
                return window.addEventListener("beforeunload", e),
                () => {
                    window.removeEventListener("beforeunload", e)
                }
            }
            , [a, c]);
            let m = (0,
            B.useCallback)( () => {
                u && h(null)
            }
            , [u, h])
              , g = (0,
            B.useCallback)(e => {
                p.current = !0,
                h(e)
            }
            , [h]);
            return B.createElement(G, {
                wallets: l,
                adapter: u,
                isUnloadingRef: y,
                onAutoConnectRequest: f,
                onConnectError: m,
                onError: i,
                onSelectWallet: g
            }, e)
        }
    }
    ,
    4091: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                viewBox: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                fillRule: "evenodd",
                d: "M8.603 3.799A4.49 4.49 0 0 1 12 2.25c1.357 0 2.573.6 3.397 1.549a4.49 4.49 0 0 1 3.498 1.307 4.491 4.491 0 0 1 1.307 3.497A4.49 4.49 0 0 1 21.75 12a4.49 4.49 0 0 1-1.549 3.397 4.491 4.491 0 0 1-1.307 3.497 4.491 4.491 0 0 1-3.497 1.307A4.49 4.49 0 0 1 12 21.75a4.49 4.49 0 0 1-3.397-1.549 4.49 4.49 0 0 1-3.498-1.306 4.491 4.491 0 0 1-1.307-3.498A4.49 4.49 0 0 1 2.25 12c0-1.357.6-2.573 1.549-3.397a4.49 4.49 0 0 1 1.307-3.497 4.49 4.49 0 0 1 3.497-1.307Zm7.007 6.387a.75.75 0 1 0-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 0 0-1.06 1.06l2.25 2.25a.75.75 0 0 0 1.14-.094l3.75-5.25Z",
                clipRule: "evenodd"
            }))
        })
    }
    ,
    6117: function(e, t, n) {
        "use strict";
        var r, i, s = this && this.__awaiter || function(e, t, n, r) {
            return new (n || (n = Promise))(function(i, s) {
                function a(e) {
                    try {
                        l(r.next(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function o(e) {
                    try {
                        l(r.throw(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function l(e) {
                    var t;
                    e.done ? i(e.value) : ((t = e.value)instanceof n ? t : new n(function(e) {
                        e(t)
                    }
                    )).then(a, o)
                }
                l((r = r.apply(e, t || [])).next())
            }
            )
        }
        , a = this && this.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            }
        }
        ;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.BaseWalletAdapter = t.WalletReadyState = t.EventEmitter = void 0,
        t.scopePollingDetectionStrategy = function(e) {
            if ("undefined" == typeof window || "undefined" == typeof document)
                return;
            let t = [];
            function n() {
                if (e())
                    for (let e of t)
                        e()
            }
            let r = setInterval(n, 1e3);
            t.push( () => clearInterval(r)),
            "loading" === document.readyState && (document.addEventListener("DOMContentLoaded", n, {
                once: !0
            }),
            t.push( () => document.removeEventListener("DOMContentLoaded", n))),
            "complete" !== document.readyState && (window.addEventListener("load", n, {
                once: !0
            }),
            t.push( () => window.removeEventListener("load", n))),
            n()
        }
        ,
        t.isIosAndRedirectable = function() {
            if (!navigator)
                return !1;
            let e = navigator.userAgent.toLowerCase()
              , t = e.includes("iphone") || e.includes("ipad")
              , n = e.includes("safari");
            return t && n
        }
        ;
        let o = a(n(93286));
        t.EventEmitter = o.default;
        let l = n(68429);
        (r = i || (t.WalletReadyState = i = {})).Installed = "Installed",
        r.NotDetected = "NotDetected",
        r.Loadable = "Loadable",
        r.Unsupported = "Unsupported";
        class c extends o.default {
            get connected() {
                return !!this.publicKey
            }
            autoConnect() {
                return s(this, void 0, void 0, function*() {
                    yield this.connect()
                })
            }
            prepareTransaction(e, t) {
                return s(this, arguments, void 0, function*(e, t, n={}) {
                    let r = this.publicKey;
                    if (!r)
                        throw new l.WalletNotConnectedError;
                    return e.feePayer = e.feePayer || r,
                    e.recentBlockhash = e.recentBlockhash || (yield t.getLatestBlockhash({
                        commitment: n.preflightCommitment,
                        minContextSlot: n.minContextSlot
                    })).blockhash,
                    e
                })
            }
        }
        t.BaseWalletAdapter = c
    },
    6727: (e, t, n) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.isWalletAdapterCompatibleStandardWallet = function(e) {
            return i.StandardConnect in e.features && i.StandardEvents in e.features && (r.SolanaSignAndSendTransaction in e.features || r.SolanaSignTransaction in e.features)
        }
        ;
        let r = n(69523)
          , i = n(51427)
    }
    ,
    6801: (e, t, n) => {
        "use strict";
        n.d(t, {
            w: () => r
        });
        let r = "standard:disconnect"
    }
    ,
    7021: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                viewBox: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                fillRule: "evenodd",
                d: "M12.516 2.17a.75.75 0 0 0-1.032 0 11.209 11.209 0 0 1-7.877 3.08.75.75 0 0 0-.722.515A12.74 12.74 0 0 0 2.25 9.75c0 5.942 4.064 10.933 9.563 12.348a.749.749 0 0 0 .374 0c5.499-1.415 9.563-6.406 9.563-12.348 0-1.39-.223-2.73-.635-3.985a.75.75 0 0 0-.722-.516l-.143.001c-2.996 0-5.717-1.17-7.734-3.08Zm3.094 8.016a.75.75 0 1 0-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 0 0-1.06 1.06l2.25 2.25a.75.75 0 0 0 1.14-.094l3.75-5.25Z",
                clipRule: "evenodd"
            }))
        })
    }
    ,
    7464: () => {}
    ,
    10434: (e, t, n) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.wrapXOFConstructorWithOpts = t.wrapConstructorWithOpts = t.wrapConstructor = t.Hash = t.nextTick = t.swap32IfBE = t.byteSwapIfBE = t.swap8IfBE = t.isLE = void 0,
        t.isBytes = i,
        t.anumber = s,
        t.abytes = a,
        t.ahash = function(e) {
            if ("function" != typeof e || "function" != typeof e.create)
                throw Error("Hash should be wrapped by utils.createHasher");
            s(e.outputLen),
            s(e.blockLen)
        }
        ,
        t.aexists = function(e, t=!0) {
            if (e.destroyed)
                throw Error("Hash instance has been destroyed");
            if (t && e.finished)
                throw Error("Hash#digest() has already been called")
        }
        ,
        t.aoutput = function(e, t) {
            a(e);
            let n = t.outputLen;
            if (e.length < n)
                throw Error("digestInto() expects output buffer of length at least " + n)
        }
        ,
        t.u8 = function(e) {
            return new Uint8Array(e.buffer,e.byteOffset,e.byteLength)
        }
        ,
        t.u32 = function(e) {
            return new Uint32Array(e.buffer,e.byteOffset,Math.floor(e.byteLength / 4))
        }
        ,
        t.clean = function(...e) {
            for (let t = 0; t < e.length; t++)
                e[t].fill(0)
        }
        ,
        t.createView = function(e) {
            return new DataView(e.buffer,e.byteOffset,e.byteLength)
        }
        ,
        t.rotr = function(e, t) {
            return e << 32 - t | e >>> t
        }
        ,
        t.rotl = function(e, t) {
            return e << t | e >>> 32 - t >>> 0
        }
        ,
        t.byteSwap = o,
        t.byteSwap32 = l,
        t.bytesToHex = function(e) {
            if (a(e),
            c)
                return e.toHex();
            let t = "";
            for (let n = 0; n < e.length; n++)
                t += d[e[n]];
            return t
        }
        ,
        t.hexToBytes = function(e) {
            if ("string" != typeof e)
                throw Error("hex string expected, got " + typeof e);
            if (c)
                return Uint8Array.fromHex(e);
            let t = e.length
              , n = t / 2;
            if (t % 2)
                throw Error("hex string expected, got unpadded hex of length " + t);
            let r = new Uint8Array(n);
            for (let t = 0, i = 0; t < n; t++,
            i += 2) {
                let n = h(e.charCodeAt(i))
                  , s = h(e.charCodeAt(i + 1));
                if (void 0 === n || void 0 === s)
                    throw Error('hex string expected, got non-hex character "' + (e[i] + e[i + 1]) + '" at index ' + i);
                r[t] = 16 * n + s
            }
            return r
        }
        ,
        t.asyncLoop = p,
        t.utf8ToBytes = f,
        t.bytesToUtf8 = function(e) {
            return new TextDecoder().decode(e)
        }
        ,
        t.toBytes = y,
        t.kdfInputToBytes = function(e) {
            return "string" == typeof e && (e = f(e)),
            a(e),
            e
        }
        ,
        t.concatBytes = function(...e) {
            let t = 0;
            for (let n = 0; n < e.length; n++) {
                let r = e[n];
                a(r),
                t += r.length
            }
            let n = new Uint8Array(t);
            for (let t = 0, r = 0; t < e.length; t++) {
                let i = e[t];
                n.set(i, r),
                r += i.length
            }
            return n
        }
        ,
        t.checkOpts = function(e, t) {
            if (void 0 !== t && "[object Object]" !== ({}).toString.call(t))
                throw Error("options should be object or undefined");
            return Object.assign(e, t)
        }
        ,
        t.createHasher = g,
        t.createOptHasher = w,
        t.createXOFer = _,
        t.randomBytes = function(e=32) {
            if (r.crypto && "function" == typeof r.crypto.getRandomValues)
                return r.crypto.getRandomValues(new Uint8Array(e));
            if (r.crypto && "function" == typeof r.crypto.randomBytes)
                return Uint8Array.from(r.crypto.randomBytes(e));
            throw Error("crypto.getRandomValues must be defined")
        }
        ;
        let r = n(40756);
        function i(e) {
            return e instanceof Uint8Array || ArrayBuffer.isView(e) && "Uint8Array" === e.constructor.name
        }
        function s(e) {
            if (!Number.isSafeInteger(e) || e < 0)
                throw Error("positive integer expected, got " + e)
        }
        function a(e, ...t) {
            if (!i(e))
                throw Error("Uint8Array expected");
            if (t.length > 0 && !t.includes(e.length))
                throw Error("Uint8Array expected of length " + t + ", got length=" + e.length)
        }
        function o(e) {
            return e << 24 & 0xff000000 | e << 8 & 0xff0000 | e >>> 8 & 65280 | e >>> 24 & 255
        }
        function l(e) {
            for (let t = 0; t < e.length; t++)
                e[t] = o(e[t]);
            return e
        }
        t.isLE = 68 === new Uint8Array(new Uint32Array([0x11223344]).buffer)[0],
        t.swap8IfBE = t.isLE ? e => e : e => o(e),
        t.byteSwapIfBE = t.swap8IfBE,
        t.swap32IfBE = t.isLE ? e => e : l;
        let c = "function" == typeof Uint8Array.from([]).toHex && "function" == typeof Uint8Array.fromHex
          , d = Array.from({
            length: 256
        }, (e, t) => t.toString(16).padStart(2, "0"))
          , u = {
            _0: 48,
            _9: 57,
            A: 65,
            F: 70,
            a: 97,
            f: 102
        };
        function h(e) {
            return e >= u._0 && e <= u._9 ? e - u._0 : e >= u.A && e <= u.F ? e - (u.A - 10) : e >= u.a && e <= u.f ? e - (u.a - 10) : void 0
        }
        async function p(e, n, r) {
            let i = Date.now();
            for (let s = 0; s < e; s++) {
                r(s);
                let e = Date.now() - i;
                e >= 0 && e < n || (await (0,
                t.nextTick)(),
                i += e)
            }
        }
        function f(e) {
            if ("string" != typeof e)
                throw Error("string expected");
            return new Uint8Array(new TextEncoder().encode(e))
        }
        function y(e) {
            return "string" == typeof e && (e = f(e)),
            a(e),
            e
        }
        t.nextTick = async () => {}
        ;
        class m {
        }
        function g(e) {
            let t = t => e().update(y(t)).digest()
              , n = e();
            return t.outputLen = n.outputLen,
            t.blockLen = n.blockLen,
            t.create = () => e(),
            t
        }
        function w(e) {
            let t = (t, n) => e(n).update(y(t)).digest()
              , n = e({});
            return t.outputLen = n.outputLen,
            t.blockLen = n.blockLen,
            t.create = t => e(t),
            t
        }
        function _(e) {
            let t = (t, n) => e(n).update(y(t)).digest()
              , n = e({});
            return t.outputLen = n.outputLen,
            t.blockLen = n.blockLen,
            t.create = t => e(t),
            t
        }
        t.Hash = m,
        t.wrapConstructor = g,
        t.wrapConstructorWithOpts = w,
        t.wrapXOFConstructorWithOpts = _
    }
    ,
    11155: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.Disconnect = t.StandardDisconnect = void 0,
        t.StandardDisconnect = "standard:disconnect",
        t.Disconnect = t.StandardDisconnect
    }
    ,
    11555: (e, t, n) => {
        "use strict";
        n.d(t, {
            Xs: () => u
        });
        var r = n(48876)
          , i = n(26432);
        let s = (0,
        i.forwardRef)( (e, t) => {
            let {as: n="div", ...i} = e;
            return (0,
            r.jsx)(n, {
                ...i,
                ref: t
            })
        }
        )
          , a = "cf-turnstile-script"
          , o = "onloadTurnstileCallback"
          , l = e => !!document.getElementById(e)
          , c = e => {
            let {render: t="explicit", onLoadCallbackName: n=o, scriptOptions: {nonce: r="", defer: i=!0, async: s=!0, id: c="", appendTo: d, onError: u, crossOrigin: h=""}={}} = e
              , p = c || a;
            if (l(p))
                return;
            let f = document.createElement("script");
            f.id = p,
            f.src = "".concat("https://challenges.cloudflare.com/turnstile/v0/api.js", "?onload=").concat(n, "&render=").concat(t),
            document.querySelector('script[src="'.concat(f.src, '"]')) || (f.defer = !!i,
            f.async = !!s,
            r && (f.nonce = r),
            h && (f.crossOrigin = h),
            u && (f.onerror = u),
            ("body" === d ? document.body : document.getElementsByTagName("head")[0]).appendChild(f))
        }
          , d = {
            normal: {
                width: 300,
                height: 65
            },
            compact: {
                width: 130,
                height: 120
            },
            invisible: {
                width: 0,
                height: 0,
                overflow: "hidden"
            },
            interactionOnly: {
                width: "fit-content",
                height: "auto"
            }
        }
          , u = (0,
        i.forwardRef)( (e, t) => {
            var n;
            let {scriptOptions: u, options: h={}, siteKey: p, onWidgetLoad: f, onSuccess: y, onExpire: m, onError: g, onBeforeInteractive: w, onAfterInteractive: _, onUnsupported: v, onLoadScript: b, id: E, style: I, as: k="div", injectScript: M=!0, ...S} = e
              , A = null != (n = h.size) ? n : "normal"
              , [T,C] = (0,
            i.useState)("execute" === h.execution ? d.invisible : "interaction-only" === h.appearance ? d.interactionOnly : d[A])
              , O = (0,
            i.useRef)(null)
              , N = (0,
            i.useRef)(!1)
              , [L,x] = (0,
            i.useState)()
              , [j,D] = (0,
            i.useState)(!1)
              , R = null != E ? E : "cf-turnstile"
              , P = M ? (null == u ? void 0 : u.id) || "".concat(a, "__").concat(R) : (null == u ? void 0 : u.id) || a
              , W = function() {
                let e = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : a
                  , [t,n] = (0,
                i.useState)(!1);
                return (0,
                i.useEffect)( () => {
                    let t = () => {
                        l(e) && n(!0)
                    }
                      , r = new MutationObserver(t);
                    return r.observe(document, {
                        childList: !0,
                        subtree: !0
                    }),
                    t(),
                    () => {
                        r.disconnect()
                    }
                }
                , [e]),
                t
            }(P)
              , U = (null == u ? void 0 : u.onLoadCallbackName) ? "".concat(u.onLoadCallbackName, "__").concat(R) : "".concat(o, "__").concat(R)
              , z = (0,
            i.useMemo)( () => {
                var e, t, n, r, i, s, a;
                return {
                    sitekey: p,
                    action: h.action,
                    cData: h.cData,
                    callback: y,
                    "error-callback": g,
                    "expired-callback": m,
                    "before-interactive-callback": w,
                    "after-interactive-callback": _,
                    "unsupported-callback": v,
                    theme: null != (e = h.theme) ? e : "auto",
                    language: null != (t = h.language) ? t : "auto",
                    tabindex: h.tabIndex,
                    "response-field": h.responseField,
                    "response-field-name": h.responseFieldName,
                    size: function(e) {
                        let t;
                        return "invisible" !== e && (t = e),
                        t
                    }(A),
                    retry: null != (n = h.retry) ? n : "auto",
                    "retry-interval": null != (r = h.retryInterval) ? r : 8e3,
                    "refresh-expired": null != (i = h.refreshExpired) ? i : "auto",
                    execution: null != (s = h.execution) ? s : "render",
                    appearance: null != (a = h.appearance) ? a : "always"
                }
            }
            , [p, h, y, g, m, A, w, _, v])
              , q = (0,
            i.useMemo)( () => JSON.stringify(z), [z]);
            return (0,
            i.useImperativeHandle)(t, () => {
                if ("undefined" == typeof window || !W)
                    return;
                let {turnstile: e} = window;
                return {
                    getResponse: () => (null == e ? void 0 : e.getResponse) && L ? e.getResponse(L) : void console.warn("Turnstile has not been loaded"),
                    reset() {
                        if (!(null == e ? void 0 : e.reset) || !L)
                            return void console.warn("Turnstile has not been loaded");
                        "execute" === h.execution && C(d.invisible);
                        try {
                            e.reset(L)
                        } catch (e) {
                            console.warn("Failed to reset Turnstile widget ".concat(L), e)
                        }
                    },
                    remove() {
                        if (!(null == e ? void 0 : e.remove) || !L)
                            return void console.warn("Turnstile has not been loaded");
                        x(""),
                        C(d.invisible),
                        e.remove(L)
                    },
                    render() {
                        if (!(null == e ? void 0 : e.render) || !O.current || L)
                            return void console.warn("Turnstile has not been loaded or widget already rendered");
                        let t = e.render(O.current, z);
                        return x(t),
                        "execute" !== h.execution && C(d[A]),
                        t
                    },
                    execute() {
                        if ("execute" === h.execution) {
                            if (!(null == e ? void 0 : e.execute) || !O.current || !L)
                                return void console.warn("Turnstile has not been loaded or widget has not been rendered");
                            e.execute(O.current, z),
                            C(d[A])
                        }
                    },
                    isExpired: () => (null == e ? void 0 : e.isExpired) && L ? e.isExpired(L) : void console.warn("Turnstile has not been loaded")
                }
            }
            , [W, L, h.execution, A, z, O]),
            (0,
            i.useEffect)( () => (window[U] = () => D(!0),
            () => {
                delete window[U]
            }
            ), [U]),
            (0,
            i.useEffect)( () => {
                M && !j && c({
                    onLoadCallbackName: U,
                    scriptOptions: {
                        ...u,
                        id: P
                    }
                })
            }
            , [M, j, U, u, P]),
            (0,
            i.useEffect)( () => {
                W && !j && window.turnstile && D(!0)
            }
            , [j, W]),
            (0,
            i.useEffect)( () => {
                if (!p)
                    return void console.warn("sitekey was not provided");
                W && O.current && j && !N.current && (x(window.turnstile.render(O.current, z)),
                N.current = !0)
            }
            , [W, p, z, N, j]),
            (0,
            i.useEffect)( () => {
                window.turnstile && O.current && L && (l(L) && window.turnstile.remove(L),
                x(window.turnstile.render(O.current, z)),
                N.current = !0)
            }
            , [q, p]),
            (0,
            i.useEffect)( () => {
                if (window.turnstile && L && l(L))
                    return null == f || f(L),
                    () => {
                        window.turnstile.remove(L)
                    }
            }
            , [L, f]),
            (0,
            i.useEffect)( () => {
                C("execute" === h.execution ? d.invisible : "interaction-only" === z.appearance ? d.interactionOnly : d[A])
            }
            , [h.execution, A, z.appearance]),
            (0,
            i.useEffect)( () => {
                W && "function" == typeof b && b()
            }
            , [W, b]),
            (0,
            r.jsx)(s, {
                ref: O,
                as: k,
                id: R,
                style: {
                    ...T,
                    ...I
                },
                ...S
            })
        }
        );
        u.displayName = "Turnstile"
    }
    ,
    12313: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => tq
        });
        let r = {
            path: "/api/v1/apps/:app_id/smart_wallets",
            method: "GET"
        };
        class i {
            getConfig() {
                return this._privyInternal.config
            }
            async getSmartWalletConfig() {
                return this._smartWalletConfig || (this._smartWalletConfig = await this._privyInternal.fetch(r, {
                    params: {
                        app_id: this.appId
                    }
                })),
                this._smartWalletConfig
            }
            get appId() {
                return this._privyInternal.appId
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        let s = {
            path: "/api/v1/apps/:app_id/cross-app/connections",
            method: "GET"
        };
        var a = n(33953);
        class o {
            static parse(e) {
                try {
                    return new o(e)
                } catch (e) {
                    return null
                }
            }
            get subject() {
                return this._decoded.sub
            }
            get expiration() {
                return this._decoded.exp
            }
            get issuer() {
                return this._decoded.iss
            }
            get audience() {
                return this._decoded.aud
            }
            isExpired(e=0) {
                return Date.now() >= 1e3 * (this.expiration - e)
            }
            constructor(e) {
                this.value = e,
                this._decoded = a.i(e)
            }
        }
        class l {
            async updateOnCrossAppAuthentication(e, t) {
                let n = t.access_token
                  , r = l.providerAccessTokenStorageKey(e);
                await this._storage.put(r, n)
            }
            async getProviderAccessToken(e) {
                let t = l.providerAccessTokenStorageKey(e)
                  , n = await this._storage.get(t);
                if ("string" != typeof n)
                    return null;
                try {
                    if (new o(n).isExpired())
                        throw Error("JWT is expired");
                    return n
                } catch {
                    return await this._storage.del(t),
                    null
                }
            }
            async getCrossAppConnections() {
                return await this._privyInternal.fetch(s, {
                    params: {
                        app_id: this._privyInternal.appId
                    }
                })
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._storage = t
            }
        }
        l.providerAccessTokenStorageKey = e => `privy:cross-app:${e}`;
        var c = n(46e3);
        class d {
            async revoke() {
                await this._privyInternal.fetch(c.xh, {})
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        var u = n(78899)
          , h = n(26736)
          , p = n(45323);
        let f = e => !!e.id && "privy-v2" === e.recovery_method;
        class y {
            async sign({message: e}) {
                return await this.request({
                    method: "sign",
                    params: {
                        message: new TextDecoder("utf8").decode(e)
                    }
                })
            }
            async signTransaction({psbt: e}) {
                return await this.request({
                    method: "signTransaction",
                    params: {
                        psbt: e
                    }
                })
            }
            async request(e) {
                if (f(this._account))
                    throw new p.wE({
                        code: "unsupported_wallet_type",
                        error: "Bitcoin wallet providers are only supported for on-device execution and this app uses TEE execution. Use the useSignRawHash hook from @privy-io/expo/extended-chains to sign over a hash with this wallet. Learn more at https://docs.privy.io/recipes/tee-wallet-migration-guide"
                    });
                if (!await this._privyInternal.getAccessToken())
                    throw new p.wE({
                        error: "Missing access token",
                        code: "attempted_rpc_call_before_logged_in"
                    });
                return this.handleIFrameRpc(e)
            }
            async handleIFrameRpc(e) {
                try {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw Error("Missing access token. User must be authenticated.");
                    return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_started", {
                        method: e.method,
                        address: this._account.address
                    }),
                    (await this._proxy.rpcWallet({
                        accessToken: t,
                        request: e,
                        entropyId: this._entropyId,
                        entropyIdVerifier: this._entropyIdVerifier,
                        hdWalletIndex: this._account.wallet_index,
                        chainType: this._account.chain_type
                    })).response.data
                } catch (n) {
                    console.error(n);
                    let t = n instanceof Error ? n.message : "Unable to make wallet request";
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_failed", {
                        method: e.method,
                        address: this._account.address,
                        error: t
                    }),
                    new p.wE({
                        code: "embedded_wallet_request_error",
                        error: t
                    })
                }
            }
            toJSON() {
                return `PrivyEmbeddedBitcoinProvider { address: '${this._account.address}', request: [Function] }`
            }
            constructor({proxy: e, privyInternal: t, account: n, entropyId: r, entropyIdVerifier: i}) {
                this._proxy = e,
                this._privyInternal = t,
                this._account = n,
                this._entropyId = r,
                this._entropyIdVerifier = i
            }
        }
        var m = n(64811)
          , g = n(46478);
        async function w({context: e, chainType: t}) {
            return {
                wallet: await (0,
                g.v)(e, {
                    request: {
                        chain_type: t,
                        owner_id: void 0
                    }
                })
            }
        }
        var _ = n(87932);
        function v(e) {
            return new Promise(t => {
                setTimeout( () => {
                    t()
                }
                , e)
            }
            )
        }
        class b {
            enqueue(e, t) {
                this.callbacks[e] = t
            }
            dequeue(e, t) {
                let n = this.callbacks[t];
                if (!n)
                    throw Error(`cannot dequeue ${e} event: no event found for id ${t}`);
                switch (delete this.callbacks[t],
                e) {
                case "privy:iframe:ready":
                case "privy:wallets:create":
                case "privy:user-signer:sign":
                case "privy:wallets:add":
                case "privy:wallets:set-recovery":
                case "privy:wallets:connect":
                case "privy:wallets:recover":
                case "privy:wallets:rpc":
                case "privy:wallet:create":
                case "privy:wallet:connect":
                case "privy:wallet:recover":
                case "privy:wallet:rpc":
                case "privy:solana-wallet:create":
                case "privy:solana-wallet:create-additional":
                case "privy:solana-wallet:connect":
                case "privy:solana-wallet:recover":
                case "privy:solana-wallet:rpc":
                case "privy:delegated-actions:consent":
                case "privy:mfa:verify":
                case "privy:mfa:init-enrollment":
                case "privy:mfa:submit-enrollment":
                case "privy:mfa:unenroll":
                case "privy:mfa:clear":
                    return n;
                default:
                    throw Error(`invalid wallet event type ${e}`)
                }
            }
            constructor() {
                this.callbacks = {}
            }
        }
        async function E(e, t, n, r, i=!1, s, a) {
            let o = i
              , l = async s => {
                if (o) {
                    s === +!i ? r() : n.current?.reject(new m.Pi("missing_or_invalid_mfa","MFA verification failed, retry."));
                    let o = await new Promise( (e, r) => {
                        t.current = {
                            resolve: e,
                            reject: r
                        },
                        setTimeout( () => {
                            let e = new m.Pi("mfa_timeout","Timed out waiting for MFA code");
                            n.current?.reject(e),
                            r(e)
                        }
                        , a)
                    }
                    );
                    return await e(o)
                }
                return await e()
            }
              , c = null;
            for (let e = 0; e < s; e++)
                try {
                    c = await l(e),
                    n.current?.resolve(void 0);
                    break
                } catch (e) {
                    if ("missing_or_invalid_mfa" !== e.type)
                        throw n.current?.resolve(void 0),
                        e;
                    o = !0
                }
            if (null === c) {
                let e = new m.Pi("mfa_verification_max_attempts_reached","Max MFA verification attempts reached");
                throw n.current?.reject(e),
                e
            }
            return c
        }
        let I = (V = 0,
        () => "id-" + V++)
          , k = (e, t) => "bigint" == typeof t ? t.toString() : t
          , M = (e, {ms: t, msg: n}) => Promise.race([e, v(t ?? 15e3).then( () => Promise.reject(Error(n)))])
          , S = new b;
        class A {
            invokeWithMfa(e, t) {
                return M(E(n => M(this.waitForReady().then( () => e(n)), {
                    msg: t.timeoutMsg,
                    ms: t.timeoutMs
                }), this.mfa.rootPromise, this.mfa.submitPromise, () => this.mfa.emit("mfaRequired"), t.mfaAlwaysRequired ?? !1, 4, 3e5), {
                    msg: "Operation reached timeout: MFA verification",
                    ms: 126e4
                })
            }
            reload() {
                return this.ready = !1,
                this._embeddedWalletMessagePoster.reload()
            }
            ping(e=15e3) {
                return M(this.invoke("privy:iframe:ready", {}), {
                    msg: "Ping reached timeout",
                    ms: e
                })
            }
            create(e) {
                return M(this.waitForReady().then( () => this.invoke("privy:wallet:create", e)), {
                    msg: "Operation reached timeout: create"
                })
            }
            rpc(e) {
                return this.invokeWithMfa(t => this.invoke("privy:wallet:rpc", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: rpc"
                })
            }
            createSolana(e) {
                return this.invokeWithMfa(t => this.invoke("privy:solana-wallet:create", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: create",
                    timeoutMs: 6e4
                })
            }
            createAdditionalSolana(e) {
                return M(this.waitForReady().then( () => this.invoke("privy:solana-wallet:create-additional", e)), {
                    msg: "Operation reached timeout: create"
                })
            }
            solanaRpc(e) {
                return this.invokeWithMfa(t => this.invoke("privy:solana-wallet:rpc", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: solana-rpc"
                })
            }
            delegateWallets(e) {
                return this.invokeWithMfa(t => this.invoke("privy:delegated-actions:consent", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: delegated-actions:consent"
                })
            }
            verifyMfa(e) {
                return this.invokeWithMfa(t => this.invoke("privy:mfa:verify", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: mfa:verify",
                    mfaAlwaysRequired: !0
                })
            }
            initEnrollMfa(e) {
                return this.invokeWithMfa(t => this.invoke("privy:mfa:init-enrollment", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: mfa:init-enrollment"
                })
            }
            submitEnrollMfa(e) {
                return this.invokeWithMfa(t => this.invoke("privy:mfa:submit-enrollment", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: mfa:submit-enrollment"
                })
            }
            unenrollMfa(e) {
                return this.invokeWithMfa(t => this.invoke("privy:mfa:unenroll", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: mfa:unenroll",
                    mfaAlwaysRequired: !0
                })
            }
            clearMfa(e) {
                return M(this.waitForReady().then( () => this.invoke("privy:mfa:clear", e)), {
                    msg: "Operation reached timeout: mfa:clear"
                })
            }
            createWallet(e) {
                return this.invokeWithMfa(t => this.invoke("privy:wallets:create", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: create",
                    timeoutMs: 6e4
                })
            }
            signWithUserSigner(e) {
                return this.invokeWithMfa(t => this.invoke("privy:user-signer:sign", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: user-signer:sign"
                })
            }
            addWallet(e) {
                return M(this.waitForReady().then( () => this.invoke("privy:wallets:add", e)), {
                    msg: "Operation reached timeout: wallets:add"
                })
            }
            setRecovery(e) {
                return this.invokeWithMfa(t => this.invoke("privy:wallets:set-recovery", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: wallets:set-recovery",
                    timeoutMs: 6e4
                })
            }
            connect(e) {
                return M(this.waitForReady().then( () => this.invoke("privy:wallets:connect", e)), {
                    msg: "Operation reached timeout: wallets:connect"
                })
            }
            recover(e) {
                return this.invokeWithMfa(t => this.invoke("privy:wallets:recover", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: wallets:recover",
                    timeoutMs: 6e4
                })
            }
            rpcWallet(e) {
                return this.invokeWithMfa(t => this.invoke("privy:wallets:rpc", {
                    ...t,
                    ...e
                }), {
                    timeoutMsg: "Operation reached timeout: wallets:rpc"
                })
            }
            handleEmbeddedWalletMessages(e) {
                let {reject: t, resolve: n} = S.dequeue(e.event, e.id);
                return void 0 !== e.error ? t(new m.Pi(e.error.type,e.error.message)) : n(e.data)
            }
            waitForReady() {
                return this.ready ? Promise.resolve() : new Promise(async (e, t) => {
                    for (; !this.ready; )
                        this.invoke("privy:iframe:ready", {}).then( () => {
                            this.ready = !0,
                            e()
                        }
                        ).catch(t),
                        await v(150)
                }
                )
            }
            invoke(e, t) {
                let n = `${e}${JSON.stringify(t, k)}`;
                if ("privy:wallet:create" === e || "privy:solana-wallet:create" === e) {
                    let e = this.cache.get(n);
                    if (e)
                        return e
                }
                let r = new Promise( (n, r) => {
                    let i = I();
                    S.enqueue(i, {
                        resolve: n,
                        reject: r
                    }),
                    this._embeddedWalletMessagePoster.postMessage(JSON.stringify({
                        id: i,
                        event: e,
                        data: t
                    }), "*")
                }
                ).finally( () => {
                    this.cache.delete(n)
                }
                );
                return this.cache.set(n, r),
                r
            }
            constructor(e, t) {
                this.ready = !1,
                this.cache = new Map,
                this._embeddedWalletMessagePoster = e,
                this.mfa = t
            }
        }
        var T = n(2589)
          , C = n(17223)
          , O = n(12627)
          , N = n(91015).Buffer;
        let L = e => /^0x[0-9a-fA-F]*$/.test(e)
          , x = e => N.from(e, "utf8")
          , j = e => `0x${e.toString("hex")}`;
        var D = n(63718)
          , R = n(91015).Buffer;
        function P(e) {
            return "number" == typeof e || "bigint" == typeof e ? `0x${BigInt(e).toString(16)}` : "string" == typeof e ? L(e) ? e : j(x(e)) : void 0
        }
        async function W({context: e, account: t, rpcRequest: n}) {
            switch (n.chainType) {
            case "ethereum":
                return async function({context: e, account: t, rpcRequest: n}) {
                    var r;
                    switch (n.method) {
                    case "personal_sign":
                        {
                            let[r] = n.params
                              , i = await (0,
                            D._)(e, e.signRequest, {
                                chain_type: "ethereum",
                                method: "personal_sign",
                                wallet_id: t.id,
                                params: r.startsWith("0x") ? {
                                    message: r.slice(2),
                                    encoding: "hex"
                                } : {
                                    message: r,
                                    encoding: "utf-8"
                                }
                            });
                            if ("personal_sign" !== i.method)
                                throw Error("Unable to sign message");
                            return {
                                data: i.data.signature
                            }
                        }
                    case "eth_signTransaction":
                        {
                            let[r] = n.params
                              , i = await (0,
                            D._)(e, e.signRequest, {
                                chain_type: "ethereum",
                                method: "eth_signTransaction",
                                wallet_id: t.id,
                                params: {
                                    transaction: {
                                        from: r.from,
                                        to: r.to ?? void 0,
                                        nonce: P(r.nonce),
                                        chain_id: P(r.chainId),
                                        data: function(e) {
                                            if (void 0 !== e)
                                                return "string" == typeof e ? L(e) ? e : j(x(e)) : j(R.from(Uint8Array.from(e)))
                                        }(r.data),
                                        value: P(r.value),
                                        type: r.type,
                                        gas_limit: P(r.gasLimit),
                                        gas_price: P(r.gasPrice),
                                        max_fee_per_gas: P(r.maxFeePerGas),
                                        max_priority_fee_per_gas: P(r.maxPriorityFeePerGas)
                                    }
                                }
                            });
                            if ("eth_signTransaction" !== i.method)
                                throw Error("Unable to sign transaction");
                            return {
                                data: i.data.signed_transaction
                            }
                        }
                    case "eth_signTypedData_v4":
                        {
                            let[,i] = n.params
                              , s = await (0,
                            D._)(e, e.signRequest, {
                                chain_type: "ethereum",
                                method: n.method,
                                wallet_id: t.id,
                                params: {
                                    typed_data: ("string" == typeof (r = i) && (r = JSON.parse(r)),
                                    {
                                        types: r.types,
                                        primary_type: String(r.primaryType),
                                        domain: r.domain,
                                        message: r.message
                                    })
                                }
                            });
                            if ("eth_signTypedData_v4" !== s.method)
                                throw Error("Unable to sign typed data");
                            return {
                                data: s.data.signature
                            }
                        }
                    case "eth_sign":
                        {
                            let[,r] = n.params
                              , i = await (0,
                            D._)(e, e.signRequest, {
                                chain_type: "ethereum",
                                method: "secp256k1_sign",
                                wallet_id: t.id,
                                params: {
                                    hash: L(r) ? r : j(x(r))
                                }
                            });
                            if ("secp256k1_sign" !== i.method)
                                throw Error("Unable to sign message");
                            return {
                                data: i.data.signature
                            }
                        }
                    case "secp256k1_sign":
                        {
                            let[r] = n.params
                              , i = await (0,
                            D._)(e, e.signRequest, {
                                chain_type: "ethereum",
                                method: "secp256k1_sign",
                                wallet_id: t.id,
                                params: {
                                    hash: L(r) ? r : j(x(r))
                                }
                            });
                            if ("secp256k1_sign" !== i.method)
                                throw Error("Unable to sign message");
                            return {
                                data: i.data.signature
                            }
                        }
                    case "csw_signUserOperation":
                    case "eth_sendTransaction":
                    case "eth_populateTransactionRequest":
                        throw Error(`This wallet does not support the method: ${n.method}`)
                    }
                }({
                    context: e,
                    account: t,
                    rpcRequest: n.request
                });
            case "solana":
                return async function({context: e, account: t, rpcRequest: n}) {
                    if ("signMessage" === n.method) {
                        let {message: r} = n.params
                          , i = await (0,
                        D._)(e, e.signRequest, {
                            chain_type: "solana",
                            method: "signMessage",
                            wallet_id: t.id,
                            params: {
                                message: r,
                                encoding: "base64"
                            }
                        });
                        if ("signMessage" !== i.method)
                            throw Error("Unable to sign message");
                        return {
                            data: i.data.signature
                        }
                    }
                }({
                    context: e,
                    account: t,
                    rpcRequest: n.request
                })
            }
        }
        n(81683);
        let U = new Set(["eth_sign", "personal_sign", "eth_signTypedData_v4", "csw_signUserOperation", "secp256k1_sign"]);
        class z extends T.A {
            async request(e) {
                if (U.has(e.method))
                    return this.handleIFrameRpc(e);
                switch (e.method) {
                case "eth_accounts":
                case "eth_requestAccounts":
                    return this._account.address ? [this._account.address] : [];
                case "eth_chainId":
                    return `0x${this._chainId.toString(16)}`;
                case "wallet_switchEthereumChain":
                    return this.handleSwitchEthereumChain(e);
                case "eth_estimateGas":
                    return this.handleEstimateGas(e);
                case "eth_signTransaction":
                    {
                        let t = e.params?.[0];
                        return this.handleSignTransaction(t)
                    }
                case "eth_sendTransaction":
                    {
                        let t = e.params?.[0];
                        return this.handleSendTransaction(t)
                    }
                case "eth_populateTransactionRequest":
                    {
                        let t = e.params?.[0];
                        return this.handlePopulateTransaction(t)
                    }
                default:
                    return this.handleJsonRpc(e)
                }
            }
            ensureChainId(e) {
                let t = {
                    chainId: this._chainId,
                    ...e
                };
                return this.internalSwitchEthereumChain(t.chainId),
                t
            }
            internalSwitchEthereumChain(e) {
                e && Number(e) !== this._chainId && (this._chainId = Number(e),
                this._client = (0,
                _.Bm)(this._chainId, this._chains, {
                    rpcUrls: []
                }, {
                    appId: this._privyInternal.appId
                }),
                this.emit("chainChanged", e))
            }
            async handlePopulateTransaction(e) {
                let t = (0,
                O._)(this.ensureChainId(e))
                  , {type: n, ...r} = await this._client.prepareTransactionRequest(t);
                return {
                    ...r,
                    type: O.k[n]
                }
            }
            async handleSignTransaction(e) {
                let t = {
                    ...e
                };
                for (let e of Object.keys(t)) {
                    let n = t[e];
                    n && "bigint" == typeof n && (t[e] = (0,
                    C.nj)(n))
                }
                return await this.handleIFrameRpc({
                    method: "eth_signTransaction",
                    params: [t]
                })
            }
            async handleSendTransaction(e) {
                let t = await this.handlePopulateTransaction(e)
                  , n = await this.handleSignTransaction(t);
                return await this.handleJsonRpc({
                    method: "eth_sendRawTransaction",
                    params: [n]
                })
            }
            async handleEstimateGas(e) {
                if (!e.params || !Array.isArray(e.params))
                    throw Error("Invalid params for eth_estimateGas");
                let t = e.params?.[0]
                  , n = (0,
                O._)(this.ensureChainId(t));
                return await this._client.estimateGas(n)
            }
            handleSwitchEthereumChain(e) {
                let t;
                if (!e.params || !Array.isArray(e.params))
                    throw new m.sj(`Invalid params for ${e.method}`,4200);
                if ("string" == typeof e.params[0])
                    t = e.params[0];
                else {
                    if (!("chainId"in e.params[0]) || "string" != typeof e.params[0].chainId)
                        throw new m.sj(`Invalid params for ${e.method}`,4200);
                    t = e.params[0].chainId
                }
                this.internalSwitchEthereumChain(t)
            }
            async handleIFrameRpc(e) {
                try {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw Error("Missing privy token. User must be logged in");
                    this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_started", {
                        method: e.method,
                        address: this._account.address
                    });
                    let n = this._account;
                    if (f(n)) {
                        let {data: r} = await W({
                            context: {
                                app: this._appApi,
                                fetchPrivyRoute: (...e) => this._privyInternal.fetch(...e),
                                getCompiledPath: (...e) => this._privyInternal.getPath(...e),
                                signRequest: ({message: e}) => this._walletProxy.signWithUserSigner({
                                    accessToken: t,
                                    message: e
                                })
                            },
                            account: n,
                            rpcRequest: {
                                chainType: "ethereum",
                                request: e
                            }
                        });
                        return r
                    }
                    try {
                        await this._walletProxy.connect({
                            entropyId: this._entropyId,
                            entropyIdVerifier: this._entropyIdVerifier,
                            accessToken: t
                        })
                    } catch (n) {
                        let e = (0,
                        m.pO)(n);
                        if (e && "privy" === this._account.recovery_method)
                            await this._walletProxy.recover({
                                entropyId: this._entropyId,
                                entropyIdVerifier: this._entropyIdVerifier,
                                accessToken: t
                            });
                        else {
                            if (!e || !this._onNeedsRecovery)
                                throw n;
                            {
                                let e;
                                await new Promise(async (t, n) => {
                                    e = setTimeout( () => n(new p.wE({
                                        code: "embedded_wallet_recovery_error",
                                        error: "User-owned recovery timed out"
                                    })), 12e4),
                                    await this._onNeedsRecovery?.({
                                        recoveryMethod: this._account.recovery_method,
                                        onRecovered: () => t(!0)
                                    })
                                }
                                ).finally( () => clearTimeout(e))
                            }
                        }
                    }
                    return (await this._walletProxy.rpcWallet({
                        accessToken: t,
                        request: e,
                        entropyId: this._entropyId,
                        entropyIdVerifier: this._entropyIdVerifier,
                        hdWalletIndex: this._account.wallet_index,
                        chainType: "ethereum"
                    })).response.data
                } catch (n) {
                    console.error(n);
                    let t = n instanceof Error ? n.message : "Unable to make wallet request";
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_failed", {
                        method: e.method,
                        address: this._account.address,
                        error: t
                    }),
                    new p.wE({
                        code: "embedded_wallet_request_error",
                        error: t
                    })
                }
            }
            async handleJsonRpc(e) {
                return this._client.request(e)
            }
            toJSON() {
                return `PrivyEIP1193Provider { address: '${this._account.address}', chainId: ${this._chainId}, request: [Function] }`
            }
            constructor({walletProxy: e, privyInternal: t, account: n, entropyId: r, entropyIdVerifier: i, chains: s, onNeedsRecovery: a, chainId: o=s[0].id, appApi: l}) {
                super(),
                this._walletProxy = e,
                this._privyInternal = t,
                this._account = n,
                this._entropyId = r,
                this._entropyIdVerifier = i,
                this._chainId = o,
                this._chains = s,
                this._onNeedsRecovery = a,
                this._client = (0,
                _.Bm)(o, s, {
                    rpcUrls: []
                }, {
                    appId: l.appId
                }),
                this._appApi = l
            }
        }
        function q(e) {
            return "version"in e
        }
        function F(e, t) {
            let n = (q(e) ? e.message : e.compileMessage()).staticAccountKeys.find(e => e.toBase58() === t);
            if (!n)
                throw Error(`Transaction does not contain public key ${t}`);
            return n
        }
        var B = n(91015).Buffer;
        class H {
            async request(e) {
                if (!await this._privyInternal.getAccessToken())
                    throw new p.wE({
                        error: "Missing access token",
                        code: "attempted_rpc_call_before_logged_in"
                    });
                switch (e.method) {
                case "signAndSendTransaction":
                    return await this.handleSignAndSendTransaction(e);
                case "signTransaction":
                    return await this.handleSignTransaction(e);
                default:
                    return await this.handleIFrameRpc(e)
                }
            }
            get _publicKey() {
                return this._account.address
            }
            async connectAndRecover(e) {
                if ("privy-v2" !== this._account.recovery_method)
                    try {
                        await this._proxy.connect({
                            entropyId: this._entropyId,
                            entropyIdVerifier: this._entropyIdVerifier,
                            accessToken: e
                        })
                    } catch (n) {
                        let t = (0,
                        m.pO)(n);
                        if (t && "privy" === this._account.recovery_method)
                            await this._proxy.recover({
                                entropyId: this._entropyId,
                                entropyIdVerifier: this._entropyIdVerifier,
                                accessToken: e
                            });
                        else {
                            if (!t || !this._onNeedsRecovery)
                                throw n;
                            {
                                let e;
                                await new Promise(async (t, n) => {
                                    e = setTimeout( () => n(new p.wE({
                                        code: "embedded_wallet_recovery_error",
                                        error: "User-owned recovery timed out"
                                    })), 12e4),
                                    await this._onNeedsRecovery?.({
                                        recoveryMethod: this._account.recovery_method,
                                        onRecovered: () => t(!0)
                                    })
                                }
                                ).finally( () => clearTimeout(e))
                            }
                        }
                    }
            }
            async signMessageRpc(e, t) {
                let n = this._account;
                if (!f(n))
                    return (await this._proxy.rpcWallet({
                        accessToken: t,
                        request: e,
                        chainType: "solana",
                        hdWalletIndex: this._account.wallet_index,
                        entropyId: this._entropyId,
                        entropyIdVerifier: this._entropyIdVerifier
                    })).response.data;
                {
                    let {data: r} = await W({
                        context: {
                            app: this._app,
                            fetchPrivyRoute: (...e) => this._privyInternal.fetch(...e),
                            getCompiledPath: (...e) => this._privyInternal.getPath(...e),
                            signRequest: ({message: e}) => this._proxy.signWithUserSigner({
                                accessToken: t,
                                message: e
                            })
                        },
                        account: n,
                        rpcRequest: {
                            chainType: "solana",
                            request: e
                        }
                    });
                    return {
                        signature: r
                    }
                }
            }
            async handleIFrameRpc(e) {
                try {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw Error("Missing privy token. User must be logged in");
                    return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_started", {
                        method: e.method,
                        address: this._account.address
                    }),
                    await this.connectAndRecover(t),
                    await this.signMessageRpc(e, t)
                } catch (n) {
                    console.error(n);
                    let t = n instanceof Error ? n.message : "Unable to make wallet request";
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_failed", {
                        method: e.method,
                        address: this._account.address,
                        error: t
                    }),
                    new p.wE({
                        code: "embedded_wallet_request_error",
                        error: t
                    })
                }
            }
            async handleSignAndSendTransaction(e) {
                try {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw Error("Missing privy token. User must be logged in");
                    this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_started", {
                        method: e.method,
                        address: this._account.address
                    }),
                    await this.connectAndRecover(t);
                    let {transaction: n, connection: r, options: i} = e.params
                      , s = F(n, this._account.address)
                      , a = q(n) ? B.from(n.message.serialize()) : n.serializeMessage()
                      , {signature: o} = await this.signMessageRpc({
                        method: "signMessage",
                        params: {
                            message: a.toString("base64")
                        }
                    }, t);
                    return n.addSignature(s, B.from(o, "base64")),
                    {
                        signature: await r.sendRawTransaction(n.serialize(), i)
                    }
                } catch (n) {
                    console.error(n);
                    let t = n instanceof Error ? n.message : "Unable to make wallet request";
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_failed", {
                        method: e.method,
                        address: this._account.address,
                        error: t
                    }),
                    new p.wE({
                        code: "embedded_wallet_request_error",
                        error: t
                    })
                }
            }
            async handleSignTransaction(e) {
                try {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw Error("Missing privy token. User must be logged in");
                    this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_started", {
                        method: e.method,
                        address: this._account.address
                    }),
                    await this.connectAndRecover(t);
                    let {transaction: n} = e.params
                      , r = F(n, this._account.address)
                      , i = q(n) ? B.from(n.message.serialize()) : n.serializeMessage()
                      , {signature: s} = await this.signMessageRpc({
                        method: "signMessage",
                        params: {
                            message: i.toString("base64")
                        }
                    }, t);
                    return n.addSignature(r, B.from(s, "base64")),
                    {
                        signedTransaction: n
                    }
                } catch (n) {
                    console.error(n);
                    let t = n instanceof Error ? n.message : "Unable to make wallet request";
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_rpc_failed", {
                        method: e.method,
                        address: this._account.wallet_index,
                        error: t
                    }),
                    new p.wE({
                        code: "embedded_wallet_request_error",
                        error: t
                    })
                }
            }
            toJSON() {
                return `PrivyEmbeddedSolanaProvider { address: '${this._account.address}', request: [Function] }`
            }
            constructor({proxy: e, privyInternal: t, account: n, entropyId: r, entropyIdVerifier: i, onNeedsRecovery: s, app: a}) {
                this._proxy = e,
                this._privyInternal = t,
                this._account = n,
                this._entropyId = r,
                this._entropyIdVerifier = i,
                this._onNeedsRecovery = s,
                this._app = a
            }
        }
        class K {
            setMessagePoster(e) {
                this._proxy = new A(e,this._mfaPromises),
                this._mfa.setProxy(this._proxy)
            }
            async signWithUserSigner(e) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                let t = await this._privyInternal.getAccessToken();
                if (!t)
                    throw new p.wE({
                        error: "User must be logged in to sign a message with the user signer",
                        code: "user_signer_sign_error"
                    });
                let {signature: n} = await this._proxy.signWithUserSigner({
                    accessToken: t,
                    message: e.message
                });
                return {
                    signature: n
                }
            }
            async add(e) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                if ("user-controlled-server-wallets-only" === this._privyInternal.config?.embedded_wallet_config.mode)
                    await w({
                        context: {
                            app: this._appApi,
                            fetchPrivyRoute: (...e) => this._privyInternal.fetch(...e),
                            getCompiledPath: (...e) => this._privyInternal.getPath(...e)
                        },
                        chainType: e.chainType
                    });
                else {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw new p.wE({
                            error: "User must be logged in to create an embedded wallet",
                            code: "embedded_wallet_creation_error"
                        });
                    await this._proxy.addWallet({
                        accessToken: t,
                        ...e
                    })
                }
                let {user: t} = await this._privyInternal.refreshSession();
                return {
                    user: t
                }
            }
            async getBitcoinProvider({wallet: e, entropyId: t, entropyIdVerifier: n, recoveryPassword: r, recoveryAccessToken: i, recoverySecretOverride: s}) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                if (!await this._privyInternal.getAccessToken())
                    throw Error("User must be logged in to create an embedded wallet");
                return f(e) || await this._load({
                    entropyId: t,
                    entropyIdVerifier: n,
                    wallet: e,
                    recoveryPassword: r,
                    recoveryAccessToken: i,
                    recoverySecretOverride: s
                }),
                new y({
                    account: e,
                    privyInternal: this._privyInternal,
                    proxy: this._proxy,
                    entropyId: t,
                    entropyIdVerifier: n
                })
            }
            async create({password: e, recoveryMethod: t, recoveryToken: n, recoveryKey: r, recoverySecretOverride: i, iCloudRecordNameOverride: s, solanaAccount: a, skipCallbacks: o}) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                if ("user-controlled-server-wallets-only" === this._privyInternal.config?.embedded_wallet_config.mode) {
                    if (t && !t.startsWith("privy"))
                        throw new p.wE({
                            error: "User-controlled server wallets do not support custom recovery methods",
                            code: "embedded_wallet_creation_error"
                        });
                    await w({
                        context: {
                            app: this._appApi,
                            fetchPrivyRoute: (...e) => this._privyInternal.fetch(...e),
                            getCompiledPath: (...e) => this._privyInternal.getPath(...e)
                        },
                        chainType: "ethereum"
                    })
                } else {
                    let o;
                    if (o = t || (e ? "user-passcode" : "privy"),
                    e && "string" != typeof e)
                        throw Error("Invalid recovery password, must be a string");
                    if ("privy" === o && this._privyInternal.config?.embedded_wallet_config.require_user_password_on_create)
                        throw Error("Password not provided yet is required by App configuration");
                    let l = await this._privyInternal.getAccessToken();
                    if (!l)
                        throw Error("User must be logged in to create an embedded wallet");
                    let {address: c} = await this._proxy.create({
                        accessToken: l,
                        recoveryMethod: o,
                        recoveryKey: r,
                        recoveryPassword: e,
                        recoveryAccessToken: n,
                        recoverySecretOverride: i,
                        iCloudRecordNameOverride: s,
                        solanaAddress: a?.address
                    });
                    if (!c)
                        throw Error("Failed to create wallet")
                }
                return await this._privyInternal.refreshSession(o)
            }
            async createSolana(e) {
                if (!this._proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_creation_error"
                    });
                if ("user-controlled-server-wallets-only" === this._privyInternal.config?.embedded_wallet_config.mode)
                    await w({
                        context: {
                            app: this._appApi,
                            fetchPrivyRoute: (...e) => this._privyInternal.fetch(...e),
                            getCompiledPath: (...e) => this._privyInternal.getPath(...e)
                        },
                        chainType: "solana"
                    });
                else {
                    let t = await this._privyInternal.getAccessToken();
                    if (!t)
                        throw new p.wE({
                            error: "User must be logged in to create an embedded wallet",
                            code: "embedded_wallet_creation_error"
                        });
                    e?.ethereumAccount && await this.getProvider(e.ethereumAccount);
                    let {publicKey: n} = await this._proxy.createSolana({
                        accessToken: t,
                        ethereumAddress: e?.ethereumAccount?.address
                    });
                    if (!n)
                        throw new p.wE({
                            error: "Failed to create wallet",
                            code: "embedded_wallet_creation_error"
                        })
                }
                return await this._privyInternal.refreshSession()
            }
            async delegateWallets({delegatedWallets: e, rootWallet: t}) {
                if (!this._proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_creation_error"
                    });
                let n = await this._privyInternal.getAccessToken();
                if (!n)
                    throw new p.wE({
                        error: "User must be logged in to create an embedded wallet",
                        code: "embedded_wallet_creation_error"
                    });
                await this._proxy.delegateWallets({
                    accessToken: n,
                    delegatedWallets: e,
                    rootWallet: t
                })
            }
            async getProvider(e, t, n, r, i) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                return f(e) || await this._load({
                    wallet: e,
                    entropyId: e.address,
                    entropyIdVerifier: "ethereum-address-verifier",
                    recoveryPassword: t,
                    recoveryKey: i,
                    recoveryAccessToken: n,
                    recoverySecretOverride: r
                }),
                new z({
                    account: e,
                    entropyId: e.address,
                    entropyIdVerifier: "ethereum-address-verifier",
                    privyInternal: this._privyInternal,
                    chains: this._chains,
                    walletProxy: this._proxy,
                    appApi: this._appApi
                })
            }
            async getEthereumProvider({wallet: e, entropyId: t, entropyIdVerifier: n, recoveryPassword: r, recoveryAccessToken: i, recoverySecretOverride: s, recoveryKey: a, onNeedsRecovery: o}) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                if (!await this._privyInternal.getAccessToken())
                    throw Error("User must be logged in to create an embedded wallet");
                return f(e) || (!o || r || i || s || a) && await this._load({
                    entropyId: t,
                    entropyIdVerifier: n,
                    wallet: e,
                    recoveryPassword: r,
                    recoveryAccessToken: i,
                    recoverySecretOverride: s,
                    recoveryKey: a
                }),
                new z({
                    account: e,
                    entropyId: t,
                    entropyIdVerifier: "ethereum-address-verifier",
                    privyInternal: this._privyInternal,
                    chains: this._chains,
                    walletProxy: this._proxy,
                    onNeedsRecovery: o,
                    appApi: this._appApi
                })
            }
            async getSolanaProvider(e, t, n, r, i, s, a) {
                if (!this._proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                return f(e) || (!a || r || i || s) && await this._load({
                    wallet: e,
                    entropyId: t,
                    entropyIdVerifier: n,
                    recoveryPassword: r,
                    recoveryAccessToken: i,
                    recoverySecretOverride: s
                }),
                new H({
                    account: e,
                    privyInternal: this._privyInternal,
                    proxy: this._proxy,
                    entropyId: t,
                    entropyIdVerifier: n,
                    onNeedsRecovery: a,
                    app: this._appApi
                })
            }
            async setRecovery(e) {
                let {wallet: t, ...n} = e;
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                if (f(t))
                    throw new p.wE({
                        error: "This wallet does not support setting recovery methods",
                        code: "unsupported_recovery_method"
                    });
                (0,
                _.Di)({
                    currentRecoveryMethod: t.recovery_method,
                    upgradeToRecoveryMethod: "icloud-native" === n.recoveryMethod ? "icloud" : n.recoveryMethod
                }),
                await this._load("solana" === t.chain_type ? {
                    wallet: t,
                    entropyId: t.address,
                    entropyIdVerifier: "solana-address-verifier"
                } : {
                    wallet: t,
                    entropyId: t.address,
                    entropyIdVerifier: "ethereum-address-verifier"
                });
                let r = await this._privyInternal.getAccessToken();
                if (!r)
                    throw Error("User must be logged in to interact with embedded wallets");
                let i = t.recovery_method;
                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_set_recovery_started", {
                    address: t.address,
                    target_recovery_method: n.recoveryMethod,
                    existing_recovery_method: i
                });
                try {
                    let e;
                    if ("user-passcode" === n.recoveryMethod)
                        e = {
                            recoveryMethod: "user-passcode",
                            recoveryPassword: n.password
                        };
                    else if ("google-drive" === n.recoveryMethod)
                        e = {
                            recoveryMethod: "google-drive",
                            recoveryAccessToken: n.recoveryAccessToken
                        };
                    else if ("icloud" === n.recoveryMethod)
                        e = {
                            recoveryMethod: "icloud",
                            recoveryAccessToken: n.recoveryAccessToken
                        };
                    else if ("icloud-native" === n.recoveryMethod)
                        e = {
                            recoveryMethod: "icloud-native",
                            iCloudRecordNameOverride: n.iCloudRecordNameOverride,
                            recoverySecretOverride: n.recoverySecretOverride
                        };
                    else if ("recovery-encryption-key" === n.recoveryMethod)
                        e = {
                            recoveryMethod: "recovery-encryption-key",
                            recoveryKey: n.recoveryKey
                        };
                    else {
                        if ("privy" !== n.recoveryMethod)
                            throw Error(`Unknown recovery method: ${n.recoveryMethod}`);
                        e = {
                            recoveryMethod: "privy"
                        }
                    }
                    await this._proxy.setRecovery({
                        accessToken: r,
                        entropyId: t.address,
                        entropyIdVerifier: "solana" === t.chain_type ? "solana-address-verifier" : "ethereum-address-verifier",
                        ...e
                    }),
                    this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_set_recovery_completed", {
                        address: t.address,
                        target_recovery_method: n.recoveryMethod,
                        existing_recovery_method: i
                    });
                    let {user: s} = await this._privyInternal.refreshSession();
                    return {
                        user: s,
                        provider: "ethereum" !== t.chain_type ? null : new z({
                            account: t,
                            entropyId: t.address,
                            entropyIdVerifier: "ethereum-address-verifier",
                            privyInternal: this._privyInternal,
                            chains: this._chains,
                            walletProxy: this._proxy,
                            appApi: this._appApi
                        })
                    }
                } catch (e) {
                    throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_set_recovery_failed", {
                        address: t.address,
                        recovery_method: t.recovery_method,
                        error: e instanceof Error ? e.message : "Unable to recover wallet"
                    }),
                    e
                }
            }
            getURL() {
                let e = new URL(`${this._privyInternal.baseUrl}/apps/${this._privyInternal.appId}/embedded-wallets`);
                return this._privyInternal.caid && e.searchParams.append("caid", this._privyInternal.caid),
                this._privyInternal.appClientId && e.searchParams.append("client_id", this._privyInternal.appClientId),
                e.href
            }
            get chains() {
                return this._chains
            }
            onMessage(e) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                return this._proxy.handleEmbeddedWalletMessages(e)
            }
            reload() {
                this._proxy ? this._proxy.reload() : console.warn("Attempted to reload proxy before initialized")
            }
            async ping(e) {
                try {
                    if (!this._proxy)
                        throw Error("Embedded wallet proxy not initialized");
                    return await this._proxy.ping(e),
                    !0
                } catch (e) {
                    return console.error(e),
                    !1
                }
            }
            async _load({entropyId: e, entropyIdVerifier: t, wallet: n, recoveryPassword: r, recoveryKey: i, recoveryAccessToken: s, recoverySecretOverride: a}) {
                if (!this._proxy)
                    throw Error("Embedded wallet proxy not initialized");
                let o = await this._privyInternal.getAccessToken();
                if (!o)
                    throw Error("User must be logged in to interact with embedded wallets");
                try {
                    return await this._proxy.connect({
                        accessToken: o,
                        entropyId: e,
                        entropyIdVerifier: t
                    }),
                    e
                } catch (l) {
                    if ((0,
                    m.pO)(l))
                        try {
                            if ("privy" === n.recovery_method) {
                                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_started", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                });
                                let r = await this._proxy.recover({
                                    accessToken: o,
                                    entropyId: e,
                                    entropyIdVerifier: t
                                });
                                return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_completed", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                }),
                                r.entropyId
                            }
                            if ("user-passcode" === n.recovery_method && r) {
                                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_started", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                });
                                let i = await this._proxy.recover({
                                    accessToken: o,
                                    recoveryPassword: r,
                                    entropyId: e,
                                    entropyIdVerifier: t
                                });
                                return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_completed", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                }),
                                i.entropyId
                            }
                            if (["google-drive", "icloud"].includes(n.recovery_method) && s) {
                                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_started", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                });
                                let r = await this._proxy.recover({
                                    accessToken: o,
                                    recoveryAccessToken: s,
                                    entropyId: e,
                                    entropyIdVerifier: t
                                });
                                return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_completed", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                }),
                                r.entropyId
                            }
                            if ("icloud" === n.recovery_method && a) {
                                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_started", {
                                    address: n.address,
                                    recovery_method: "icloud-native"
                                });
                                let r = await this._proxy.recover({
                                    accessToken: o,
                                    recoverySecretOverride: a,
                                    entropyId: e,
                                    entropyIdVerifier: t
                                });
                                return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_completed", {
                                    address: n.address,
                                    recovery_method: "icloud-native"
                                }),
                                r.entropyId
                            }
                            if ("recovery-encryption-key" === n.recovery_method && i) {
                                this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_started", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                });
                                let r = await this._proxy.recover({
                                    accessToken: o,
                                    recoveryKey: i,
                                    entropyId: e,
                                    entropyIdVerifier: t
                                });
                                return this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_completed", {
                                    address: n.address,
                                    recovery_method: n.recovery_method
                                }),
                                r.entropyId
                            }
                        } catch (t) {
                            throw this._privyInternal.createAnalyticsEvent("embedded_wallet_sdk_recovery_failed", {
                                address: n.address,
                                recovery_method: n.recovery_method,
                                error: t instanceof Error ? t.message : `Unable to recover wallet: ${e}`
                            }),
                            t
                        }
                    throw l
                }
            }
            constructor(e, t, n, r, i, s) {
                if (this._chains = Array.from(u.m),
                this._privyInternal = e,
                t && (this._proxy = new A(t,i),
                r.setProxy(this._proxy)),
                n) {
                    let e = (0,
                    h._)(n);
                    this._chains = e
                }
                this._mfa = r,
                this._mfaPromises = i,
                this._appApi = s
            }
        }
        class Q extends T.A {
            constructor() {
                super(),
                this.rootPromise = {
                    current: null
                },
                this.submitPromise = {
                    current: null
                }
            }
        }
        var V, Y, G = n(74192), Z = n(52002), $ = ((Y = {}).OAUTH_ACCOUNT_SUSPENDED = "oauth_account_suspended",
        Y.MISSING_OR_INVALID_PRIVY_APP_ID = "missing_or_invalid_privy_app_id",
        Y.MISSING_OR_INVALID_PRIVY_CLIENT_ID = "missing_or_invalid_privy_client_id",
        Y.MISSING_OR_INVALID_PRIVY_ACCOUNT_ID = "missing_or_invalid_privy_account_id",
        Y.MISSING_OR_INVALID_TOKEN = "missing_or_invalid_token",
        Y.MISSING_MFA_ENROLLMENT = "missing_mfa_enrollment",
        Y.MISSING_OR_INVALID_MFA = "missing_or_invalid_mfa",
        Y.EXPIRED_OR_INVALID_MFA_TOKEN = "expired_or_invalid_mfa_token",
        Y.INVALID_DATA = "invalid_data",
        Y.INVALID_CREDENTIALS = "invalid_credentials",
        Y.INVALID_CAPTCHA = "invalid_captcha",
        Y.LINKED_TO_ANOTHER_USER = "linked_to_another_user",
        Y.ALLOWLIST_REJECTED = "allowlist_rejected",
        Y.CANNOT_UNLINK_EMBEDDED_WALLET = "cannot_unlink_embedded_wallet",
        Y.CANNOT_UNLINK_SOLE_ACCOUNT = "cannot_unlink_sole_account",
        Y.CANNOT_LINK_MORE_OF_TYPE = "cannot_link_more_of_type",
        Y.LINKED_ACCOUNT_NOT_FOUND = "linked_account_not_found",
        Y.TOO_MANY_REQUESTS = "too_many_requests",
        Y.RESOURCE_CONFLICT = "resource_conflict",
        Y.INVALID_ORIGIN = "invalid_origin",
        Y.MISSING_ORIGIN = "missing_origin",
        Y.INVALID_NATIVE_APP_ID = "invalid_native_app_id",
        Y.TOKEN_ALREADY_USED = "token_already_used",
        Y.ALREADY_LOGGED_OUT = "already_logged_out",
        Y.NOT_SUPPORTED = "not_supported",
        Y.USER_UNSUBSCRIBED = "user_unsubscribed",
        Y.MAX_APPS_REACHED = "max_apps_reached",
        Y.USER_LIMIT_REACHED = "max_accounts_reached",
        Y.DEVICE_REVOKED = "device_revoked",
        Y.WALLET_PASSWORD_EXISTS = "wallet_password_exists",
        Y.OAUTH_STATE_MISMATCH = "oauth_state_mismatch",
        Y.MAX_DENYLIST_ENTRIES_REACHED = "max_denylist_entries_reached",
        Y.MAX_TEST_ACCOUNTS_REACHED = "max_test_accounts_reached",
        Y.DISALLOWED_LOGIN_METHOD = "disallowed_login_method",
        Y.DISALLOWED_PLUS_EMAIL = "disallowed_plus_email",
        Y.DISALLOWED_RECOVERY_METHOD = "disallowed_recovery_method",
        Y.LEGACY_DASHBOARD_LOGIN_CONFIGURATION = "legacy_dashboard_login_configuration",
        Y.CANNOT_SET_PASSWORD = "cannot_set_password",
        Y.INVALID_PKCE_PARAMETERS = "invalid_pkce_parameters",
        Y.INVALID_APP_URL_SCHEME_CONFIGURATION = "invalid_app_url_scheme_configuration",
        Y.CROSS_APP_CONNECTION_NOT_ALLOWED = "cross_app_connection_not_allowed",
        Y.USER_DOES_NOT_EXIST = "user_does_not_exist",
        Y.ALREADY_EXISTS = "resource_already_exists",
        Y.ACCOUNT_TRANSFER_REQUIRED = "account_transfer_required",
        Y.USER_HAS_NOT_DELEGATED_WALLET = "user_has_not_delegated_wallet",
        Y.FEATURE_NOT_ENABLED = "feature_not_enabled",
        Y.INSUFFICIENT_FUNDS = "insufficient_funds",
        Y.TRANSACTION_BROADCAST_FAILURE = "transaction_broadcast_failure",
        Y.INVALID_POLICY_FORMAT = "invalid_policy_format",
        Y.POLICY_VIOLATION = "policy_violation",
        Y.AUTHORIZATION_KEY_HAS_ASSOCIATED_WALLETS = "authorization_key_has_associated_wallets",
        Y.INVALID_REQUEST = "invalid_request",
        Y.SIGNUP_DISABLED = "signup_disabled",
        Y);
        let J = (e, t) => t ? Object.entries(t).reduce( (e, [t,n]) => e.replace(`:${t}`, `${n}`), e) : e
          , X = {
            path: "/api/v1/apps/:app_id",
            method: "GET"
        }
          , ee = {
            path: "/api/v1/analytics_events",
            method: "POST"
        }
          , et = {
            path: "/api/v1/sessions",
            method: "POST"
        }
          , en = {
            path: "/api/v1/sessions/logout",
            method: "POST"
        };
        var er = n(74235)
          , ei = n(33635);
        let es = Promise.allSettled.bind(Promise) ?? (e => Promise.all(e.map(e => e.then(e => ({
            status: "fulfilled",
            value: e
        })).catch(e => ({
            status: "rejected",
            reason: e
        })))))
          , ea = "privy:token"
          , eo = "privy-token"
          , el = "privy:refresh_token"
          , ec = "privy-refresh-token"
          , ed = "privy:id-token"
          , eu = "privy-id-token"
          , eh = "privy-session";
        class ep extends T.A {
            set isUsingServerCookies(e) {
                this._isUsingServerCookies = e
            }
            async getToken() {
                let e = await this._storage.get(ea);
                try {
                    return "string" == typeof e ? new o(e).value : null
                } catch (e) {
                    return console.error(e),
                    await this.destroyLocalState({
                        reason: "getToken_error"
                    }),
                    null
                }
            }
            async getRefreshToken() {
                let e = await this._storage.get(el);
                return "string" == typeof e ? e : null
            }
            async getIdentityToken() {
                let e = await this._storage.get(ed);
                return "string" == typeof e ? e : null
            }
            get mightHaveServerCookies() {
                try {
                    let e = ei.A.get(eh);
                    return void 0 !== e && e.length > 0
                } catch (e) {
                    console.error(e)
                }
                return !1
            }
            hasRefreshCredentials(e, t) {
                return this.mightHaveServerCookies || "string" == typeof e && "string" == typeof t
            }
            tokenIsActive(e) {
                if (!e)
                    return !1;
                let t = o.parse(e);
                return null !== t && !t.isExpired(30)
            }
            async destroyLocalState(e) {
                await es([this._storage.del(ea), this._storage.del(el), this._storage.del(ed), this._storage.del(this.GUEST_CREDENTIAL_STORAGE_KEY)]),
                ei.A.remove(eo),
                ei.A.remove(ec),
                ei.A.remove(eu),
                ei.A.remove(eh),
                e?.reason && this.emit("storage_cleared", {
                    reason: e.reason
                })
            }
            async storeToken(e) {
                if ("string" == typeof e) {
                    let t = await this._storage.get(ea);
                    if (await this._storage.put(ea, e),
                    !this._isUsingServerCookies) {
                        let t = o.parse(e)?.expiration;
                        ei.A.set(eo, e, {
                            sameSite: "Strict",
                            secure: !0,
                            expires: t ? new Date(1e3 * t) : void 0
                        })
                    }
                    t !== e && this.emit("token_stored", {
                        cookiesEnabled: this._isUsingServerCookies
                    })
                } else {
                    let e = await this._storage.get(ea);
                    await this._storage.del(ea),
                    ei.A.remove(eo),
                    null !== e && this.emit("token_cleared", {
                        reason: "set_with_non_string_value"
                    })
                }
            }
            async storeRefreshToken(e) {
                "string" == typeof e ? (await this._storage.put(el, e),
                this._isUsingServerCookies || (ei.A.set(eh, "t", {
                    sameSite: "Strict",
                    secure: !0,
                    expires: 30
                }),
                ei.A.set(ec, e, {
                    sameSite: "Strict",
                    secure: !0,
                    expires: 30
                })),
                this.emit("refresh_token_stored", {
                    cookiesEnabled: this._isUsingServerCookies
                })) : (await this._storage.del(el),
                ei.A.remove(ec),
                ei.A.remove(eh),
                this.emit("refresh_token_cleared", {
                    reason: "set_with_non_string_value"
                }))
            }
            async updateWithTokensResponse(e) {
                let t = (await es([this.storeToken(e.token), this.storeRefreshToken(e.refresh_token), this.storeIdentityToken(e.identity_token), this.processOAuthTokens(e.oauth_tokens)])).filter(e => "rejected" === e.status);
                t.length > 0 && this.emit("error_storing_tokens", t.map(e => String(e.reason)).join(", "))
            }
            async processOAuthTokens(e) {
                e && this.emit("oauth_tokens_granted", e)
            }
            async storeIdentityToken(e) {
                if ("string" == typeof e) {
                    let t = await this._storage.get(ed);
                    if (await this._storage.put(ed, e),
                    !this._isUsingServerCookies) {
                        let t = o.parse(e)?.expiration;
                        ei.A.set(eu, e, {
                            sameSite: "Strict",
                            secure: !0,
                            expires: t ? new Date(1e3 * t) : void 0
                        })
                    }
                    t !== e && this.emit("identity_token_stored", {
                        cookiesEnabled: this._isUsingServerCookies
                    })
                } else {
                    let e = await this._storage.get(ed);
                    await this._storage.del(ed),
                    ei.A.remove(eu),
                    null !== e && this.emit("identity_token_cleared", {
                        reason: "set_with_non_string_value"
                    })
                }
            }
            async getOrCreateGuestCredential() {
                let e = this._storage.get(this.GUEST_CREDENTIAL_STORAGE_KEY);
                if (e && "string" == typeof e)
                    return e;
                let t = er.l(crypto.getRandomValues(new Uint8Array(32)));
                return await this._storage.put(this.GUEST_CREDENTIAL_STORAGE_KEY, t),
                t
            }
            constructor(e) {
                super(),
                this._isUsingServerCookies = !1,
                this._storage = e.storage,
                this.GUEST_CREDENTIAL_STORAGE_KEY = `privy:guest:${e.appId}`
            }
        }
        ep.events = ["storage_cleared", "token_cleared", "refresh_token_cleared", "identity_token_cleared", "token_stored", "refresh_token_stored", "identity_token_stored", "oauth_tokens_granted", "error_storing_tokens"];
        var ef = e => {
            let t = new AbortController;
            return setTimeout( () => t.abort(), e),
            t.signal
        }
        ;
        let ey = () => {}
          , em = {
            NONE: Number.NEGATIVE_INFINITY,
            ERROR: 1,
            WARN: 2,
            INFO: 3,
            DEBUG: Number.POSITIVE_INFINITY
        }
          , eg = ({level: e}={
            level: "ERROR"
        }) => ({
            get level() {
                return e
            },
            error: em[e] >= em.ERROR ? console.error : ey,
            warn: em[e] >= em.WARN ? console.warn : ey,
            info: em[e] >= em.INFO ? console.info : ey,
            debug: em[e] >= em.DEBUG ? console.debug : ey
        })
          , ew = "privy:caid";
        class e_ {
            setCallbacks(e) {
                this.callbacks = {
                    ...this.callbacks,
                    ...e
                }
            }
            get isReady() {
                return !!this._config
            }
            get config() {
                return this._config
            }
            get caid() {
                return this._analyticsId
            }
            async _initialize() {
                if (this.isReady)
                    this.callbacks?.setIsReady?.(!0);
                else {
                    if (!await this.isStorageAccessible())
                        throw new p.wE({
                            code: "storage_error",
                            error: "Unable to access storage"
                        });
                    this._config = await this.getAppConfig(),
                    this._config?.custom_api_url && (this.baseUrl = this._config.custom_api_url,
                    this.session.isUsingServerCookies = !0),
                    this.callbacks?.setIsReady?.(!0),
                    this._sdkVersion.startsWith("react-auth:") || this.createAnalyticsEvent("sdk_initialize", {})
                }
            }
            getPath(e, {params: t, query: n}) {
                return `${this.baseUrl}${J(e.path, t)}${function(e) {
                    let t = new URLSearchParams;
                    for (let n in e)
                        null != e[n] && t.append(n, String(e[n]));
                    return Array.from(t).length ? "?" + t.toString() : ""
                }(n)}`
            }
            async fetch(e, {body: t, params: n, query: r, headers: i, onRequest: s=this._beforeRequest.bind(this)}) {
                let a = new Request(this.getPath(e, {
                    params: n,
                    query: r
                }),{
                    method: e.method,
                    body: JSON.stringify(t),
                    headers: i
                })
                  , o = await s(a)
                  , l = await this._fetch(a, o)
                  , c = await l.json();
                if (l.status > 299)
                    throw new p.VQ(c);
                return c
            }
            async _beforeRequestWithoutInitialize(e) {
                let t = await this.session.getToken()
                  , n = new Headers(e.headers);
                n.set("privy-app-id", this.appId),
                this.appClientId && n.set("privy-client-id", this.appClientId),
                n.set("privy-client", this._sdkVersion),
                t && n.set("Authorization", `Bearer ${t}`),
                n.set("Content-Type", "application/json"),
                n.set("Accept", "application/json");
                let r = await this._getOrGenerateClientAnalyticsId();
                return r && n.set("privy-ca-id", r),
                this.nativeAppIdentifier && n.set("x-native-app-identifier", this.nativeAppIdentifier),
                {
                    signal: ef(2e4),
                    headers: n,
                    credentials: "include"
                }
            }
            async beforeRequestWithoutRefresh(e) {
                return await this._initialize(),
                this._beforeRequestWithoutInitialize(e)
            }
            async _beforeRequest(e) {
                return await this._initialize(),
                await this.getAccessToken(),
                this.beforeRequestWithoutRefresh(e)
            }
            async getAppConfig() {
                return await this.fetch(X, {
                    params: {
                        app_id: this.appId
                    },
                    onRequest: this._beforeRequestWithoutInitialize.bind(this)
                })
            }
            async _getOrGenerateClientAnalyticsId() {
                if (this._analyticsId)
                    return this._analyticsId;
                try {
                    let e = await this._storage.get(ew);
                    if ("string" == typeof e && e.length > 0)
                        return this._analyticsId = e,
                        e
                } catch (e) {
                    this.logger.error("Unable to load clientId", e)
                }
                try {
                    this._analyticsId = (0,
                    Z.A)()
                } catch (e) {
                    this.logger.error("Unable to generate uuidv4", e)
                }
                if (!this._analyticsId)
                    return null;
                try {
                    await this._storage.put(ew, this._analyticsId)
                } catch (e) {
                    this.logger.error(`Unable to store clientId: ${this._analyticsId}`, e)
                }
                return this._analyticsId
            }
            async destroyClientAnalyticsId() {
                try {
                    return await this._storage.del(ew)
                } catch (e) {
                    this.logger.error("Unable to delete clientId", e)
                }
            }
            async createAnalyticsEvent(e, t) {
                try {
                    await this.fetch(ee, {
                        body: {
                            event_name: e,
                            client_id: await this._getOrGenerateClientAnalyticsId(),
                            payload: t
                        },
                        onRequest: this.beforeRequestWithoutRefresh.bind(this)
                    })
                } catch (e) {}
            }
            async refreshSession(e=!1) {
                if (!await this.isStorageAccessible())
                    throw new p.wE({
                        code: "storage_error",
                        error: "Unable to access storage"
                    });
                let t = await this.session.getRefreshToken() ?? void 0
                  , n = t ?? "key"
                  , r = this._cache.get(n);
                if (r)
                    return this.logger.debug("[privy:refresh] found in-flight session refresh request, deduping"),
                    await r;
                let i = this._refreshSession(t, e);
                this._cache.set(n, i);
                try {
                    return await i
                } finally {
                    this._cache.delete(n)
                }
            }
            async _refreshSession(e, t) {
                let n = await this.session.getToken();
                if (!this.session.hasRefreshCredentials(n, e ?? null))
                    throw this.logger.debug("[privy:refresh] missing tokens, skipping request"),
                    await this._initialize(),
                    new p.VQ({
                        code: $.MISSING_OR_INVALID_TOKEN,
                        error: "No tokens found in storage"
                    });
                try {
                    this.logger.debug(`[privy:refresh] fetching: ${et.path}`);
                    let n = await this.fetch(et, {
                        body: {
                            refresh_token: e
                        },
                        onRequest: this.beforeRequestWithoutRefresh.bind(this)
                    })
                      , r = n.session_update_action;
                    return this.logger.debug(`[privy:refresh] response: ${r}`),
                    t || this.callbacks?.setUser?.(n.user),
                    "set" === r && (await this.session.updateWithTokensResponse(n),
                    this.logger.debug("[privy:refresh] tokens stored")),
                    "clear" === r && (await this.session.destroyLocalState(),
                    this.logger.debug("[privy:refresh] tokens cleared"),
                    t || this.callbacks?.setUser?.(null)),
                    "ignore" === r && n.token && (await this.session.storeToken(n.token),
                    this.logger.debug("[privy:refresh] access token stored"),
                    n.identity_token && (this.logger.debug("[privy:refresh] identity token stored"),
                    await this.session.storeIdentityToken(n.identity_token))),
                    this.logger.debug("[privy:refresh] returning response"),
                    n
                } catch (e) {
                    throw this.logger.debug(`[privy:refresh] error: ${e.message ?? "unknown error"}`),
                    e instanceof p.VQ && e.code === $.MISSING_OR_INVALID_TOKEN && (await this.session.destroyLocalState(),
                    t || this.callbacks?.setUser?.(null)),
                    e
                }
            }
            async getAccessToken() {
                let[e,t] = await Promise.all([this.session.getToken(), this.session.getRefreshToken()]);
                if (!this.session.tokenIsActive(e) && this.session.hasRefreshCredentials(e, t)) {
                    let t = await this.refreshSession()
                      , n = await this.session.getToken();
                    return t.token || this.logger.debug("[privy:getAccessToken] expected token received null"),
                    t.token === e && this.logger.debug("[privy:getAccessToken] expected new token in response received existing"),
                    n === e && this.logger.debug("[privy:getAccessToken] expected new token in storage received existing"),
                    t.token ?? n
                }
                return e
            }
            async getIdentityToken() {
                return await this.session.getIdentityToken()
            }
            async isStorageAccessible() {
                try {
                    let e = `privy:__storage__test-${(0,
                    Z.A)()}`
                      , t = "blobby";
                    await this._storage.put(e, t);
                    let n = await this._storage.get(e);
                    return await this._storage.del(e),
                    n === t
                } catch (e) {
                    return this.logger.error(e),
                    !1
                }
            }
            constructor(e) {
                this._sdkVersion = "js-sdk-core:0.56.1",
                this._cache = new Map,
                this.logger = eg({
                    level: e.logLevel ?? "ERROR"
                }),
                this._storage = e.storage,
                this._analyticsId = null,
                this._getOrGenerateClientAnalyticsId(),
                this.baseUrl = e.baseUrl ?? "https://auth.privy.io",
                this.appId = e.appId,
                this.appClientId = e.appClientId,
                this._sdkVersion = e.sdkVersion ?? this._sdkVersion,
                this.callbacks = e.callbacks,
                this.nativeAppIdentifier = e.nativeAppIdentifier,
                this.session = new ep({
                    storage: this._storage,
                    isUsingServerCookies: !1,
                    appId: e.appId
                }),
                this._fetch = G(globalThis.fetch, {
                    retries: 3,
                    retryDelay: e => 3 ** e * 500,
                    retryOn: [408, 409, 425, 500, 502, 503, 504]
                }),
                this.session.on("error_storing_tokens", e => {
                    this.createAnalyticsEvent("error_updating_tokens_in_storage", {
                        reason: e
                    })
                }
                )
            }
        }
        let ev = {
            path: "/api/v1/users/me/accept_terms",
            method: "POST"
        };
        class eb {
            async get() {
                let {user: e} = await this._privyInternal.refreshSession();
                return {
                    user: e
                }
            }
            async acceptTerms() {
                return {
                    user: await this._privyInternal.fetch(ev, {})
                }
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        let eE = {
            path: "/api/v1/custom_jwt_account/authenticate",
            method: "POST"
        }
          , eI = e => "wallet" === e.type && "privy" === e.wallet_client_type && "embedded" === e.connector_type
          , ek = e => e ? e.linked_accounts.filter(eI).filter(e => "ethereum" === e.chain_type).sort( (e, t) => e.wallet_index - t.wallet_index) : []
          , eM = e => ek(e).find(e => 0 === e.wallet_index) ?? null
          , eS = e => e ? e.linked_accounts.filter(eI).filter(e => "solana" === e.chain_type).sort( (e, t) => e.wallet_index - t.wallet_index) : []
          , eA = e => eS(e).find(e => 0 === e.wallet_index) ?? null
          , eT = (e, t) => !("off" === t || ek(e).length > 0 || e.linked_accounts.filter(e => "wallet" === e.type && "ethereum" === e.chain_type).length > 0 && "all-users" !== t)
          , eC = (e, t) => !("off" === t || eS(e).length > 0 || e.linked_accounts.filter(e => "wallet" === e.type && "solana" === e.chain_type).length > 0 && "all-users" !== t)
          , eO = async (e, t, n) => {
            let r = eT(t.user, n?.ethereum?.createOnLogin ?? "off")
              , i = eC(t.user, n?.solana?.createOnLogin ?? "off");
            if (r && i) {
                let n = await e.create({
                    recoveryMethod: "privy",
                    skipCallbacks: !0
                });
                return {
                    ...await e.createSolana({
                        ethereumAccount: eM(n.user) ?? void 0
                    }),
                    is_new_user: t.is_new_user,
                    oauth_tokens: t.oauth_tokens
                }
            }
            return r ? {
                ...await e.create({
                    recoveryMethod: "privy",
                    solanaAccount: eA(t.user) ?? void 0
                }),
                is_new_user: t.is_new_user,
                oauth_tokens: t.oauth_tokens
            } : i ? {
                ...await e.createSolana({
                    ethereumAccount: eM(t.user) ?? void 0
                }),
                is_new_user: t.is_new_user,
                oauth_tokens: t.oauth_tokens
            } : t
        }
        ;
        class eN {
            async syncWithToken(e, t, n) {
                let r = await this._privyInternal.fetch(eE, {
                    body: {
                        token: e,
                        mode: n
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(r);
                let i = await eO(this._embedded, r, t?.embedded);
                return this._privyInternal.callbacks?.setUser?.(i.user),
                i
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let eL = {
            path: "/api/v1/passwordless/authenticate",
            method: "POST"
        }
          , ex = {
            path: "/api/v1/passwordless/init",
            method: "POST"
        }
          , ej = {
            path: "/api/v1/passwordless/link",
            method: "POST"
        }
          , eD = {
            path: "/api/v1/passwordless/unlink",
            method: "POST"
        }
          , eR = {
            path: "/api/v1/passwordless/update",
            method: "POST"
        };
        class eP {
            async sendCode(e, t) {
                return this._privyInternal.fetch(ex, {
                    body: {
                        email: e,
                        token: t
                    }
                })
            }
            async loginWithCode(e, t, n, r) {
                let i = await this._privyInternal.fetch(eL, {
                    body: {
                        email: e,
                        code: t,
                        mode: n
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(i);
                let s = await eO(this._embedded, i, r?.embedded);
                return this._privyInternal.callbacks?.setUser?.(s.user),
                s
            }
            async linkWithCode(e, t) {
                await this._privyInternal.fetch(ej, {
                    body: {
                        email: e,
                        code: t
                    }
                });
                let n = await this._privyInternal.refreshSession();
                return {
                    user: n.user,
                    identity_token: n.identity_token
                }
            }
            async updateEmail({oldEmailAddress: e, newEmailAddress: t, code: n}) {
                await this._privyInternal.fetch(eR, {
                    body: {
                        oldAddress: e,
                        newAddress: t,
                        code: n
                    }
                });
                let r = await this._privyInternal.refreshSession();
                return {
                    user: r.user,
                    identity_token: r.identity_token
                }
            }
            async unlink(e) {
                await this._privyInternal.fetch(eD, {
                    body: {
                        address: e
                    }
                });
                let t = await this._privyInternal.refreshSession();
                return {
                    user: t.user,
                    identity_token: t.identity_token
                }
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let eW = {
            path: "/api/v1/farcaster/init",
            method: "POST"
        }
          , eU = {
            path: "/api/v1/farcaster/authenticate",
            method: "POST"
        }
          , ez = {
            path: "/api/v1/farcaster/link",
            method: "POST"
        }
          , eq = {
            path: "/api/v1/farcaster/unlink",
            method: "POST"
        }
          , eF = {
            path: "/api/v1/farcaster/status",
            method: "GET"
        }
          , eB = {
            path: "/api/v2/farcaster/init",
            method: "POST"
        }
          , eH = {
            path: "/api/v2/farcaster/authenticate",
            method: "POST"
        };
        class eK {
            async initializeAuth({relyingParty: e, redirectUrl: t, token: n}) {
                return await this._privyInternal.fetch(eW, {
                    body: {
                        relying_party: e,
                        redirect_url: t,
                        token: n
                    }
                })
            }
            async getFarcasterStatus({channel_token: e}) {
                return await this._privyInternal.fetch(eF, {
                    headers: {
                        "farcaster-channel-token": e
                    }
                })
            }
            async authenticate({channel_token: e, message: t, signature: n, fid: r, mode: i}, s) {
                let a = await this._privyInternal.fetch(eU, {
                    body: {
                        channel_token: e,
                        message: t,
                        signature: n,
                        fid: r,
                        mode: i
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(a);
                let o = await eO(this._embedded, a, s?.embedded);
                return this._privyInternal.callbacks?.setUser?.(o.user),
                o
            }
            async link({channel_token: e, message: t, signature: n, fid: r}) {
                await this._privyInternal.fetch(ez, {
                    body: {
                        channel_token: e,
                        message: t,
                        signature: n,
                        fid: r
                    }
                });
                let i = await this._privyInternal.refreshSession();
                return {
                    user: i.user,
                    identity_token: i.identity_token
                }
            }
            async unlink({fid: e}) {
                await this._privyInternal.fetch(eq, {
                    body: {
                        fid: e
                    }
                });
                let t = await this._privyInternal.refreshSession();
                return {
                    user: t.user,
                    identity_token: t.identity_token
                }
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        class eQ {
            async initializeAuth() {
                return await this._privyInternal.fetch(eB, {
                    body: {}
                })
            }
            async authenticate({message: e, signature: t, fid: n}, r) {
                let i = await this._privyInternal.fetch(eH, {
                    body: {
                        message: e,
                        signature: t,
                        fid: n
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(i);
                let s = await eO(this._embedded, i, r?.embedded);
                return this._privyInternal.callbacks?.setUser?.(s.user),
                s
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let eV = {
            path: "/api/v1/guest/authenticate",
            method: "POST"
        };
        class eY {
            async create(e) {
                let t = await this._privyInternal.session.getOrCreateGuestCredential()
                  , n = await this._privyInternal.fetch(eV, {
                    body: {
                        guest_credential: t
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(n);
                let r = await eO(this._embedded, n, e?.embedded);
                return this._privyInternal.callbacks?.setUser?.(r.user),
                r
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let eG = {
            path: "/api/v1/oauth/authenticate",
            method: "POST"
        }
          , eZ = {
            path: "/api/v1/oauth/init",
            method: "POST"
        }
          , e$ = {
            path: "/api/v1/oauth/link",
            method: "POST"
        }
          , eJ = {
            path: "/api/v1/oauth/unlink",
            method: "POST"
        }
          , eX = "privy:state_code"
          , e0 = "privy:code_verifier";
        async function e1(e, t) {
            let n = (new TextEncoder).encode(e);
            return new Uint8Array(await t("SHA-256", n))
        }
        function e2() {
            return er.l(crypto.getRandomValues(new Uint8Array(36)))
        }
        async function e4({codeVerifier: e, method: t="S256", digest: n=crypto.subtle.digest.bind(crypto.subtle)}) {
            if ("S256" != t)
                return e;
            {
                let t = await e1(e, n);
                return er.l(t)
            }
        }
        class e5 {
            async generateURL(e, t) {
                let n = e2()
                  , r = e2()
                  , i = await e4({
                    codeVerifier: n,
                    digest: this._crypto?.digest
                });
                return await Promise.all([this._storage.put(e0, n), this._storage.put(eX, r)]),
                this._privyInternal.fetch(eZ, {
                    body: {
                        redirect_to: t,
                        provider: e,
                        code_challenge: i,
                        state_code: r
                    }
                })
            }
            async loginWithCode(e, t, n, r, i, s) {
                let[a,o] = await Promise.all([this._storage.get(e0), this._storage.get(eX)]);
                if (o !== t)
                    throw this._privyInternal.createAnalyticsEvent("possible_phishing_attempt", {
                        flow: "oauth",
                        provider: n,
                        storedStateCode: o ?? "",
                        returnedStateCode: t ?? ""
                    }),
                    new p.wE({
                        code: "pkce_state_code_mismatch",
                        error: "Unexpected auth flow. This may be a phishing attempt."
                    });
                let l = await this._privyInternal.fetch(eG, {
                    body: {
                        authorization_code: e,
                        code_type: r,
                        state_code: o,
                        code_verifier: a,
                        mode: i
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(l);
                let c = await eO(this._embedded, l, s?.embedded);
                return await Promise.all([this._storage.del(e0), this._storage.del(eX)]),
                this._privyInternal.callbacks?.setUser?.(c.user),
                c
            }
            async linkWithCode(e, t, n, r) {
                let[i,s] = await Promise.all([this._storage.get(e0), this._storage.get(eX)]);
                if (s !== t)
                    throw this._privyInternal.createAnalyticsEvent("possible_phishing_attempt", {
                        flow: "oauth",
                        provider: n,
                        storedStateCode: s ?? "",
                        returnedStateCode: t ?? ""
                    }),
                    new p.wE({
                        code: "pkce_state_code_mismatch",
                        error: "Unexpected auth flow. This may be a phishing attempt."
                    });
                let a = await this._privyInternal.fetch(e$, {
                    body: {
                        authorization_code: e,
                        code_type: r,
                        state_code: s,
                        code_verifier: i
                    }
                });
                await this._privyInternal.session.processOAuthTokens(a.oauth_tokens);
                let o = await this._privyInternal.refreshSession();
                return await Promise.all([this._storage.del(e0), this._storage.del(eX)]),
                {
                    user: o.user,
                    identity_token: o.identity_token
                }
            }
            async unlink(e, t) {
                await this._privyInternal.fetch(eJ, {
                    body: {
                        provider: e,
                        subject: t
                    }
                });
                let n = await this._privyInternal.refreshSession();
                return {
                    user: n.user,
                    identity_token: n.identity_token
                }
            }
            constructor(e, t, n, r) {
                this._privyInternal = e,
                this._embedded = t,
                this._storage = n,
                this._crypto = r
            }
        }
        let e3 = {
            path: "/api/v1/passkeys/link",
            method: "POST"
        }
          , e6 = {
            path: "/api/v1/passkeys/authenticate",
            method: "POST"
        }
          , e7 = {
            path: "/api/v1/passkeys/register",
            method: "POST"
        }
          , e8 = {
            path: "/api/v1/passkeys/authenticate/init",
            method: "POST"
        }
          , e9 = {
            path: "/api/v1/passkeys/register/init",
            method: "POST"
        }
          , te = {
            path: "/api/v1/passkeys/link/init",
            method: "POST"
        };
        class tt {
            async generateRegistrationOptions(e) {
                return await this._privyInternal.fetch(te, {
                    body: {
                        relying_party: e
                    }
                })
            }
            async generateAuthenticationOptions(e) {
                return await this._privyInternal.fetch(e8, {
                    body: {
                        relying_party: e
                    }
                })
            }
            async generateSignupOptions(e) {
                return await this._privyInternal.fetch(e9, {
                    body: {
                        relying_party: e
                    }
                })
            }
            async loginWithPasskey(e, t, n, r) {
                let i = await this._privyInternal.fetch(e6, {
                    body: {
                        relying_party: n,
                        challenge: t,
                        authenticator_response: this._transformAuthenticationResponseToSnakeCase(e)
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(i);
                let s = await eO(this._embedded, i, r?.embedded);
                return this._privyInternal.callbacks?.setUser?.(s.user),
                s
            }
            async signupWithPasskey(e, t, n) {
                let r = await this._privyInternal.fetch(e7, {
                    body: {
                        relying_party: t,
                        authenticator_response: this._transformRegistrationResponseToSnakeCase(e)
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(r);
                let i = await eO(this._embedded, r, n?.embedded);
                return this._privyInternal.callbacks?.setUser?.(i.user),
                i
            }
            async linkWithPasskey(e, t) {
                await this._privyInternal.fetch(e3, {
                    body: {
                        relying_party: t,
                        authenticator_response: this._transformRegistrationResponseToSnakeCase(e)
                    }
                });
                let n = await this._privyInternal.refreshSession();
                return {
                    user: n.user,
                    identity_token: n.identity_token
                }
            }
            _transformRegistrationResponseToSnakeCase(e) {
                return {
                    type: e.type,
                    id: e.id,
                    raw_id: e.rawId,
                    response: {
                        client_data_json: e.response.clientDataJSON,
                        attestation_object: e.response.attestationObject,
                        authenticator_data: e.response.authenticatorData || void 0,
                        transports: e.response.transports || void 0,
                        public_key: e.response.publicKey || void 0,
                        public_key_algorithm: e.response.publicKeyAlgorithm || void 0
                    },
                    authenticator_attachment: e.authenticatorAttachment || void 0,
                    client_extension_results: {
                        app_id: e.clientExtensionResults.appid || void 0,
                        cred_props: e.clientExtensionResults.credProps || void 0,
                        hmac_create_secret: e.clientExtensionResults.hmacCreateSecret || void 0
                    }
                }
            }
            _transformAuthenticationResponseToSnakeCase(e) {
                return {
                    type: e.type,
                    id: e.id,
                    raw_id: e.rawId,
                    response: {
                        signature: e.response.signature,
                        client_data_json: e.response.clientDataJSON,
                        authenticator_data: e.response.authenticatorData,
                        user_handle: e.response.userHandle || void 0
                    },
                    authenticator_attachment: e.authenticatorAttachment || void 0,
                    client_extension_results: {
                        app_id: e.clientExtensionResults.appid || void 0,
                        cred_props: e.clientExtensionResults.credProps || void 0,
                        hmac_create_secret: e.clientExtensionResults.hmacCreateSecret || void 0
                    }
                }
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let tn = {
            path: "/api/v1/passwordless_sms/authenticate",
            method: "POST"
        }
          , tr = {
            path: "/api/v1/passwordless_sms/init",
            method: "POST"
        }
          , ti = {
            path: "/api/v1/passwordless_sms/link",
            method: "POST"
        }
          , ts = {
            path: "/api/v1/passwordless_sms/unlink",
            method: "POST"
        }
          , ta = {
            path: "/api/v1/passwordless_sms/update",
            method: "POST"
        };
        class to {
            async sendCode(e, t) {
                return this._privyInternal.fetch(tr, {
                    body: {
                        phoneNumber: e,
                        token: t
                    }
                })
            }
            async loginWithCode(e, t, n, r) {
                let i = await this._privyInternal.fetch(tn, {
                    body: {
                        phoneNumber: e,
                        code: t,
                        mode: n
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(i);
                let s = await eO(this._embedded, i, r?.embedded);
                return this._privyInternal.callbacks?.setUser?.(s.user),
                s
            }
            async linkWithCode(e, t) {
                await this._privyInternal.fetch(ti, {
                    body: {
                        phoneNumber: e,
                        code: t
                    }
                });
                let n = await this._privyInternal.refreshSession();
                return {
                    user: n.user,
                    identity_token: n.identity_token
                }
            }
            async updatePhone({oldPhoneNumber: e, newPhoneNumber: t, code: n}) {
                await this._privyInternal.fetch(ta, {
                    body: {
                        old_phone_number: e,
                        new_phone_number: t,
                        code: n
                    }
                });
                let r = await this._privyInternal.refreshSession();
                return {
                    user: r.user,
                    identity_token: r.identity_token
                }
            }
            async unlink(e) {
                await this._privyInternal.fetch(ts, {
                    body: {
                        phoneNumber: e
                    }
                });
                let t = await this._privyInternal.refreshSession();
                return {
                    user: t.user,
                    identity_token: t.identity_token
                }
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let tl = {
            path: "/api/v1/siwe/init",
            method: "POST"
        }
          , tc = {
            path: "/api/v1/siwe/authenticate",
            method: "POST"
        }
          , td = {
            path: "/api/v1/siwe/link",
            method: "POST"
        }
          , tu = {
            path: "/api/v1/siwe/link_smart_wallet",
            method: "POST"
        }
          , th = {
            path: "/api/v1/siwe/unlink",
            method: "POST"
        };
        class tp {
            async unlinkWallet(e) {
                await this._privyInternal.fetch(th, {
                    body: {
                        address: e
                    }
                });
                let t = await this._privyInternal.refreshSession();
                return {
                    user: t.user,
                    identity_token: t.identity_token
                }
            }
            async linkWithSiwe(e, t, n) {
                let r = t || this._wallet
                  , i = n || this._preparedMessage;
                if (!r)
                    throw Error("A wallet must be provided in the init step or as an argument to linkWithSiwe");
                if (!i)
                    throw Error("A message must be generated and signed before being used to link a wallet to privy");
                await this._privyInternal.fetch(td, {
                    body: {
                        message: i,
                        signature: e,
                        chainId: r.chainId,
                        walletClientType: r.walletClientType,
                        connectorType: r.connectorType
                    }
                });
                let s = await this._privyInternal.refreshSession();
                return {
                    user: s.user,
                    identity_token: s.identity_token
                }
            }
            async loginWithSiwe(e, t, n, r, i) {
                let s = t || this._wallet
                  , a = n || this._preparedMessage;
                if (!s)
                    throw Error("A wallet must be provided in the init step or as an argument to loginWithSiwe");
                if (!a)
                    throw Error("A message must be generated and signed before being used to login to privy with a wallet");
                let o = await this._privyInternal.fetch(tc, {
                    body: {
                        signature: e,
                        message: a,
                        chainId: s.chainId,
                        walletClientType: s.walletClientType,
                        connectorType: s.connectorType,
                        mode: r
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(o);
                let l = await eO(this._embedded, o, i?.embedded);
                return this._privyInternal.callbacks?.setUser?.(l.user),
                l
            }
            async init(e, t, n) {
                var r;
                this._wallet = e;
                let {nonce: i} = await this._privyInternal.fetch(tl, {
                    body: {
                        address: e.address
                    }
                })
                  , s = `${(r = {
                    chainId: e.chainId.toString().replace("eip155:", ""),
                    address: e.address,
                    issuedAt: (new Date).toISOString(),
                    statement: "By signing, you are proving you own this wallet and logging in. This does not initiate a transaction or cost any fees.",
                    domain: t,
                    nonce: i,
                    uri: n
                }).domain} wants you to sign in with your Ethereum account:
${r.address}

${r.statement}

URI: ${r.uri}
Version: 1
Chain ID: ${r.chainId}
Nonce: ${r.nonce}
Issued At: ${r.issuedAt}
Resources:
- https://privy.io`;
                return this._preparedMessage = s,
                {
                    nonce: i,
                    message: s
                }
            }
            constructor(e, t) {
                this._wallet = void 0,
                this._privyInternal = e,
                this._embedded = t
            }
        }
        let tf = {
            path: "/api/v1/siws/init",
            method: "POST"
        }
          , ty = {
            path: "/api/v1/siws/authenticate",
            method: "POST"
        }
          , tm = {
            path: "/api/v1/siws/link",
            method: "POST"
        }
          , tg = {
            path: "/api/v1/siws/unlink",
            method: "POST"
        };
        class tw {
            async unlink({address: e}) {
                await this._privyInternal.fetch(tg, {
                    body: {
                        address: e
                    }
                });
                let t = await this._privyInternal.refreshSession();
                return {
                    user: t.user,
                    identity_token: t.identity_token
                }
            }
            async link({message: e, signature: t, walletClientType: n, connectorType: r}) {
                await this._privyInternal.fetch(tm, {
                    body: {
                        message: e,
                        signature: t,
                        walletClientType: n,
                        connectorType: r
                    }
                });
                let i = await this._privyInternal.refreshSession();
                return {
                    user: i.user,
                    identity_token: i.identity_token
                }
            }
            async login({mode: e, message: t, signature: n, walletClientType: r, connectorType: i, opts: s}) {
                let a = await this._privyInternal.fetch(ty, {
                    body: {
                        signature: n,
                        message: t,
                        walletClientType: r,
                        connectorType: i,
                        mode: e
                    }
                });
                await this._privyInternal.session.updateWithTokensResponse(a);
                let o = await eO(this._embedded, a, s?.embedded);
                return this._privyInternal.callbacks?.setUser?.(o.user),
                o
            }
            async fetchNonce({address: e}) {
                let {nonce: t} = await this._privyInternal.fetch(tf, {
                    body: {
                        address: e
                    }
                });
                return {
                    nonce: t
                }
            }
            constructor(e, t) {
                this._privyInternal = e,
                this._embedded = t
            }
        }
        class t_ {
            async link(e, t, n, r) {
                await this._privyInternal.fetch(tu, {
                    body: {
                        message: e,
                        signature: t,
                        smart_wallet_type: n,
                        smart_wallet_version: r
                    }
                });
                let i = await this._privyInternal.refreshSession();
                return {
                    user: i.user,
                    identity_token: i.identity_token
                }
            }
            async init(e) {
                var t;
                let {nonce: n} = await this._privyInternal.fetch(tl, {
                    body: {
                        address: e.address
                    }
                });
                return {
                    nonce: n,
                    message: `${(t = {
                        chainId: e.chainId.toString().replace("eip155:", ""),
                        address: e.address,
                        issuedAt: (new Date).toISOString(),
                        statement: "By signing, you are proving you own this wallet and logging in. This does not initiate a transaction or cost any fees.",
                        domain: "privy.io",
                        uri: "https://auth.privy.io",
                        nonce: n
                    }).domain} wants you to sign in with your Ethereum account:
${t.address}

${t.statement}

URI: ${t.uri}
Version: 1
Chain ID: ${t.chainId}
Nonce: ${t.nonce}
Issued At: ${t.issuedAt}
Resources:
- https://privy.io`
                }
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        class tv {
            async logout() {
                try {
                    let e = await this._privyInternal.session.getRefreshToken() ?? void 0;
                    await this._privyInternal.fetch(en, {
                        body: {
                            refresh_token: e
                        }
                    })
                } catch (e) {
                    console.warn("Error destroying session")
                }
                await Promise.all([this._privyInternal.session.destroyLocalState({
                    reason: "logout"
                }), this._privyInternal.destroyClientAnalyticsId()]),
                this._privyInternal.callbacks?.setUser?.(null)
            }
            constructor(e, t, n, r) {
                this._privyInternal = e,
                this.customProvider = new eN(this._privyInternal,t),
                this.phone = new to(this._privyInternal,t),
                this.email = new eP(this._privyInternal,t),
                this.oauth = new e5(this._privyInternal,t,n,r),
                this.guest = new eY(this._privyInternal,t),
                this.siwe = new tp(this._privyInternal,t),
                this.siws = new tw(this._privyInternal,t),
                this.smartWallet = new t_(this._privyInternal),
                this.passkey = new tt(this._privyInternal,t),
                this.farcaster = new eK(this._privyInternal,t),
                this.farcasterV2 = new eQ(this._privyInternal,t)
            }
        }
        let tb = {
            path: "/api/v1/funding/coinbase_on_ramp/init",
            method: "POST"
        }
          , tE = {
            path: "/api/v1/funding/coinbase_on_ramp/status",
            method: "GET"
        };
        class tI {
            async initOnRampSession(e) {
                return await this._privyInternal.fetch(tb, {
                    body: e
                })
            }
            async getStatus(e) {
                return await this._privyInternal.fetch(tE, {
                    query: {
                        partnerUserId: e
                    }
                })
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        let tk = {
            path: "/api/v1/plugins/moonpay_on_ramp/sign",
            method: "POST"
        };
        var tM = n(36544);
        let tS = class {
            async sign(e) {
                return await this._privyInternal.fetch(tk, {
                    body: e
                })
            }
            async getTransactionStatus({transactionId: e, useSandbox: t}) {
                let {url: n, key: r} = tM.IX[t ? "sandbox" : "prod"]
                  , i = await G(fetch, {
                    retries: 3,
                    retryDelay: 500
                })(`${n}/transactions/ext/${e}?apiKey=${r}`);
                if (!i.ok)
                    throw new p.gn({
                        error: `Failed to fetch transaction status for Transaction ${e}`,
                        code: "failed_to_fetch_moonpay_transaction_status",
                        response: i
                    });
                let s = await i.json();
                return Array.isArray(s) ? s.at(0) : void 0
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        ;
        class tA {
            constructor(e) {
                this.moonpay = new tS(e),
                this.coinbase = new tI(e)
            }
        }
        let tT = {
            path: "/api/v1/mfa/passkeys/init",
            method: "POST"
        }
          , tC = class {
            async generateAuthenticationOptions(e) {
                return await this._privyInternal.fetch(tT, {
                    body: e
                })
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
          , tO = {
            path: "/api/v1/mfa/passwordless_sms/init",
            method: "POST"
        };
        class tN {
            async sendCode(e) {
                return await this._privyInternal.fetch(tO, {
                    body: e
                })
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        class tL {
            setProxy(e) {
                this.proxy = e
            }
            async getAccessToken() {
                let e = await this.privyInternal.getAccessToken();
                if (!e)
                    throw new p.wE({
                        error: "Missing access token",
                        code: "attempted_rpc_call_before_logged_in"
                    });
                return e
            }
            async verifyMfa() {
                if (!this.proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                return await this.proxy.verifyMfa({
                    accessToken: await this.getAccessToken()
                })
            }
            async initEnrollMfa(e) {
                if (!this.proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                return await this.proxy.initEnrollMfa({
                    ...e,
                    accessToken: await this.getAccessToken()
                })
            }
            async submitEnrollMfa(e) {
                if (!this.proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                let t = await this.proxy.submitEnrollMfa({
                    ...e,
                    accessToken: await this.getAccessToken()
                });
                return await this.privyInternal.refreshSession(),
                t
            }
            async unenrollMfa(e) {
                if (!this.proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                let t = await this.proxy.unenrollMfa({
                    method: e,
                    accessToken: await this.getAccessToken()
                });
                return await this.privyInternal.refreshSession(),
                t
            }
            async clearMfa(e) {
                if (!this.proxy)
                    throw new p.wE({
                        error: "Embedded wallet proxy not initialized",
                        code: "embedded_wallet_webview_not_loaded"
                    });
                return await this.proxy.clearMfa(e)
            }
            constructor(e, t) {
                this.proxy = t,
                this.privyInternal = e,
                this.sms = new tN(e),
                this.passkey = new tC(e)
            }
        }
        let tx = {
            path: "/api/v1/embedded_wallets/:address/recovery/key_material",
            method: "POST"
        }
          , tj = {
            path: "/api/v1/recovery/oauth/init",
            method: "POST"
        }
          , tD = {
            path: "/api/v1/recovery/oauth/authenticate",
            method: "POST"
        }
          , tR = {
            path: "/api/v1/recovery/oauth/init_icloud",
            method: "POST"
        }
          , tP = {
            path: "/api/v1/recovery/configuration_icloud",
            method: "POST"
        };
        class tW {
            async init(e) {
                return this._privyInternal.fetch(tR, {
                    body: {
                        client_type: e
                    }
                })
            }
            async getICloudConfiguration(e) {
                return this._privyInternal.fetch(tP, {
                    body: {
                        client_type: e
                    }
                })
            }
            constructor(e) {
                this._privyInternal = e
            }
        }
        class tU {
            async generateURL(e) {
                let t = e2()
                  , n = e2()
                  , r = await e4({
                    codeVerifier: t,
                    digest: this._crypto?.digest
                });
                return await Promise.all([this._storage.put(e0, t), this._storage.put(eX, n)]),
                this._privyInternal.fetch(tj, {
                    body: {
                        redirect_to: e,
                        code_challenge: r,
                        state_code: n
                    }
                })
            }
            async authorize(e, t) {
                let[n,r] = await Promise.all([this._storage.get(e0), this._storage.get(eX)]);
                if (r !== t)
                    throw this._privyInternal.createAnalyticsEvent("possible_phishing_attempt", {
                        flow: "recovery_oauth",
                        storedStateCode: r ?? "",
                        returnedStateCode: t ?? ""
                    }),
                    new p.wE({
                        code: "pkce_state_code_mismatch",
                        error: "Unexpected auth flow. This may be a phishing attempt."
                    });
                let i = await this._privyInternal.fetch(tD, {
                    body: {
                        authorization_code: e,
                        state_code: r,
                        code_verifier: n
                    }
                });
                return await Promise.all([this._storage.del(e0), this._storage.del(eX)]),
                i
            }
            constructor(e, t, n) {
                this._privyInternal = e,
                this._storage = t,
                this._crypto = n
            }
        }
        class tz {
            async getRecoveryKeyMaterial(e, t) {
                return this._privyInternal.fetch(tx, {
                    body: {
                        chain_type: t
                    },
                    params: {
                        address: e
                    }
                })
            }
            constructor(e, t, n) {
                this._privyInternal = e,
                this.auth = new tU(this._privyInternal,t,n),
                this.icloudAuth = new tW(this._privyInternal)
            }
        }
        class tq {
            async initialize() {
                await this._privyInternal._initialize()
            }
            setMessagePoster(e) {
                this.embeddedWallet.setMessagePoster(e)
            }
            addOAuthTokensListener(e) {
                return this._privyInternal.session.on("oauth_tokens_granted", e),
                {
                    unsubscribe: () => {
                        this._privyInternal.session.removeListener("oauth_tokens_granted", e)
                    }
                }
            }
            setCallbacks(e) {
                this._privyInternal.setCallbacks(e)
            }
            getAccessToken() {
                return this._privyInternal.getAccessToken()
            }
            getIdentityToken() {
                return this._privyInternal.getIdentityToken()
            }
            getCompiledPath(e, t) {
                return this._privyInternal.getPath(e, t)
            }
            async fetchPrivyRoute(e, t) {
                return this._privyInternal.fetch(e, t)
            }
            get logger() {
                return this._privyInternal.logger
            }
            constructor({clientId: e, ...t}) {
                this._privyInternal = new e_({
                    ...t,
                    appClientId: e
                }),
                this.mfa = new tL(this._privyInternal),
                this.mfaPromises = new Q,
                this.app = new i(this._privyInternal),
                this.embeddedWallet = new K(this._privyInternal,t.embeddedWalletMessagePoster,t.supportedChains,this.mfa,this.mfaPromises,this.app),
                this.user = new eb(this._privyInternal),
                this.auth = new tv(this._privyInternal,this.embeddedWallet,t.storage,t.crypto),
                this.recovery = new tz(this._privyInternal,t.storage,t.crypto),
                this.funding = new tA(this._privyInternal),
                this.delegated = new d(this._privyInternal),
                this.crossApp = new l(this._privyInternal,t.storage)
            }
        }
    }
    ,
    12727: (e, t, n) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        });
        var r, i, s, a, o, l, c, d, u, h, p, f, y, m, g, w, _, v, b, E, I, k, M, S, A, T, C, O, N, L, x, j, D, R, P, W, U, z, q, F, B, H, K, Q, V, Y, G, Z, $, J, X, ee, et, en, er, ei, es, ea, eo, el, ec, ed, eu, eh, ep, ef, ey, em, eg, ew, e_, ev, eb, eE, eI, ek, eM, eS, eA, eT = n(69523), eC = n(82252), eO = n(74096), eN = n(51427), eL = n(79323), ex = n(70410);
        function ej(e) {
            return e && "object" == typeof e && "default"in e ? e : {
                default: e
            }
        }
        var eD = ej(eC)
          , eR = ej(eL);
        function eP(e, t, n, r) {
            return new (n || (n = Promise))(function(i, s) {
                function a(e) {
                    try {
                        l(r.next(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function o(e) {
                    try {
                        l(r.throw(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function l(e) {
                    var t;
                    e.done ? i(e.value) : ((t = e.value)instanceof n ? t : new n(function(e) {
                        e(t)
                    }
                    )).then(a, o)
                }
                l((r = r.apply(e, t || [])).next())
            }
            )
        }
        function eW(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        }
        function eU(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        }
        let ez = `
<div class="mobile-wallet-adapter-embedded-modal-container" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div data-modal-close style="position: absolute; width: 100%; height: 100%;"></div>
	<div class="mobile-wallet-adapter-embedded-modal-card">
		<div>
			<button data-modal-close class="mobile-wallet-adapter-embedded-modal-close">
				<svg width="14" height="14">
					<path d="M 6.7125,8.3036995 1.9082,13.108199 c -0.2113,0.2112 -0.4765,0.3168 -0.7957,0.3168 -0.3192,0 -0.5844,-0.1056 -0.7958,-0.3168 C 0.1056,12.896899 0,12.631699 0,12.312499 c 0,-0.3192 0.1056,-0.5844 0.3167,-0.7958 L 5.1212,6.7124995 0.3167,1.9082 C 0.1056,1.6969 0,1.4317 0,1.1125 0,0.7933 0.1056,0.5281 0.3167,0.3167 0.5281,0.1056 0.7933,0 1.1125,0 1.4317,0 1.6969,0.1056 1.9082,0.3167 L 6.7125,5.1212 11.5167,0.3167 C 11.7281,0.1056 11.9933,0 12.3125,0 c 0.3192,0 0.5844,0.1056 0.7957,0.3167 0.2112,0.2114 0.3168,0.4766 0.3168,0.7958 0,0.3192 -0.1056,0.5844 -0.3168,0.7957 L 8.3037001,6.7124995 13.1082,11.516699 c 0.2112,0.2114 0.3168,0.4766 0.3168,0.7958 0,0.3192 -0.1056,0.5844 -0.3168,0.7957 -0.2113,0.2112 -0.4765,0.3168 -0.7957,0.3168 -0.3192,0 -0.5844,-0.1056 -0.7958,-0.3168 z" />
				</svg>
			</button>
		</div>
		<div class="mobile-wallet-adapter-embedded-modal-content"></div>
	</div>
</div>
`
          , eq = `
.mobile-wallet-adapter-embedded-modal-container {
    display: flex; /* Use flexbox to center content */
    justify-content: center; /* Center horizontally */
    align-items: center; /* Center vertically */
    position: fixed; /* Stay in place */
    z-index: 1; /* Sit on top */
    left: 0;
    top: 0;
    width: 100%; /* Full width */
    height: 100%; /* Full height */
    background-color: rgba(0,0,0,0.4); /* Black w/ opacity */
    overflow-y: auto; /* enable scrolling */
}

.mobile-wallet-adapter-embedded-modal-card {
    display: flex;
    flex-direction: column;
    margin: auto 20px;
    max-width: 780px;
    padding: 20px;
    border-radius: 24px;
    background: #ffffff;
    font-family: "Inter Tight", "PT Sans", Calibri, sans-serif;
    transform: translateY(-200%);
    animation: slide-in 0.5s forwards;
}

@keyframes slide-in {
    100% { transform: translateY(0%); }
}

.mobile-wallet-adapter-embedded-modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    cursor: pointer;
    background: #e4e9e9;
    border: none;
    border-radius: 50%;
}

.mobile-wallet-adapter-embedded-modal-close:focus-visible {
    outline-color: red;
}

.mobile-wallet-adapter-embedded-modal-close svg {
    fill: #546266;
    transition: fill 200ms ease 0s;
}

.mobile-wallet-adapter-embedded-modal-close:hover svg {
    fill: #fff;
}
`
          , eF = `
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter+Tight:ital,wght@0,100..900;1,100..900&display=swap" rel="stylesheet">
`;
        class eB {
            constructor() {
                r.add(this),
                i.set(this, null),
                s.set(this, {}),
                a.set(this, !1),
                this.dom = null,
                this.open = () => {
                    console.debug("Modal open"),
                    eW(this, r, "m", l).call(this),
                    eW(this, i, "f") && (eW(this, i, "f").style.display = "flex")
                }
                ,
                this.close = e => {
                    var t;
                    console.debug("Modal close"),
                    eW(this, r, "m", c).call(this),
                    eW(this, i, "f") && (eW(this, i, "f").style.display = "none"),
                    null == (t = eW(this, s, "f").close) || t.forEach(t => t(e))
                }
                ,
                d.set(this, e => {
                    "Escape" === e.key && this.close(e)
                }
                ),
                this.init = this.init.bind(this),
                eU(this, i, document.getElementById("mobile-wallet-adapter-embedded-root-ui"), "f")
            }
            init() {
                return eP(this, void 0, void 0, function*() {
                    console.log("Injecting modal"),
                    eW(this, r, "m", o).call(this)
                })
            }
            addEventListener(e, t) {
                var n;
                return (null == (n = eW(this, s, "f")[e]) ? void 0 : n.push(t)) || (eW(this, s, "f")[e] = [t]),
                () => this.removeEventListener(e, t)
            }
            removeEventListener(e, t) {
                var n;
                eW(this, s, "f")[e] = null == (n = eW(this, s, "f")[e]) ? void 0 : n.filter(e => t !== e)
            }
        }
        i = new WeakMap,
        s = new WeakMap,
        a = new WeakMap,
        d = new WeakMap,
        r = new WeakSet,
        o = function() {
            if (document.getElementById("mobile-wallet-adapter-embedded-root-ui")) {
                eW(this, i, "f") || eU(this, i, document.getElementById("mobile-wallet-adapter-embedded-root-ui"), "f");
                return
            }
            eU(this, i, document.createElement("div"), "f"),
            eW(this, i, "f").id = "mobile-wallet-adapter-embedded-root-ui",
            eW(this, i, "f").innerHTML = ez,
            eW(this, i, "f").style.display = "none";
            let e = eW(this, i, "f").querySelector(".mobile-wallet-adapter-embedded-modal-content");
            e && (e.innerHTML = this.contentHtml);
            let t = document.createElement("style");
            t.id = "mobile-wallet-adapter-embedded-modal-styles",
            t.textContent = eq + this.contentStyles;
            let n = document.createElement("div");
            n.innerHTML = eF,
            this.dom = n.attachShadow({
                mode: "closed"
            }),
            this.dom.appendChild(t),
            this.dom.appendChild(eW(this, i, "f")),
            document.body.appendChild(n)
        }
        ,
        l = function() {
            !eW(this, i, "f") || eW(this, a, "f") || ([...eW(this, i, "f").querySelectorAll("[data-modal-close]")].forEach(e => null == e ? void 0 : e.addEventListener("click", this.close)),
            window.addEventListener("load", this.close),
            document.addEventListener("keydown", eW(this, d, "f")),
            eU(this, a, !0, "f"))
        }
        ,
        c = function() {
            if (eW(this, a, "f"))
                window.removeEventListener("load", this.close),
                document.removeEventListener("keydown", eW(this, d, "f")),
                eW(this, i, "f") && ([...eW(this, i, "f").querySelectorAll("[data-modal-close]")].forEach(e => null == e ? void 0 : e.removeEventListener("click", this.close)),
                eU(this, a, !1, "f"))
        }
        ;
        class eH extends eB {
            constructor() {
                super(...arguments),
                this.contentStyles = eQ,
                this.contentHtml = eK
            }
            initWithQR(e) {
                let t = Object.create(null, {
                    init: {
                        get: () => super.init
                    }
                });
                return eP(this, void 0, void 0, function*() {
                    t.init.call(this),
                    this.populateQRCode(e)
                })
            }
            populateQRCode(e) {
                var t;
                return eP(this, void 0, void 0, function*() {
                    let n = null == (t = this.dom) ? void 0 : t.getElementById("mobile-wallet-adapter-embedded-modal-qr-code-container");
                    if (n) {
                        let t = yield eD.default.toCanvas(e, {
                            width: 200,
                            margin: 0
                        });
                        null !== n.firstElementChild ? n.replaceChild(t, n.firstElementChild) : n.appendChild(t)
                    } else
                        console.error("QRCode Container not found")
                })
            }
        }
        let eK = `
<div class="mobile-wallet-adapter-embedded-modal-qr-content">
    <div>
        <svg class="mobile-wallet-adapter-embedded-modal-icon" width="100%" height="100%">
            <circle r="52" cx="53" cy="53" fill="#99b3be" stroke="#000000" stroke-width="2"/>
            <path d="m 53,82.7305 c -3.3116,0 -6.1361,-1.169 -8.4735,-3.507 -2.338,-2.338 -3.507,-5.1625 -3.507,-8.4735 0,-3.3116 1.169,-6.1364 3.507,-8.4744 2.3374,-2.338 5.1619,-3.507 8.4735,-3.507 3.3116,0 6.1361,1.169 8.4735,3.507 2.338,2.338 3.507,5.1628 3.507,8.4744 0,3.311 -1.169,6.1355 -3.507,8.4735 -2.3374,2.338 -5.1619,3.507 -8.4735,3.507 z m 0.007,-5.25 c 1.8532,0 3.437,-0.6598 4.7512,-1.9793 1.3149,-1.3195 1.9723,-2.9058 1.9723,-4.7591 0,-1.8526 -0.6598,-3.4364 -1.9793,-4.7512 -1.3195,-1.3149 -2.9055,-1.9723 -4.7582,-1.9723 -1.8533,0 -3.437,0.6598 -4.7513,1.9793 -1.3148,1.3195 -1.9722,2.9058 -1.9722,4.7591 0,1.8527 0.6597,3.4364 1.9792,4.7512 1.3195,1.3149 2.9056,1.9723 4.7583,1.9723 z m -28,-33.5729 -3.85,-3.6347 c 4.1195,-4.025 8.8792,-7.1984 14.2791,-9.52 5.4005,-2.3223 11.2551,-3.4834 17.5639,-3.4834 6.3087,0 12.1634,1.1611 17.5639,3.4834 5.3999,2.3216 10.1596,5.495 14.2791,9.52 l -3.85,3.6347 C 77.2999,40.358 73.0684,37.5726 68.2985,35.5514 63.5292,33.5301 58.4296,32.5195 53,32.5195 c -5.4297,0 -10.5292,1.0106 -15.2985,3.0319 -4.7699,2.0212 -9.0014,4.8066 -12.6945,8.3562 z m 44.625,10.8771 c -2.2709,-2.1046 -4.7962,-3.7167 -7.5758,-4.8361 -2.7795,-1.12 -5.7983,-1.68 -9.0562,-1.68 -3.2579,0 -6.2621,0.56 -9.0125,1.68 -2.7504,1.1194 -5.2903,2.7315 -7.6195,4.8361 L 32.5189,51.15 c 2.8355,-2.6028 5.9777,-4.6086 9.4263,-6.0174 3.4481,-1.4087 7.133,-2.1131 11.0548,-2.1131 3.9217,0 7.5979,0.7044 11.0285,2.1131 3.43,1.4088 6.5631,3.4146 9.3992,6.0174 z"/>
        </svg>
        <div class="mobile-wallet-adapter-embedded-modal-title">Remote Mobile Wallet Adapter</div>
    </div>
    <div>
        <div>
            <h4 class="mobile-wallet-adapter-embedded-modal-qr-label">
                Open your wallet and scan this code
            </h4>
        </div>
        <div id="mobile-wallet-adapter-embedded-modal-qr-code-container" class="mobile-wallet-adapter-embedded-modal-qr-code-container"></div>
    </div>
</div>
<div class="mobile-wallet-adapter-embedded-modal-divider"><hr></div>
<div class="mobile-wallet-adapter-embedded-modal-footer">
    <div class="mobile-wallet-adapter-embedded-modal-subtitle">
        Follow the instructions on your device. When you're finished, this screen will update.
    </div>
    <div class="mobile-wallet-adapter-embedded-modal-progress-badge">
        <div>
            <div class="spinner">
                <div class="leftWrapper">
                    <div class="left">
                        <div class="circle"></div>
                    </div>
                </div>
                <div class="rightWrapper">
                    <div class="right">
                        <div class="circle"></div>
                    </div>
                </div>
            </div>
        </div>
        <div>Waiting for scan</div>
    </div>
</div>
`
          , eQ = `
.mobile-wallet-adapter-embedded-modal-qr-content {
    display: flex; 
    margin-top: 10px;
    padding: 10px;
}

.mobile-wallet-adapter-embedded-modal-qr-content > div:first-child {
    display: flex;
    flex-direction: column;
    flex: 2;
    margin-top: auto;
    margin-right: 30px;
}

.mobile-wallet-adapter-embedded-modal-qr-content > div:nth-child(2) {
    display: flex;
    flex-direction: column;
    flex: 1;
    margin-left: auto;
}

.mobile-wallet-adapter-embedded-modal-footer {
    display: flex;
    padding: 10px;
}

.mobile-wallet-adapter-embedded-modal-icon {}

.mobile-wallet-adapter-embedded-modal-title {
    color: #000000;
    font-size: 2.5em;
    font-weight: 600;
}

.mobile-wallet-adapter-embedded-modal-qr-label {
    text-align: right;
    color: #000000;
}

.mobile-wallet-adapter-embedded-modal-qr-code-container {
    margin-left: auto;
}

.mobile-wallet-adapter-embedded-modal-divider {
    margin-top: 20px;
    padding-left: 10px;
    padding-right: 10px;
}

.mobile-wallet-adapter-embedded-modal-divider hr {
    border-top: 1px solid #D9DEDE;
}

.mobile-wallet-adapter-embedded-modal-subtitle {
    margin: auto;
    margin-right: 60px;
    padding: 20px;
    color: #6E8286;
}

.mobile-wallet-adapter-embedded-modal-progress-badge {
    display: flex;
    background: #F7F8F8;
    height: 56px;
    min-width: 200px;
    margin: auto;
    padding-left: 20px;
    padding-right: 20px;
    border-radius: 18px;
    color: #A8B6B8;
    align-items: center;
}

.mobile-wallet-adapter-embedded-modal-progress-badge > div:first-child {
    margin-left: auto;
    margin-right: 20px;
}

.mobile-wallet-adapter-embedded-modal-progress-badge > div:nth-child(2) {
    margin-right: auto;
}

/* Smaller screens */
@media all and (max-width: 600px) {
    .mobile-wallet-adapter-embedded-modal-card {
        text-align: center;
    }
    .mobile-wallet-adapter-embedded-modal-qr-content {
        flex-direction: column;
    }
    .mobile-wallet-adapter-embedded-modal-qr-content > div:first-child {
        margin: auto;
    }
    .mobile-wallet-adapter-embedded-modal-qr-content > div:nth-child(2) {
        margin: auto;
        flex: 2 auto;
    }
    .mobile-wallet-adapter-embedded-modal-footer {
        flex-direction: column;
    }
    .mobile-wallet-adapter-embedded-modal-icon {
        display: none;
    }
    .mobile-wallet-adapter-embedded-modal-title {
        font-size: 1.5em;
    }
    .mobile-wallet-adapter-embedded-modal-subtitle {
        margin-right: unset;
    }
    .mobile-wallet-adapter-embedded-modal-qr-label {
        text-align: center;
    }
    .mobile-wallet-adapter-embedded-modal-qr-code-container {
        margin: auto;
    }
}

/* Spinner */
@keyframes spinLeft {
    0% {
        transform: rotate(20deg);
    }
    50% {
        transform: rotate(160deg);
    }
    100% {
        transform: rotate(20deg);
    }
}
@keyframes spinRight {
    0% {
        transform: rotate(160deg);
    }
    50% {
        transform: rotate(20deg);
    }
    100% {
        transform: rotate(160deg);
    }
}
@keyframes spin {
    0% {
        transform: rotate(0deg);
    }
    100% {
        transform: rotate(2520deg);
    }
}

.spinner {
    position: relative;
    width: 1.5em;
    height: 1.5em;
    margin: auto;
    animation: spin 10s linear infinite;
}
.spinner::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
}
.right, .rightWrapper, .left, .leftWrapper {
    position: absolute;
    top: 0;
    overflow: hidden;
    width: .75em;
    height: 1.5em;
}
.left, .leftWrapper {
    left: 0;
}
.right {
    left: -12px;
}
.rightWrapper {
    right: 0;
}
.circle {
    border: .125em solid #A8B6B8;
    width: 1.25em; /* 1.5em - 2*0.125em border */
    height: 1.25em; /* 1.5em - 2*0.125em border */
    border-radius: 0.75em; /* 0.5*1.5em spinner size 8 */
}
.left {
    transform-origin: 100% 50%;
    animation: spinLeft 2.5s cubic-bezier(.2,0,.8,1) infinite;
}
.right {
    transform-origin: 100% 50%;
    animation: spinRight 2.5s cubic-bezier(.2,0,.8,1) infinite;
}
`
          , eV = "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik03IDIuNUgxN0MxNy44Mjg0IDIuNSAxOC41IDMuMTcxNTcgMTguNSA0VjIwQzE4LjUgMjAuODI4NCAxNy44Mjg0IDIxLjUgMTcgMjEuNUg3QzYuMTcxNTcgMjEuNSA1LjUgMjAuODI4NCA1LjUgMjBWNEM1LjUgMy4xNzE1NyA2LjE3MTU3IDIuNSA3IDIuNVpNMyA0QzMgMS43OTA4NiA0Ljc5MDg2IDAgNyAwSDE3QzE5LjIwOTEgMCAyMSAxLjc5MDg2IDIxIDRWMjBDMjEgMjIuMjA5MSAxOS4yMDkxIDI0IDE3IDI0SDdDNC43OTA4NiAyNCAzIDIyLjIwOTEgMyAyMFY0Wk0xMSA0LjYxNTM4QzEwLjQ0NzcgNC42MTUzOCAxMCA1LjA2MzEgMTAgNS42MTUzOFY2LjM4NDYyQzEwIDYuOTM2OSAxMC40NDc3IDcuMzg0NjIgMTEgNy4zODQ2MkgxM0MxMy41NTIzIDcuMzg0NjIgMTQgNi45MzY5IDE0IDYuMzg0NjJWNS42MTUzOEMxNCA1LjA2MzEgMTMuNTUyMyA0LjYxNTM4IDEzIDQuNjE1MzhIMTFaIiBmaWxsPSIjRENCOEZGIi8+Cjwvc3ZnPgo=";
        function eY(e) {
            return window.btoa(String.fromCharCode.call(null, ...e))
        }
        function eG(e) {
            return new Uint8Array(window.atob(e).split("").map(e => e.charCodeAt(0)))
        }
        let eZ = "Mobile Wallet Adapter"
          , e$ = [eT.SolanaSignAndSendTransaction, eT.SolanaSignTransaction, eT.SolanaSignMessage, eT.SolanaSignIn];
        class eJ {
            constructor(e) {
                u.add(this),
                h.set(this, {}),
                p.set(this, "1.0.0"),
                f.set(this, eZ),
                y.set(this, "https://solanamobile.com/wallets"),
                m.set(this, eV),
                g.set(this, void 0),
                w.set(this, void 0),
                _.set(this, void 0),
                v.set(this, !1),
                b.set(this, 0),
                E.set(this, []),
                I.set(this, void 0),
                k.set(this, void 0),
                M.set(this, void 0),
                S.set(this, (e, t) => {
                    var n;
                    return (null == (n = eW(this, h, "f")[e]) ? void 0 : n.push(t)) || (eW(this, h, "f")[e] = [t]),
                    () => eW(this, u, "m", T).call(this, e, t)
                }
                ),
                C.set(this, ({silent: e}={}) => eP(this, void 0, void 0, function*() {
                    if (eW(this, v, "f") || this.connected)
                        return {
                            accounts: this.accounts
                        };
                    eU(this, v, !0, "f");
                    try {
                        if (e) {
                            let e = yield eW(this, _, "f").get();
                            if (!e)
                                return {
                                    accounts: this.accounts
                                };
                            yield eW(this, L, "f").call(this, e.capabilities),
                            yield eW(this, N, "f").call(this, e)
                        } else
                            yield eW(this, O, "f").call(this)
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    } finally {
                        eU(this, v, !1, "f")
                    }
                    return {
                        accounts: this.accounts
                    }
                })),
                O.set(this, e => eP(this, void 0, void 0, function*() {
                    try {
                        let t = yield eW(this, _, "f").get();
                        if (t)
                            return eW(this, N, "f").call(this, t),
                            t;
                        let n = yield eW(this, I, "f").select(eW(this, E, "f"));
                        return yield eW(this, D, "f").call(this, t => eP(this, void 0, void 0, function*() {
                            let[r,i] = yield Promise.all([t.getCapabilities(), t.authorize({
                                chain: n,
                                identity: eW(this, g, "f"),
                                sign_in_payload: e
                            })])
                              , s = eW(this, P, "f").call(this, i.accounts)
                              , a = Object.assign(Object.assign({}, i), {
                                accounts: s,
                                chain: n,
                                capabilities: r
                            });
                            return Promise.all([eW(this, L, "f").call(this, r), eW(this, _, "f").set(a), eW(this, N, "f").call(this, a)]),
                            a
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                N.set(this, e => eP(this, void 0, void 0, function*() {
                    var t;
                    let n = null == eW(this, w, "f") || (null == (t = eW(this, w, "f")) ? void 0 : t.accounts.length) !== e.accounts.length || eW(this, w, "f").accounts.some( (t, n) => t.address !== e.accounts[n].address);
                    eU(this, w, e, "f"),
                    n && eW(this, u, "m", A).call(this, "change", {
                        accounts: this.accounts
                    })
                })),
                L.set(this, e => eP(this, void 0, void 0, function*() {
                    let t = e.features.includes("solana:signTransactions")
                      , n = e.supports_sign_and_send_transactions
                      , r = eT.SolanaSignAndSendTransaction in this.features !== n || eT.SolanaSignTransaction in this.features !== t;
                    eU(this, k, Object.assign(Object.assign({}, (n || !n && !t) && {
                        [eT.SolanaSignAndSendTransaction]: {
                            version: "1.0.0",
                            supportedTransactionVersions: ["legacy", 0],
                            signAndSendTransaction: eW(this, z, "f")
                        }
                    }), t && {
                        [eT.SolanaSignTransaction]: {
                            version: "1.0.0",
                            supportedTransactionVersions: ["legacy", 0],
                            signTransaction: eW(this, q, "f")
                        }
                    }), "f"),
                    r && eW(this, u, "m", A).call(this, "change", {
                        features: this.features
                    })
                })),
                x.set(this, (e, t, n) => eP(this, void 0, void 0, function*() {
                    var r, i;
                    try {
                        let[s,a] = yield Promise.all([null != (i = null == (r = eW(this, w, "f")) ? void 0 : r.capabilities) ? i : yield e.getCapabilities(), e.authorize({
                            auth_token: t,
                            identity: eW(this, g, "f"),
                            chain: n
                        })])
                          , o = eW(this, P, "f").call(this, a.accounts)
                          , l = Object.assign(Object.assign({}, a), {
                            accounts: o,
                            chain: n,
                            capabilities: s
                        });
                        Promise.all([eW(this, _, "f").set(l), eW(this, N, "f").call(this, l)])
                    } catch (e) {
                        throw eW(this, j, "f").call(this),
                        Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                j.set(this, () => eP(this, void 0, void 0, function*() {
                    var e;
                    eW(this, _, "f").clear(),
                    eU(this, v, !1, "f"),
                    eU(this, b, (e = eW(this, b, "f"),
                    ++e), "f"),
                    eU(this, w, void 0, "f"),
                    eW(this, u, "m", A).call(this, "change", {
                        accounts: this.accounts
                    })
                })),
                D.set(this, e => eP(this, void 0, void 0, function*() {
                    var t;
                    let n = null == (t = eW(this, w, "f")) ? void 0 : t.wallet_uri_base
                      , r = eW(this, b, "f");
                    try {
                        return yield eO.transact(e, n ? {
                            baseUri: n
                        } : void 0)
                    } catch (e) {
                        throw eW(this, b, "f") !== r && (yield new Promise( () => {}
                        )),
                        e instanceof Error && "SolanaMobileWalletAdapterError" === e.name && "ERROR_WALLET_NOT_FOUND" === e.code && (yield eW(this, M, "f").call(this, this)),
                        e
                    }
                })),
                R.set(this, () => {
                    if (!eW(this, w, "f"))
                        throw Error("Wallet not connected");
                    return {
                        authToken: eW(this, w, "f").auth_token,
                        chain: eW(this, w, "f").chain
                    }
                }
                ),
                P.set(this, e => e.map(e => {
                    var t, n;
                    let r = eG(e.address);
                    return {
                        address: eR.default.encode(r),
                        publicKey: r,
                        label: e.label,
                        icon: e.icon,
                        chains: null != (t = e.chains) ? t : eW(this, E, "f"),
                        features: null != (n = e.features) ? n : e$
                    }
                }
                )),
                W.set(this, e => eP(this, void 0, void 0, function*() {
                    let {authToken: t, chain: n} = eW(this, R, "f").call(this);
                    try {
                        let r = e.map(e => eY(e));
                        return yield eW(this, D, "f").call(this, e => eP(this, void 0, void 0, function*() {
                            return yield eW(this, x, "f").call(this, e, t, n),
                            (yield e.signTransactions({
                                payloads: r
                            })).signed_payloads.map(eG)
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                U.set(this, (e, t) => eP(this, void 0, void 0, function*() {
                    let {authToken: n, chain: r} = eW(this, R, "f").call(this);
                    try {
                        return yield eW(this, D, "f").call(this, i => eP(this, void 0, void 0, function*() {
                            let[s,a] = yield Promise.all([i.getCapabilities(), eW(this, x, "f").call(this, i, n, r)]);
                            if (s.supports_sign_and_send_transactions) {
                                let n = eY(e);
                                return (yield i.signAndSendTransactions(Object.assign(Object.assign({}, t), {
                                    payloads: [n]
                                }))).signatures.map(eG)[0]
                            }
                            throw Error("connected wallet does not support signAndSendTransaction")
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                z.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let t = [];
                    for (let n of e) {
                        let e = yield eW(this, U, "f").call(this, n.transaction, n.options);
                        t.push({
                            signature: e
                        })
                    }
                    return t
                })),
                q.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    return (yield eW(this, W, "f").call(this, e.map( ({transaction: e}) => e))).map(e => ({
                        signedTransaction: e
                    }))
                })),
                F.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let {authToken: t, chain: n} = eW(this, R, "f").call(this)
                      , r = e.map( ({account: e}) => eY(e.publicKey))
                      , i = e.map( ({message: e}) => eY(e));
                    try {
                        return yield eW(this, D, "f").call(this, e => eP(this, void 0, void 0, function*() {
                            return yield eW(this, x, "f").call(this, e, t, n),
                            (yield e.signMessages({
                                addresses: r,
                                payloads: i
                            })).signed_payloads.map(eG).map(e => ({
                                signedMessage: e,
                                signature: e.slice(-64)
                            }))
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                B.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let t = [];
                    if (!(e.length > 1))
                        return [(yield eW(this, H, "f").call(this, e[0]))];
                    for (let n of e)
                        t.push((yield eW(this, H, "f").call(this, n)));
                    return t
                })),
                H.set(this, e => eP(this, void 0, void 0, function*() {
                    var t, n, r;
                    eU(this, v, !0, "f");
                    try {
                        let i = yield eW(this, O, "f").call(this, Object.assign(Object.assign({}, e), {
                            domain: null != (t = null == e ? void 0 : e.domain) ? t : window.location.host
                        }));
                        if (!i.sign_in_result)
                            throw Error("Sign in failed, no sign in result returned by wallet");
                        let s = i.sign_in_result.address
                          , a = i.accounts.find(e => e.address == s);
                        return {
                            account: Object.assign(Object.assign({}, null != a ? a : {
                                address: eR.default.encode(eG(s))
                            }), {
                                publicKey: eG(s),
                                chains: null != (n = null == a ? void 0 : a.chains) ? n : eW(this, E, "f"),
                                features: null != (r = null == a ? void 0 : a.features) ? r : i.capabilities.features
                            }),
                            signedMessage: eG(i.sign_in_result.signed_message),
                            signature: eG(i.sign_in_result.signature)
                        }
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    } finally {
                        eU(this, v, !1, "f")
                    }
                })),
                eU(this, _, e.authorizationCache, "f"),
                eU(this, g, e.appIdentity, "f"),
                eU(this, E, e.chains, "f"),
                eU(this, I, e.chainSelector, "f"),
                eU(this, M, e.onWalletNotFound, "f"),
                eU(this, k, {
                    [eT.SolanaSignAndSendTransaction]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signAndSendTransaction: eW(this, z, "f")
                    },
                    [eT.SolanaSignTransaction]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signTransaction: eW(this, q, "f")
                    }
                }, "f")
            }
            get version() {
                return eW(this, p, "f")
            }
            get name() {
                return eW(this, f, "f")
            }
            get url() {
                return eW(this, y, "f")
            }
            get icon() {
                return eW(this, m, "f")
            }
            get chains() {
                return eW(this, E, "f")
            }
            get features() {
                return Object.assign({
                    [eN.StandardConnect]: {
                        version: "1.0.0",
                        connect: eW(this, C, "f")
                    },
                    [eN.StandardDisconnect]: {
                        version: "1.0.0",
                        disconnect: eW(this, j, "f")
                    },
                    [eN.StandardEvents]: {
                        version: "1.0.0",
                        on: eW(this, S, "f")
                    },
                    [eT.SolanaSignMessage]: {
                        version: "1.0.0",
                        signMessage: eW(this, F, "f")
                    },
                    [eT.SolanaSignIn]: {
                        version: "1.0.0",
                        signIn: eW(this, B, "f")
                    }
                }, eW(this, k, "f"))
            }
            get accounts() {
                var e, t;
                return null != (t = null == (e = eW(this, w, "f")) ? void 0 : e.accounts) ? t : []
            }
            get connected() {
                return !!eW(this, w, "f")
            }
            get isAuthorized() {
                return !!eW(this, w, "f")
            }
            get currentAuthorization() {
                return eW(this, w, "f")
            }
            get cachedAuthorizationResult() {
                return eW(this, _, "f").get()
            }
        }
        h = new WeakMap,
        p = new WeakMap,
        f = new WeakMap,
        y = new WeakMap,
        m = new WeakMap,
        g = new WeakMap,
        w = new WeakMap,
        _ = new WeakMap,
        v = new WeakMap,
        b = new WeakMap,
        E = new WeakMap,
        I = new WeakMap,
        k = new WeakMap,
        M = new WeakMap,
        S = new WeakMap,
        C = new WeakMap,
        O = new WeakMap,
        N = new WeakMap,
        L = new WeakMap,
        x = new WeakMap,
        j = new WeakMap,
        D = new WeakMap,
        R = new WeakMap,
        P = new WeakMap,
        W = new WeakMap,
        U = new WeakMap,
        z = new WeakMap,
        q = new WeakMap,
        F = new WeakMap,
        B = new WeakMap,
        H = new WeakMap,
        u = new WeakSet,
        A = function(e, ...t) {
            var n;
            null == (n = eW(this, h, "f")[e]) || n.forEach(e => e.apply(null, t))
        }
        ,
        T = function(e, t) {
            var n;
            eW(this, h, "f")[e] = null == (n = eW(this, h, "f")[e]) ? void 0 : n.filter(e => t !== e)
        }
        ;
        class eX {
            constructor(e) {
                K.add(this),
                Q.set(this, {}),
                V.set(this, "1.0.0"),
                Y.set(this, eZ),
                G.set(this, "https://solanamobile.com/wallets"),
                Z.set(this, eV),
                $.set(this, void 0),
                J.set(this, void 0),
                X.set(this, void 0),
                ee.set(this, !1),
                et.set(this, 0),
                en.set(this, []),
                er.set(this, void 0),
                ei.set(this, void 0),
                es.set(this, void 0),
                ea.set(this, void 0),
                eo.set(this, void 0),
                el.set(this, (e, t) => {
                    var n;
                    return (null == (n = eW(this, Q, "f")[e]) ? void 0 : n.push(t)) || (eW(this, Q, "f")[e] = [t]),
                    () => eW(this, K, "m", ed).call(this, e, t)
                }
                ),
                eu.set(this, ({silent: e}={}) => eP(this, void 0, void 0, function*() {
                    if (eW(this, ee, "f") || this.connected)
                        return {
                            accounts: this.accounts
                        };
                    eU(this, ee, !0, "f");
                    try {
                        yield eW(this, eh, "f").call(this)
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    } finally {
                        eU(this, ee, !1, "f")
                    }
                    return {
                        accounts: this.accounts
                    }
                })),
                eh.set(this, e => eP(this, void 0, void 0, function*() {
                    try {
                        let t = yield eW(this, X, "f").get();
                        if (t)
                            return eW(this, ep, "f").call(this, t),
                            t;
                        eW(this, eo, "f") && eU(this, eo, void 0, "f");
                        let n = yield eW(this, er, "f").select(eW(this, en, "f"));
                        return yield eW(this, eg, "f").call(this, t => eP(this, void 0, void 0, function*() {
                            let[r,i] = yield Promise.all([t.getCapabilities(), t.authorize({
                                chain: n,
                                identity: eW(this, $, "f"),
                                sign_in_payload: e
                            })])
                              , s = eW(this, e_, "f").call(this, i.accounts)
                              , a = Object.assign(Object.assign({}, i), {
                                accounts: s,
                                chain: n,
                                capabilities: r
                            });
                            return Promise.all([eW(this, ef, "f").call(this, r), eW(this, X, "f").set(a), eW(this, ep, "f").call(this, a)]),
                            a
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                ep.set(this, e => eP(this, void 0, void 0, function*() {
                    var t;
                    let n = null == eW(this, J, "f") || (null == (t = eW(this, J, "f")) ? void 0 : t.accounts.length) !== e.accounts.length || eW(this, J, "f").accounts.some( (t, n) => t.address !== e.accounts[n].address);
                    eU(this, J, e, "f"),
                    n && eW(this, K, "m", ec).call(this, "change", {
                        accounts: this.accounts
                    })
                })),
                ef.set(this, e => eP(this, void 0, void 0, function*() {
                    let t = e.features.includes("solana:signTransactions")
                      , n = e.supports_sign_and_send_transactions || e.features.includes("solana:signAndSendTransaction")
                      , r = eT.SolanaSignAndSendTransaction in this.features !== n || eT.SolanaSignTransaction in this.features !== t;
                    eU(this, ei, Object.assign(Object.assign({}, n && {
                        [eT.SolanaSignAndSendTransaction]: {
                            version: "1.0.0",
                            supportedTransactionVersions: e.supported_transaction_versions,
                            signAndSendTransaction: eW(this, eE, "f")
                        }
                    }), t && {
                        [eT.SolanaSignTransaction]: {
                            version: "1.0.0",
                            supportedTransactionVersions: e.supported_transaction_versions,
                            signTransaction: eW(this, eI, "f")
                        }
                    }), "f"),
                    r && eW(this, K, "m", ec).call(this, "change", {
                        features: this.features
                    })
                })),
                ey.set(this, (e, t, n) => eP(this, void 0, void 0, function*() {
                    var r, i;
                    try {
                        let[s,a] = yield Promise.all([null != (i = null == (r = eW(this, J, "f")) ? void 0 : r.capabilities) ? i : yield e.getCapabilities(), e.authorize({
                            auth_token: t,
                            identity: eW(this, $, "f"),
                            chain: n
                        })])
                          , o = eW(this, e_, "f").call(this, a.accounts)
                          , l = Object.assign(Object.assign({}, a), {
                            accounts: o,
                            chain: n,
                            capabilities: s
                        });
                        Promise.all([eW(this, X, "f").set(l), eW(this, ep, "f").call(this, l)])
                    } catch (e) {
                        throw eW(this, em, "f").call(this),
                        Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                em.set(this, () => eP(this, void 0, void 0, function*() {
                    var e, t;
                    null == (e = eW(this, eo, "f")) || e.close(),
                    eW(this, X, "f").clear(),
                    eU(this, ee, !1, "f"),
                    eU(this, et, (t = eW(this, et, "f"),
                    ++t), "f"),
                    eU(this, J, void 0, "f"),
                    eU(this, eo, void 0, "f"),
                    eW(this, K, "m", ec).call(this, "change", {
                        accounts: this.accounts
                    })
                })),
                eg.set(this, e => eP(this, void 0, void 0, function*() {
                    var t;
                    let n = null == (t = eW(this, J, "f")) ? void 0 : t.wallet_uri_base
                      , r = Object.assign(Object.assign({}, n ? {
                        baseUri: n
                    } : void 0), {
                        remoteHostAuthority: eW(this, ea, "f")
                    })
                      , i = eW(this, et, "f")
                      , s = new eH;
                    if (eW(this, eo, "f"))
                        return e(eW(this, eo, "f").wallet);
                    try {
                        let {associationUrl: t, close: n, wallet: i} = yield eO.startRemoteScenario(r)
                          , a = s.addEventListener("close", e => {
                            e && n()
                        }
                        );
                        return s.initWithQR(t.toString()),
                        s.open(),
                        eU(this, eo, {
                            close: n,
                            wallet: yield i
                        }, "f"),
                        a(),
                        s.close(),
                        yield e(eW(this, eo, "f").wallet)
                    } catch (e) {
                        throw s.close(),
                        eW(this, et, "f") !== i && (yield new Promise( () => {}
                        )),
                        e instanceof Error && "SolanaMobileWalletAdapterError" === e.name && "ERROR_WALLET_NOT_FOUND" === e.code && (yield eW(this, es, "f").call(this, this)),
                        e
                    }
                })),
                ew.set(this, () => {
                    if (!eW(this, J, "f"))
                        throw Error("Wallet not connected");
                    return {
                        authToken: eW(this, J, "f").auth_token,
                        chain: eW(this, J, "f").chain
                    }
                }
                ),
                e_.set(this, e => e.map(e => {
                    var t, n;
                    let r = eG(e.address);
                    return {
                        address: eR.default.encode(r),
                        publicKey: r,
                        label: e.label,
                        icon: e.icon,
                        chains: null != (t = e.chains) ? t : eW(this, en, "f"),
                        features: null != (n = e.features) ? n : e$
                    }
                }
                )),
                ev.set(this, e => eP(this, void 0, void 0, function*() {
                    let {authToken: t, chain: n} = eW(this, ew, "f").call(this);
                    try {
                        return yield eW(this, eg, "f").call(this, r => eP(this, void 0, void 0, function*() {
                            return yield eW(this, ey, "f").call(this, r, t, n),
                            (yield r.signTransactions({
                                payloads: e.map(eY)
                            })).signed_payloads.map(eG)
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                eb.set(this, (e, t) => eP(this, void 0, void 0, function*() {
                    let {authToken: n, chain: r} = eW(this, ew, "f").call(this);
                    try {
                        return yield eW(this, eg, "f").call(this, i => eP(this, void 0, void 0, function*() {
                            let[s,a] = yield Promise.all([i.getCapabilities(), eW(this, ey, "f").call(this, i, n, r)]);
                            if (s.supports_sign_and_send_transactions)
                                return (yield i.signAndSendTransactions(Object.assign(Object.assign({}, t), {
                                    payloads: [eY(e)]
                                }))).signatures.map(eG)[0];
                            throw Error("connected wallet does not support signAndSendTransaction")
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                eE.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let t = [];
                    for (let n of e) {
                        let e = yield eW(this, eb, "f").call(this, n.transaction, n.options);
                        t.push({
                            signature: e
                        })
                    }
                    return t
                })),
                eI.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    return (yield eW(this, ev, "f").call(this, e.map( ({transaction: e}) => e))).map(e => ({
                        signedTransaction: e
                    }))
                })),
                ek.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let {authToken: t, chain: n} = eW(this, ew, "f").call(this)
                      , r = e.map( ({account: e}) => eY(e.publicKey))
                      , i = e.map( ({message: e}) => eY(e));
                    try {
                        return yield eW(this, eg, "f").call(this, e => eP(this, void 0, void 0, function*() {
                            return yield eW(this, ey, "f").call(this, e, t, n),
                            (yield e.signMessages({
                                addresses: r,
                                payloads: i
                            })).signed_payloads.map(eG).map(e => ({
                                signedMessage: e,
                                signature: e.slice(-64)
                            }))
                        }))
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    }
                })),
                eM.set(this, (...e) => eP(this, void 0, void 0, function*() {
                    let t = [];
                    if (!(e.length > 1))
                        return [(yield eW(this, eS, "f").call(this, e[0]))];
                    for (let n of e)
                        t.push((yield eW(this, eS, "f").call(this, n)));
                    return t
                })),
                eS.set(this, e => eP(this, void 0, void 0, function*() {
                    var t, n, r;
                    eU(this, ee, !0, "f");
                    try {
                        let i = yield eW(this, eh, "f").call(this, Object.assign(Object.assign({}, e), {
                            domain: null != (t = null == e ? void 0 : e.domain) ? t : window.location.host
                        }));
                        if (!i.sign_in_result)
                            throw Error("Sign in failed, no sign in result returned by wallet");
                        let s = i.sign_in_result.address
                          , a = i.accounts.find(e => e.address == s);
                        return {
                            account: Object.assign(Object.assign({}, null != a ? a : {
                                address: eR.default.encode(eG(s))
                            }), {
                                publicKey: eG(s),
                                chains: null != (n = null == a ? void 0 : a.chains) ? n : eW(this, en, "f"),
                                features: null != (r = null == a ? void 0 : a.features) ? r : i.capabilities.features
                            }),
                            signedMessage: eG(i.sign_in_result.signed_message),
                            signature: eG(i.sign_in_result.signature)
                        }
                    } catch (e) {
                        throw Error(e instanceof Error && e.message || "Unknown error")
                    } finally {
                        eU(this, ee, !1, "f")
                    }
                })),
                eU(this, X, e.authorizationCache, "f"),
                eU(this, $, e.appIdentity, "f"),
                eU(this, en, e.chains, "f"),
                eU(this, er, e.chainSelector, "f"),
                eU(this, ea, e.remoteHostAuthority, "f"),
                eU(this, es, e.onWalletNotFound, "f"),
                eU(this, ei, {
                    [eT.SolanaSignAndSendTransaction]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signAndSendTransaction: eW(this, eE, "f")
                    },
                    [eT.SolanaSignTransaction]: {
                        version: "1.0.0",
                        supportedTransactionVersions: ["legacy", 0],
                        signTransaction: eW(this, eI, "f")
                    }
                }, "f")
            }
            get version() {
                return eW(this, V, "f")
            }
            get name() {
                return eW(this, Y, "f")
            }
            get url() {
                return eW(this, G, "f")
            }
            get icon() {
                return eW(this, Z, "f")
            }
            get chains() {
                return eW(this, en, "f")
            }
            get features() {
                return Object.assign({
                    [eN.StandardConnect]: {
                        version: "1.0.0",
                        connect: eW(this, eu, "f")
                    },
                    [eN.StandardDisconnect]: {
                        version: "1.0.0",
                        disconnect: eW(this, em, "f")
                    },
                    [eN.StandardEvents]: {
                        version: "1.0.0",
                        on: eW(this, el, "f")
                    },
                    [eT.SolanaSignMessage]: {
                        version: "1.0.0",
                        signMessage: eW(this, ek, "f")
                    },
                    [eT.SolanaSignIn]: {
                        version: "1.0.0",
                        signIn: eW(this, eM, "f")
                    }
                }, eW(this, ei, "f"))
            }
            get accounts() {
                var e, t;
                return null != (t = null == (e = eW(this, J, "f")) ? void 0 : e.accounts) ? t : []
            }
            get connected() {
                return !!eW(this, eo, "f") && !!eW(this, J, "f")
            }
            get isAuthorized() {
                return !!eW(this, J, "f")
            }
            get currentAuthorization() {
                return eW(this, J, "f")
            }
            get cachedAuthorizationResult() {
                return eW(this, X, "f").get()
            }
        }
        Q = new WeakMap,
        V = new WeakMap,
        Y = new WeakMap,
        G = new WeakMap,
        Z = new WeakMap,
        $ = new WeakMap,
        J = new WeakMap,
        X = new WeakMap,
        ee = new WeakMap,
        et = new WeakMap,
        en = new WeakMap,
        er = new WeakMap,
        ei = new WeakMap,
        es = new WeakMap,
        ea = new WeakMap,
        eo = new WeakMap,
        el = new WeakMap,
        eu = new WeakMap,
        eh = new WeakMap,
        ep = new WeakMap,
        ef = new WeakMap,
        ey = new WeakMap,
        em = new WeakMap,
        eg = new WeakMap,
        ew = new WeakMap,
        e_ = new WeakMap,
        ev = new WeakMap,
        eb = new WeakMap,
        eE = new WeakMap,
        eI = new WeakMap,
        ek = new WeakMap,
        eM = new WeakMap,
        eS = new WeakMap,
        K = new WeakSet,
        ec = function(e, ...t) {
            var n;
            null == (n = eW(this, Q, "f")[e]) || n.forEach(e => e.apply(null, t))
        }
        ,
        ed = function(e, t) {
            var n;
            eW(this, Q, "f")[e] = null == (n = eW(this, Q, "f")[e]) ? void 0 : n.filter(e => t !== e)
        }
        ;
        var e0 = function(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        }
          , e1 = function(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        };
        function e2(e) {
            let t = ({register: t}) => t(e);
            try {
                window.dispatchEvent(new e4(t))
            } catch (e) {
                console.error("wallet-standard:register-wallet event could not be dispatched\n", e)
            }
            try {
                window.addEventListener("wallet-standard:app-ready", ({detail: e}) => t(e))
            } catch (e) {
                console.error("wallet-standard:app-ready event listener could not be added\n", e)
            }
        }
        class e4 extends Event {
            constructor(e) {
                super("wallet-standard:register-wallet", {
                    bubbles: !1,
                    cancelable: !1,
                    composed: !1
                }),
                eA.set(this, void 0),
                e0(this, eA, e, "f")
            }
            get detail() {
                return e1(this, eA, "f")
            }
            get type() {
                return "wallet-standard:register-wallet"
            }
            preventDefault() {
                throw Error("preventDefault cannot be called")
            }
            stopImmediatePropagation() {
                throw Error("stopImmediatePropagation cannot be called")
            }
            stopPropagation() {
                throw Error("stopPropagation cannot be called")
            }
        }
        eA = new WeakMap;
        class e5 extends eB {
            constructor() {
                super(...arguments),
                this.contentStyles = e6,
                this.contentHtml = e3
            }
            initWithError(e) {
                super.init(),
                this.populateError(e)
            }
            populateError(e) {
                var t, n;
                let r = null == (t = this.dom) ? void 0 : t.getElementById("mobile-wallet-adapter-error-message")
                  , i = null == (n = this.dom) ? void 0 : n.getElementById("mobile-wallet-adapter-error-action");
                if (r) {
                    if ("SolanaMobileWalletAdapterError" === e.name)
                        switch (e.code) {
                        case "ERROR_WALLET_NOT_FOUND":
                            r.innerHTML = "To use mobile wallet adapter, you must have a compatible mobile wallet application installed on your device.",
                            i && i.addEventListener("click", () => {
                                window.location.href = "https://solanamobile.com/wallets"
                            }
                            );
                            return;
                        case "ERROR_BROWSER_NOT_SUPPORTED":
                            r.innerHTML = "This browser appears to be incompatible with mobile wallet adapter. Open this page in a compatible mobile browser app and try again.",
                            i && (i.style.display = "none");
                            return
                        }
                    r.innerHTML = `An unexpected error occurred: ${e.message}`
                } else
                    console.log("Failed to locate error dialog element")
            }
        }
        let e3 = `
<svg class="mobile-wallet-adapter-embedded-modal-error-icon" xmlns="http://www.w3.org/2000/svg" height="50px" viewBox="0 -960 960 960" width="50px" fill="#000000"><path d="M 280,-80 Q 197,-80 138.5,-138.5 80,-197 80,-280 80,-363 138.5,-421.5 197,-480 280,-480 q 83,0 141.5,58.5 58.5,58.5 58.5,141.5 0,83 -58.5,141.5 Q 363,-80 280,-80 Z M 824,-120 568,-376 Q 556,-389 542.5,-402.5 529,-416 516,-428 q 38,-24 61,-64 23,-40 23,-88 0,-75 -52.5,-127.5 Q 495,-760 420,-760 345,-760 292.5,-707.5 240,-655 240,-580 q 0,6 0.5,11.5 0.5,5.5 1.5,11.5 -18,2 -39.5,8 -21.5,6 -38.5,14 -2,-11 -3,-22 -1,-11 -1,-23 0,-109 75.5,-184.5 Q 311,-840 420,-840 q 109,0 184.5,75.5 75.5,75.5 75.5,184.5 0,43 -13.5,81.5 Q 653,-460 629,-428 l 251,252 z m -615,-61 71,-71 70,71 29,-28 -71,-71 71,-71 -28,-28 -71,71 -71,-71 -28,28 71,71 -71,71 z"/></svg>
<div class="mobile-wallet-adapter-embedded-modal-title">We can't find a wallet.</div>
<div id="mobile-wallet-adapter-error-message" class="mobile-wallet-adapter-embedded-modal-subtitle"></div>
<div>
    <button data-error-action id="mobile-wallet-adapter-error-action" class="mobile-wallet-adapter-embedded-modal-error-action">
        Find a wallet
    </button>
</div>
`
          , e6 = `
.mobile-wallet-adapter-embedded-modal-content {
    text-align: center;
}

.mobile-wallet-adapter-embedded-modal-error-icon {
    margin-top: 24px;
}

.mobile-wallet-adapter-embedded-modal-title {
    margin: 18px 100px auto 100px;
    color: #000000;
    font-size: 2.75em;
    font-weight: 600;
}

.mobile-wallet-adapter-embedded-modal-subtitle {
    margin: 30px 60px 40px 60px;
    color: #000000;
    font-size: 1.25em;
    font-weight: 400;
}

.mobile-wallet-adapter-embedded-modal-error-action {
    display: block;
    width: 100%;
    height: 56px;
    /*margin-top: 40px;*/
    font-size: 1.25em;
    /*line-height: 24px;*/
    /*letter-spacing: -1%;*/
    background: #000000;
    color: #FFFFFF;
    border-radius: 18px;
}

/* Smaller screens */
@media all and (max-width: 600px) {
    .mobile-wallet-adapter-embedded-modal-title {
        font-size: 1.5em;
        margin-right: 12px;
        margin-left: 12px;
    }
    .mobile-wallet-adapter-embedded-modal-subtitle {
        margin-right: 12px;
        margin-left: 12px;
    }
}
`;
        function e7() {
            return eP(this, void 0, void 0, function*() {
                if ("undefined" != typeof window) {
                    let e = window.navigator.userAgent.toLowerCase()
                      , t = new e5;
                    e.includes("wv") ? t.initWithError({
                        name: "SolanaMobileWalletAdapterError",
                        code: "ERROR_BROWSER_NOT_SUPPORTED",
                        message: ""
                    }) : t.initWithError({
                        name: "SolanaMobileWalletAdapterError",
                        code: "ERROR_WALLET_NOT_FOUND",
                        message: ""
                    }),
                    t.open()
                }
            })
        }
        let e8 = "SolanaMobileWalletAdapterDefaultAuthorizationCache";
        t.LocalSolanaMobileWalletAdapterWallet = eJ,
        t.RemoteSolanaMobileWalletAdapterWallet = eX,
        t.SolanaMobileWalletAdapterWalletName = eZ,
        t.createDefaultAuthorizationCache = function() {
            let e;
            try {
                e = window.localStorage
            } catch (e) {}
            return {
                clear() {
                    return eP(this, void 0, void 0, function*() {
                        if (e)
                            try {
                                e.removeItem(e8)
                            } catch (e) {}
                    })
                },
                get() {
                    return eP(this, void 0, void 0, function*() {
                        if (e)
                            try {
                                let t = JSON.parse(e.getItem(e8));
                                if (!t || !t.accounts)
                                    return t || void 0;
                                {
                                    let e = t.accounts.map(e => Object.assign(Object.assign({}, e), {
                                        publicKey: "publicKey"in e ? new Uint8Array(Object.values(e.publicKey)) : eR.default.decode(e.address)
                                    }));
                                    return Object.assign(Object.assign({}, t), {
                                        accounts: e
                                    })
                                }
                            } catch (e) {}
                    })
                },
                set(t) {
                    return eP(this, void 0, void 0, function*() {
                        if (e)
                            try {
                                e.setItem(e8, JSON.stringify(t))
                            } catch (e) {}
                    })
                }
            }
        }
        ,
        t.createDefaultChainSelector = function() {
            return {
                select(e) {
                    return eP(this, void 0, void 0, function*() {
                        return 1 === e.length ? e[0] : e.includes(ex.SOLANA_MAINNET_CHAIN) ? ex.SOLANA_MAINNET_CHAIN : e[0]
                    })
                }
            }
        }
        ,
        t.createDefaultWalletNotFoundHandler = function() {
            return () => eP(this, void 0, void 0, function*() {
                e7()
            })
        }
        ,
        t.defaultErrorModalWalletNotFoundHandler = e7,
        t.registerMwa = function(e) {
            "undefined" != typeof window && window.isSecureContext && "undefined" != typeof document && /android/i.test(navigator.userAgent) ? e2(new eJ(e)) : "undefined" != typeof window && window.isSecureContext && "undefined" != typeof document && !/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) && void 0 !== e.remoteHostAuthority && e2(new eX(Object.assign(Object.assign({}, e), {
                remoteHostAuthority: e.remoteHostAuthority
            })))
        }
    }
    ,
    14661: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.Connect = t.StandardConnect = void 0,
        t.StandardConnect = "standard:connect",
        t.Connect = t.StandardConnect
    }
    ,
    16248: (e, t, n) => {
        "use strict";
        n.d(t, {
            _: () => S,
            e: () => A
        });
        var r = n(26432)
          , i = n(17394)
          , s = n(5049)
          , a = n(52718)
          , o = n(76603)
          , l = n(50748)
          , c = n(297)
          , d = n(21917)
          , u = n(94578)
          , h = n(15377)
          , p = n(80612)
          , f = n(4515)
          , y = n(89478);
        function m(e) {
            var t;
            return !!(e.enter || e.enterFrom || e.enterTo || e.leave || e.leaveFrom || e.leaveTo) || !(0,
            y.zv)(null != (t = e.as) ? t : E) || 1 === r.Children.count(e.children)
        }
        let g = (0,
        r.createContext)(null);
        g.displayName = "TransitionContext";
        var w = (e => (e.Visible = "visible",
        e.Hidden = "hidden",
        e))(w || {});
        let _ = (0,
        r.createContext)(null);
        function v(e) {
            return "children"in e ? v(e.children) : e.current.filter(e => {
                let {el: t} = e;
                return null !== t.current
            }
            ).filter(e => {
                let {state: t} = e;
                return "visible" === t
            }
            ).length > 0
        }
        function b(e, t) {
            let n = (0,
            l.Y)(e)
              , o = (0,
            r.useRef)([])
              , c = (0,
            a.a)()
              , d = (0,
            i.L)()
              , u = (0,
            s._)(function(e) {
                let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : y.mK.Hidden
                  , r = o.current.findIndex(t => {
                    let {el: n} = t;
                    return n === e
                }
                );
                -1 !== r && ((0,
                f.Y)(t, {
                    [y.mK.Unmount]() {
                        o.current.splice(r, 1)
                    },
                    [y.mK.Hidden]() {
                        o.current[r].state = "hidden"
                    }
                }),
                d.microTask( () => {
                    var e;
                    !v(o) && c.current && (null == (e = n.current) || e.call(n))
                }
                ))
            })
              , h = (0,
            s._)(e => {
                let t = o.current.find(t => {
                    let {el: n} = t;
                    return n === e
                }
                );
                return t ? "visible" !== t.state && (t.state = "visible") : o.current.push({
                    el: e,
                    state: "visible"
                }),
                () => u(e, y.mK.Unmount)
            }
            )
              , p = (0,
            r.useRef)([])
              , m = (0,
            r.useRef)(Promise.resolve())
              , g = (0,
            r.useRef)({
                enter: [],
                leave: []
            })
              , w = (0,
            s._)( (e, n, r) => {
                p.current.splice(0),
                t && (t.chains.current[n] = t.chains.current[n].filter(t => {
                    let[n] = t;
                    return n !== e
                }
                )),
                null == t || t.chains.current[n].push([e, new Promise(e => {
                    p.current.push(e)
                }
                )]),
                null == t || t.chains.current[n].push([e, new Promise(e => {
                    Promise.all(g.current[n].map(e => {
                        let[t,n] = e;
                        return n
                    }
                    )).then( () => e())
                }
                )]),
                "enter" === n ? m.current = m.current.then( () => null == t ? void 0 : t.wait.current).then( () => r(n)) : r(n)
            }
            )
              , _ = (0,
            s._)( (e, t, n) => {
                Promise.all(g.current[t].splice(0).map(e => {
                    let[t,n] = e;
                    return n
                }
                )).then( () => {
                    var e;
                    null == (e = p.current.shift()) || e()
                }
                ).then( () => n(t))
            }
            );
            return (0,
            r.useMemo)( () => ({
                children: o,
                register: h,
                unregister: u,
                onStart: w,
                onStop: _,
                wait: m,
                chains: g
            }), [h, u, o, w, _, g, m])
        }
        _.displayName = "NestingContext";
        let E = r.Fragment
          , I = y.Ac.RenderStrategy
          , k = (0,
        y.FX)(function(e, t) {
            let {show: n, appear: i=!1, unmount: a=!0, ...l} = e
              , u = (0,
            r.useRef)(null)
              , p = m(e)
              , f = (0,
            d.P)(...p ? [u, t] : null === t ? [] : [t]);
            (0,
            c.g)();
            let w = (0,
            h.O_)();
            if (void 0 === n && null !== w && (n = (w & h.Uw.Open) === h.Uw.Open),
            void 0 === n)
                throw Error("A <Transition /> is used but it is missing a `show={true | false}` prop.");
            let[E,k] = (0,
            r.useState)(n ? "visible" : "hidden")
              , S = b( () => {
                n || k("hidden")
            }
            )
              , [A,T] = (0,
            r.useState)(!0)
              , C = (0,
            r.useRef)([n]);
            (0,
            o.s)( () => {
                !1 !== A && C.current[C.current.length - 1] !== n && (C.current.push(n),
                T(!1))
            }
            , [C, n]);
            let O = (0,
            r.useMemo)( () => ({
                show: n,
                appear: i,
                initial: A
            }), [n, i, A]);
            (0,
            o.s)( () => {
                n ? k("visible") : v(S) || null === u.current || k("hidden")
            }
            , [n, S]);
            let N = {
                unmount: a
            }
              , L = (0,
            s._)( () => {
                var t;
                A && T(!1),
                null == (t = e.beforeEnter) || t.call(e)
            }
            )
              , x = (0,
            s._)( () => {
                var t;
                A && T(!1),
                null == (t = e.beforeLeave) || t.call(e)
            }
            )
              , j = (0,
            y.Ci)();
            return r.createElement(_.Provider, {
                value: S
            }, r.createElement(g.Provider, {
                value: O
            }, j({
                ourProps: {
                    ...N,
                    as: r.Fragment,
                    children: r.createElement(M, {
                        ref: f,
                        ...N,
                        ...l,
                        beforeEnter: L,
                        beforeLeave: x
                    })
                },
                theirProps: {},
                defaultTag: r.Fragment,
                features: I,
                visible: "visible" === E,
                name: "Transition"
            })))
        })
          , M = (0,
        y.FX)(function(e, t) {
            var n, i;
            let {transition: a=!0, beforeEnter: l, afterEnter: w, beforeLeave: k, afterLeave: M, enter: S, enterFrom: A, enterTo: T, entered: C, leave: O, leaveFrom: N, leaveTo: L, ...x} = e
              , [j,D] = (0,
            r.useState)(null)
              , R = (0,
            r.useRef)(null)
              , P = m(e)
              , W = (0,
            d.P)(...P ? [R, t, D] : null === t ? [] : [t])
              , U = null == (n = x.unmount) || n ? y.mK.Unmount : y.mK.Hidden
              , {show: z, appear: q, initial: F} = function() {
                let e = (0,
                r.useContext)(g);
                if (null === e)
                    throw Error("A <Transition.Child /> is used but it is missing a parent <Transition /> or <Transition.Root />.");
                return e
            }()
              , [B,H] = (0,
            r.useState)(z ? "visible" : "hidden")
              , K = function() {
                let e = (0,
                r.useContext)(_);
                if (null === e)
                    throw Error("A <Transition.Child /> is used but it is missing a parent <Transition /> or <Transition.Root />.");
                return e
            }()
              , {register: Q, unregister: V} = K;
            (0,
            o.s)( () => Q(R), [Q, R]),
            (0,
            o.s)( () => {
                if (U === y.mK.Hidden && R.current)
                    return z && "visible" !== B ? void H("visible") : (0,
                    f.Y)(B, {
                        hidden: () => V(R),
                        visible: () => Q(R)
                    })
            }
            , [B, R, Q, V, z, U]);
            let Y = (0,
            c.g)();
            (0,
            o.s)( () => {
                if (P && Y && "visible" === B && null === R.current)
                    throw Error("Did you forget to passthrough the `ref` to the actual DOM node?")
            }
            , [R, B, Y, P]);
            let G = F && !q
              , Z = q && z && F
              , $ = (0,
            r.useRef)(!1)
              , J = b( () => {
                $.current || (H("hidden"),
                V(R))
            }
            , K)
              , X = (0,
            s._)(e => {
                $.current = !0,
                J.onStart(R, e ? "enter" : "leave", e => {
                    "enter" === e ? null == l || l() : "leave" === e && (null == k || k())
                }
                )
            }
            )
              , ee = (0,
            s._)(e => {
                let t = e ? "enter" : "leave";
                $.current = !1,
                J.onStop(R, t, e => {
                    "enter" === e ? null == w || w() : "leave" === e && (null == M || M())
                }
                ),
                "leave" !== t || v(J) || (H("hidden"),
                V(R))
            }
            );
            (0,
            r.useEffect)( () => {
                P && a || (X(z),
                ee(z))
            }
            , [z, P, a]);
            let et = !(!a || !P || !Y || G)
              , [,en] = (0,
            u.p)(et, j, z, {
                start: X,
                end: ee
            })
              , er = (0,
            y.oE)({
                ref: W,
                className: (null == (i = (0,
                p.x)(x.className, Z && S, Z && A, en.enter && S, en.enter && en.closed && A, en.enter && !en.closed && T, en.leave && O, en.leave && !en.closed && N, en.leave && en.closed && L, !en.transition && z && C)) ? void 0 : i.trim()) || void 0,
                ...(0,
                u.B)(en)
            })
              , ei = 0;
            "visible" === B && (ei |= h.Uw.Open),
            "hidden" === B && (ei |= h.Uw.Closed),
            z && "hidden" === B && (ei |= h.Uw.Opening),
            z || "visible" !== B || (ei |= h.Uw.Closing);
            let es = (0,
            y.Ci)();
            return r.createElement(_.Provider, {
                value: J
            }, r.createElement(h.El, {
                value: ei
            }, es({
                ourProps: er,
                theirProps: x,
                defaultTag: E,
                features: I,
                visible: "visible" === B,
                name: "Transition.Child"
            })))
        })
          , S = (0,
        y.FX)(function(e, t) {
            let n = null !== (0,
            r.useContext)(g)
              , i = null !== (0,
            h.O_)();
            return r.createElement(r.Fragment, null, !n && i ? r.createElement(k, {
                ref: t,
                ...e
            }) : r.createElement(M, {
                ref: t,
                ...e
            }))
        })
          , A = Object.assign(k, {
            Child: S,
            Root: k
        })
    }
    ,
    17151: (e, t, n) => {
        "use strict";
        var r, i, s, a, o, l, c, d, u, h, p, f, y, m = n(81650), g = n(40476), w = n(69523), _ = n(12727);
        function v(e, t, n, r) {
            return new (n || (n = Promise))(function(i, s) {
                function a(e) {
                    try {
                        l(r.next(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function o(e) {
                    try {
                        l(r.throw(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function l(e) {
                    var t;
                    e.done ? i(e.value) : ((t = e.value)instanceof n ? t : new n(function(e) {
                        e(t)
                    }
                    )).then(a, o)
                }
                l((r = r.apply(e, t || [])).next())
            }
            )
        }
        function b(e, t, n, r) {
            if ("a" === n && !r)
                throw TypeError("Private accessor was defined without a getter");
            if ("function" == typeof t ? e !== t || !r : !t.has(e))
                throw TypeError("Cannot read private member from an object whose class did not declare it");
            return "m" === n ? r : "a" === n ? r.call(e) : r ? r.value : t.get(e)
        }
        function E(e, t, n, r, i) {
            if ("m" === r)
                throw TypeError("Private method is not writable");
            if ("a" === r && !i)
                throw TypeError("Private accessor was defined without a setter");
            if ("function" == typeof t ? e !== t || !i : !t.has(e))
                throw TypeError("Cannot write private member to an object whose class did not declare it");
            return "a" === r ? i.call(e, n) : i ? i.value = n : t.set(e, n),
            n
        }
        let I = "standard:connect";
        function k(e) {
            return window.btoa(String.fromCharCode.call(null, ...e))
        }
        function M(e) {
            switch (e) {
            case "mainnet-beta":
                return "solana:mainnet";
            case "testnet":
                return "solana:testnet";
            case "devnet":
                return "solana:devnet";
            default:
                return e
            }
        }
        class S extends m.BaseSignInMessageSignerWalletAdapter {
            constructor(e, t) {
                super(),
                r.add(this),
                this.supportedTransactionVersions = new Set(["legacy", 0]),
                i.set(this, void 0),
                s.set(this, !1),
                a.set(this, "undefined" != typeof window && window.isSecureContext && "undefined" != typeof document && /android/i.test(navigator.userAgent) ? m.WalletReadyState.Loadable : m.WalletReadyState.Unsupported),
                o.set(this, void 0),
                l.set(this, void 0),
                c.set(this, void 0),
                d.set(this, e => v(this, void 0, void 0, function*() {
                    if (e.accounts && e.accounts.length > 0) {
                        b(this, r, "m", h).call(this);
                        let t = yield b(this, o, "f").call(this, e.accounts);
                        t !== b(this, l, "f") && (E(this, l, t, "f"),
                        E(this, c, void 0, "f"),
                        this.emit("connect", this.publicKey))
                    }
                })),
                E(this, o, e => v(this, void 0, void 0, function*() {
                    var n;
                    let r = yield t.addressSelector.select(e.map( ({publicKey: e}) => k(e)));
                    return null != (n = e.find( ({publicKey: e}) => k(e) === r)) ? n : e[0]
                }), "f"),
                E(this, i, e, "f"),
                b(this, i, "f").features["standard:events"].on("change", b(this, d, "f")),
                this.name = b(this, i, "f").name,
                this.icon = b(this, i, "f").icon,
                this.url = b(this, i, "f").url
            }
            get publicKey() {
                var e;
                if (!b(this, c, "f") && b(this, l, "f"))
                    try {
                        E(this, c, new g.PublicKey(b(this, l, "f").publicKey), "f")
                    } catch (e) {
                        throw new m.WalletPublicKeyError(e instanceof Error && (null == e ? void 0 : e.message) || "Unknown error",e)
                    }
                return null != (e = b(this, c, "f")) ? e : null
            }
            get connected() {
                return b(this, i, "f").connected
            }
            get connecting() {
                return b(this, s, "f")
            }
            get readyState() {
                return b(this, a, "f")
            }
            autoConnect_DO_NOT_USE_OR_YOU_WILL_BE_FIRED() {
                return v(this, void 0, void 0, function*() {
                    return yield this.autoConnect()
                })
            }
            autoConnect() {
                return v(this, void 0, void 0, function*() {
                    b(this, r, "m", u).call(this, !0)
                })
            }
            connect() {
                return v(this, void 0, void 0, function*() {
                    b(this, r, "m", u).call(this)
                })
            }
            performAuthorization(e) {
                return v(this, void 0, void 0, function*() {
                    try {
                        let t = yield b(this, i, "f").cachedAuthorizationResult;
                        if (t)
                            return yield b(this, i, "f").features[I].connect({
                                silent: !0
                            }),
                            t;
                        return e ? yield b(this, i, "f").features[w.SolanaSignIn].signIn(e) : yield b(this, i, "f").features[I].connect(),
                        yield yield b(this, i, "f").cachedAuthorizationResult
                    } catch (e) {
                        throw new m.WalletConnectionError(e instanceof Error && e.message || "Unknown error",e)
                    }
                })
            }
            disconnect() {
                return v(this, void 0, void 0, function*() {
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        E(this, s, !1, "f"),
                        E(this, c, void 0, "f"),
                        E(this, l, void 0, "f"),
                        yield b(this, i, "f").features["standard:disconnect"].disconnect(),
                        this.emit("disconnect")
                    }))
                })
            }
            signIn(e) {
                return v(this, void 0, void 0, function*() {
                    return b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        var t;
                        if (b(this, a, "f") !== m.WalletReadyState.Installed && b(this, a, "f") !== m.WalletReadyState.Loadable)
                            throw new m.WalletNotReadyError;
                        E(this, s, !0, "f");
                        try {
                            let n = yield b(this, i, "f").features[w.SolanaSignIn].signIn(Object.assign(Object.assign({}, e), {
                                domain: null != (t = null == e ? void 0 : e.domain) ? t : window.location.host
                            }));
                            if (n.length > 0)
                                return n[0];
                            throw Error("Sign in failed, no sign in result returned by wallet")
                        } catch (e) {
                            throw new m.WalletConnectionError(e instanceof Error && e.message || "Unknown error",e)
                        } finally {
                            E(this, s, !1, "f")
                        }
                    }))
                })
            }
            signMessage(e) {
                return v(this, void 0, void 0, function*() {
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        let t = b(this, r, "m", p).call(this);
                        try {
                            return (yield b(this, i, "f").features[w.SolanaSignMessage].signMessage({
                                account: t,
                                message: e
                            }))[0].signature
                        } catch (e) {
                            throw new m.WalletSignMessageError(null == e ? void 0 : e.message,e)
                        }
                    }))
                })
            }
            sendTransaction(e, t, n) {
                return v(this, void 0, void 0, function*() {
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        let s = b(this, r, "m", p).call(this);
                        try {
                            if (w.SolanaSignAndSendTransaction in b(this, i, "f").features) {
                                let t = M(b(this, i, "f").currentAuthorization.chain)
                                  , [r] = (yield b(this, i, "f").features[w.SolanaSignAndSendTransaction].signAndSendTransaction({
                                    account: s,
                                    transaction: e.serialize(),
                                    chain: t,
                                    options: n ? {
                                        skipPreflight: n.skipPreflight,
                                        maxRetries: n.maxRetries
                                    } : void 0
                                })).map(e => k(e.signature));
                                return r
                            }
                            {
                                let[i] = yield b(this, r, "m", f).call(this, [e]);
                                if ("version"in i)
                                    return yield t.sendTransaction(i);
                                {
                                    let e = i.serialize();
                                    return yield t.sendRawTransaction(e, Object.assign(Object.assign({}, n), {
                                        preflightCommitment: function() {
                                            let e, r;
                                            switch (t.commitment) {
                                            case "confirmed":
                                            case "finalized":
                                            case "processed":
                                                e = t.commitment;
                                                break;
                                            default:
                                                e = "finalized"
                                            }
                                            switch (null == n ? void 0 : n.preflightCommitment) {
                                            case "confirmed":
                                            case "finalized":
                                            case "processed":
                                                r = n.preflightCommitment;
                                                break;
                                            case void 0:
                                                r = e;
                                                break;
                                            default:
                                                r = "finalized"
                                            }
                                            let i = "finalized" === r ? 2 : +("confirmed" === r)
                                              , s = "finalized" === e ? 2 : +("confirmed" === e);
                                            return i < s ? r : e
                                        }()
                                    }))
                                }
                            }
                        } catch (e) {
                            throw new m.WalletSendTransactionError(null == e ? void 0 : e.message,e)
                        }
                    }))
                })
            }
            signTransaction(e) {
                return v(this, void 0, void 0, function*() {
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        let[t] = yield b(this, r, "m", f).call(this, [e]);
                        return t
                    }))
                })
            }
            signAllTransactions(e) {
                return v(this, void 0, void 0, function*() {
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        return yield b(this, r, "m", f).call(this, e)
                    }))
                })
            }
        }
        i = new WeakMap,
        s = new WeakMap,
        a = new WeakMap,
        o = new WeakMap,
        l = new WeakMap,
        c = new WeakMap,
        d = new WeakMap,
        r = new WeakSet,
        u = function(e=!1) {
            return v(this, void 0, void 0, function*() {
                if (!this.connecting && !this.connected)
                    return yield b(this, r, "m", y).call(this, () => v(this, void 0, void 0, function*() {
                        if (b(this, a, "f") !== m.WalletReadyState.Installed && b(this, a, "f") !== m.WalletReadyState.Loadable)
                            throw new m.WalletNotReadyError;
                        E(this, s, !0, "f");
                        try {
                            yield b(this, i, "f").features[I].connect({
                                silent: e
                            })
                        } catch (e) {
                            throw new m.WalletConnectionError(e instanceof Error && e.message || "Unknown error",e)
                        } finally {
                            E(this, s, !1, "f")
                        }
                    }))
            })
        }
        ,
        h = function() {
            b(this, a, "f") !== m.WalletReadyState.Installed && this.emit("readyStateChange", E(this, a, m.WalletReadyState.Installed, "f"))
        }
        ,
        p = function() {
            if (!b(this, i, "f").isAuthorized || !b(this, l, "f"))
                throw new m.WalletNotConnectedError;
            return b(this, l, "f")
        }
        ,
        f = function(e) {
            return v(this, void 0, void 0, function*() {
                let t = b(this, r, "m", p).call(this);
                try {
                    if (w.SolanaSignTransaction in b(this, i, "f").features)
                        return b(this, i, "f").features[w.SolanaSignTransaction].signTransaction(...e.map(e => ({
                            account: t,
                            transaction: e.serialize()
                        }))).then(e => e.map(e => {
                            let t = e.signedTransaction
                              , n = t[0]
                              , r = g.VersionedMessage.deserializeMessageVersion(t.slice(64 * n + 1, t.length));
                            return "legacy" === r ? g.Transaction.from(t) : g.VersionedTransaction.deserialize(t)
                        }
                        ));
                    throw Error("Connected wallet does not support signing transactions")
                } catch (e) {
                    throw new m.WalletSignTransactionError(null == e ? void 0 : e.message,e)
                }
            })
        }
        ,
        y = function(e) {
            return v(this, void 0, void 0, function*() {
                try {
                    return yield e()
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            })
        }
        ;
        class A extends S {
            constructor(e) {
                var t;
                let n = M(null != (t = e.chain) ? t : e.cluster);
                super(new _.LocalSolanaMobileWalletAdapterWallet({
                    appIdentity: e.appIdentity,
                    authorizationCache: {
                        set: e.authorizationResultCache.set,
                        get: () => v(this, void 0, void 0, function*() {
                            return yield e.authorizationResultCache.get()
                        }),
                        clear: e.authorizationResultCache.clear
                    },
                    chains: [n],
                    chainSelector: _.createDefaultChainSelector(),
                    onWalletNotFound: () => v(this, void 0, void 0, function*() {
                        e.onWalletNotFound(this)
                    })
                }), {
                    addressSelector: e.addressSelector,
                    chain: n
                })
            }
        }
        class T extends A {
        }
        function C(e) {
            return v(this, void 0, void 0, function*() {
                return _.defaultErrorModalWalletNotFoundHandler()
            })
        }
        t.Ne = T,
        t.fG = "Mobile Wallet Adapter",
        t.RP = function() {
            return {
                select(e) {
                    return v(this, void 0, void 0, function*() {
                        return e[0]
                    })
                }
            }
        }
        ,
        t.u = function() {
            return _.createDefaultAuthorizationCache()
        }
        ,
        t.O0 = function() {
            return C
        }
    }
    ,
    19687: (e, t, n) => {
        "use strict";
        n.d(t, {
            c: () => l
        });
        var r = n(50901)
          , i = n(72286)
          , s = n(17972)
          , a = n(89037)
          , o = n(40476);
        class l extends r.DE {
            constructor(e={}) {
                super(),
                this.name = "Phantom",
                this.url = "https://phantom.app",
                this.icon = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxMDgiIGhlaWdodD0iMTA4IiB2aWV3Qm94PSIwIDAgMTA4IDEwOCIgZmlsbD0ibm9uZSI+CjxyZWN0IHdpZHRoPSIxMDgiIGhlaWdodD0iMTA4IiByeD0iMjYiIGZpbGw9IiNBQjlGRjIiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik00Ni41MjY3IDY5LjkyMjlDNDIuMDA1NCA3Ni44NTA5IDM0LjQyOTIgODUuNjE4MiAyNC4zNDggODUuNjE4MkMxOS41ODI0IDg1LjYxODIgMTUgODMuNjU2MyAxNSA3NS4xMzQyQzE1IDUzLjQzMDUgNDQuNjMyNiAxOS44MzI3IDcyLjEyNjggMTkuODMyN0M4Ny43NjggMTkuODMyNyA5NCAzMC42ODQ2IDk0IDQzLjAwNzlDOTQgNTguODI1OCA4My43MzU1IDc2LjkxMjIgNzMuNTMyMSA3Ni45MTIyQzcwLjI5MzkgNzYuOTEyMiA2OC43MDUzIDc1LjEzNDIgNjguNzA1MyA3Mi4zMTRDNjguNzA1MyA3MS41NzgzIDY4LjgyNzUgNzAuNzgxMiA2OS4wNzE5IDY5LjkyMjlDNjUuNTg5MyA3NS44Njk5IDU4Ljg2ODUgODEuMzg3OCA1Mi41NzU0IDgxLjM4NzhDNDcuOTkzIDgxLjM4NzggNDUuNjcxMyA3OC41MDYzIDQ1LjY3MTMgNzQuNDU5OEM0NS42NzEzIDcyLjk4ODQgNDUuOTc2OCA3MS40NTU2IDQ2LjUyNjcgNjkuOTIyOVpNODMuNjc2MSA0Mi41Nzk0QzgzLjY3NjEgNDYuMTcwNCA4MS41NTc1IDQ3Ljk2NTggNzkuMTg3NSA0Ny45NjU4Qzc2Ljc4MTYgNDcuOTY1OCA3NC42OTg5IDQ2LjE3MDQgNzQuNjk4OSA0Mi41Nzk0Qzc0LjY5ODkgMzguOTg4NSA3Ni43ODE2IDM3LjE5MzEgNzkuMTg3NSAzNy4xOTMxQzgxLjU1NzUgMzcuMTkzMSA4My42NzYxIDM4Ljk4ODUgODMuNjc2MSA0Mi41Nzk0Wk03MC4yMTAzIDQyLjU3OTVDNzAuMjEwMyA0Ni4xNzA0IDY4LjA5MTYgNDcuOTY1OCA2NS43MjE2IDQ3Ljk2NThDNjMuMzE1NyA0Ny45NjU4IDYxLjIzMyA0Ni4xNzA0IDYxLjIzMyA0Mi41Nzk1QzYxLjIzMyAzOC45ODg1IDYzLjMxNTcgMzcuMTkzMSA2NS43MjE2IDM3LjE5MzFDNjguMDkxNiAzNy4xOTMxIDcwLjIxMDMgMzguOTg4NSA3MC4yMTAzIDQyLjU3OTVaIiBmaWxsPSIjRkZGREY4Ii8+Cjwvc3ZnPg==",
                this.supportedTransactionVersions = new Set(["legacy", 0]),
                this._readyState = "undefined" == typeof window || "undefined" == typeof document ? i.Ok.Unsupported : i.Ok.NotDetected,
                this._disconnected = () => {
                    let e = this._wallet;
                    e && (e.off("disconnect", this._disconnected),
                    e.off("accountChanged", this._accountChanged),
                    this._wallet = null,
                    this._publicKey = null,
                    this.emit("error", new s.PQ),
                    this.emit("disconnect"))
                }
                ,
                this._accountChanged = e => {
                    let t = this._publicKey;
                    if (t) {
                        try {
                            e = new o.PublicKey(e.toBytes())
                        } catch (e) {
                            this.emit("error", new s.Kd(e?.message,e));
                            return
                        }
                        t.equals(e) || (this._publicKey = e,
                        this.emit("connect", e))
                    }
                }
                ,
                this._connecting = !1,
                this._wallet = null,
                this._publicKey = null,
                this._readyState !== i.Ok.Unsupported && ((0,
                i.Br)() ? (this._readyState = i.Ok.Loadable,
                this.emit("readyStateChange", this._readyState)) : (0,
                i.qG)( () => !!(window.phantom?.solana?.isPhantom || window.solana?.isPhantom) && (this._readyState = i.Ok.Installed,
                this.emit("readyStateChange", this._readyState),
                !0)))
            }
            get publicKey() {
                return this._publicKey
            }
            get connecting() {
                return this._connecting
            }
            get readyState() {
                return this._readyState
            }
            async autoConnect() {
                this.readyState === i.Ok.Installed && await this.connect()
            }
            async connect() {
                try {
                    let e;
                    if (this.connected || this.connecting)
                        return;
                    if (this.readyState === i.Ok.Loadable) {
                        let e = encodeURIComponent(window.location.href)
                          , t = encodeURIComponent(window.location.origin);
                        window.location.href = `https://phantom.app/ul/browse/${e}?ref=${t}`;
                        return
                    }
                    if (this.readyState !== i.Ok.Installed)
                        throw new s.AE;
                    this._connecting = !0;
                    let t = window.phantom?.solana || window.solana;
                    if (!t.isConnected)
                        try {
                            await t.connect()
                        } catch (e) {
                            throw new s.Y6(e?.message,e)
                        }
                    if (!t.publicKey)
                        throw new s.fk;
                    try {
                        e = new o.PublicKey(t.publicKey.toBytes())
                    } catch (e) {
                        throw new s.Kd(e?.message,e)
                    }
                    t.on("disconnect", this._disconnected),
                    t.on("accountChanged", this._accountChanged),
                    this._wallet = t,
                    this._publicKey = e,
                    this.emit("connect", e)
                } catch (e) {
                    throw this.emit("error", e),
                    e
                } finally {
                    this._connecting = !1
                }
            }
            async disconnect() {
                let e = this._wallet;
                if (e) {
                    e.off("disconnect", this._disconnected),
                    e.off("accountChanged", this._accountChanged),
                    this._wallet = null,
                    this._publicKey = null;
                    try {
                        await e.disconnect()
                    } catch (e) {
                        this.emit("error", new s.Y8(e?.message,e))
                    }
                }
                this.emit("disconnect")
            }
            async sendTransaction(e, t, n={}) {
                try {
                    let r = this._wallet;
                    if (!r)
                        throw new s.kW;
                    try {
                        let {signers: i, ...s} = n;
                        (0,
                        a.Y)(e) ? i?.length && e.sign(i) : (e = await this.prepareTransaction(e, t, s),
                        i?.length && e.partialSign(...i)),
                        s.preflightCommitment = s.preflightCommitment || t.commitment;
                        let {signature: o} = await r.signAndSendTransaction(e, s);
                        return o
                    } catch (e) {
                        if (e instanceof s.m7)
                            throw e;
                        throw new s.UF(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signTransaction(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        return await t.signTransaction(e) || e
                    } catch (e) {
                        throw new s.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signAllTransactions(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        return await t.signAllTransactions(e) || e
                    } catch (e) {
                        throw new s.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signMessage(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        let {signature: n} = await t.signMessage(e);
                        return n
                    } catch (e) {
                        throw new s.K3(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
        }
    }
    ,
    20627: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => r
        });
        let r = (0,
        n(44074).A)("CloudUpload", [["path", {
            d: "M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242",
            key: "1pljnt"
        }], ["path", {
            d: "M12 12v9",
            key: "192myk"
        }], ["path", {
            d: "m16 16-4-4-4 4",
            key: "119tzi"
        }]])
    }
    ,
    20638: e => {
        e.exports = {
            style: {
                fontFamily: "'Geist', 'Geist Fallback'",
                fontStyle: "normal"
            },
            className: "__className_188709",
            variable: "__variable_188709"
        }
    }
    ,
    21715: (e, t, n) => {
        "use strict";
        n.d(t, {
            q: () => r
        });
        let r = "solana:signTransaction"
    }
    ,
    22482: e => {
        "use strict";
        e.exports = function(e) {
            if (e.length >= 255)
                throw TypeError("Alphabet too long");
            for (var t = new Uint8Array(256), n = 0; n < t.length; n++)
                t[n] = 255;
            for (var r = 0; r < e.length; r++) {
                var i = e.charAt(r)
                  , s = i.charCodeAt(0);
                if (255 !== t[s])
                    throw TypeError(i + " is ambiguous");
                t[s] = r
            }
            var a = e.length
              , o = e.charAt(0)
              , l = Math.log(a) / Math.log(256)
              , c = Math.log(256) / Math.log(a);
            function d(e) {
                if ("string" != typeof e)
                    throw TypeError("Expected String");
                if (0 === e.length)
                    return new Uint8Array;
                for (var n = 0, r = 0, i = 0; e[n] === o; )
                    r++,
                    n++;
                for (var s = (e.length - n) * l + 1 >>> 0, c = new Uint8Array(s); e[n]; ) {
                    var d = e.charCodeAt(n);
                    if (d > 255)
                        return;
                    var u = t[d];
                    if (255 === u)
                        return;
                    for (var h = 0, p = s - 1; (0 !== u || h < i) && -1 !== p; p--,
                    h++)
                        u += a * c[p] >>> 0,
                        c[p] = u % 256 >>> 0,
                        u = u / 256 >>> 0;
                    if (0 !== u)
                        throw Error("Non-zero carry");
                    i = h,
                    n++
                }
                for (var f = s - i; f !== s && 0 === c[f]; )
                    f++;
                for (var y = new Uint8Array(r + (s - f)), m = r; f !== s; )
                    y[m++] = c[f++];
                return y
            }
            return {
                encode: function(t) {
                    if (t instanceof Uint8Array || (ArrayBuffer.isView(t) ? t = new Uint8Array(t.buffer,t.byteOffset,t.byteLength) : Array.isArray(t) && (t = Uint8Array.from(t))),
                    !(t instanceof Uint8Array))
                        throw TypeError("Expected Uint8Array");
                    if (0 === t.length)
                        return "";
                    for (var n = 0, r = 0, i = 0, s = t.length; i !== s && 0 === t[i]; )
                        i++,
                        n++;
                    for (var l = (s - i) * c + 1 >>> 0, d = new Uint8Array(l); i !== s; ) {
                        for (var u = t[i], h = 0, p = l - 1; (0 !== u || h < r) && -1 !== p; p--,
                        h++)
                            u += 256 * d[p] >>> 0,
                            d[p] = u % a >>> 0,
                            u = u / a >>> 0;
                        if (0 !== u)
                            throw Error("Non-zero carry");
                        r = h,
                        i++
                    }
                    for (var f = l - r; f !== l && 0 === d[f]; )
                        f++;
                    for (var y = o.repeat(n); f < l; ++f)
                        y += e.charAt(d[f]);
                    return y
                },
                decodeUnsafe: d,
                decode: function(e) {
                    var t = d(e);
                    if (t)
                        return t;
                    throw Error("Non-base" + a + " character")
                }
            }
        }
    }
    ,
    23713: (e, t, n) => {
        "use strict";
        n.d(t, {
            j: () => r
        });
        let r = "standard:events"
    }
    ,
    24636: (e, t, n) => {
        "use strict";
        n.d(t, {
            _: () => tQ
        });
        class r {
            constructor(e, t) {
                this.scope = e,
                this.module = t
            }
            storeObject(e, t) {
                this.setItem(e, JSON.stringify(t))
            }
            loadObject(e) {
                let t = this.getItem(e);
                return t ? JSON.parse(t) : void 0
            }
            setItem(e, t) {
                localStorage.setItem(this.scopedKey(e), t)
            }
            getItem(e) {
                return localStorage.getItem(this.scopedKey(e))
            }
            removeItem(e) {
                localStorage.removeItem(this.scopedKey(e))
            }
            clear() {
                let e = this.scopedKey("")
                  , t = [];
                for (let n = 0; n < localStorage.length; n++) {
                    let r = localStorage.key(n);
                    "string" == typeof r && r.startsWith(e) && t.push(r)
                }
                t.forEach(e => localStorage.removeItem(e))
            }
            scopedKey(e) {
                return `-${this.scope}${this.module ? `:${this.module}` : ""}:${e}`
            }
            static clearAll() {
                new r("CBWSDK").clear(),
                new r("walletlink").clear()
            }
        }
        let i = {
            rpc: {
                invalidInput: -32e3,
                resourceNotFound: -32001,
                resourceUnavailable: -32002,
                transactionRejected: -32003,
                methodNotSupported: -32004,
                limitExceeded: -32005,
                parse: -32700,
                invalidRequest: -32600,
                methodNotFound: -32601,
                invalidParams: -32602,
                internal: -32603
            },
            provider: {
                userRejectedRequest: 4001,
                unauthorized: 4100,
                unsupportedMethod: 4200,
                disconnected: 4900,
                chainDisconnected: 4901,
                unsupportedChain: 4902
            }
        }
          , s = {
            "-32700": {
                standard: "JSON RPC 2.0",
                message: "Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text."
            },
            "-32600": {
                standard: "JSON RPC 2.0",
                message: "The JSON sent is not a valid Request object."
            },
            "-32601": {
                standard: "JSON RPC 2.0",
                message: "The method does not exist / is not available."
            },
            "-32602": {
                standard: "JSON RPC 2.0",
                message: "Invalid method parameter(s)."
            },
            "-32603": {
                standard: "JSON RPC 2.0",
                message: "Internal JSON-RPC error."
            },
            "-32000": {
                standard: "EIP-1474",
                message: "Invalid input."
            },
            "-32001": {
                standard: "EIP-1474",
                message: "Resource not found."
            },
            "-32002": {
                standard: "EIP-1474",
                message: "Resource unavailable."
            },
            "-32003": {
                standard: "EIP-1474",
                message: "Transaction rejected."
            },
            "-32004": {
                standard: "EIP-1474",
                message: "Method not supported."
            },
            "-32005": {
                standard: "EIP-1474",
                message: "Request limit exceeded."
            },
            4001: {
                standard: "EIP-1193",
                message: "User rejected the request."
            },
            4100: {
                standard: "EIP-1193",
                message: "The requested account and/or method has not been authorized by the user."
            },
            4200: {
                standard: "EIP-1193",
                message: "The requested method is not supported by this Ethereum provider."
            },
            4900: {
                standard: "EIP-1193",
                message: "The provider is disconnected from all chains."
            },
            4901: {
                standard: "EIP-1193",
                message: "The provider is disconnected from the specified chain."
            },
            4902: {
                standard: "EIP-3085",
                message: "Unrecognized chain ID."
            }
        }
          , a = "Unspecified error message.";
        function o(e, t=a) {
            if (e && Number.isInteger(e)) {
                var n;
                let t = e.toString();
                if (c(s, t))
                    return s[t].message;
                if ((n = e) >= -32099 && n <= -32e3)
                    return "Unspecified server error."
            }
            return t
        }
        function l(e) {
            return e && "object" == typeof e && !Array.isArray(e) ? Object.assign({}, e) : e
        }
        function c(e, t) {
            return Object.prototype.hasOwnProperty.call(e, t)
        }
        function d(e, t) {
            return "object" == typeof e && null !== e && t in e && "string" == typeof e[t]
        }
        let u = {
            rpc: {
                parse: e => h(i.rpc.parse, e),
                invalidRequest: e => h(i.rpc.invalidRequest, e),
                invalidParams: e => h(i.rpc.invalidParams, e),
                methodNotFound: e => h(i.rpc.methodNotFound, e),
                internal: e => h(i.rpc.internal, e),
                server: e => {
                    if (!e || "object" != typeof e || Array.isArray(e))
                        throw Error("Ethereum RPC Server errors must provide single object argument.");
                    let {code: t} = e;
                    if (!Number.isInteger(t) || t > -32005 || t < -32099)
                        throw Error('"code" must be an integer such that: -32099 <= code <= -32005');
                    return h(t, e)
                }
                ,
                invalidInput: e => h(i.rpc.invalidInput, e),
                resourceNotFound: e => h(i.rpc.resourceNotFound, e),
                resourceUnavailable: e => h(i.rpc.resourceUnavailable, e),
                transactionRejected: e => h(i.rpc.transactionRejected, e),
                methodNotSupported: e => h(i.rpc.methodNotSupported, e),
                limitExceeded: e => h(i.rpc.limitExceeded, e)
            },
            provider: {
                userRejectedRequest: e => p(i.provider.userRejectedRequest, e),
                unauthorized: e => p(i.provider.unauthorized, e),
                unsupportedMethod: e => p(i.provider.unsupportedMethod, e),
                disconnected: e => p(i.provider.disconnected, e),
                chainDisconnected: e => p(i.provider.chainDisconnected, e),
                unsupportedChain: e => p(i.provider.unsupportedChain, e),
                custom: e => {
                    if (!e || "object" != typeof e || Array.isArray(e))
                        throw Error("Ethereum Provider custom errors must provide single object argument.");
                    let {code: t, message: n, data: r} = e;
                    if (!n || "string" != typeof n)
                        throw Error('"message" must be a nonempty string');
                    return new m(t,n,r)
                }
            }
        };
        function h(e, t) {
            let[n,r] = f(t);
            return new y(e,n || o(e),r)
        }
        function p(e, t) {
            let[n,r] = f(t);
            return new m(e,n || o(e),r)
        }
        function f(e) {
            if (e) {
                if ("string" == typeof e)
                    return [e];
                else if ("object" == typeof e && !Array.isArray(e)) {
                    let {message: t, data: n} = e;
                    if (t && "string" != typeof t)
                        throw Error("Must specify string message.");
                    return [t || void 0, n]
                }
            }
            return []
        }
        class y extends Error {
            constructor(e, t, n) {
                if (!Number.isInteger(e))
                    throw Error('"code" must be an integer.');
                if (!t || "string" != typeof t)
                    throw Error('"message" must be a nonempty string.');
                super(t),
                this.code = e,
                void 0 !== n && (this.data = n)
            }
        }
        class m extends y {
            constructor(e, t, n) {
                if (!function(e) {
                    return Number.isInteger(e) && e >= 1e3 && e <= 4999
                }(e))
                    throw Error('"code" must be an integer such that: 1000 <= code <= 4999');
                super(e, t, n)
            }
        }
        let g = e => e
          , w = e => e
          , _ = e => e;
        function v(e) {
            return Math.floor(e)
        }
        var b = n(91015).Buffer;
        let E = /^[0-9]*$/
          , I = /^[a-f0-9]*$/;
        function k(e) {
            return M(crypto.getRandomValues(new Uint8Array(e)))
        }
        function M(e) {
            return [...e].map(e => e.toString(16).padStart(2, "0")).join("")
        }
        function S(e) {
            return new Uint8Array(e.match(/.{1,2}/g).map(e => Number.parseInt(e, 16)))
        }
        function A(e, t=!1) {
            let n = e.toString("hex");
            return g(t ? `0x${n}` : n)
        }
        function T(e) {
            return A(P(e), !0)
        }
        function C(e) {
            return _(e.toString(10))
        }
        function O(e) {
            return g(`0x${BigInt(e).toString(16)}`)
        }
        function N(e) {
            return e.startsWith("0x") || e.startsWith("0X")
        }
        function L(e) {
            return N(e) ? e.slice(2) : e
        }
        function x(e) {
            return N(e) ? `0x${e.slice(2)}` : `0x${e}`
        }
        function j(e) {
            if ("string" != typeof e)
                return !1;
            let t = L(e).toLowerCase();
            return I.test(t)
        }
        function D(e, t=!1) {
            let n = function(e, t=!1) {
                if ("string" == typeof e) {
                    let n = L(e).toLowerCase();
                    if (I.test(n))
                        return g(t ? `0x${n}` : n)
                }
                throw u.rpc.invalidParams(`"${String(e)}" is not a hexadecimal string`)
            }(e, !1);
            return n.length % 2 == 1 && (n = g(`0${n}`)),
            t ? g(`0x${n}`) : n
        }
        function R(e) {
            if ("string" == typeof e) {
                let t = L(e).toLowerCase();
                if (j(t) && 40 === t.length)
                    return w(x(t))
            }
            throw u.rpc.invalidParams(`Invalid Ethereum address: ${String(e)}`)
        }
        function P(e) {
            if (b.isBuffer(e))
                return e;
            if ("string" == typeof e) {
                if (j(e)) {
                    let t = D(e, !1);
                    return b.from(t, "hex")
                }
                return b.from(e, "utf8")
            }
            throw u.rpc.invalidParams(`Not binary data: ${String(e)}`)
        }
        function W(e) {
            if ("number" == typeof e && Number.isInteger(e))
                return v(e);
            if ("string" == typeof e) {
                if (E.test(e))
                    return v(Number(e));
                if (j(e))
                    return v(Number(BigInt(D(e, !0))))
            }
            throw u.rpc.invalidParams(`Not an integer: ${String(e)}`)
        }
        function U(e) {
            if (null !== e && ("bigint" == typeof e || function(e) {
                if (null == e || "function" != typeof e.constructor)
                    return !1;
                let {constructor: t} = e;
                return "function" == typeof t.config && "number" == typeof t.EUCLID
            }(e)))
                return BigInt(e.toString(10));
            if ("number" == typeof e)
                return BigInt(W(e));
            if ("string" == typeof e) {
                if (E.test(e))
                    return BigInt(e);
                if (j(e))
                    return BigInt(D(e, !0))
            }
            throw u.rpc.invalidParams(`Not an integer: ${String(e)}`)
        }
        async function z() {
            return crypto.subtle.generateKey({
                name: "ECDH",
                namedCurve: "P-256"
            }, !0, ["deriveKey"])
        }
        async function q(e, t) {
            return crypto.subtle.deriveKey({
                name: "ECDH",
                public: t
            }, e, {
                name: "AES-GCM",
                length: 256
            }, !1, ["encrypt", "decrypt"])
        }
        async function F(e, t) {
            let n = crypto.getRandomValues(new Uint8Array(12))
              , r = await crypto.subtle.encrypt({
                name: "AES-GCM",
                iv: n
            }, e, new TextEncoder().encode(t));
            return {
                iv: n,
                cipherText: r
            }
        }
        async function B(e, {iv: t, cipherText: n}) {
            let r = await crypto.subtle.decrypt({
                name: "AES-GCM",
                iv: t
            }, e, n);
            return new TextDecoder().decode(r)
        }
        function H(e) {
            switch (e) {
            case "public":
                return "spki";
            case "private":
                return "pkcs8"
            }
        }
        async function K(e, t) {
            let n = H(e);
            return M(new Uint8Array(await crypto.subtle.exportKey(n, t)))
        }
        async function Q(e, t) {
            let n = H(e)
              , r = S(t).buffer;
            return await crypto.subtle.importKey(n, new Uint8Array(r), {
                name: "ECDH",
                namedCurve: "P-256"
            }, !0, "private" === e ? ["deriveKey"] : [])
        }
        async function V(e, t) {
            return F(t, JSON.stringify(e, (e, t) => t instanceof Error ? Object.assign(Object.assign({}, t.code ? {
                code: t.code
            } : {}), {
                message: t.message
            }) : t))
        }
        async function Y(e, t) {
            return JSON.parse(await B(t, e))
        }
        let G = {
            storageKey: "ownPrivateKey",
            keyType: "private"
        }
          , Z = {
            storageKey: "ownPublicKey",
            keyType: "public"
        }
          , $ = {
            storageKey: "peerPublicKey",
            keyType: "public"
        };
        class J {
            constructor() {
                this.storage = new r("CBWSDK","SCWKeyManager"),
                this.ownPrivateKey = null,
                this.ownPublicKey = null,
                this.peerPublicKey = null,
                this.sharedSecret = null
            }
            async getOwnPublicKey() {
                return await this.loadKeysIfNeeded(),
                this.ownPublicKey
            }
            async getSharedSecret() {
                return await this.loadKeysIfNeeded(),
                this.sharedSecret
            }
            async setPeerPublicKey(e) {
                this.sharedSecret = null,
                this.peerPublicKey = e,
                await this.storeKey($, e),
                await this.loadKeysIfNeeded()
            }
            async clear() {
                this.ownPrivateKey = null,
                this.ownPublicKey = null,
                this.peerPublicKey = null,
                this.sharedSecret = null,
                this.storage.removeItem(Z.storageKey),
                this.storage.removeItem(G.storageKey),
                this.storage.removeItem($.storageKey)
            }
            async generateKeyPair() {
                let e = await z();
                this.ownPrivateKey = e.privateKey,
                this.ownPublicKey = e.publicKey,
                await this.storeKey(G, e.privateKey),
                await this.storeKey(Z, e.publicKey)
            }
            async loadKeysIfNeeded() {
                null === this.ownPrivateKey && (this.ownPrivateKey = await this.loadKey(G)),
                null === this.ownPublicKey && (this.ownPublicKey = await this.loadKey(Z)),
                (null === this.ownPrivateKey || null === this.ownPublicKey) && await this.generateKeyPair(),
                null === this.peerPublicKey && (this.peerPublicKey = await this.loadKey($)),
                null === this.sharedSecret && null !== this.ownPrivateKey && null !== this.peerPublicKey && (this.sharedSecret = await q(this.ownPrivateKey, this.peerPublicKey))
            }
            async loadKey(e) {
                let t = this.storage.getItem(e.storageKey);
                return t ? Q(e.keyType, t) : null
            }
            async storeKey(e, t) {
                let n = await K(e.keyType, t);
                this.storage.setItem(e.storageKey, n)
            }
        }
        let X = "4.3.2"
          , ee = "@coinbase/wallet-sdk";
        async function et(e, t) {
            let n = Object.assign(Object.assign({}, e), {
                jsonrpc: "2.0",
                id: crypto.randomUUID()
            })
              , r = await window.fetch(t, {
                method: "POST",
                body: JSON.stringify(n),
                mode: "cors",
                headers: {
                    "Content-Type": "application/json",
                    "X-Cbw-Sdk-Version": X,
                    "X-Cbw-Sdk-Platform": ee
                }
            })
              , {result: i, error: s} = await r.json();
            if (s)
                throw s;
            return i
        }
        let en = "accounts"
          , er = "activeChain"
          , ei = "availableChains"
          , es = "walletCapabilities";
        class ea {
            constructor(e) {
                var t, n, i;
                this.metadata = e.metadata,
                this.communicator = e.communicator,
                this.callback = e.callback,
                this.keyManager = new J,
                this.storage = new r("CBWSDK","SCWStateManager"),
                this.accounts = null != (t = this.storage.loadObject(en)) ? t : [],
                this.chain = this.storage.loadObject(er) || {
                    id: null != (i = null == (n = e.metadata.appChainIds) ? void 0 : n[0]) ? i : 1
                },
                this.handshake = this.handshake.bind(this),
                this.request = this.request.bind(this),
                this.createRequestMessage = this.createRequestMessage.bind(this),
                this.decryptResponseMessage = this.decryptResponseMessage.bind(this)
            }
            async handshake(e) {
                var t, n, r, i;
                await (null == (n = (t = this.communicator).waitForPopupLoaded) ? void 0 : n.call(t));
                let s = await this.createRequestMessage({
                    handshake: {
                        method: e.method,
                        params: Object.assign({}, this.metadata, null != (r = e.params) ? r : {})
                    }
                })
                  , a = await this.communicator.postRequestAndWaitForResponse(s);
                if ("failure"in a.content)
                    throw a.content.failure;
                let o = await Q("public", a.sender);
                await this.keyManager.setPeerPublicKey(o);
                let l = (await this.decryptResponseMessage(a)).result;
                if ("error"in l)
                    throw l.error;
                if ("eth_requestAccounts" === e.method) {
                    let e = l.value;
                    this.accounts = e,
                    this.storage.storeObject(en, e),
                    null == (i = this.callback) || i.call(this, "accountsChanged", e)
                }
            }
            async request(e) {
                var t;
                if (0 === this.accounts.length)
                    if ("wallet_sendCalls" === e.method)
                        return this.sendRequestToPopup(e);
                    else
                        throw u.provider.unauthorized();
                switch (e.method) {
                case "eth_requestAccounts":
                    return null == (t = this.callback) || t.call(this, "connect", {
                        chainId: O(this.chain.id)
                    }),
                    this.accounts;
                case "eth_accounts":
                    return this.accounts;
                case "eth_coinbase":
                    return this.accounts[0];
                case "net_version":
                    return this.chain.id;
                case "eth_chainId":
                    return O(this.chain.id);
                case "wallet_getCapabilities":
                    return this.storage.loadObject(es);
                case "wallet_switchEthereumChain":
                    return this.handleSwitchChainRequest(e);
                case "eth_ecRecover":
                case "personal_sign":
                case "wallet_sign":
                case "personal_ecRecover":
                case "eth_signTransaction":
                case "eth_sendTransaction":
                case "eth_signTypedData_v1":
                case "eth_signTypedData_v3":
                case "eth_signTypedData_v4":
                case "eth_signTypedData":
                case "wallet_addEthereumChain":
                case "wallet_watchAsset":
                case "wallet_sendCalls":
                case "wallet_showCallsStatus":
                case "wallet_grantPermissions":
                    return this.sendRequestToPopup(e);
                default:
                    if (!this.chain.rpcUrl)
                        throw u.rpc.internal("No RPC URL set for chain");
                    return et(e, this.chain.rpcUrl)
                }
            }
            async sendRequestToPopup(e) {
                var t, n;
                await (null == (n = (t = this.communicator).waitForPopupLoaded) ? void 0 : n.call(t));
                let r = await this.sendEncryptedRequest(e)
                  , i = (await this.decryptResponseMessage(r)).result;
                if ("error"in i)
                    throw i.error;
                return i.value
            }
            async cleanup() {
                var e, t;
                this.storage.clear(),
                await this.keyManager.clear(),
                this.accounts = [],
                this.chain = {
                    id: null != (t = null == (e = this.metadata.appChainIds) ? void 0 : e[0]) ? t : 1
                }
            }
            async handleSwitchChainRequest(e) {
                var t;
                let n = e.params;
                if (!n || !(null == (t = n[0]) ? void 0 : t.chainId))
                    throw u.rpc.invalidParams();
                let r = W(n[0].chainId);
                if (this.updateChain(r))
                    return null;
                let i = await this.sendRequestToPopup(e);
                return null === i && this.updateChain(r),
                i
            }
            async sendEncryptedRequest(e) {
                let t = await this.keyManager.getSharedSecret();
                if (!t)
                    throw u.provider.unauthorized("No valid session found, try requestAccounts before other methods");
                let n = await V({
                    action: e,
                    chainId: this.chain.id
                }, t)
                  , r = await this.createRequestMessage({
                    encrypted: n
                });
                return this.communicator.postRequestAndWaitForResponse(r)
            }
            async createRequestMessage(e) {
                let t = await K("public", await this.keyManager.getOwnPublicKey());
                return {
                    id: crypto.randomUUID(),
                    sender: t,
                    content: e,
                    timestamp: new Date
                }
            }
            async decryptResponseMessage(e) {
                var t, n;
                let r = e.content;
                if ("failure"in r)
                    throw r.failure;
                let i = await this.keyManager.getSharedSecret();
                if (!i)
                    throw u.provider.unauthorized("Invalid session");
                let s = await Y(r.encrypted, i)
                  , a = null == (t = s.data) ? void 0 : t.chains;
                if (a) {
                    let e = Object.entries(a).map( ([e,t]) => ({
                        id: Number(e),
                        rpcUrl: t
                    }));
                    this.storage.storeObject(ei, e),
                    this.updateChain(this.chain.id, e)
                }
                let o = null == (n = s.data) ? void 0 : n.capabilities;
                return o && this.storage.storeObject(es, o),
                s
            }
            updateChain(e, t) {
                var n;
                let r = null != t ? t : this.storage.loadObject(ei)
                  , i = null == r ? void 0 : r.find(t => t.id === e);
                return !!i && (i !== this.chain && (this.chain = i,
                this.storage.storeObject(er, i),
                null == (n = this.callback) || n.call(this, "chainChanged", O(i.id))),
                !0)
            }
        }
        var eo = n(77024);
        let el = "Addresses";
        function ec(e) {
            return void 0 !== e.errorMessage
        }
        class ed {
            constructor(e) {
                this.secret = e
            }
            async encrypt(e) {
                let t = this.secret;
                if (64 !== t.length)
                    throw Error("secret must be 256 bits");
                let n = crypto.getRandomValues(new Uint8Array(12))
                  , r = await crypto.subtle.importKey("raw", S(t), {
                    name: "aes-gcm"
                }, !1, ["encrypt", "decrypt"])
                  , i = new TextEncoder
                  , s = await window.crypto.subtle.encrypt({
                    name: "AES-GCM",
                    iv: n
                }, r, i.encode(e))
                  , a = s.slice(s.byteLength - 16)
                  , o = s.slice(0, s.byteLength - 16)
                  , l = new Uint8Array(a)
                  , c = new Uint8Array(o);
                return M(new Uint8Array([...n, ...l, ...c]))
            }
            async decrypt(e) {
                let t = this.secret;
                if (64 !== t.length)
                    throw Error("secret must be 256 bits");
                return new Promise( (n, r) => {
                    !async function() {
                        let i = await crypto.subtle.importKey("raw", S(t), {
                            name: "aes-gcm"
                        }, !1, ["encrypt", "decrypt"])
                          , s = S(e)
                          , a = s.slice(0, 12)
                          , o = s.slice(12, 28)
                          , l = new Uint8Array([...s.slice(28), ...o])
                          , c = {
                            name: "AES-GCM",
                            iv: new Uint8Array(a)
                        };
                        try {
                            let e = await window.crypto.subtle.decrypt(c, i, l)
                              , t = new TextDecoder;
                            n(t.decode(e))
                        } catch (e) {
                            r(e)
                        }
                    }()
                }
                )
            }
        }
        class eu {
            constructor(e, t, n) {
                this.linkAPIUrl = e,
                this.sessionId = t;
                let r = `${t}:${n}`;
                this.auth = `Basic ${btoa(r)}`
            }
            async markUnseenEventsAsSeen(e) {
                return Promise.all(e.map(e => fetch(`${this.linkAPIUrl}/events/${e.eventId}/seen`, {
                    method: "POST",
                    headers: {
                        Authorization: this.auth
                    }
                }))).catch(e => console.error("Unabled to mark event as failed:", e))
            }
            async fetchUnseenEvents() {
                var e;
                let t = await fetch(`${this.linkAPIUrl}/events?unseen=true`, {
                    headers: {
                        Authorization: this.auth
                    }
                });
                if (t.ok) {
                    let {events: n, error: r} = await t.json();
                    if (r)
                        throw Error(`Check unseen events failed: ${r}`);
                    let i = null != (e = null == n ? void 0 : n.filter(e => "Web3Response" === e.event).map(e => ({
                        type: "Event",
                        sessionId: this.sessionId,
                        eventId: e.id,
                        event: e.event,
                        data: e.data
                    }))) ? e : [];
                    return this.markUnseenEventsAsSeen(i),
                    i
                }
                throw Error(`Check unseen events failed: ${t.status}`)
            }
        }
        !function(e) {
            e[e.DISCONNECTED = 0] = "DISCONNECTED",
            e[e.CONNECTING = 1] = "CONNECTING",
            e[e.CONNECTED = 2] = "CONNECTED"
        }(eI || (eI = {}));
        class eh {
            setConnectionStateListener(e) {
                this.connectionStateListener = e
            }
            setIncomingDataListener(e) {
                this.incomingDataListener = e
            }
            constructor(e, t=WebSocket) {
                this.WebSocketClass = t,
                this.webSocket = null,
                this.pendingData = [],
                this.url = e.replace(/^http/, "ws")
            }
            async connect() {
                if (this.webSocket)
                    throw Error("webSocket object is not null");
                return new Promise( (e, t) => {
                    var n;
                    let r;
                    try {
                        this.webSocket = r = new this.WebSocketClass(this.url)
                    } catch (e) {
                        t(e);
                        return
                    }
                    null == (n = this.connectionStateListener) || n.call(this, eI.CONNECTING),
                    r.onclose = e => {
                        var n;
                        this.clearWebSocket(),
                        t(Error(`websocket error ${e.code}: ${e.reason}`)),
                        null == (n = this.connectionStateListener) || n.call(this, eI.DISCONNECTED)
                    }
                    ,
                    r.onopen = t => {
                        var n;
                        e(),
                        null == (n = this.connectionStateListener) || n.call(this, eI.CONNECTED),
                        this.pendingData.length > 0 && ([...this.pendingData].forEach(e => this.sendData(e)),
                        this.pendingData = [])
                    }
                    ,
                    r.onmessage = e => {
                        var t, n;
                        if ("h" === e.data)
                            null == (t = this.incomingDataListener) || t.call(this, {
                                type: "Heartbeat"
                            });
                        else
                            try {
                                let t = JSON.parse(e.data);
                                null == (n = this.incomingDataListener) || n.call(this, t)
                            } catch (e) {}
                    }
                }
                )
            }
            disconnect() {
                var e;
                let {webSocket: t} = this;
                if (t) {
                    this.clearWebSocket(),
                    null == (e = this.connectionStateListener) || e.call(this, eI.DISCONNECTED),
                    this.connectionStateListener = void 0,
                    this.incomingDataListener = void 0;
                    try {
                        t.close()
                    } catch (e) {}
                }
            }
            sendData(e) {
                let {webSocket: t} = this;
                if (!t) {
                    this.pendingData.push(e),
                    this.connect();
                    return
                }
                t.send(e)
            }
            clearWebSocket() {
                let {webSocket: e} = this;
                e && (this.webSocket = null,
                e.onclose = null,
                e.onerror = null,
                e.onmessage = null,
                e.onopen = null)
            }
        }
        class ep {
            constructor({session: e, linkAPIUrl: t, listener: n}) {
                this.destroyed = !1,
                this.lastHeartbeatResponse = 0,
                this.nextReqId = v(1),
                this._connected = !1,
                this._linked = !1,
                this.shouldFetchUnseenEventsOnConnect = !1,
                this.requestResolutions = new Map,
                this.handleSessionMetadataUpdated = e => {
                    e && new Map([["__destroyed", this.handleDestroyed], ["EthereumAddress", this.handleAccountUpdated], ["WalletUsername", this.handleWalletUsernameUpdated], ["AppVersion", this.handleAppVersionUpdated], ["ChainId", t => e.JsonRpcUrl && this.handleChainUpdated(t, e.JsonRpcUrl)]]).forEach( (t, n) => {
                        let r = e[n];
                        void 0 !== r && t(r)
                    }
                    )
                }
                ,
                this.handleDestroyed = e => {
                    var t;
                    "1" === e && (null == (t = this.listener) || t.resetAndReload())
                }
                ,
                this.handleAccountUpdated = async e => {
                    var t;
                    let n = await this.cipher.decrypt(e);
                    null == (t = this.listener) || t.accountUpdated(n)
                }
                ,
                this.handleMetadataUpdated = async (e, t) => {
                    var n;
                    let r = await this.cipher.decrypt(t);
                    null == (n = this.listener) || n.metadataUpdated(e, r)
                }
                ,
                this.handleWalletUsernameUpdated = async e => {
                    this.handleMetadataUpdated("walletUsername", e)
                }
                ,
                this.handleAppVersionUpdated = async e => {
                    this.handleMetadataUpdated("AppVersion", e)
                }
                ,
                this.handleChainUpdated = async (e, t) => {
                    var n;
                    let r = await this.cipher.decrypt(e)
                      , i = await this.cipher.decrypt(t);
                    null == (n = this.listener) || n.chainUpdated(r, i)
                }
                ,
                this.session = e,
                this.cipher = new ed(e.secret),
                this.listener = n;
                let r = new eh(`${t}/rpc`,WebSocket);
                r.setConnectionStateListener(async e => {
                    let t = !1;
                    switch (e) {
                    case eI.DISCONNECTED:
                        if (!this.destroyed) {
                            let e = async () => {
                                await new Promise(e => setTimeout(e, 5e3)),
                                this.destroyed || r.connect().catch( () => {
                                    e()
                                }
                                )
                            }
                            ;
                            e()
                        }
                        break;
                    case eI.CONNECTED:
                        t = await this.handleConnected(),
                        this.updateLastHeartbeat(),
                        setInterval( () => {
                            this.heartbeat()
                        }
                        , 1e4),
                        this.shouldFetchUnseenEventsOnConnect && this.fetchUnseenEventsAPI();
                    case eI.CONNECTING:
                    }
                    this.connected !== t && (this.connected = t)
                }
                ),
                r.setIncomingDataListener(e => {
                    var t;
                    switch (e.type) {
                    case "Heartbeat":
                        this.updateLastHeartbeat();
                        return;
                    case "IsLinkedOK":
                    case "Linked":
                        {
                            let t = "IsLinkedOK" === e.type ? e.linked : void 0;
                            this.linked = t || e.onlineGuests > 0;
                            break
                        }
                    case "GetSessionConfigOK":
                    case "SessionConfigUpdated":
                        this.handleSessionMetadataUpdated(e.metadata);
                        break;
                    case "Event":
                        this.handleIncomingEvent(e)
                    }
                    void 0 !== e.id && (null == (t = this.requestResolutions.get(e.id)) || t(e))
                }
                ),
                this.ws = r,
                this.http = new eu(t,e.id,e.key)
            }
            connect() {
                if (this.destroyed)
                    throw Error("instance is destroyed");
                this.ws.connect()
            }
            async destroy() {
                this.destroyed || (await this.makeRequest({
                    type: "SetSessionConfig",
                    id: v(this.nextReqId++),
                    sessionId: this.session.id,
                    metadata: {
                        __destroyed: "1"
                    }
                }, {
                    timeout: 1e3
                }),
                this.destroyed = !0,
                this.ws.disconnect(),
                this.listener = void 0)
            }
            get connected() {
                return this._connected
            }
            set connected(e) {
                this._connected = e
            }
            get linked() {
                return this._linked
            }
            set linked(e) {
                var t, n;
                this._linked = e,
                e && (null == (t = this.onceLinked) || t.call(this)),
                null == (n = this.listener) || n.linkedUpdated(e)
            }
            setOnceLinked(e) {
                return new Promise(t => {
                    this.linked ? e().then(t) : this.onceLinked = () => {
                        e().then(t),
                        this.onceLinked = void 0
                    }
                }
                )
            }
            async handleIncomingEvent(e) {
                var t;
                if ("Event" !== e.type || "Web3Response" !== e.event)
                    return;
                let n = JSON.parse(await this.cipher.decrypt(e.data));
                if ("WEB3_RESPONSE" !== n.type)
                    return;
                let {id: r, response: i} = n;
                null == (t = this.listener) || t.handleWeb3ResponseMessage(r, i)
            }
            async checkUnseenEvents() {
                if (!this.connected) {
                    this.shouldFetchUnseenEventsOnConnect = !0;
                    return
                }
                await new Promise(e => setTimeout(e, 250));
                try {
                    await this.fetchUnseenEventsAPI()
                } catch (e) {
                    console.error("Unable to check for unseen events", e)
                }
            }
            async fetchUnseenEventsAPI() {
                this.shouldFetchUnseenEventsOnConnect = !1,
                (await this.http.fetchUnseenEvents()).forEach(e => this.handleIncomingEvent(e))
            }
            async publishEvent(e, t, n=!1) {
                let r = await this.cipher.encrypt(JSON.stringify(Object.assign(Object.assign({}, t), {
                    origin: location.origin,
                    location: location.href,
                    relaySource: "coinbaseWalletExtension"in window && window.coinbaseWalletExtension ? "injected_sdk" : "sdk"
                })))
                  , i = {
                    type: "PublishEvent",
                    id: v(this.nextReqId++),
                    sessionId: this.session.id,
                    event: e,
                    data: r,
                    callWebhook: n
                };
                return this.setOnceLinked(async () => {
                    let e = await this.makeRequest(i);
                    if ("Fail" === e.type)
                        throw Error(e.error || "failed to publish event");
                    return e.eventId
                }
                )
            }
            sendData(e) {
                this.ws.sendData(JSON.stringify(e))
            }
            updateLastHeartbeat() {
                this.lastHeartbeatResponse = Date.now()
            }
            heartbeat() {
                if (Date.now() - this.lastHeartbeatResponse > 2e4)
                    return void this.ws.disconnect();
                try {
                    this.ws.sendData("h")
                } catch (e) {}
            }
            async makeRequest(e, t={
                timeout: 6e4
            }) {
                let n, r = e.id;
                return this.sendData(e),
                Promise.race([new Promise( (e, i) => {
                    n = window.setTimeout( () => {
                        i(Error(`request ${r} timed out`))
                    }
                    , t.timeout)
                }
                ), new Promise(e => {
                    this.requestResolutions.set(r, t => {
                        clearTimeout(n),
                        e(t),
                        this.requestResolutions.delete(r)
                    }
                    )
                }
                )])
            }
            async handleConnected() {
                return "Fail" !== (await this.makeRequest({
                    type: "HostSession",
                    id: v(this.nextReqId++),
                    sessionId: this.session.id,
                    sessionKey: this.session.key
                })).type && (this.sendData({
                    type: "IsLinked",
                    id: v(this.nextReqId++),
                    sessionId: this.session.id
                }),
                this.sendData({
                    type: "GetSessionConfig",
                    id: v(this.nextReqId++),
                    sessionId: this.session.id
                }),
                !0)
            }
        }
        class ef {
            constructor() {
                this._nextRequestId = 0,
                this.callbacks = new Map
            }
            makeRequestId() {
                this._nextRequestId = (this._nextRequestId + 1) % 0x7fffffff;
                let e = this._nextRequestId
                  , t = x(e.toString(16));
                return this.callbacks.get(t) && this.callbacks.delete(t),
                e
            }
        }
        var ey = n(30680)
          , em = n(36940);
        let eg = "session:id"
          , ew = "session:secret"
          , e_ = "session:linked";
        class ev {
            constructor(e, t, n, r=!1) {
                this.storage = e,
                this.id = t,
                this.secret = n,
                this.key = (0,
                em.My)((0,
                ey.sc)(`${t}, ${n} WalletLink`)),
                this._linked = !!r
            }
            static create(e) {
                return new ev(e,k(16),k(32)).save()
            }
            static load(e) {
                let t = e.getItem(eg)
                  , n = e.getItem(e_)
                  , r = e.getItem(ew);
                return t && r ? new ev(e,t,r,"1" === n) : null
            }
            get linked() {
                return this._linked
            }
            set linked(e) {
                this._linked = e,
                this.persistLinked()
            }
            save() {
                return this.storage.setItem(eg, this.id),
                this.storage.setItem(ew, this.secret),
                this.persistLinked(),
                this
            }
            persistLinked() {
                this.storage.setItem(e_, this._linked ? "1" : "0")
            }
        }
        function eb() {
            var e, t;
            return null != (t = null == (e = null == window ? void 0 : window.matchMedia) ? void 0 : e.call(window, "(prefers-color-scheme: dark)").matches) && t
        }
        function eE() {
            let e = document.createElement("style");
            e.type = "text/css",
            e.appendChild(document.createTextNode('@namespace svg "http://www.w3.org/2000/svg";.-cbwsdk-css-reset,.-cbwsdk-css-reset *{animation:none;animation-delay:0;animation-direction:normal;animation-duration:0;animation-fill-mode:none;animation-iteration-count:1;animation-name:none;animation-play-state:running;animation-timing-function:ease;backface-visibility:visible;background:0;background-attachment:scroll;background-clip:border-box;background-color:rgba(0,0,0,0);background-image:none;background-origin:padding-box;background-position:0 0;background-position-x:0;background-position-y:0;background-repeat:repeat;background-size:auto auto;border:0;border-style:none;border-width:medium;border-color:inherit;border-bottom:0;border-bottom-color:inherit;border-bottom-left-radius:0;border-bottom-right-radius:0;border-bottom-style:none;border-bottom-width:medium;border-collapse:separate;border-image:none;border-left:0;border-left-color:inherit;border-left-style:none;border-left-width:medium;border-radius:0;border-right:0;border-right-color:inherit;border-right-style:none;border-right-width:medium;border-spacing:0;border-top:0;border-top-color:inherit;border-top-left-radius:0;border-top-right-radius:0;border-top-style:none;border-top-width:medium;box-shadow:none;box-sizing:border-box;caption-side:top;clear:none;clip:auto;color:inherit;columns:auto;column-count:auto;column-fill:balance;column-gap:normal;column-rule:medium none currentColor;column-rule-color:currentColor;column-rule-style:none;column-rule-width:none;column-span:1;column-width:auto;counter-increment:none;counter-reset:none;direction:ltr;empty-cells:show;float:none;font:normal;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Helvetica Neue",Arial,sans-serif;font-size:medium;font-style:normal;font-variant:normal;font-weight:normal;height:auto;hyphens:none;letter-spacing:normal;line-height:normal;list-style:none;list-style-image:none;list-style-position:outside;list-style-type:disc;margin:0;margin-bottom:0;margin-left:0;margin-right:0;margin-top:0;opacity:1;orphans:0;outline:0;outline-color:invert;outline-style:none;outline-width:medium;overflow:visible;overflow-x:visible;overflow-y:visible;padding:0;padding-bottom:0;padding-left:0;padding-right:0;padding-top:0;page-break-after:auto;page-break-before:auto;page-break-inside:auto;perspective:none;perspective-origin:50% 50%;pointer-events:auto;position:static;quotes:"\\201C" "\\201D" "\\2018" "\\2019";tab-size:8;table-layout:auto;text-align:inherit;text-align-last:auto;text-decoration:none;text-decoration-color:inherit;text-decoration-line:none;text-decoration-style:solid;text-indent:0;text-shadow:none;text-transform:none;transform:none;transform-style:flat;transition:none;transition-delay:0s;transition-duration:0s;transition-property:none;transition-timing-function:ease;unicode-bidi:normal;vertical-align:baseline;visibility:visible;white-space:normal;widows:0;word-spacing:normal;z-index:auto}.-cbwsdk-css-reset strong{font-weight:bold}.-cbwsdk-css-reset *{box-sizing:border-box;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Helvetica Neue",Arial,sans-serif;line-height:1}.-cbwsdk-css-reset [class*=container]{margin:0;padding:0}.-cbwsdk-css-reset style{display:none}')),
            document.documentElement.appendChild(e)
        }
        var eI, ek, eM, eS, eA, eT, eC, eO, eN, eL, ex, ej, eD = n(79482), eR = {}, eP = [], eW = /acit|ex(?:s|g|n|p|$)|rph|grid|ows|mnc|ntw|ine[ch]|zoo|^ord|itera/i, eU = Array.isArray;
        function ez(e, t) {
            for (var n in t)
                e[n] = t[n];
            return e
        }
        function eq(e) {
            e && e.parentNode && e.parentNode.removeChild(e)
        }
        function eF(e, t, n) {
            var r, i, s, a = {};
            for (s in t)
                "key" == s ? r = t[s] : "ref" == s ? i = t[s] : a[s] = t[s];
            if (arguments.length > 2 && (a.children = arguments.length > 3 ? ek.call(arguments, 2) : n),
            "function" == typeof e && null != e.defaultProps)
                for (s in e.defaultProps)
                    void 0 === a[s] && (a[s] = e.defaultProps[s]);
            return eB(e, a, r, i, null)
        }
        function eB(e, t, n, r, i) {
            var s = {
                type: e,
                props: t,
                key: n,
                ref: r,
                __k: null,
                __: null,
                __b: 0,
                __e: null,
                __c: null,
                constructor: void 0,
                __v: null == i ? ++eS : i,
                __i: -1,
                __u: 0
            };
            return null == i && null != eM.vnode && eM.vnode(s),
            s
        }
        function eH(e) {
            return e.children
        }
        function eK(e, t) {
            this.props = e,
            this.context = t
        }
        function eQ(e, t) {
            if (null == t)
                return e.__ ? eQ(e.__, e.__i + 1) : null;
            for (var n; t < e.__k.length; t++)
                if (null != (n = e.__k[t]) && null != n.__e)
                    return n.__e;
            return "function" == typeof e.type ? eQ(e) : null
        }
        function eV(e) {
            (!e.__d && (e.__d = !0) && eA.push(e) && !eY.__r++ || eT != eM.debounceRendering) && ((eT = eM.debounceRendering) || eC)(eY)
        }
        function eY() {
            for (var e, t, n, r, i, s, a = 1; eA.length; )
                eA.length > a && eA.sort(eO),
                e = eA.shift(),
                a = eA.length,
                e.__d && (t = void 0,
                n = void 0,
                r = (n = e.__v).__e,
                i = [],
                s = [],
                e.__P && ((t = ez({}, n)).__v = n.__v + 1,
                eM.vnode && eM.vnode(t),
                eX(e.__P, t, n, e.__n, e.__P.namespaceURI, 32 & n.__u ? [r] : null, i, null == r ? eQ(n) : r, !!(32 & n.__u), s),
                t.__v = n.__v,
                t.__.__k[t.__i] = t,
                e1(i, t, s),
                n.__e = n.__ = null,
                t.__e != r && function e(t) {
                    var n, r;
                    if (null != (t = t.__) && null != t.__c) {
                        for (t.__e = t.__c.base = null,
                        n = 0; n < t.__k.length; n++)
                            if (null != (r = t.__k[n]) && null != r.__e) {
                                t.__e = t.__c.base = r.__e;
                                break
                            }
                        return e(t)
                    }
                }(t)));
            eY.__r = 0
        }
        function eG(e, t, n, r, i, s, a, o, l, c, d) {
            var u, h, p, f, y, m, g, w = r && r.__k || eP, _ = t.length;
            for (l = function(e, t, n, r, i) {
                var s, a, o, l, c, d = n.length, u = d, h = 0;
                for (e.__k = Array(i),
                s = 0; s < i; s++)
                    null != (a = t[s]) && "boolean" != typeof a && "function" != typeof a ? (l = s + h,
                    (a = e.__k[s] = "string" == typeof a || "number" == typeof a || "bigint" == typeof a || a.constructor == String ? eB(null, a, null, null, null) : eU(a) ? eB(eH, {
                        children: a
                    }, null, null, null) : null == a.constructor && a.__b > 0 ? eB(a.type, a.props, a.key, a.ref ? a.ref : null, a.__v) : a).__ = e,
                    a.__b = e.__b + 1,
                    o = null,
                    -1 != (c = a.__i = function(e, t, n, r) {
                        var i, s, a, o = e.key, l = e.type, c = t[n], d = null != c && 0 == (2 & c.__u);
                        if (null === c && null == e.key || d && o == c.key && l == c.type)
                            return n;
                        if (r > +!!d) {
                            for (i = n - 1,
                            s = n + 1; i >= 0 || s < t.length; )
                                if (null != (c = t[a = i >= 0 ? i-- : s++]) && 0 == (2 & c.__u) && o == c.key && l == c.type)
                                    return a
                        }
                        return -1
                    }(a, n, l, u)) && (u--,
                    (o = n[c]) && (o.__u |= 2)),
                    null == o || null == o.__v ? (-1 == c && (i > d ? h-- : i < d && h++),
                    "function" != typeof a.type && (a.__u |= 4)) : c != l && (c == l - 1 ? h-- : c == l + 1 ? h++ : (c > l ? h-- : h++,
                    a.__u |= 4))) : e.__k[s] = null;
                if (u)
                    for (s = 0; s < d; s++)
                        null != (o = n[s]) && 0 == (2 & o.__u) && (o.__e == r && (r = eQ(o)),
                        function e(t, n, r) {
                            var i, s;
                            if (eM.unmount && eM.unmount(t),
                            (i = t.ref) && (i.current && i.current != t.__e || e2(i, null, n)),
                            null != (i = t.__c)) {
                                if (i.componentWillUnmount)
                                    try {
                                        i.componentWillUnmount()
                                    } catch (e) {
                                        eM.__e(e, n)
                                    }
                                i.base = i.__P = null
                            }
                            if (i = t.__k)
                                for (s = 0; s < i.length; s++)
                                    i[s] && e(i[s], n, r || "function" != typeof t.type);
                            r || eq(t.__e),
                            t.__c = t.__ = t.__e = void 0
                        }(o, o));
                return r
            }(n, t, w, l, _),
            u = 0; u < _; u++)
                null != (p = n.__k[u]) && (h = -1 == p.__i ? eR : w[p.__i] || eR,
                p.__i = u,
                m = eX(e, p, h, i, s, a, o, l, c, d),
                f = p.__e,
                p.ref && h.ref != p.ref && (h.ref && e2(h.ref, null, p),
                d.push(p.ref, p.__c || f, p)),
                null == y && null != f && (y = f),
                (g = !!(4 & p.__u)) || h.__k === p.__k ? l = function e(t, n, r, i) {
                    var s, a;
                    if ("function" == typeof t.type) {
                        for (s = t.__k,
                        a = 0; s && a < s.length; a++)
                            s[a] && (s[a].__ = t,
                            n = e(s[a], n, r, i));
                        return n
                    }
                    t.__e != n && (i && (n && t.type && !n.parentNode && (n = eQ(t)),
                    r.insertBefore(t.__e, n || null)),
                    n = t.__e);
                    do
                        n = n && n.nextSibling;
                    while (null != n && 8 == n.nodeType);
                    return n
                }(p, l, e, g) : "function" == typeof p.type && void 0 !== m ? l = m : f && (l = f.nextSibling),
                p.__u &= -7);
            return n.__e = y,
            l
        }
        function eZ(e, t, n) {
            "-" == t[0] ? e.setProperty(t, null == n ? "" : n) : e[t] = null == n ? "" : "number" != typeof n || eW.test(t) ? n : n + "px"
        }
        function e$(e, t, n, r, i) {
            var s, a;
            e: if ("style" == t)
                if ("string" == typeof n)
                    e.style.cssText = n;
                else {
                    if ("string" == typeof r && (e.style.cssText = r = ""),
                    r)
                        for (t in r)
                            n && t in n || eZ(e.style, t, "");
                    if (n)
                        for (t in n)
                            r && n[t] == r[t] || eZ(e.style, t, n[t])
                }
            else if ("o" == t[0] && "n" == t[1])
                s = t != (t = t.replace(eN, "$1")),
                t = (a = t.toLowerCase())in e || "onFocusOut" == t || "onFocusIn" == t ? a.slice(2) : t.slice(2),
                e.l || (e.l = {}),
                e.l[t + s] = n,
                n ? r ? n.u = r.u : (n.u = eL,
                e.addEventListener(t, s ? ej : ex, s)) : e.removeEventListener(t, s ? ej : ex, s);
            else {
                if ("http://www.w3.org/2000/svg" == i)
                    t = t.replace(/xlink(H|:h)/, "h").replace(/sName$/, "s");
                else if ("width" != t && "height" != t && "href" != t && "list" != t && "form" != t && "tabIndex" != t && "download" != t && "rowSpan" != t && "colSpan" != t && "role" != t && "popover" != t && t in e)
                    try {
                        e[t] = null == n ? "" : n;
                        break e
                    } catch (e) {}
                "function" == typeof n || (null == n || !1 === n && "-" != t[4] ? e.removeAttribute(t) : e.setAttribute(t, "popover" == t && 1 == n ? "" : n))
            }
        }
        function eJ(e) {
            return function(t) {
                if (this.l) {
                    var n = this.l[t.type + e];
                    if (null == t.t)
                        t.t = eL++;
                    else if (t.t < n.u)
                        return;
                    return n(eM.event ? eM.event(t) : t)
                }
            }
        }
        function eX(e, t, n, r, i, s, a, o, l, c) {
            var d, u, h, p, f, y, m, g, w, _, v, b, E, I, k, M, S, A = t.type;
            if (null != t.constructor)
                return null;
            128 & n.__u && (l = !!(32 & n.__u),
            s = [o = t.__e = n.__e]),
            (d = eM.__b) && d(t);
            e: if ("function" == typeof A)
                try {
                    if (g = t.props,
                    w = "prototype"in A && A.prototype.render,
                    _ = (d = A.contextType) && r[d.__c],
                    v = d ? _ ? _.props.value : d.__ : r,
                    n.__c ? m = (u = t.__c = n.__c).__ = u.__E : (w ? t.__c = u = new A(g,v) : (t.__c = u = new eK(g,v),
                    u.constructor = A,
                    u.render = e4),
                    _ && _.sub(u),
                    u.props = g,
                    u.state || (u.state = {}),
                    u.context = v,
                    u.__n = r,
                    h = u.__d = !0,
                    u.__h = [],
                    u._sb = []),
                    w && null == u.__s && (u.__s = u.state),
                    w && null != A.getDerivedStateFromProps && (u.__s == u.state && (u.__s = ez({}, u.__s)),
                    ez(u.__s, A.getDerivedStateFromProps(g, u.__s))),
                    p = u.props,
                    f = u.state,
                    u.__v = t,
                    h)
                        w && null == A.getDerivedStateFromProps && null != u.componentWillMount && u.componentWillMount(),
                        w && null != u.componentDidMount && u.__h.push(u.componentDidMount);
                    else {
                        if (w && null == A.getDerivedStateFromProps && g !== p && null != u.componentWillReceiveProps && u.componentWillReceiveProps(g, v),
                        !u.__e && null != u.shouldComponentUpdate && !1 === u.shouldComponentUpdate(g, u.__s, v) || t.__v == n.__v) {
                            for (t.__v != n.__v && (u.props = g,
                            u.state = u.__s,
                            u.__d = !1),
                            t.__e = n.__e,
                            t.__k = n.__k,
                            t.__k.some(function(e) {
                                e && (e.__ = t)
                            }),
                            b = 0; b < u._sb.length; b++)
                                u.__h.push(u._sb[b]);
                            u._sb = [],
                            u.__h.length && a.push(u);
                            break e
                        }
                        null != u.componentWillUpdate && u.componentWillUpdate(g, u.__s, v),
                        w && null != u.componentDidUpdate && u.__h.push(function() {
                            u.componentDidUpdate(p, f, y)
                        })
                    }
                    if (u.context = v,
                    u.props = g,
                    u.__P = e,
                    u.__e = !1,
                    E = eM.__r,
                    I = 0,
                    w) {
                        for (u.state = u.__s,
                        u.__d = !1,
                        E && E(t),
                        d = u.render(u.props, u.state, u.context),
                        k = 0; k < u._sb.length; k++)
                            u.__h.push(u._sb[k]);
                        u._sb = []
                    } else
                        do
                            u.__d = !1,
                            E && E(t),
                            d = u.render(u.props, u.state, u.context),
                            u.state = u.__s;
                        while (u.__d && ++I < 25);
                    u.state = u.__s,
                    null != u.getChildContext && (r = ez(ez({}, r), u.getChildContext())),
                    w && !h && null != u.getSnapshotBeforeUpdate && (y = u.getSnapshotBeforeUpdate(p, f)),
                    M = d,
                    null != d && d.type === eH && null == d.key && (M = function e(t) {
                        return "object" != typeof t || null == t || t.__b && t.__b > 0 ? t : eU(t) ? t.map(e) : ez({}, t)
                    }(d.props.children)),
                    o = eG(e, eU(M) ? M : [M], t, n, r, i, s, a, o, l, c),
                    u.base = t.__e,
                    t.__u &= -161,
                    u.__h.length && a.push(u),
                    m && (u.__E = u.__ = null)
                } catch (e) {
                    if (t.__v = null,
                    l || null != s)
                        if (e.then) {
                            for (t.__u |= l ? 160 : 128; o && 8 == o.nodeType && o.nextSibling; )
                                o = o.nextSibling;
                            s[s.indexOf(o)] = null,
                            t.__e = o
                        } else {
                            for (S = s.length; S--; )
                                eq(s[S]);
                            e0(t)
                        }
                    else
                        t.__e = n.__e,
                        t.__k = n.__k,
                        e.then || e0(t);
                    eM.__e(e, t, n)
                }
            else
                null == s && t.__v == n.__v ? (t.__k = n.__k,
                t.__e = n.__e) : o = t.__e = function(e, t, n, r, i, s, a, o, l) {
                    var c, d, u, h, p, f, y, m = n.props, g = t.props, w = t.type;
                    if ("svg" == w ? i = "http://www.w3.org/2000/svg" : "math" == w ? i = "http://www.w3.org/1998/Math/MathML" : i || (i = "http://www.w3.org/1999/xhtml"),
                    null != s) {
                        for (c = 0; c < s.length; c++)
                            if ((p = s[c]) && "setAttribute"in p == !!w && (w ? p.localName == w : 3 == p.nodeType)) {
                                e = p,
                                s[c] = null;
                                break
                            }
                    }
                    if (null == e) {
                        if (null == w)
                            return document.createTextNode(g);
                        e = document.createElementNS(i, w, g.is && g),
                        o && (eM.__m && eM.__m(t, s),
                        o = !1),
                        s = null
                    }
                    if (null == w)
                        m === g || o && e.data == g || (e.data = g);
                    else {
                        if (s = s && ek.call(e.childNodes),
                        m = n.props || eR,
                        !o && null != s)
                            for (m = {},
                            c = 0; c < e.attributes.length; c++)
                                m[(p = e.attributes[c]).name] = p.value;
                        for (c in m)
                            if (p = m[c],
                            "children" == c)
                                ;
                            else if ("dangerouslySetInnerHTML" == c)
                                u = p;
                            else if (!(c in g)) {
                                if ("value" == c && "defaultValue"in g || "checked" == c && "defaultChecked"in g)
                                    continue;
                                e$(e, c, null, p, i)
                            }
                        for (c in g)
                            p = g[c],
                            "children" == c ? h = p : "dangerouslySetInnerHTML" == c ? d = p : "value" == c ? f = p : "checked" == c ? y = p : o && "function" != typeof p || m[c] === p || e$(e, c, p, m[c], i);
                        if (d)
                            o || u && (d.__html == u.__html || d.__html == e.innerHTML) || (e.innerHTML = d.__html),
                            t.__k = [];
                        else if (u && (e.innerHTML = ""),
                        eG("template" == t.type ? e.content : e, eU(h) ? h : [h], t, n, r, "foreignObject" == w ? "http://www.w3.org/1999/xhtml" : i, s, a, s ? s[0] : n.__k && eQ(n, 0), o, l),
                        null != s)
                            for (c = s.length; c--; )
                                eq(s[c]);
                        o || (c = "value",
                        "progress" == w && null == f ? e.removeAttribute("value") : null == f || f === e[c] && ("progress" != w || f) && ("option" != w || f == m[c]) || e$(e, c, f, m[c], i),
                        c = "checked",
                        null != y && y != e[c] && e$(e, c, y, m[c], i))
                    }
                    return e
                }(n.__e, t, n, r, i, s, a, l, c);
            return (d = eM.diffed) && d(t),
            128 & t.__u ? void 0 : o
        }
        function e0(e) {
            e && e.__c && (e.__c.__e = !0),
            e && e.__k && e.__k.forEach(e0)
        }
        function e1(e, t, n) {
            for (var r = 0; r < n.length; r++)
                e2(n[r], n[++r], n[++r]);
            eM.__c && eM.__c(t, e),
            e.some(function(t) {
                try {
                    e = t.__h,
                    t.__h = [],
                    e.some(function(e) {
                        e.call(t)
                    })
                } catch (e) {
                    eM.__e(e, t.__v)
                }
            })
        }
        function e2(e, t, n) {
            try {
                if ("function" == typeof e) {
                    var r = "function" == typeof e.__u;
                    r && e.__u(),
                    r && null == t || (e.__u = e(t))
                } else
                    e.current = t
            } catch (e) {
                eM.__e(e, n)
            }
        }
        function e4(e, t, n) {
            return this.constructor(e, n)
        }
        function e5(e, t, n) {
            var r, i, s, a;
            t == document && (t = document.documentElement),
            eM.__ && eM.__(e, t),
            i = (r = "function" == typeof n) ? null : n && n.__k || t.__k,
            s = [],
            a = [],
            eX(t, e = (!r && n || t).__k = eF(eH, null, [e]), i || eR, eR, t.namespaceURI, !r && n ? [n] : i ? null : t.firstChild ? ek.call(t.childNodes) : null, s, !r && n ? n : i ? i.__e : t.firstChild, r, a),
            e1(s, e, a)
        }
        function e3(e, t) {
            e5(e, t, e3)
        }
        ek = eP.slice,
        eM = {
            __e: function(e, t, n, r) {
                for (var i, s, a; t = t.__; )
                    if ((i = t.__c) && !i.__)
                        try {
                            if ((s = i.constructor) && null != s.getDerivedStateFromError && (i.setState(s.getDerivedStateFromError(e)),
                            a = i.__d),
                            null != i.componentDidCatch && (i.componentDidCatch(e, r || {}),
                            a = i.__d),
                            a)
                                return i.__E = i
                        } catch (t) {
                            e = t
                        }
                throw e
            }
        },
        eS = 0,
        eK.prototype.setState = function(e, t) {
            var n;
            n = null != this.__s && this.__s != this.state ? this.__s : this.__s = ez({}, this.state),
            "function" == typeof e && (e = e(ez({}, n), this.props)),
            e && ez(n, e),
            null != e && this.__v && (t && this._sb.push(t),
            eV(this))
        }
        ,
        eK.prototype.forceUpdate = function(e) {
            this.__v && (this.__e = !0,
            e && this.__h.push(e),
            eV(this))
        }
        ,
        eK.prototype.render = eH,
        eA = [],
        eC = "function" == typeof Promise ? Promise.prototype.then.bind(Promise.resolve()) : setTimeout,
        eO = function(e, t) {
            return e.__v.__b - t.__v.__b
        }
        ,
        eY.__r = 0,
        eN = /(PointerCapture)$|Capture$/i,
        eL = 0,
        ex = eJ(!1),
        ej = eJ(!0);
        var e6, e7, e8, e9, te = 0, tt = [], tn = eM, tr = tn.__b, ti = tn.__r, ts = tn.diffed, ta = tn.__c, to = tn.unmount, tl = tn.__;
        function tc(e, t) {
            tn.__h && tn.__h(e7, e, te || t),
            te = 0;
            var n = e7.__H || (e7.__H = {
                __: [],
                __h: []
            });
            return e >= n.__.length && n.__.push({}),
            n.__[e]
        }
        function td(e) {
            return te = 1,
            function(e, t, n) {
                var r = tc(e6++, 2);
                if (r.t = e,
                !r.__c && (r.__ = [tm(void 0, t), function(e) {
                    var t = r.__N ? r.__N[0] : r.__[0]
                      , n = r.t(t, e);
                    t !== n && (r.__N = [n, r.__[1]],
                    r.__c.setState({}))
                }
                ],
                r.__c = e7,
                !e7.__f)) {
                    var i = function(e, t, n) {
                        if (!r.__c.__H)
                            return !0;
                        var i = r.__c.__H.__.filter(function(e) {
                            return !!e.__c
                        });
                        if (i.every(function(e) {
                            return !e.__N
                        }))
                            return !s || s.call(this, e, t, n);
                        var a = r.__c.props !== e;
                        return i.forEach(function(e) {
                            if (e.__N) {
                                var t = e.__[0];
                                e.__ = e.__N,
                                e.__N = void 0,
                                t !== e.__[0] && (a = !0)
                            }
                        }),
                        s && s.call(this, e, t, n) || a
                    };
                    e7.__f = !0;
                    var s = e7.shouldComponentUpdate
                      , a = e7.componentWillUpdate;
                    e7.componentWillUpdate = function(e, t, n) {
                        if (this.__e) {
                            var r = s;
                            s = void 0,
                            i(e, t, n),
                            s = r
                        }
                        a && a.call(this, e, t, n)
                    }
                    ,
                    e7.shouldComponentUpdate = i
                }
                return r.__N || r.__
            }(tm, e)
        }
        function tu() {
            for (var e; e = tt.shift(); )
                if (e.__P && e.__H)
                    try {
                        e.__H.__h.forEach(tp),
                        e.__H.__h.forEach(tf),
                        e.__H.__h = []
                    } catch (t) {
                        e.__H.__h = [],
                        tn.__e(t, e.__v)
                    }
        }
        tn.__b = function(e) {
            e7 = null,
            tr && tr(e)
        }
        ,
        tn.__ = function(e, t) {
            e && t.__k && t.__k.__m && (e.__m = t.__k.__m),
            tl && tl(e, t)
        }
        ,
        tn.__r = function(e) {
            ti && ti(e),
            e6 = 0;
            var t = (e7 = e.__c).__H;
            t && (e8 === e7 ? (t.__h = [],
            e7.__h = [],
            t.__.forEach(function(e) {
                e.__N && (e.__ = e.__N),
                e.u = e.__N = void 0
            })) : (t.__h.forEach(tp),
            t.__h.forEach(tf),
            t.__h = [],
            e6 = 0)),
            e8 = e7
        }
        ,
        tn.diffed = function(e) {
            ts && ts(e);
            var t = e.__c;
            t && t.__H && (t.__H.__h.length && (1 !== tt.push(t) && e9 === tn.requestAnimationFrame || ((e9 = tn.requestAnimationFrame) || function(e) {
                var t, n = function() {
                    clearTimeout(r),
                    th && cancelAnimationFrame(t),
                    setTimeout(e)
                }, r = setTimeout(n, 35);
                th && (t = requestAnimationFrame(n))
            }
            )(tu)),
            t.__H.__.forEach(function(e) {
                e.u && (e.__H = e.u),
                e.u = void 0
            })),
            e8 = e7 = null
        }
        ,
        tn.__c = function(e, t) {
            t.some(function(e) {
                try {
                    e.__h.forEach(tp),
                    e.__h = e.__h.filter(function(e) {
                        return !e.__ || tf(e)
                    })
                } catch (n) {
                    t.some(function(e) {
                        e.__h && (e.__h = [])
                    }),
                    t = [],
                    tn.__e(n, e.__v)
                }
            }),
            ta && ta(e, t)
        }
        ,
        tn.unmount = function(e) {
            to && to(e);
            var t, n = e.__c;
            n && n.__H && (n.__H.__.forEach(function(e) {
                try {
                    tp(e)
                } catch (e) {
                    t = e
                }
            }),
            n.__H = void 0,
            t && tn.__e(t, n.__v))
        }
        ;
        var th = "function" == typeof requestAnimationFrame;
        function tp(e) {
            var t = e7
              , n = e.__c;
            "function" == typeof n && (e.__c = void 0,
            n()),
            e7 = t
        }
        function tf(e) {
            var t = e7;
            e.__c = e.__(),
            e7 = t
        }
        function ty(e, t) {
            return !e || e.length !== t.length || t.some(function(t, n) {
                return t !== e[n]
            })
        }
        function tm(e, t) {
            return "function" == typeof t ? t(e) : t
        }
        class tg {
            constructor() {
                this.items = new Map,
                this.nextItemKey = 0,
                this.root = null,
                this.darkMode = eb()
            }
            attach(e) {
                this.root = document.createElement("div"),
                this.root.className = "-cbwsdk-snackbar-root",
                e.appendChild(this.root),
                this.render()
            }
            presentItem(e) {
                let t = this.nextItemKey++;
                return this.items.set(t, e),
                this.render(),
                () => {
                    this.items.delete(t),
                    this.render()
                }
            }
            clear() {
                this.items.clear(),
                this.render()
            }
            render() {
                this.root && e5(eF("div", null, eF(tw, {
                    darkMode: this.darkMode
                }, Array.from(this.items.entries()).map( ([e,t]) => eF(t_, Object.assign({}, t, {
                    key: e
                }))))), this.root)
            }
        }
        let tw = e => eF("div", {
            class: (0,
            eD.$)("-cbwsdk-snackbar-container")
        }, eF("style", null, ".-cbwsdk-css-reset .-gear-container{margin-left:16px !important;margin-right:9px !important;display:flex;align-items:center;justify-content:center;width:24px;height:24px;transition:opacity .25s}.-cbwsdk-css-reset .-gear-container *{user-select:none}.-cbwsdk-css-reset .-gear-container svg{opacity:0;position:absolute}.-cbwsdk-css-reset .-gear-icon{height:12px;width:12px;z-index:10000}.-cbwsdk-css-reset .-cbwsdk-snackbar{align-items:flex-end;display:flex;flex-direction:column;position:fixed;right:0;top:0;z-index:2147483647}.-cbwsdk-css-reset .-cbwsdk-snackbar *{user-select:none}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance{display:flex;flex-direction:column;margin:8px 16px 0 16px;overflow:visible;text-align:left;transform:translateX(0);transition:opacity .25s,transform .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-header:hover .-gear-container svg{opacity:1}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-header{display:flex;align-items:center;background:#fff;overflow:hidden;border:1px solid #e7ebee;box-sizing:border-box;border-radius:8px;cursor:pointer}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-header-cblogo{margin:8px 8px 8px 8px}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-header *{cursor:pointer}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-header-message{color:#000;font-size:13px;line-height:1.5;user-select:none}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu{background:#fff;transition:opacity .25s ease-in-out,transform .25s linear,visibility 0s;visibility:hidden;border:1px solid #e7ebee;box-sizing:border-box;border-radius:8px;opacity:0;flex-direction:column;padding-left:8px;padding-right:8px}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item:last-child{margin-bottom:8px !important}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item:hover{background:#f5f7f8;border-radius:6px;transition:background .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item:hover span{color:#050f19;transition:color .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item:hover svg path{fill:#000;transition:fill .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item{visibility:inherit;height:35px;margin-top:8px;margin-bottom:0;display:flex;flex-direction:row;align-items:center;padding:8px;cursor:pointer}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item *{visibility:inherit;cursor:pointer}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item-is-red:hover{background:rgba(223,95,103,.2);transition:background .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item-is-red:hover *{cursor:pointer}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item-is-red:hover svg path{fill:#df5f67;transition:fill .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item-is-red:hover span{color:#df5f67;transition:color .25s}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-menu-item-info{color:#aaa;font-size:13px;margin:0 8px 0 32px;position:absolute}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-hidden{opacity:0;text-align:left;transform:translateX(25%);transition:opacity .5s linear}.-cbwsdk-css-reset .-cbwsdk-snackbar-instance-expanded .-cbwsdk-snackbar-instance-menu{opacity:1;display:flex;transform:translateY(8px);visibility:visible}"), eF("div", {
            class: "-cbwsdk-snackbar"
        }, e.children))
          , t_ = ({autoExpand: e, message: t, menuItems: n}) => {
            let[r,i] = td(!0)
              , [s,a] = td(null != e && e);
            return !function(e, t) {
                var n = tc(e6++, 3);
                !tn.__s && ty(n.__H, void 0) && (n.__ = e,
                n.u = void 0,
                e7.__H.__h.push(n))
            }( () => {
                let e = [window.setTimeout( () => {
                    i(!1)
                }
                , 1), window.setTimeout( () => {
                    a(!0)
                }
                , 1e4)];
                return () => {
                    e.forEach(window.clearTimeout)
                }
            }
            ),
            eF("div", {
                class: (0,
                eD.$)("-cbwsdk-snackbar-instance", r && "-cbwsdk-snackbar-instance-hidden", s && "-cbwsdk-snackbar-instance-expanded")
            }, eF("div", {
                class: "-cbwsdk-snackbar-instance-header",
                onClick: () => {
                    a(!s)
                }
            }, eF("img", {
                src: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PHBhdGggZD0iTTEuNDkyIDEwLjQxOWE4LjkzIDguOTMgMCAwMTguOTMtOC45M2gxMS4xNjNhOC45MyA4LjkzIDAgMDE4LjkzIDguOTN2MTEuMTYzYTguOTMgOC45MyAwIDAxLTguOTMgOC45M0gxMC40MjJhOC45MyA4LjkzIDAgMDEtOC45My04LjkzVjEwLjQxOXoiIGZpbGw9IiMxNjUyRjAiLz48cGF0aCBmaWxsLXJ1bGU9ImV2ZW5vZGQiIGNsaXAtcnVsZT0iZXZlbm9kZCIgZD0iTTEwLjQxOSAwSDIxLjU4QzI3LjMzNSAwIDMyIDQuNjY1IDMyIDEwLjQxOVYyMS41OEMzMiAyNy4zMzUgMjcuMzM1IDMyIDIxLjU4MSAzMkgxMC40MkM0LjY2NSAzMiAwIDI3LjMzNSAwIDIxLjU4MVYxMC40MkMwIDQuNjY1IDQuNjY1IDAgMTAuNDE5IDB6bTAgMS40ODhhOC45MyA4LjkzIDAgMDAtOC45MyA4LjkzdjExLjE2M2E4LjkzIDguOTMgMCAwMDguOTMgOC45M0gyMS41OGE4LjkzIDguOTMgMCAwMDguOTMtOC45M1YxMC40MmE4LjkzIDguOTMgMCAwMC04LjkzLTguOTNIMTAuNDJ6IiBmaWxsPSIjZmZmIi8+PHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik0xNS45OTggMjYuMDQ5Yy01LjU0OSAwLTEwLjA0Ny00LjQ5OC0xMC4wNDctMTAuMDQ3IDAtNS41NDggNC40OTgtMTAuMDQ2IDEwLjA0Ny0xMC4wNDYgNS41NDggMCAxMC4wNDYgNC40OTggMTAuMDQ2IDEwLjA0NiAwIDUuNTQ5LTQuNDk4IDEwLjA0Ny0xMC4wNDYgMTAuMDQ3eiIgZmlsbD0iI2ZmZiIvPjxwYXRoIGQ9Ik0xMi43NjIgMTQuMjU0YzAtLjgyMi42NjctMS40ODkgMS40ODktMS40ODloMy40OTdjLjgyMiAwIDEuNDg4LjY2NiAxLjQ4OCAxLjQ4OXYzLjQ5N2MwIC44MjItLjY2NiAxLjQ4OC0xLjQ4OCAxLjQ4OGgtMy40OTdhMS40ODggMS40ODggMCAwMS0xLjQ4OS0xLjQ4OHYtMy40OTh6IiBmaWxsPSIjMTY1MkYwIi8+PC9zdmc+",
                class: "-cbwsdk-snackbar-instance-header-cblogo"
            }), " ", eF("div", {
                class: "-cbwsdk-snackbar-instance-header-message"
            }, t), eF("div", {
                class: "-gear-container"
            }, !s && eF("svg", {
                width: "24",
                height: "24",
                viewBox: "0 0 24 24",
                fill: "none",
                xmlns: "http://www.w3.org/2000/svg"
            }, eF("circle", {
                cx: "12",
                cy: "12",
                r: "12",
                fill: "#F5F7F8"
            })), eF("img", {
                src: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTIiIGhlaWdodD0iMTIiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PHBhdGggZD0iTTEyIDYuNzV2LTEuNWwtMS43Mi0uNTdjLS4wOC0uMjctLjE5LS41Mi0uMzItLjc3bC44MS0xLjYyLTEuMDYtMS4wNi0xLjYyLjgxYy0uMjQtLjEzLS41LS4yNC0uNzctLjMyTDYuNzUgMGgtMS41bC0uNTcgMS43MmMtLjI3LjA4LS41My4xOS0uNzcuMzJsLTEuNjItLjgxLTEuMDYgMS4wNi44MSAxLjYyYy0uMTMuMjQtLjI0LjUtLjMyLjc3TDAgNS4yNXYxLjVsMS43Mi41N2MuMDguMjcuMTkuNTMuMzIuNzdsLS44MSAxLjYyIDEuMDYgMS4wNiAxLjYyLS44MWMuMjQuMTMuNS4yMy43Ny4zMkw1LjI1IDEyaDEuNWwuNTctMS43MmMuMjctLjA4LjUyLS4xOS43Ny0uMzJsMS42Mi44MSAxLjA2LTEuMDYtLjgxLTEuNjJjLjEzLS4yNC4yMy0uNS4zMi0uNzdMMTIgNi43NXpNNiA4LjVhMi41IDIuNSAwIDAxMC01IDIuNSAyLjUgMCAwMTAgNXoiIGZpbGw9IiMwNTBGMTkiLz48L3N2Zz4=",
                class: "-gear-icon",
                title: "Expand"
            }))), n && n.length > 0 && eF("div", {
                class: "-cbwsdk-snackbar-instance-menu"
            }, n.map( (e, t) => eF("div", {
                class: (0,
                eD.$)("-cbwsdk-snackbar-instance-menu-item", e.isRed && "-cbwsdk-snackbar-instance-menu-item-is-red"),
                onClick: e.onClick,
                key: t
            }, eF("svg", {
                width: e.svgWidth,
                height: e.svgHeight,
                viewBox: "0 0 10 11",
                fill: "none",
                xmlns: "http://www.w3.org/2000/svg"
            }, eF("path", {
                "fill-rule": e.defaultFillRule,
                "clip-rule": e.defaultClipRule,
                d: e.path,
                fill: "#AAAAAA"
            })), eF("span", {
                class: (0,
                eD.$)("-cbwsdk-snackbar-instance-menu-item-info", e.isRed && "-cbwsdk-snackbar-instance-menu-item-info-is-red")
            }, e.info)))))
        }
          , tv = "M5.00008 0.96875C6.73133 0.96875 8.23758 1.94375 9.00008 3.375L10.0001 2.375V5.5H9.53133H7.96883H6.87508L7.80633 4.56875C7.41258 3.3875 6.31258 2.53125 5.00008 2.53125C3.76258 2.53125 2.70633 3.2875 2.25633 4.36875L0.812576 3.76875C1.50008 2.125 3.11258 0.96875 5.00008 0.96875ZM2.19375 6.43125C2.5875 7.6125 3.6875 8.46875 5 8.46875C6.2375 8.46875 7.29375 7.7125 7.74375 6.63125L9.1875 7.23125C8.5 8.875 6.8875 10.0312 5 10.0312C3.26875 10.0312 1.7625 9.05625 1 7.625L0 8.625V5.5H0.46875H2.03125H3.125L2.19375 6.43125Z";
        class tb {
            constructor() {
                this.attached = !1,
                this.snackbar = new tg
            }
            attach() {
                if (this.attached)
                    throw Error("Coinbase Wallet SDK UI is already attached");
                let e = document.documentElement
                  , t = document.createElement("div");
                t.className = "-cbwsdk-css-reset",
                e.appendChild(t),
                this.snackbar.attach(t),
                this.attached = !0,
                eE()
            }
            showConnecting(e) {
                let t;
                return t = e.isUnlinkedErrorState ? {
                    autoExpand: !0,
                    message: "Connection lost",
                    menuItems: [{
                        isRed: !1,
                        info: "Reset connection",
                        svgWidth: "10",
                        svgHeight: "11",
                        path: "M5.00008 0.96875C6.73133 0.96875 8.23758 1.94375 9.00008 3.375L10.0001 2.375V5.5H9.53133H7.96883H6.87508L7.80633 4.56875C7.41258 3.3875 6.31258 2.53125 5.00008 2.53125C3.76258 2.53125 2.70633 3.2875 2.25633 4.36875L0.812576 3.76875C1.50008 2.125 3.11258 0.96875 5.00008 0.96875ZM2.19375 6.43125C2.5875 7.6125 3.6875 8.46875 5 8.46875C6.2375 8.46875 7.29375 7.7125 7.74375 6.63125L9.1875 7.23125C8.5 8.875 6.8875 10.0312 5 10.0312C3.26875 10.0312 1.7625 9.05625 1 7.625L0 8.625V5.5H0.46875H2.03125H3.125L2.19375 6.43125Z",
                        defaultFillRule: "evenodd",
                        defaultClipRule: "evenodd",
                        onClick: e.onResetConnection
                    }]
                } : {
                    message: "Confirm on phone",
                    menuItems: [{
                        isRed: !0,
                        info: "Cancel transaction",
                        svgWidth: "11",
                        svgHeight: "11",
                        path: "M10.3711 1.52346L9.21775 0.370117L5.37109 4.21022L1.52444 0.370117L0.371094 1.52346L4.2112 5.37012L0.371094 9.21677L1.52444 10.3701L5.37109 6.53001L9.21775 10.3701L10.3711 9.21677L6.53099 5.37012L10.3711 1.52346Z",
                        defaultFillRule: "inherit",
                        defaultClipRule: "inherit",
                        onClick: e.onCancel
                    }, {
                        isRed: !1,
                        info: "Reset connection",
                        svgWidth: "10",
                        svgHeight: "11",
                        path: tv,
                        defaultFillRule: "evenodd",
                        defaultClipRule: "evenodd",
                        onClick: e.onResetConnection
                    }]
                },
                this.snackbar.presentItem(t)
            }
        }
        class tE {
            constructor() {
                this.root = null,
                this.darkMode = eb()
            }
            attach() {
                let e = document.documentElement;
                this.root = document.createElement("div"),
                this.root.className = "-cbwsdk-css-reset",
                e.appendChild(this.root),
                eE()
            }
            present(e) {
                this.render(e)
            }
            clear() {
                this.render(null)
            }
            render(e) {
                this.root && (e5(null, this.root),
                e && e5(eF(tI, Object.assign({}, e, {
                    onDismiss: () => {
                        this.clear()
                    }
                    ,
                    darkMode: this.darkMode
                })), this.root))
            }
        }
        let tI = ({title: e, buttonText: t, darkMode: n, onButtonClick: r, onDismiss: i}) => eF(tw, {
            darkMode: n
        }, eF("div", {
            class: "-cbwsdk-redirect-dialog"
        }, eF("style", null, ".-cbwsdk-css-reset .-cbwsdk-redirect-dialog-backdrop{position:fixed;top:0;left:0;right:0;bottom:0;transition:opacity .25s;background-color:rgba(10,11,13,.5)}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-backdrop-hidden{opacity:0}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box{display:block;position:fixed;top:50%;left:50%;transform:translate(-50%, -50%);padding:20px;border-radius:8px;background-color:#fff;color:#0a0b0d}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box p{display:block;font-weight:400;font-size:14px;line-height:20px;padding-bottom:12px;color:#5b636e}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box button{appearance:none;border:none;background:none;color:#0052ff;padding:0;text-decoration:none;display:block;font-weight:600;font-size:16px;line-height:24px}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box.dark{background-color:#0a0b0d;color:#fff}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box.dark button{color:#0052ff}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box.light{background-color:#fff;color:#0a0b0d}.-cbwsdk-css-reset .-cbwsdk-redirect-dialog-box.light button{color:#0052ff}"), eF("div", {
            class: "-cbwsdk-redirect-dialog-backdrop",
            onClick: i
        }), eF("div", {
            class: (0,
            eD.$)("-cbwsdk-redirect-dialog-box", n ? "dark" : "light")
        }, eF("p", null, e), eF("button", {
            onClick: r
        }, t))))
          , tk = "https://www.walletlink.org";
        class tM {
            constructor() {
                this.attached = !1,
                this.redirectDialog = new tE
            }
            attach() {
                if (this.attached)
                    throw Error("Coinbase Wallet SDK UI is already attached");
                this.redirectDialog.attach(),
                this.attached = !0
            }
            redirectToCoinbaseWallet(e) {
                let t = new URL("https://go.cb-w.com/walletlink");
                t.searchParams.append("redirect_url", function() {
                    try {
                        if (function() {
                            try {
                                return null !== window.frameElement
                            } catch (e) {
                                return !1
                            }
                        }() && window.top)
                            return window.top.location;
                        return window.location
                    } catch (e) {
                        return window.location
                    }
                }().href),
                e && t.searchParams.append("wl_url", e);
                let n = document.createElement("a");
                n.target = "cbw-opener",
                n.href = t.href,
                n.rel = "noreferrer noopener",
                n.click()
            }
            openCoinbaseWalletDeeplink(e) {
                this.redirectDialog.present({
                    title: "Redirecting to Coinbase Wallet...",
                    buttonText: "Open",
                    onButtonClick: () => {
                        this.redirectToCoinbaseWallet(e)
                    }
                }),
                setTimeout( () => {
                    this.redirectToCoinbaseWallet(e)
                }
                , 99)
            }
            showConnecting(e) {
                return () => {
                    this.redirectDialog.clear()
                }
            }
        }
        class tS {
            constructor(e) {
                this.chainCallbackParams = {
                    chainId: "",
                    jsonRpcUrl: ""
                },
                this.isMobileWeb = function() {
                    var e;
                    return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(null == (e = null == window ? void 0 : window.navigator) ? void 0 : e.userAgent)
                }(),
                this.linkedUpdated = e => {
                    this.isLinked = e;
                    let t = this.storage.getItem(el);
                    if (e && (this._session.linked = e),
                    this.isUnlinkedErrorState = !1,
                    t) {
                        let n = t.split(" ")
                          , r = "true" === this.storage.getItem("IsStandaloneSigning");
                        "" === n[0] || e || !this._session.linked || r || (this.isUnlinkedErrorState = !0)
                    }
                }
                ,
                this.metadataUpdated = (e, t) => {
                    this.storage.setItem(e, t)
                }
                ,
                this.chainUpdated = (e, t) => {
                    (this.chainCallbackParams.chainId !== e || this.chainCallbackParams.jsonRpcUrl !== t) && (this.chainCallbackParams = {
                        chainId: e,
                        jsonRpcUrl: t
                    },
                    this.chainCallback && this.chainCallback(t, Number.parseInt(e, 10)))
                }
                ,
                this.accountUpdated = e => {
                    this.accountsCallback && this.accountsCallback([e]),
                    tS.accountRequestCallbackIds.size > 0 && (Array.from(tS.accountRequestCallbackIds.values()).forEach(t => {
                        this.invokeCallback(t, {
                            method: "requestEthereumAccounts",
                            result: [e]
                        })
                    }
                    ),
                    tS.accountRequestCallbackIds.clear())
                }
                ,
                this.resetAndReload = this.resetAndReload.bind(this),
                this.linkAPIUrl = e.linkAPIUrl,
                this.storage = e.storage,
                this.metadata = e.metadata,
                this.accountsCallback = e.accountsCallback,
                this.chainCallback = e.chainCallback;
                let {session: t, ui: n, connection: r} = this.subscribe();
                this._session = t,
                this.connection = r,
                this.relayEventManager = new ef,
                this.ui = n,
                this.ui.attach()
            }
            subscribe() {
                let e = ev.load(this.storage) || ev.create(this.storage)
                  , {linkAPIUrl: t} = this
                  , n = new ep({
                    session: e,
                    linkAPIUrl: t,
                    listener: this
                })
                  , r = this.isMobileWeb ? new tM : new tb;
                return n.connect(),
                {
                    session: e,
                    ui: r,
                    connection: n
                }
            }
            resetAndReload() {
                this.connection.destroy().then( () => {
                    let e = ev.load(this.storage);
                    (null == e ? void 0 : e.id) === this._session.id && r.clearAll(),
                    document.location.reload()
                }
                ).catch(e => {}
                )
            }
            signEthereumTransaction(e) {
                return this.sendRequest({
                    method: "signEthereumTransaction",
                    params: {
                        fromAddress: e.fromAddress,
                        toAddress: e.toAddress,
                        weiValue: C(e.weiValue),
                        data: A(e.data, !0),
                        nonce: e.nonce,
                        gasPriceInWei: e.gasPriceInWei ? C(e.gasPriceInWei) : null,
                        maxFeePerGas: e.gasPriceInWei ? C(e.gasPriceInWei) : null,
                        maxPriorityFeePerGas: e.gasPriceInWei ? C(e.gasPriceInWei) : null,
                        gasLimit: e.gasLimit ? C(e.gasLimit) : null,
                        chainId: e.chainId,
                        shouldSubmit: !1
                    }
                })
            }
            signAndSubmitEthereumTransaction(e) {
                return this.sendRequest({
                    method: "signEthereumTransaction",
                    params: {
                        fromAddress: e.fromAddress,
                        toAddress: e.toAddress,
                        weiValue: C(e.weiValue),
                        data: A(e.data, !0),
                        nonce: e.nonce,
                        gasPriceInWei: e.gasPriceInWei ? C(e.gasPriceInWei) : null,
                        maxFeePerGas: e.maxFeePerGas ? C(e.maxFeePerGas) : null,
                        maxPriorityFeePerGas: e.maxPriorityFeePerGas ? C(e.maxPriorityFeePerGas) : null,
                        gasLimit: e.gasLimit ? C(e.gasLimit) : null,
                        chainId: e.chainId,
                        shouldSubmit: !0
                    }
                })
            }
            submitEthereumTransaction(e, t) {
                return this.sendRequest({
                    method: "submitEthereumTransaction",
                    params: {
                        signedTransaction: A(e, !0),
                        chainId: t
                    }
                })
            }
            getWalletLinkSession() {
                return this._session
            }
            sendRequest(e) {
                let t = null
                  , n = k(8)
                  , r = r => {
                    this.publishWeb3RequestCanceledEvent(n),
                    this.handleErrorResponse(n, e.method, r),
                    null == t || t()
                }
                ;
                return new Promise( (i, s) => {
                    t = this.ui.showConnecting({
                        isUnlinkedErrorState: this.isUnlinkedErrorState,
                        onCancel: r,
                        onResetConnection: this.resetAndReload
                    }),
                    this.relayEventManager.callbacks.set(n, e => {
                        if (null == t || t(),
                        ec(e))
                            return s(Error(e.errorMessage));
                        i(e)
                    }
                    ),
                    this.publishWeb3RequestEvent(n, e)
                }
                )
            }
            publishWeb3RequestEvent(e, t) {
                let n = {
                    type: "WEB3_REQUEST",
                    id: e,
                    request: t
                };
                this.publishEvent("Web3Request", n, !0).then(e => {}
                ).catch(e => {
                    this.handleWeb3ResponseMessage(n.id, {
                        method: t.method,
                        errorMessage: e.message
                    })
                }
                ),
                this.isMobileWeb && this.openCoinbaseWalletDeeplink(t.method)
            }
            openCoinbaseWalletDeeplink(e) {
                if (this.ui instanceof tM)
                    switch (e) {
                    case "requestEthereumAccounts":
                    case "switchEthereumChain":
                        return;
                    default:
                        window.addEventListener("blur", () => {
                            window.addEventListener("focus", () => {
                                this.connection.checkUnseenEvents()
                            }
                            , {
                                once: !0
                            })
                        }
                        , {
                            once: !0
                        }),
                        this.ui.openCoinbaseWalletDeeplink()
                    }
            }
            publishWeb3RequestCanceledEvent(e) {
                this.publishEvent("Web3RequestCanceled", {
                    type: "WEB3_REQUEST_CANCELED",
                    id: e
                }, !1).then()
            }
            publishEvent(e, t, n) {
                return this.connection.publishEvent(e, t, n)
            }
            handleWeb3ResponseMessage(e, t) {
                if ("requestEthereumAccounts" === t.method) {
                    tS.accountRequestCallbackIds.forEach(e => this.invokeCallback(e, t)),
                    tS.accountRequestCallbackIds.clear();
                    return
                }
                this.invokeCallback(e, t)
            }
            handleErrorResponse(e, t, n) {
                var r;
                let i = null != (r = null == n ? void 0 : n.message) ? r : "Unspecified error message.";
                this.handleWeb3ResponseMessage(e, {
                    method: t,
                    errorMessage: i
                })
            }
            invokeCallback(e, t) {
                let n = this.relayEventManager.callbacks.get(e);
                n && (n(t),
                this.relayEventManager.callbacks.delete(e))
            }
            requestEthereumAccounts() {
                let {appName: e, appLogoUrl: t} = this.metadata
                  , n = {
                    method: "requestEthereumAccounts",
                    params: {
                        appName: e,
                        appLogoUrl: t
                    }
                }
                  , r = k(8);
                return new Promise( (e, t) => {
                    this.relayEventManager.callbacks.set(r, n => {
                        if (ec(n))
                            return t(Error(n.errorMessage));
                        e(n)
                    }
                    ),
                    tS.accountRequestCallbackIds.add(r),
                    this.publishWeb3RequestEvent(r, n)
                }
                )
            }
            watchAsset(e, t, n, r, i, s) {
                let a = {
                    method: "watchAsset",
                    params: {
                        type: e,
                        options: {
                            address: t,
                            symbol: n,
                            decimals: r,
                            image: i
                        },
                        chainId: s
                    }
                }
                  , o = null
                  , l = k(8);
                return o = this.ui.showConnecting({
                    isUnlinkedErrorState: this.isUnlinkedErrorState,
                    onCancel: e => {
                        this.publishWeb3RequestCanceledEvent(l),
                        this.handleErrorResponse(l, a.method, e),
                        null == o || o()
                    }
                    ,
                    onResetConnection: this.resetAndReload
                }),
                new Promise( (e, t) => {
                    this.relayEventManager.callbacks.set(l, n => {
                        if (null == o || o(),
                        ec(n))
                            return t(Error(n.errorMessage));
                        e(n)
                    }
                    ),
                    this.publishWeb3RequestEvent(l, a)
                }
                )
            }
            addEthereumChain(e, t, n, r, i, s) {
                let a = {
                    method: "addEthereumChain",
                    params: {
                        chainId: e,
                        rpcUrls: t,
                        blockExplorerUrls: r,
                        chainName: i,
                        iconUrls: n,
                        nativeCurrency: s
                    }
                }
                  , o = null
                  , l = k(8);
                return o = this.ui.showConnecting({
                    isUnlinkedErrorState: this.isUnlinkedErrorState,
                    onCancel: e => {
                        this.publishWeb3RequestCanceledEvent(l),
                        this.handleErrorResponse(l, a.method, e),
                        null == o || o()
                    }
                    ,
                    onResetConnection: this.resetAndReload
                }),
                new Promise( (e, t) => {
                    this.relayEventManager.callbacks.set(l, n => {
                        if (null == o || o(),
                        ec(n))
                            return t(Error(n.errorMessage));
                        e(n)
                    }
                    ),
                    this.publishWeb3RequestEvent(l, a)
                }
                )
            }
            switchEthereumChain(e, t) {
                let n = {
                    method: "switchEthereumChain",
                    params: Object.assign({
                        chainId: e
                    }, {
                        address: t
                    })
                }
                  , r = null
                  , i = k(8);
                return r = this.ui.showConnecting({
                    isUnlinkedErrorState: this.isUnlinkedErrorState,
                    onCancel: e => {
                        this.publishWeb3RequestCanceledEvent(i),
                        this.handleErrorResponse(i, n.method, e),
                        null == r || r()
                    }
                    ,
                    onResetConnection: this.resetAndReload
                }),
                new Promise( (e, t) => {
                    this.relayEventManager.callbacks.set(i, n => (null == r || r(),
                    ec(n) && n.errorCode) ? t(u.provider.custom({
                        code: n.errorCode,
                        message: "Unrecognized chain ID. Try adding the chain using addEthereumChain first."
                    })) : ec(n) ? t(Error(n.errorMessage)) : void e(n)),
                    this.publishWeb3RequestEvent(i, n)
                }
                )
            }
        }
        tS.accountRequestCallbackIds = new Set;
        var tA = n(91015).Buffer;
        let tT = "DefaultChainId"
          , tC = "DefaultJsonRpcUrl";
        class tO {
            constructor(e) {
                this._relay = null,
                this._addresses = [],
                this.metadata = e.metadata,
                this._storage = new r("walletlink",tk),
                this.callback = e.callback || null;
                let t = this._storage.getItem(el);
                if (t) {
                    let e = t.split(" ");
                    "" !== e[0] && (this._addresses = e.map(e => R(e)))
                }
                this.initializeRelay()
            }
            getSession() {
                let {id: e, secret: t} = this.initializeRelay().getWalletLinkSession();
                return {
                    id: e,
                    secret: t
                }
            }
            async handshake() {
                await this._eth_requestAccounts()
            }
            get selectedAddress() {
                return this._addresses[0] || void 0
            }
            get jsonRpcUrl() {
                var e;
                return null != (e = this._storage.getItem(tC)) ? e : void 0
            }
            set jsonRpcUrl(e) {
                this._storage.setItem(tC, e)
            }
            updateProviderInfo(e, t) {
                var n;
                this.jsonRpcUrl = e;
                let r = this.getChainId();
                this._storage.setItem(tT, t.toString(10)),
                W(t) !== r && (null == (n = this.callback) || n.call(this, "chainChanged", O(t)))
            }
            async watchAsset(e) {
                let t = Array.isArray(e) ? e[0] : e;
                if (!t.type)
                    throw u.rpc.invalidParams("Type is required");
                if ((null == t ? void 0 : t.type) !== "ERC20")
                    throw u.rpc.invalidParams(`Asset of type '${t.type}' is not supported`);
                if (!(null == t ? void 0 : t.options))
                    throw u.rpc.invalidParams("Options are required");
                if (!(null == t ? void 0 : t.options.address))
                    throw u.rpc.invalidParams("Address is required");
                let n = this.getChainId()
                  , {address: r, symbol: i, image: s, decimals: a} = t.options
                  , o = this.initializeRelay()
                  , l = await o.watchAsset(t.type, r, i, a, s, null == n ? void 0 : n.toString());
                return !ec(l) && !!l.result
            }
            async addEthereumChain(e) {
                var t, n;
                let r = e[0];
                if ((null == (t = r.rpcUrls) ? void 0 : t.length) === 0)
                    throw u.rpc.invalidParams("please pass in at least 1 rpcUrl");
                if (!r.chainName || "" === r.chainName.trim())
                    throw u.rpc.invalidParams("chainName is a required field");
                if (!r.nativeCurrency)
                    throw u.rpc.invalidParams("nativeCurrency is a required field");
                let i = Number.parseInt(r.chainId, 16);
                if (i === this.getChainId())
                    return !1;
                let s = this.initializeRelay()
                  , {rpcUrls: a=[], blockExplorerUrls: o=[], chainName: l, iconUrls: c=[], nativeCurrency: d} = r
                  , h = await s.addEthereumChain(i.toString(), a, c, o, l, d);
                if (ec(h))
                    return !1;
                if ((null == (n = h.result) ? void 0 : n.isApproved) === !0)
                    return this.updateProviderInfo(a[0], i),
                    null;
                throw u.rpc.internal("unable to add ethereum chain")
            }
            async switchEthereumChain(e) {
                let t = Number.parseInt(e[0].chainId, 16)
                  , n = this.initializeRelay()
                  , r = await n.switchEthereumChain(t.toString(10), this.selectedAddress || void 0);
                if (ec(r))
                    throw r;
                let i = r.result;
                return i.isApproved && i.rpcUrl.length > 0 && this.updateProviderInfo(i.rpcUrl, t),
                null
            }
            async cleanup() {
                this.callback = null,
                this._relay && this._relay.resetAndReload(),
                this._storage.clear()
            }
            _setAddresses(e, t) {
                var n;
                if (!Array.isArray(e))
                    throw Error("addresses is not an array");
                let r = e.map(e => R(e));
                JSON.stringify(r) !== JSON.stringify(this._addresses) && (this._addresses = r,
                null == (n = this.callback) || n.call(this, "accountsChanged", r),
                this._storage.setItem(el, r.join(" ")))
            }
            async request(e) {
                let t = e.params || [];
                switch (e.method) {
                case "eth_accounts":
                    return [...this._addresses];
                case "eth_coinbase":
                    return this.selectedAddress || null;
                case "net_version":
                    return this.getChainId().toString(10);
                case "eth_chainId":
                    return O(this.getChainId());
                case "eth_requestAccounts":
                    return this._eth_requestAccounts();
                case "eth_ecRecover":
                case "personal_ecRecover":
                    return this.ecRecover(e);
                case "personal_sign":
                    return this.personalSign(e);
                case "eth_signTransaction":
                    return this._eth_signTransaction(t);
                case "eth_sendRawTransaction":
                    return this._eth_sendRawTransaction(t);
                case "eth_sendTransaction":
                    return this._eth_sendTransaction(t);
                case "eth_signTypedData_v1":
                case "eth_signTypedData_v3":
                case "eth_signTypedData_v4":
                case "eth_signTypedData":
                    return this.signTypedData(e);
                case "wallet_addEthereumChain":
                    return this.addEthereumChain(t);
                case "wallet_switchEthereumChain":
                    return this.switchEthereumChain(t);
                case "wallet_watchAsset":
                    return this.watchAsset(t);
                default:
                    if (!this.jsonRpcUrl)
                        throw u.rpc.internal("No RPC URL set for chain");
                    return et(e, this.jsonRpcUrl)
                }
            }
            _ensureKnownAddress(e) {
                let t = R(e);
                if (!this._addresses.map(e => R(e)).includes(t))
                    throw Error("Unknown Ethereum address")
            }
            _prepareTransactionParams(e) {
                let t = e.from ? R(e.from) : this.selectedAddress;
                if (!t)
                    throw Error("Ethereum address is unavailable");
                this._ensureKnownAddress(t);
                let n = e.to ? R(e.to) : null
                  , r = null != e.value ? U(e.value) : BigInt(0)
                  , i = e.data ? P(e.data) : tA.alloc(0)
                  , s = null != e.nonce ? W(e.nonce) : null
                  , a = null != e.gasPrice ? U(e.gasPrice) : null
                  , o = null != e.maxFeePerGas ? U(e.maxFeePerGas) : null
                  , l = null != e.maxPriorityFeePerGas ? U(e.maxPriorityFeePerGas) : null;
                return {
                    fromAddress: t,
                    toAddress: n,
                    weiValue: r,
                    data: i,
                    nonce: s,
                    gasPriceInWei: a,
                    maxFeePerGas: o,
                    maxPriorityFeePerGas: l,
                    gasLimit: null != e.gas ? U(e.gas) : null,
                    chainId: e.chainId ? W(e.chainId) : this.getChainId()
                }
            }
            async ecRecover(e) {
                let {method: t, params: n} = e;
                if (!Array.isArray(n))
                    throw u.rpc.invalidParams();
                let r = this.initializeRelay()
                  , i = await r.sendRequest({
                    method: "ethereumAddressFromSignedMessage",
                    params: {
                        message: T(n[0]),
                        signature: T(n[1]),
                        addPrefix: "personal_ecRecover" === t
                    }
                });
                if (ec(i))
                    throw i;
                return i.result
            }
            getChainId() {
                var e;
                return Number.parseInt(null != (e = this._storage.getItem(tT)) ? e : "1", 10)
            }
            async _eth_requestAccounts() {
                var e, t;
                if (this._addresses.length > 0)
                    return null == (e = this.callback) || e.call(this, "connect", {
                        chainId: O(this.getChainId())
                    }),
                    this._addresses;
                let n = this.initializeRelay()
                  , r = await n.requestEthereumAccounts();
                if (ec(r))
                    throw r;
                if (!r.result)
                    throw Error("accounts received is empty");
                return this._setAddresses(r.result),
                null == (t = this.callback) || t.call(this, "connect", {
                    chainId: O(this.getChainId())
                }),
                this._addresses
            }
            async personalSign({params: e}) {
                if (!Array.isArray(e))
                    throw u.rpc.invalidParams();
                let t = e[1]
                  , n = e[0];
                this._ensureKnownAddress(t);
                let r = this.initializeRelay()
                  , i = await r.sendRequest({
                    method: "signEthereumMessage",
                    params: {
                        address: R(t),
                        message: T(n),
                        addPrefix: !0,
                        typedDataJson: null
                    }
                });
                if (ec(i))
                    throw i;
                return i.result
            }
            async _eth_signTransaction(e) {
                let t = this._prepareTransactionParams(e[0] || {})
                  , n = this.initializeRelay()
                  , r = await n.signEthereumTransaction(t);
                if (ec(r))
                    throw r;
                return r.result
            }
            async _eth_sendRawTransaction(e) {
                let t = P(e[0])
                  , n = this.initializeRelay()
                  , r = await n.submitEthereumTransaction(t, this.getChainId());
                if (ec(r))
                    throw r;
                return r.result
            }
            async _eth_sendTransaction(e) {
                let t = this._prepareTransactionParams(e[0] || {})
                  , n = this.initializeRelay()
                  , r = await n.signAndSubmitEthereumTransaction(t);
                if (ec(r))
                    throw r;
                return r.result
            }
            async signTypedData(e) {
                let {method: t, params: n} = e;
                if (!Array.isArray(n))
                    throw u.rpc.invalidParams();
                let r = n[+("eth_signTypedData_v1" === t)]
                  , i = n[+("eth_signTypedData_v1" !== t)];
                this._ensureKnownAddress(r);
                let s = this.initializeRelay()
                  , a = await s.sendRequest({
                    method: "signEthereumMessage",
                    params: {
                        address: R(r),
                        message: A(({
                            eth_signTypedData_v1: eo.hashForSignTypedDataLegacy,
                            eth_signTypedData_v3: eo.hashForSignTypedData_v3,
                            eth_signTypedData_v4: eo.hashForSignTypedData_v4,
                            eth_signTypedData: eo.hashForSignTypedData_v4
                        })[t]({
                            data: function(e) {
                                if ("string" == typeof e)
                                    return JSON.parse(e);
                                if ("object" == typeof e)
                                    return e;
                                throw u.rpc.invalidParams(`Not a JSON string or an object: ${String(e)}`)
                            }(i)
                        }), !0),
                        typedDataJson: JSON.stringify(i, null, 2),
                        addPrefix: !1
                    }
                });
                if (ec(a))
                    throw a;
                return a.result
            }
            initializeRelay() {
                return this._relay || (this._relay = new tS({
                    linkAPIUrl: tk,
                    storage: this._storage,
                    metadata: this.metadata,
                    accountsCallback: this._setAddresses.bind(this),
                    chainCallback: this.updateProviderInfo.bind(this)
                })),
                this._relay
            }
        }
        let tN = "SignerType"
          , tL = new r("CBWSDK","SignerConfigurator");
        async function tx(e) {
            let {communicator: t, metadata: n, handshakeRequest: r, callback: i} = e;
            tj(t, n, i).catch( () => {}
            );
            let s = {
                id: crypto.randomUUID(),
                event: "selectSignerType",
                data: Object.assign(Object.assign({}, e.preference), {
                    handshakeRequest: r
                })
            }
              , {data: a} = await t.postRequestAndWaitForResponse(s);
            return a
        }
        async function tj(e, t, n) {
            await e.onMessage( ({event: e}) => "WalletLinkSessionRequest" === e);
            let r = new tO({
                metadata: t,
                callback: n
            });
            e.postMessage({
                event: "WalletLinkUpdate",
                data: {
                    session: r.getSession()
                }
            }),
            await r.handshake(),
            e.postMessage({
                event: "WalletLinkUpdate",
                data: {
                    connected: !0
                }
            })
        }
        let tD = `Coinbase Wallet SDK requires the Cross-Origin-Opener-Policy header to not be set to 'same-origin'. This is to ensure that the SDK can communicate with the Coinbase Smart Wallet app.

Please see https://www.smartwallet.dev/guides/tips/popup-tips#cross-origin-opener-policy for more information.`
          , {checkCrossOriginOpenerPolicy: tR, getCrossOriginOpenerPolicy: tP} = ( () => {
            let e;
            return {
                getCrossOriginOpenerPolicy: () => void 0 === e ? "undefined" : e,
                checkCrossOriginOpenerPolicy: async () => {
                    if ("undefined" == typeof window) {
                        e = "non-browser-env";
                        return
                    }
                    try {
                        let t = `${window.location.origin}${window.location.pathname}`
                          , n = await fetch(t, {
                            method: "HEAD"
                        });
                        if (!n.ok)
                            throw Error(`HTTP error! status: ${n.status}`);
                        let r = n.headers.get("Cross-Origin-Opener-Policy");
                        e = null != r ? r : "null",
                        "same-origin" === e && console.error(tD)
                    } catch (t) {
                        console.error("Error checking Cross-Origin-Opener-Policy:", t.message),
                        e = "error"
                    }
                }
            }
        }
        )()
          , tW = {
            isRed: !1,
            info: "Retry",
            svgWidth: "10",
            svgHeight: "11",
            path: tv,
            defaultFillRule: "evenodd",
            defaultClipRule: "evenodd"
        }
          , tU = null;
        class tz {
            constructor({url: e="https://keys.coinbase.com/connect", metadata: t, preference: n}) {
                this.popup = null,
                this.listeners = new Map,
                this.postMessage = async e => {
                    (await this.waitForPopupLoaded()).postMessage(e, this.url.origin)
                }
                ,
                this.postRequestAndWaitForResponse = async e => {
                    let t = this.onMessage( ({requestId: t}) => t === e.id);
                    return this.postMessage(e),
                    await t
                }
                ,
                this.onMessage = async e => new Promise( (t, n) => {
                    let r = n => {
                        if (n.origin !== this.url.origin)
                            return;
                        let i = n.data;
                        e(i) && (t(i),
                        window.removeEventListener("message", r),
                        this.listeners.delete(r))
                    }
                    ;
                    window.addEventListener("message", r),
                    this.listeners.set(r, {
                        reject: n
                    })
                }
                ),
                this.disconnect = () => {
                    !function(e) {
                        e && !e.closed && e.close()
                    }(this.popup),
                    this.popup = null,
                    this.listeners.forEach( ({reject: e}, t) => {
                        e(u.provider.userRejectedRequest("Request rejected")),
                        window.removeEventListener("message", t)
                    }
                    ),
                    this.listeners.clear()
                }
                ,
                this.waitForPopupLoaded = async () => this.popup && !this.popup.closed ? (this.popup.focus(),
                this.popup) : (this.popup = await function(e) {
                    let t = (window.innerWidth - 420) / 2 + window.screenX
                      , n = (window.innerHeight - 540) / 2 + window.screenY;
                    function r() {
                        let r = `wallet_${crypto.randomUUID()}`
                          , i = window.open(e, r, `width=420, height=540, left=${t}, top=${n}`);
                        return (null == i || i.focus(),
                        i) ? i : null
                    }
                    var i = e;
                    for (let[e,t] of Object.entries({
                        sdkName: ee,
                        sdkVersion: X,
                        origin: window.location.origin,
                        coop: tP()
                    }))
                        i.searchParams.append(e, t.toString());
                    let s = r();
                    if (!s) {
                        let e = function() {
                            if (!tU) {
                                let e = document.createElement("div");
                                e.className = "-cbwsdk-css-reset",
                                document.body.appendChild(e),
                                (tU = new tg).attach(e)
                            }
                            return tU
                        }();
                        return new Promise( (t, n) => {
                            e.presentItem({
                                autoExpand: !0,
                                message: "Popup was blocked. Try again.",
                                menuItems: [Object.assign(Object.assign({}, tW), {
                                    onClick: () => {
                                        (s = r()) ? t(s) : n(u.rpc.internal("Popup window was blocked")),
                                        e.clear()
                                    }
                                })]
                            })
                        }
                        )
                    }
                    return Promise.resolve(s)
                }(this.url),
                this.onMessage( ({event: e}) => "PopupUnload" === e).then(this.disconnect).catch( () => {}
                ),
                this.onMessage( ({event: e}) => "PopupLoaded" === e).then(e => {
                    this.postMessage({
                        requestId: e.id,
                        data: {
                            version: X,
                            metadata: this.metadata,
                            preference: this.preference,
                            location: window.location.toString()
                        }
                    })
                }
                ).then( () => {
                    if (!this.popup)
                        throw u.rpc.internal();
                    return this.popup
                }
                )),
                this.url = new URL(e),
                this.metadata = t,
                this.preference = n
            }
        }
        var tq = n(2589);
        class tF extends tq.b {
        }
        var tB = function(e, t) {
            var n = {};
            for (var r in e)
                Object.prototype.hasOwnProperty.call(e, r) && 0 > t.indexOf(r) && (n[r] = e[r]);
            if (null != e && "function" == typeof Object.getOwnPropertySymbols)
                for (var i = 0, r = Object.getOwnPropertySymbols(e); i < r.length; i++)
                    0 > t.indexOf(r[i]) && Object.prototype.propertyIsEnumerable.call(e, r[i]) && (n[r[i]] = e[r[i]]);
            return n
        };
        class tH extends tF {
            constructor(e) {
                var {metadata: t} = e
                  , n = e.preference
                  , {keysUrl: r} = n
                  , i = tB(n, ["keysUrl"]);
                super(),
                this.signer = null,
                this.isCoinbaseWallet = !0,
                this.metadata = t,
                this.preference = i,
                this.communicator = new tz({
                    url: r,
                    metadata: t,
                    preference: i
                });
                let s = tL.getItem(tN);
                s && (this.signer = this.initSigner(s))
            }
            async request(e) {
                try {
                    if (!e || "object" != typeof e || Array.isArray(e))
                        throw u.rpc.invalidParams({
                            message: "Expected a single, non-array, object argument.",
                            data: e
                        });
                    let {method: t, params: n} = e;
                    if ("string" != typeof t || 0 === t.length)
                        throw u.rpc.invalidParams({
                            message: "'args.method' must be a non-empty string.",
                            data: e
                        });
                    if (void 0 !== n && !Array.isArray(n) && ("object" != typeof n || null === n))
                        throw u.rpc.invalidParams({
                            message: "'args.params' must be an object or array if provided.",
                            data: e
                        });
                    switch (t) {
                    case "eth_sign":
                    case "eth_signTypedData_v2":
                    case "eth_subscribe":
                    case "eth_unsubscribe":
                        throw u.provider.unsupportedMethod()
                    }
                    if (!this.signer)
                        switch (e.method) {
                        case "eth_requestAccounts":
                            {
                                let t = await this.requestSignerSelection(e)
                                  , n = this.initSigner(t);
                                await n.handshake(e),
                                this.signer = n,
                                tL.setItem(tN, t);
                                break
                            }
                        case "wallet_sendCalls":
                            {
                                let t = this.initSigner("scw");
                                await t.handshake({
                                    method: "handshake"
                                });
                                let n = await t.request(e);
                                return await t.cleanup(),
                                n
                            }
                        case "wallet_getCallsStatus":
                            return et(e, "https://rpc.wallet.coinbase.com");
                        case "net_version":
                            return 1;
                        case "eth_chainId":
                            return O(1);
                        default:
                            throw u.provider.unauthorized("Must call 'eth_requestAccounts' before other methods")
                        }
                    return await this.signer.request(e)
                } catch (t) {
                    let {code: e} = t;
                    return e === i.provider.unauthorized && this.disconnect(),
                    Promise.reject(function(e) {
                        let t = function(e, {shouldIncludeStack: t=!1}={}) {
                            var n, r;
                            let u = {};
                            if (e && "object" == typeof e && !Array.isArray(e) && c(e, "code") && Number.isInteger(n = e.code) && (s[n.toString()] || (r = n) >= -32099 && r <= -32e3))
                                u.code = e.code,
                                e.message && "string" == typeof e.message ? (u.message = e.message,
                                c(e, "data") && (u.data = e.data)) : (u.message = o(u.code),
                                u.data = {
                                    originalError: l(e)
                                });
                            else
                                u.code = i.rpc.internal,
                                u.message = d(e, "message") ? e.message : a,
                                u.data = {
                                    originalError: l(e)
                                };
                            return t && (u.stack = d(e, "stack") ? e.stack : void 0),
                            u
                        }(function(e) {
                            var t;
                            if ("string" == typeof e)
                                return {
                                    message: e,
                                    code: i.rpc.internal
                                };
                            if (ec(e)) {
                                let n = e.errorMessage
                                  , r = null != (t = e.errorCode) ? t : n.match(/(denied|rejected)/i) ? i.provider.userRejectedRequest : void 0;
                                return Object.assign(Object.assign({}, e), {
                                    message: n,
                                    code: r,
                                    data: {
                                        method: e.method
                                    }
                                })
                            }
                            return e
                        }(e), {
                            shouldIncludeStack: !0
                        })
                          , n = new URL("https://docs.cloud.coinbase.com/wallet-sdk/docs/errors");
                        return n.searchParams.set("version", X),
                        n.searchParams.set("code", t.code.toString()),
                        n.searchParams.set("message", t.message),
                        Object.assign(Object.assign({}, t), {
                            docUrl: n.href
                        })
                    }(t))
                }
            }
            async enable() {
                return console.warn('.enable() has been deprecated. Please use .request({ method: "eth_requestAccounts" }) instead.'),
                await this.request({
                    method: "eth_requestAccounts"
                })
            }
            async disconnect() {
                var e;
                await (null == (e = this.signer) ? void 0 : e.cleanup()),
                this.signer = null,
                r.clearAll(),
                this.emit("disconnect", u.provider.disconnected("User initiated disconnection"))
            }
            requestSignerSelection(e) {
                return tx({
                    communicator: this.communicator,
                    preference: this.preference,
                    metadata: this.metadata,
                    handshakeRequest: e,
                    callback: this.emit.bind(this)
                })
            }
            initSigner(e) {
                let {signerType: t, metadata: n, communicator: r, callback: i} = {
                    signerType: e,
                    metadata: this.metadata,
                    communicator: this.communicator,
                    callback: this.emit.bind(this)
                };
                switch (t) {
                case "scw":
                    return new ea({
                        metadata: n,
                        callback: i,
                        communicator: r
                    });
                case "walletlink":
                    return new tO({
                        metadata: n,
                        callback: i
                    })
                }
            }
        }
        let tK = {
            options: "all"
        };
        function tQ(e) {
            new r("CBWSDK").setItem("VERSION", X),
            tR();
            let t = {
                metadata: {
                    appName: e.appName || "Dapp",
                    appLogoUrl: e.appLogoUrl || "",
                    appChainIds: e.appChainIds || []
                },
                preference: Object.assign(tK, null != (n = e.preference) ? n : {})
            };
            var n, i = t.preference;
            if (i) {
                if (!["all", "smartWalletOnly", "eoaOnly"].includes(i.options))
                    throw Error(`Invalid options: ${i.options}`);
                if (i.attribution && void 0 !== i.attribution.auto && void 0 !== i.attribution.dataSuffix)
                    throw Error("Attribution cannot contain both auto and dataSuffix properties")
            }
            let s = null;
            return {
                getProvider: () => (s || (s = function(e) {
                    var t;
                    let n = {
                        metadata: e.metadata,
                        preference: e.preference
                    };
                    return null != (t = function({metadata: e, preference: t}) {
                        var n, r;
                        let {appName: i, appLogoUrl: s, appChainIds: a} = e;
                        if ("smartWalletOnly" !== t.options) {
                            let e = globalThis.coinbaseWalletExtension;
                            if (e)
                                return null == (n = e.setAppInfo) || n.call(e, i, s, a, t),
                                e
                        }
                        let o = function() {
                            var e, t;
                            try {
                                let n = globalThis;
                                return null != (e = n.ethereum) ? e : null == (t = n.top) ? void 0 : t.ethereum
                            } catch (e) {
                                return
                            }
                        }();
                        if (null == o ? void 0 : o.isCoinbaseBrowser)
                            return null == (r = o.setAppInfo) || r.call(o, i, s, a, t),
                            o
                    }(n)) ? t : new tH(n)
                }(t)),
                s)
            }
        }
    }
    ,
    33635: (e, t, n) => {
        "use strict";
        function r(e) {
            for (var t = 1; t < arguments.length; t++) {
                var n = arguments[t];
                for (var r in n)
                    e[r] = n[r]
            }
            return e
        }
        n.d(t, {
            A: () => i
        });
        var i = function e(t, n) {
            function i(e, i, s) {
                if ("undefined" != typeof document) {
                    "number" == typeof (s = r({}, n, s)).expires && (s.expires = new Date(Date.now() + 864e5 * s.expires)),
                    s.expires && (s.expires = s.expires.toUTCString()),
                    e = encodeURIComponent(e).replace(/%(2[346B]|5E|60|7C)/g, decodeURIComponent).replace(/[()]/g, escape);
                    var a = "";
                    for (var o in s)
                        s[o] && (a += "; " + o,
                        !0 !== s[o] && (a += "=" + s[o].split(";")[0]));
                    return document.cookie = e + "=" + t.write(i, e) + a
                }
            }
            return Object.create({
                set: i,
                get: function(e) {
                    if ("undefined" != typeof document && (!arguments.length || e)) {
                        for (var n = document.cookie ? document.cookie.split("; ") : [], r = {}, i = 0; i < n.length; i++) {
                            var s = n[i].split("=")
                              , a = s.slice(1).join("=");
                            try {
                                var o = decodeURIComponent(s[0]);
                                if (r[o] = t.read(a, o),
                                e === o)
                                    break
                            } catch (e) {}
                        }
                        return e ? r[e] : r
                    }
                },
                remove: function(e, t) {
                    i(e, "", r({}, t, {
                        expires: -1
                    }))
                },
                withAttributes: function(t) {
                    return e(this.converter, r({}, this.attributes, t))
                },
                withConverter: function(t) {
                    return e(r({}, this.converter, t), this.attributes)
                }
            }, {
                attributes: {
                    value: Object.freeze(n)
                },
                converter: {
                    value: Object.freeze(t)
                }
            })
        }({
            read: function(e) {
                return '"' === e[0] && (e = e.slice(1, -1)),
                e.replace(/(%[\dA-F]{2})+/gi, decodeURIComponent)
            },
            write: function(e) {
                return encodeURIComponent(e).replace(/%(2[346BF]|3[AC-F]|40|5[BDE]|60|7[BCD])/g, decodeURIComponent)
            }
        }, {
            path: "/"
        })
    }
    ,
    33953: (e, t, n) => {
        "use strict";
        n.d(t, {
            i: () => o
        });
        var r = n(74235)
          , i = n(93487);
        class s extends Error {
            static get code() {
                return "ERR_JOSE_GENERIC"
            }
            constructor(e) {
                var t;
                super(e),
                this.code = "ERR_JOSE_GENERIC",
                this.name = this.constructor.name,
                null == (t = Error.captureStackTrace) || t.call(Error, this, this.constructor)
            }
        }
        class a extends s {
            constructor() {
                super(...arguments),
                this.code = "ERR_JWT_INVALID"
            }
            static get code() {
                return "ERR_JWT_INVALID"
            }
        }
        function o(e) {
            let t, n;
            if ("string" != typeof e)
                throw new a("JWTs must use Compact JWS serialization, JWT must be a string");
            let {1: s, length: o} = e.split(".");
            if (5 === o)
                throw new a("Only JWTs using Compact JWS serialization can be decoded");
            if (3 !== o)
                throw new a("Invalid JWT");
            if (!s)
                throw new a("JWTs must contain a payload");
            try {
                t = (0,
                r.D)(s)
            } catch (e) {
                throw new a("Failed to base64url decode the payload")
            }
            try {
                n = JSON.parse(i.D0.decode(t))
            } catch (e) {
                throw new a("Failed to parse the decoded payload as JSON")
            }
            if (!function(e) {
                if ("object" != typeof e || null === e || "[object Object]" !== Object.prototype.toString.call(e))
                    return !1;
                if (null === Object.getPrototypeOf(e))
                    return !0;
                let t = e;
                for (; null !== Object.getPrototypeOf(t); )
                    t = Object.getPrototypeOf(t);
                return Object.getPrototypeOf(e) === t
            }(n))
                throw new a("Invalid JWT Claims Set");
            return n
        }
        Symbol.asyncIterator
    }
    ,
    34994: (e, t, n) => {
        "use strict";
        n.d(t, {
            u: () => r
        });
        let r = "standard:connect"
    }
    ,
    36803: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SignAndSendAllTransactions = void 0,
        t.SignAndSendAllTransactions = "solana:signAndSendAllTransactions"
    }
    ,
    37082: e => {
        e.exports = {
            style: {
                fontFamily: "'recklessNeue', 'recklessNeue Fallback'"
            },
            className: "__className_08e203",
            variable: "__variable_08e203"
        }
    }
    ,
    38138: (e, t, n) => {
        var r = n(91015).Buffer;
        let i = n(46586);
        function s(e) {
            if (e.startsWith("int["))
                return "int256" + e.slice(3);
            if ("int" === e)
                return "int256";
            if (e.startsWith("uint["))
                return "uint256" + e.slice(4);
            if ("uint" === e)
                return "uint256";
            if (e.startsWith("fixed["))
                return "fixed128x128" + e.slice(5);
            else if ("fixed" === e)
                return "fixed128x128";
            else if (e.startsWith("ufixed["))
                return "ufixed128x128" + e.slice(6);
            else if ("ufixed" === e)
                return "ufixed128x128";
            return e
        }
        function a(e) {
            return Number.parseInt(/^\D+(\d+)$/.exec(e)[1], 10)
        }
        function o(e) {
            var t = /^\D+(\d+)x(\d+)$/.exec(e);
            return [Number.parseInt(t[1], 10), Number.parseInt(t[2], 10)]
        }
        function l(e) {
            var t = e.match(/(.*)\[(.*?)\]$/);
            return t ? "" === t[2] ? "dynamic" : Number.parseInt(t[2], 10) : null
        }
        function c(e) {
            var t = typeof e;
            if ("string" === t || "number" === t)
                return BigInt(e);
            if ("bigint" === t)
                return e;
            throw Error("Argument is not a number")
        }
        function d(e, t) {
            if ("address" === e)
                return d("uint160", c(t));
            if ("bool" === e)
                return d("uint8", +!!t);
            if ("string" === e)
                return d("bytes", new r(t,"utf8"));
            if ((p = e).lastIndexOf("]") === p.length - 1) {
                if (void 0 === t.length)
                    throw Error("Not an array?");
                if ("dynamic" !== (n = l(e)) && 0 !== n && t.length > n)
                    throw Error("Elements exceed array size: " + n);
                for (h in u = [],
                e = e.slice(0, e.lastIndexOf("[")),
                "string" == typeof t && (t = JSON.parse(t)),
                t)
                    u.push(d(e, t[h]));
                if ("dynamic" === n) {
                    var n, s, u, h, p, f = d("uint256", t.length);
                    u.unshift(f)
                }
                return r.concat(u)
            } else if ("bytes" === e)
                return t = new r(t),
                u = r.concat([d("uint256", t.length), t]),
                t.length % 32 != 0 && (u = r.concat([u, i.zeros(32 - t.length % 32)])),
                u;
            else if (e.startsWith("bytes")) {
                if ((n = a(e)) < 1 || n > 32)
                    throw Error("Invalid bytes<N> width: " + n);
                return i.setLengthRight(t, 32)
            } else if (e.startsWith("uint")) {
                if ((n = a(e)) % 8 || n < 8 || n > 256)
                    throw Error("Invalid uint<N> width: " + n);
                s = c(t);
                let r = i.bitLengthFromBigInt(s);
                if (r > n)
                    throw Error("Supplied uint exceeds width: " + n + " vs " + r);
                if (s < 0)
                    throw Error("Supplied uint is negative");
                return i.bufferBEFromBigInt(s, 32)
            } else if (e.startsWith("int")) {
                if ((n = a(e)) % 8 || n < 8 || n > 256)
                    throw Error("Invalid int<N> width: " + n);
                s = c(t);
                let r = i.bitLengthFromBigInt(s);
                if (r > n)
                    throw Error("Supplied int exceeds width: " + n + " vs " + r);
                let o = i.twosFromBigInt(s, 256);
                return i.bufferBEFromBigInt(o, 32)
            } else if (e.startsWith("ufixed")) {
                if (n = o(e),
                (s = c(t)) < 0)
                    throw Error("Supplied ufixed is negative");
                return d("uint256", s * BigInt(2) ** BigInt(n[1]))
            } else if (e.startsWith("fixed"))
                return n = o(e),
                d("int256", c(t) * BigInt(2) ** BigInt(n[1]));
            throw Error("Unsupported or invalid type: " + e)
        }
        function u(e, t) {
            if (e.length !== t.length)
                throw Error("Number of types are not matching the values");
            for (var n, o, l = [], d = 0; d < e.length; d++) {
                var u = s(e[d])
                  , h = t[d];
                if ("bytes" === u)
                    l.push(h);
                else if ("string" === u)
                    l.push(new r(h,"utf8"));
                else if ("bool" === u)
                    l.push(new r(h ? "01" : "00","hex"));
                else if ("address" === u)
                    l.push(i.setLength(h, 20));
                else if (u.startsWith("bytes")) {
                    if ((n = a(u)) < 1 || n > 32)
                        throw Error("Invalid bytes<N> width: " + n);
                    l.push(i.setLengthRight(h, n))
                } else if (u.startsWith("uint")) {
                    if ((n = a(u)) % 8 || n < 8 || n > 256)
                        throw Error("Invalid uint<N> width: " + n);
                    o = c(h);
                    let e = i.bitLengthFromBigInt(o);
                    if (e > n)
                        throw Error("Supplied uint exceeds width: " + n + " vs " + e);
                    l.push(i.bufferBEFromBigInt(o, n / 8))
                } else if (u.startsWith("int")) {
                    if ((n = a(u)) % 8 || n < 8 || n > 256)
                        throw Error("Invalid int<N> width: " + n);
                    o = c(h);
                    let e = i.bitLengthFromBigInt(o);
                    if (e > n)
                        throw Error("Supplied int exceeds width: " + n + " vs " + e);
                    let t = i.twosFromBigInt(o, n);
                    l.push(i.bufferBEFromBigInt(t, n / 8))
                } else
                    throw Error("Unsupported or invalid type: " + u)
            }
            return r.concat(l)
        }
        e.exports = {
            rawEncode: function(e, t) {
                var n = []
                  , i = []
                  , a = 32 * e.length;
                for (var o in e) {
                    var c = s(e[o])
                      , u = d(c, t[o]);
                    "string" === c || "bytes" === c || "dynamic" === l(c) ? (n.push(d("uint256", a)),
                    i.push(u),
                    a += u.length) : n.push(u)
                }
                return r.concat(n.concat(i))
            },
            solidityPack: u,
            soliditySHA3: function(e, t) {
                return i.keccak(u(e, t))
            }
        }
    }
    ,
    40756: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.crypto = void 0,
        t.crypto = "object" == typeof globalThis && "crypto"in globalThis ? globalThis.crypto : void 0
    }
    ,
    41052: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SolanaSignTransaction = void 0,
        t.SolanaSignTransaction = "solana:signTransaction"
    }
    ,
    45041: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => r
        });
        let r = (0,
        n(44074).A)("CircleAlert", [["circle", {
            cx: "12",
            cy: "12",
            r: "10",
            key: "1mglay"
        }], ["line", {
            x1: "12",
            x2: "12",
            y1: "8",
            y2: "12",
            key: "1pkeuh"
        }], ["line", {
            x1: "12",
            x2: "12.01",
            y1: "16",
            y2: "16",
            key: "4dfq90"
        }]])
    }
    ,
    45915: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.toBig = t.shrSL = t.shrSH = t.rotrSL = t.rotrSH = t.rotrBL = t.rotrBH = t.rotr32L = t.rotr32H = t.rotlSL = t.rotlSH = t.rotlBL = t.rotlBH = t.add5L = t.add5H = t.add4L = t.add4H = t.add3L = t.add3H = void 0,
        t.add = _,
        t.fromBig = i,
        t.split = s;
        let n = BigInt(0x100000000 - 1)
          , r = BigInt(32);
        function i(e, t=!1) {
            return t ? {
                h: Number(e & n),
                l: Number(e >> r & n)
            } : {
                h: 0 | Number(e >> r & n),
                l: 0 | Number(e & n)
            }
        }
        function s(e, t=!1) {
            let n = e.length
              , r = new Uint32Array(n)
              , a = new Uint32Array(n);
            for (let s = 0; s < n; s++) {
                let {h: n, l: o} = i(e[s], t);
                [r[s],a[s]] = [n, o]
            }
            return [r, a]
        }
        let a = (e, t) => BigInt(e >>> 0) << r | BigInt(t >>> 0);
        t.toBig = a;
        let o = (e, t, n) => e >>> n;
        t.shrSH = o;
        let l = (e, t, n) => e << 32 - n | t >>> n;
        t.shrSL = l;
        let c = (e, t, n) => e >>> n | t << 32 - n;
        t.rotrSH = c;
        let d = (e, t, n) => e << 32 - n | t >>> n;
        t.rotrSL = d;
        let u = (e, t, n) => e << 64 - n | t >>> n - 32;
        t.rotrBH = u;
        let h = (e, t, n) => e >>> n - 32 | t << 64 - n;
        t.rotrBL = h;
        let p = (e, t) => t;
        t.rotr32H = p;
        let f = (e, t) => e;
        t.rotr32L = f;
        let y = (e, t, n) => e << n | t >>> 32 - n;
        t.rotlSH = y;
        let m = (e, t, n) => t << n | e >>> 32 - n;
        t.rotlSL = m;
        let g = (e, t, n) => t << n - 32 | e >>> 64 - n;
        t.rotlBH = g;
        let w = (e, t, n) => e << n - 32 | t >>> 64 - n;
        function _(e, t, n, r) {
            let i = (t >>> 0) + (r >>> 0);
            return {
                h: e + n + (i / 0x100000000 | 0) | 0,
                l: 0 | i
            }
        }
        t.rotlBL = w;
        let v = (e, t, n) => (e >>> 0) + (t >>> 0) + (n >>> 0);
        t.add3L = v;
        let b = (e, t, n, r) => t + n + r + (e / 0x100000000 | 0) | 0;
        t.add3H = b;
        let E = (e, t, n, r) => (e >>> 0) + (t >>> 0) + (n >>> 0) + (r >>> 0);
        t.add4L = E;
        let I = (e, t, n, r, i) => t + n + r + i + (e / 0x100000000 | 0) | 0;
        t.add4H = I;
        let k = (e, t, n, r, i) => (e >>> 0) + (t >>> 0) + (n >>> 0) + (r >>> 0) + (i >>> 0);
        t.add5L = k;
        let M = (e, t, n, r, i, s) => t + n + r + i + s + (e / 0x100000000 | 0) | 0;
        t.add5H = M,
        t.default = {
            fromBig: i,
            split: s,
            toBig: a,
            shrSH: o,
            shrSL: l,
            rotrSH: c,
            rotrSL: d,
            rotrBH: u,
            rotrBL: h,
            rotr32H: p,
            rotr32L: f,
            rotlSH: y,
            rotlSL: m,
            rotlBH: g,
            rotlBL: w,
            add: _,
            add3L: v,
            add3H: b,
            add4L: E,
            add4H: I,
            add5H: M,
            add5L: k
        }
    }
    ,
    46586: (e, t, n) => {
        var r = n(91015).Buffer;
        let {keccak_256: i} = n(55366);
        function s(e) {
            return r.allocUnsafe(e).fill(0)
        }
        function a(e, t) {
            let n = e.toString(16);
            n.length % 2 != 0 && (n = "0" + n);
            let i = n.match(/.{1,2}/g).map(e => parseInt(e, 16));
            for (; i.length < t; )
                i.unshift(0);
            return r.from(i)
        }
        function o(e, t, n) {
            let r = s(t);
            return (e = l(e),
            n) ? e.length < t ? (e.copy(r),
            r) : e.slice(0, t) : e.length < t ? (e.copy(r, t - e.length),
            r) : e.slice(-t)
        }
        function l(e) {
            if (!r.isBuffer(e))
                if (Array.isArray(e))
                    e = r.from(e);
                else if ("string" == typeof e) {
                    var t;
                    e = c(e) ? r.from((t = d(e)).length % 2 ? "0" + t : t, "hex") : r.from(e)
                } else if ("number" == typeof e)
                    e = intToBuffer(e);
                else if (null == e)
                    e = r.allocUnsafe(0);
                else if ("bigint" == typeof e)
                    e = a(e);
                else if (e.toArray)
                    e = r.from(e.toArray());
                else
                    throw Error("invalid type");
            return e
        }
        function c(e) {
            return "string" == typeof e && e.match(/^0x[0-9A-Fa-f]*$/)
        }
        function d(e) {
            return "string" == typeof e && e.startsWith("0x") ? e.slice(2) : e
        }
        e.exports = {
            zeros: s,
            setLength: o,
            setLengthRight: function(e, t) {
                return o(e, t, !0)
            },
            isHexString: c,
            stripHexPrefix: d,
            toBuffer: l,
            bufferToHex: function(e) {
                return "0x" + (e = l(e)).toString("hex")
            },
            keccak: function(e, t) {
                if (e = l(e),
                t || (t = 256),
                256 !== t)
                    throw Error("unsupported");
                return r.from(i(new Uint8Array(e)))
            },
            bitLengthFromBigInt: function(e) {
                return e.toString(2).length
            },
            bufferBEFromBigInt: a,
            twosFromBigInt: function(e, t) {
                let n;
                return (e < 0n ? (~e & (1n << BigInt(t)) - 1n) + 1n : e) & (1n << BigInt(t)) - 1n
            }
        }
    }
    ,
    47880: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "m8.25 4.5 7.5 7.5-7.5 7.5"
            }))
        })
    }
    ,
    47960: (e, t, n) => {
        "use strict";
        n.d(t, {
            E: () => y
        });
        var r = n(91934)
          , i = n(90223)
          , s = n(57011)
          , a = n(23196)
          , o = class extends a.Q {
            constructor(e={}) {
                super(),
                this.config = e,
                this.#e = new Map
            }
            #e;
            build(e, t, n) {
                let s = t.queryKey
                  , a = t.queryHash ?? (0,
                r.F$)(s, t)
                  , o = this.get(a);
                return o || (o = new i.X({
                    client: e,
                    queryKey: s,
                    queryHash: a,
                    options: e.defaultQueryOptions(t),
                    state: n,
                    defaultOptions: e.getQueryDefaults(s)
                }),
                this.add(o)),
                o
            }
            add(e) {
                this.#e.has(e.queryHash) || (this.#e.set(e.queryHash, e),
                this.notify({
                    type: "added",
                    query: e
                }))
            }
            remove(e) {
                let t = this.#e.get(e.queryHash);
                t && (e.destroy(),
                t === e && this.#e.delete(e.queryHash),
                this.notify({
                    type: "removed",
                    query: e
                }))
            }
            clear() {
                s.jG.batch( () => {
                    this.getAll().forEach(e => {
                        this.remove(e)
                    }
                    )
                }
                )
            }
            get(e) {
                return this.#e.get(e)
            }
            getAll() {
                return [...this.#e.values()]
            }
            find(e) {
                let t = {
                    exact: !0,
                    ...e
                };
                return this.getAll().find(e => (0,
                r.MK)(t, e))
            }
            findAll(e={}) {
                let t = this.getAll();
                return Object.keys(e).length > 0 ? t.filter(t => (0,
                r.MK)(e, t)) : t
            }
            notify(e) {
                s.jG.batch( () => {
                    this.listeners.forEach(t => {
                        t(e)
                    }
                    )
                }
                )
            }
            onFocus() {
                s.jG.batch( () => {
                    this.getAll().forEach(e => {
                        e.onFocus()
                    }
                    )
                }
                )
            }
            onOnline() {
                s.jG.batch( () => {
                    this.getAll().forEach(e => {
                        e.onOnline()
                    }
                    )
                }
                )
            }
        }
          , l = n(37818)
          , c = class extends a.Q {
            constructor(e={}) {
                super(),
                this.config = e,
                this.#t = new Set,
                this.#n = new Map,
                this.#r = 0
            }
            #t;
            #n;
            #r;
            build(e, t, n) {
                let r = new l.s({
                    client: e,
                    mutationCache: this,
                    mutationId: ++this.#r,
                    options: e.defaultMutationOptions(t),
                    state: n
                });
                return this.add(r),
                r
            }
            add(e) {
                this.#t.add(e);
                let t = d(e);
                if ("string" == typeof t) {
                    let n = this.#n.get(t);
                    n ? n.push(e) : this.#n.set(t, [e])
                }
                this.notify({
                    type: "added",
                    mutation: e
                })
            }
            remove(e) {
                if (this.#t.delete(e)) {
                    let t = d(e);
                    if ("string" == typeof t) {
                        let n = this.#n.get(t);
                        if (n)
                            if (n.length > 1) {
                                let t = n.indexOf(e);
                                -1 !== t && n.splice(t, 1)
                            } else
                                n[0] === e && this.#n.delete(t)
                    }
                }
                this.notify({
                    type: "removed",
                    mutation: e
                })
            }
            canRun(e) {
                let t = d(e);
                if ("string" != typeof t)
                    return !0;
                {
                    let n = this.#n.get(t)
                      , r = n?.find(e => "pending" === e.state.status);
                    return !r || r === e
                }
            }
            runNext(e) {
                let t = d(e);
                if ("string" != typeof t)
                    return Promise.resolve();
                {
                    let n = this.#n.get(t)?.find(t => t !== e && t.state.isPaused);
                    return n?.continue() ?? Promise.resolve()
                }
            }
            clear() {
                s.jG.batch( () => {
                    this.#t.forEach(e => {
                        this.notify({
                            type: "removed",
                            mutation: e
                        })
                    }
                    ),
                    this.#t.clear(),
                    this.#n.clear()
                }
                )
            }
            getAll() {
                return Array.from(this.#t)
            }
            find(e) {
                let t = {
                    exact: !0,
                    ...e
                };
                return this.getAll().find(e => (0,
                r.nJ)(t, e))
            }
            findAll(e={}) {
                return this.getAll().filter(t => (0,
                r.nJ)(e, t))
            }
            notify(e) {
                s.jG.batch( () => {
                    this.listeners.forEach(t => {
                        t(e)
                    }
                    )
                }
                )
            }
            resumePausedMutations() {
                let e = this.getAll().filter(e => e.state.isPaused);
                return s.jG.batch( () => Promise.all(e.map(e => e.continue().catch(r.lQ))))
            }
        }
        ;
        function d(e) {
            return e.options.scope?.id
        }
        var u = n(79734)
          , h = n(15565);
        function p(e) {
            return {
                onFetch: (t, n) => {
                    let i = t.options
                      , s = t.fetchOptions?.meta?.fetchMore?.direction
                      , a = t.state.data?.pages || []
                      , o = t.state.data?.pageParams || []
                      , l = {
                        pages: [],
                        pageParams: []
                    }
                      , c = 0
                      , d = async () => {
                        let n = !1
                          , d = e => {
                            Object.defineProperty(e, "signal", {
                                enumerable: !0,
                                get: () => (t.signal.aborted ? n = !0 : t.signal.addEventListener("abort", () => {
                                    n = !0
                                }
                                ),
                                t.signal)
                            })
                        }
                          , u = (0,
                        r.ZM)(t.options, t.fetchOptions)
                          , h = async (e, i, s) => {
                            if (n)
                                return Promise.reject();
                            if (null == i && e.pages.length)
                                return Promise.resolve(e);
                            let a = ( () => {
                                let e = {
                                    client: t.client,
                                    queryKey: t.queryKey,
                                    pageParam: i,
                                    direction: s ? "backward" : "forward",
                                    meta: t.options.meta
                                };
                                return d(e),
                                e
                            }
                            )()
                              , o = await u(a)
                              , {maxPages: l} = t.options
                              , c = s ? r.ZZ : r.y9;
                            return {
                                pages: c(e.pages, o, l),
                                pageParams: c(e.pageParams, i, l)
                            }
                        }
                        ;
                        if (s && a.length) {
                            let e = "backward" === s
                              , t = {
                                pages: a,
                                pageParams: o
                            }
                              , n = (e ? function(e, {pages: t, pageParams: n}) {
                                return t.length > 0 ? e.getPreviousPageParam?.(t[0], t, n[0], n) : void 0
                            }
                            : f)(i, t);
                            l = await h(t, n, e)
                        } else {
                            let t = e ?? a.length;
                            do {
                                let e = 0 === c ? o[0] ?? i.initialPageParam : f(i, l);
                                if (c > 0 && null == e)
                                    break;
                                l = await h(l, e),
                                c++
                            } while (c < t)
                        }
                        return l
                    }
                    ;
                    t.options.persister ? t.fetchFn = () => t.options.persister?.(d, {
                        client: t.client,
                        queryKey: t.queryKey,
                        meta: t.options.meta,
                        signal: t.signal
                    }, n) : t.fetchFn = d
                }
            }
        }
        function f(e, {pages: t, pageParams: n}) {
            let r = t.length - 1;
            return t.length > 0 ? e.getNextPageParam(t[r], t, n[r], n) : void 0
        }
        var y = class {
            #i;
            #s;
            #a;
            #o;
            #l;
            #c;
            #d;
            #u;
            constructor(e={}) {
                this.#i = e.queryCache || new o,
                this.#s = e.mutationCache || new c,
                this.#a = e.defaultOptions || {},
                this.#o = new Map,
                this.#l = new Map,
                this.#c = 0
            }
            mount() {
                this.#c++,
                1 === this.#c && (this.#d = u.m.subscribe(async e => {
                    e && (await this.resumePausedMutations(),
                    this.#i.onFocus())
                }
                ),
                this.#u = h.t.subscribe(async e => {
                    e && (await this.resumePausedMutations(),
                    this.#i.onOnline())
                }
                ))
            }
            unmount() {
                this.#c--,
                0 === this.#c && (this.#d?.(),
                this.#d = void 0,
                this.#u?.(),
                this.#u = void 0)
            }
            isFetching(e) {
                return this.#i.findAll({
                    ...e,
                    fetchStatus: "fetching"
                }).length
            }
            isMutating(e) {
                return this.#s.findAll({
                    ...e,
                    status: "pending"
                }).length
            }
            getQueryData(e) {
                let t = this.defaultQueryOptions({
                    queryKey: e
                });
                return this.#i.get(t.queryHash)?.state.data
            }
            ensureQueryData(e) {
                let t = this.defaultQueryOptions(e)
                  , n = this.#i.build(this, t)
                  , i = n.state.data;
                return void 0 === i ? this.fetchQuery(e) : (e.revalidateIfStale && n.isStaleByTime((0,
                r.d2)(t.staleTime, n)) && this.prefetchQuery(t),
                Promise.resolve(i))
            }
            getQueriesData(e) {
                return this.#i.findAll(e).map( ({queryKey: e, state: t}) => [e, t.data])
            }
            setQueryData(e, t, n) {
                let i = this.defaultQueryOptions({
                    queryKey: e
                })
                  , s = this.#i.get(i.queryHash)
                  , a = s?.state.data
                  , o = (0,
                r.Zw)(t, a);
                if (void 0 !== o)
                    return this.#i.build(this, i).setData(o, {
                        ...n,
                        manual: !0
                    })
            }
            setQueriesData(e, t, n) {
                return s.jG.batch( () => this.#i.findAll(e).map( ({queryKey: e}) => [e, this.setQueryData(e, t, n)]))
            }
            getQueryState(e) {
                let t = this.defaultQueryOptions({
                    queryKey: e
                });
                return this.#i.get(t.queryHash)?.state
            }
            removeQueries(e) {
                let t = this.#i;
                s.jG.batch( () => {
                    t.findAll(e).forEach(e => {
                        t.remove(e)
                    }
                    )
                }
                )
            }
            resetQueries(e, t) {
                let n = this.#i;
                return s.jG.batch( () => (n.findAll(e).forEach(e => {
                    e.reset()
                }
                ),
                this.refetchQueries({
                    type: "active",
                    ...e
                }, t)))
            }
            cancelQueries(e, t={}) {
                let n = {
                    revert: !0,
                    ...t
                };
                return Promise.all(s.jG.batch( () => this.#i.findAll(e).map(e => e.cancel(n)))).then(r.lQ).catch(r.lQ)
            }
            invalidateQueries(e, t={}) {
                return s.jG.batch( () => (this.#i.findAll(e).forEach(e => {
                    e.invalidate()
                }
                ),
                e?.refetchType === "none") ? Promise.resolve() : this.refetchQueries({
                    ...e,
                    type: e?.refetchType ?? e?.type ?? "active"
                }, t))
            }
            refetchQueries(e, t={}) {
                let n = {
                    ...t,
                    cancelRefetch: t.cancelRefetch ?? !0
                };
                return Promise.all(s.jG.batch( () => this.#i.findAll(e).filter(e => !e.isDisabled() && !e.isStatic()).map(e => {
                    let t = e.fetch(void 0, n);
                    return n.throwOnError || (t = t.catch(r.lQ)),
                    "paused" === e.state.fetchStatus ? Promise.resolve() : t
                }
                ))).then(r.lQ)
            }
            fetchQuery(e) {
                let t = this.defaultQueryOptions(e);
                void 0 === t.retry && (t.retry = !1);
                let n = this.#i.build(this, t);
                return n.isStaleByTime((0,
                r.d2)(t.staleTime, n)) ? n.fetch(t) : Promise.resolve(n.state.data)
            }
            prefetchQuery(e) {
                return this.fetchQuery(e).then(r.lQ).catch(r.lQ)
            }
            fetchInfiniteQuery(e) {
                return e.behavior = p(e.pages),
                this.fetchQuery(e)
            }
            prefetchInfiniteQuery(e) {
                return this.fetchInfiniteQuery(e).then(r.lQ).catch(r.lQ)
            }
            ensureInfiniteQueryData(e) {
                return e.behavior = p(e.pages),
                this.ensureQueryData(e)
            }
            resumePausedMutations() {
                return h.t.isOnline() ? this.#s.resumePausedMutations() : Promise.resolve()
            }
            getQueryCache() {
                return this.#i
            }
            getMutationCache() {
                return this.#s
            }
            getDefaultOptions() {
                return this.#a
            }
            setDefaultOptions(e) {
                this.#a = e
            }
            setQueryDefaults(e, t) {
                this.#o.set((0,
                r.EN)(e), {
                    queryKey: e,
                    defaultOptions: t
                })
            }
            getQueryDefaults(e) {
                let t = [...this.#o.values()]
                  , n = {};
                return t.forEach(t => {
                    (0,
                    r.Cp)(e, t.queryKey) && Object.assign(n, t.defaultOptions)
                }
                ),
                n
            }
            setMutationDefaults(e, t) {
                this.#l.set((0,
                r.EN)(e), {
                    mutationKey: e,
                    defaultOptions: t
                })
            }
            getMutationDefaults(e) {
                let t = [...this.#l.values()]
                  , n = {};
                return t.forEach(t => {
                    (0,
                    r.Cp)(e, t.mutationKey) && Object.assign(n, t.defaultOptions)
                }
                ),
                n
            }
            defaultQueryOptions(e) {
                if (e._defaulted)
                    return e;
                let t = {
                    ...this.#a.queries,
                    ...this.getQueryDefaults(e.queryKey),
                    ...e,
                    _defaulted: !0
                };
                return t.queryHash || (t.queryHash = (0,
                r.F$)(t.queryKey, t)),
                void 0 === t.refetchOnReconnect && (t.refetchOnReconnect = "always" !== t.networkMode),
                void 0 === t.throwOnError && (t.throwOnError = !!t.suspense),
                !t.networkMode && t.persister && (t.networkMode = "offlineFirst"),
                t.queryFn === r.hT && (t.enabled = !1),
                t
            }
            defaultMutationOptions(e) {
                return e?._defaulted ? e : {
                    ...this.#a.mutations,
                    ...e?.mutationKey && this.getMutationDefaults(e.mutationKey),
                    ...e,
                    _defaulted: !0
                }
            }
            clear() {
                this.#i.clear(),
                this.#s.clear()
            }
        }
    }
    ,
    48279: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "M8.25 9V5.25A2.25 2.25 0 0 1 10.5 3h6a2.25 2.25 0 0 1 2.25 2.25v13.5A2.25 2.25 0 0 1 16.5 21h-6a2.25 2.25 0 0 1-2.25-2.25V15M12 9l3 3m0 0-3 3m3-3H2.25"
            }))
        })
    }
    ,
    50358: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                viewBox: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                fillRule: "evenodd",
                d: "M4.5 3.75a3 3 0 0 0-3 3v10.5a3 3 0 0 0 3 3h15a3 3 0 0 0 3-3V6.75a3 3 0 0 0-3-3h-15Zm4.125 3a2.25 2.25 0 1 0 0 4.5 2.25 2.25 0 0 0 0-4.5Zm-3.873 8.703a4.126 4.126 0 0 1 7.746 0 .75.75 0 0 1-.351.92 7.47 7.47 0 0 1-3.522.877 7.47 7.47 0 0 1-3.522-.877.75.75 0 0 1-.351-.92ZM15 8.25a.75.75 0 0 0 0 1.5h3.75a.75.75 0 0 0 0-1.5H15ZM14.25 12a.75.75 0 0 1 .75-.75h3.75a.75.75 0 0 1 0 1.5H15a.75.75 0 0 1-.75-.75Zm.75 2.25a.75.75 0 0 0 0 1.5h3.75a.75.75 0 0 0 0-1.5H15Z",
                clipRule: "evenodd"
            }))
        })
    }
    ,
    50901: (e, t, n) => {
        "use strict";
        n.d(t, {
            DE: () => o
        });
        var r = n(72286)
          , i = n(17972)
          , s = n(89037);
        class a extends r.Ce {
            async sendTransaction(e, t, n={}) {
                let r = !0;
                try {
                    if ((0,
                    s.Y)(e)) {
                        if (!this.supportedTransactionVersions)
                            throw new i.UF("Sending versioned transactions isn't supported by this wallet");
                        if (!this.supportedTransactionVersions.has(e.version))
                            throw new i.UF(`Sending transaction version ${e.version} isn't supported by this wallet`);
                        try {
                            let r = (e = await this.signTransaction(e)).serialize();
                            return await t.sendRawTransaction(r, n)
                        } catch (e) {
                            if (e instanceof i.z4)
                                throw r = !1,
                                e;
                            throw new i.UF(e?.message,e)
                        }
                    }
                    try {
                        let {signers: r, ...i} = n;
                        e = await this.prepareTransaction(e, t, i),
                        r?.length && e.partialSign(...r);
                        let s = (e = await this.signTransaction(e)).serialize();
                        return await t.sendRawTransaction(s, i)
                    } catch (e) {
                        if (e instanceof i.z4)
                            throw r = !1,
                            e;
                        throw new i.UF(e?.message,e)
                    }
                } catch (e) {
                    throw r && this.emit("error", e),
                    e
                }
            }
            async signAllTransactions(e) {
                for (let t of e)
                    if ((0,
                    s.Y)(t)) {
                        if (!this.supportedTransactionVersions)
                            throw new i.z4("Signing versioned transactions isn't supported by this wallet");
                        if (!this.supportedTransactionVersions.has(t.version))
                            throw new i.z4(`Signing transaction version ${t.version} isn't supported by this wallet`)
                    }
                let t = [];
                for (let n of e)
                    t.push(await this.signTransaction(n));
                return t
            }
        }
        class o extends a {
        }
    }
    ,
    51366: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.isVersionedTransaction = function(e) {
            return "version"in e
        }
    }
    ,
    51427: function(e, t, n) {
        "use strict";
        var r = this && this.__createBinding || (Object.create ? function(e, t, n, r) {
            void 0 === r && (r = n);
            var i = Object.getOwnPropertyDescriptor(t, n);
            (!i || ("get"in i ? !t.__esModule : i.writable || i.configurable)) && (i = {
                enumerable: !0,
                get: function() {
                    return t[n]
                }
            }),
            Object.defineProperty(e, r, i)
        }
        : function(e, t, n, r) {
            void 0 === r && (r = n),
            e[r] = t[n]
        }
        )
          , i = this && this.__exportStar || function(e, t) {
            for (var n in e)
                "default" === n || Object.prototype.hasOwnProperty.call(t, n) || r(t, e, n)
        }
        ;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        i(n(14661), t),
        i(n(11155), t),
        i(n(60096), t)
    },
    52002: (e, t, n) => {
        "use strict";
        let r;
        n.d(t, {
            A: () => o
        });
        let i = {
            randomUUID: "undefined" != typeof crypto && crypto.randomUUID && crypto.randomUUID.bind(crypto)
        }
          , s = new Uint8Array(16)
          , a = [];
        for (let e = 0; e < 256; ++e)
            a.push((e + 256).toString(16).slice(1));
        let o = function(e, t, n) {
            if (i.randomUUID && !t && !e)
                return i.randomUUID();
            let o = (e = e || {}).random || (e.rng || function() {
                if (!r && !(r = "undefined" != typeof crypto && crypto.getRandomValues && crypto.getRandomValues.bind(crypto)))
                    throw Error("crypto.getRandomValues() not supported. See https://github.com/uuidjs/uuid#getrandomvalues-not-supported");
                return r(s)
            }
            )();
            if (o[6] = 15 & o[6] | 64,
            o[8] = 63 & o[8] | 128,
            t) {
                n = n || 0;
                for (let e = 0; e < 16; ++e)
                    t[n + e] = o[e];
                return t
            }
            return function(e, t=0) {
                return a[e[t + 0]] + a[e[t + 1]] + a[e[t + 2]] + a[e[t + 3]] + "-" + a[e[t + 4]] + a[e[t + 5]] + "-" + a[e[t + 6]] + a[e[t + 7]] + "-" + a[e[t + 8]] + a[e[t + 9]] + "-" + a[e[t + 10]] + a[e[t + 11]] + a[e[t + 12]] + a[e[t + 13]] + a[e[t + 14]] + a[e[t + 15]]
            }(o)
        }
    }
    ,
    52718: (e, t, n) => {
        "use strict";
        n.d(t, {
            a: () => s
        });
        var r = n(26432)
          , i = n(76603);
        function s() {
            let e = (0,
            r.useRef)(!1);
            return (0,
            i.s)( () => (e.current = !0,
            () => {
                e.current = !1
            }
            ), []),
            e
        }
    }
    ,
    53676: (e, t, n) => {
        "use strict";
        n.d(t, {
            R: () => r
        });
        let r = "solana:signAndSendTransaction"
    }
    ,
    55037: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SolanaSignAndSendTransaction = void 0,
        t.SolanaSignAndSendTransaction = "solana:signAndSendTransaction"
    }
    ,
    55366: (e, t, n) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.shake256 = t.shake128 = t.keccak_512 = t.keccak_384 = t.keccak_256 = t.keccak_224 = t.sha3_512 = t.sha3_384 = t.sha3_256 = t.sha3_224 = t.Keccak = void 0,
        t.keccakP = _;
        let r = n(45915)
          , i = n(10434)
          , s = BigInt(0)
          , a = BigInt(1)
          , o = BigInt(2)
          , l = BigInt(7)
          , c = BigInt(256)
          , d = BigInt(113)
          , u = []
          , h = []
          , p = [];
        for (let e = 0, t = a, n = 1, r = 0; e < 24; e++) {
            [n,r] = [r, (2 * n + 3 * r) % 5],
            u.push(2 * (5 * r + n)),
            h.push((e + 1) * (e + 2) / 2 % 64);
            let i = s;
            for (let e = 0; e < 7; e++)
                (t = (t << a ^ (t >> l) * d) % c) & o && (i ^= a << (a << BigInt(e)) - a);
            p.push(i)
        }
        let f = (0,
        r.split)(p, !0)
          , y = f[0]
          , m = f[1]
          , g = (e, t, n) => n > 32 ? (0,
        r.rotlBH)(e, t, n) : (0,
        r.rotlSH)(e, t, n)
          , w = (e, t, n) => n > 32 ? (0,
        r.rotlBL)(e, t, n) : (0,
        r.rotlSL)(e, t, n);
        function _(e, t=24) {
            let n = new Uint32Array(10);
            for (let r = 24 - t; r < 24; r++) {
                for (let t = 0; t < 10; t++)
                    n[t] = e[t] ^ e[t + 10] ^ e[t + 20] ^ e[t + 30] ^ e[t + 40];
                for (let t = 0; t < 10; t += 2) {
                    let r = (t + 8) % 10
                      , i = (t + 2) % 10
                      , s = n[i]
                      , a = n[i + 1]
                      , o = g(s, a, 1) ^ n[r]
                      , l = w(s, a, 1) ^ n[r + 1];
                    for (let n = 0; n < 50; n += 10)
                        e[t + n] ^= o,
                        e[t + n + 1] ^= l
                }
                let t = e[2]
                  , i = e[3];
                for (let n = 0; n < 24; n++) {
                    let r = h[n]
                      , s = g(t, i, r)
                      , a = w(t, i, r)
                      , o = u[n];
                    t = e[o],
                    i = e[o + 1],
                    e[o] = s,
                    e[o + 1] = a
                }
                for (let t = 0; t < 50; t += 10) {
                    for (let r = 0; r < 10; r++)
                        n[r] = e[t + r];
                    for (let r = 0; r < 10; r++)
                        e[t + r] ^= ~n[(r + 2) % 10] & n[(r + 4) % 10]
                }
                e[0] ^= y[r],
                e[1] ^= m[r]
            }
            (0,
            i.clean)(n)
        }
        class v extends i.Hash {
            constructor(e, t, n, r=!1, s=24) {
                if (super(),
                this.pos = 0,
                this.posOut = 0,
                this.finished = !1,
                this.destroyed = !1,
                this.enableXOF = !1,
                this.blockLen = e,
                this.suffix = t,
                this.outputLen = n,
                this.enableXOF = r,
                this.rounds = s,
                (0,
                i.anumber)(n),
                !(0 < e && e < 200))
                    throw Error("only keccak-f1600 function is supported");
                this.state = new Uint8Array(200),
                this.state32 = (0,
                i.u32)(this.state)
            }
            clone() {
                return this._cloneInto()
            }
            keccak() {
                (0,
                i.swap32IfBE)(this.state32),
                _(this.state32, this.rounds),
                (0,
                i.swap32IfBE)(this.state32),
                this.posOut = 0,
                this.pos = 0
            }
            update(e) {
                (0,
                i.aexists)(this),
                e = (0,
                i.toBytes)(e),
                (0,
                i.abytes)(e);
                let {blockLen: t, state: n} = this
                  , r = e.length;
                for (let i = 0; i < r; ) {
                    let s = Math.min(t - this.pos, r - i);
                    for (let t = 0; t < s; t++)
                        n[this.pos++] ^= e[i++];
                    this.pos === t && this.keccak()
                }
                return this
            }
            finish() {
                if (this.finished)
                    return;
                this.finished = !0;
                let {state: e, suffix: t, pos: n, blockLen: r} = this;
                e[n] ^= t,
                (128 & t) != 0 && n === r - 1 && this.keccak(),
                e[r - 1] ^= 128,
                this.keccak()
            }
            writeInto(e) {
                (0,
                i.aexists)(this, !1),
                (0,
                i.abytes)(e),
                this.finish();
                let t = this.state
                  , {blockLen: n} = this;
                for (let r = 0, i = e.length; r < i; ) {
                    this.posOut >= n && this.keccak();
                    let s = Math.min(n - this.posOut, i - r);
                    e.set(t.subarray(this.posOut, this.posOut + s), r),
                    this.posOut += s,
                    r += s
                }
                return e
            }
            xofInto(e) {
                if (!this.enableXOF)
                    throw Error("XOF is not possible for this instance");
                return this.writeInto(e)
            }
            xof(e) {
                return (0,
                i.anumber)(e),
                this.xofInto(new Uint8Array(e))
            }
            digestInto(e) {
                if ((0,
                i.aoutput)(e, this),
                this.finished)
                    throw Error("digest() was already called");
                return this.writeInto(e),
                this.destroy(),
                e
            }
            digest() {
                return this.digestInto(new Uint8Array(this.outputLen))
            }
            destroy() {
                this.destroyed = !0,
                (0,
                i.clean)(this.state)
            }
            _cloneInto(e) {
                let {blockLen: t, suffix: n, outputLen: r, rounds: i, enableXOF: s} = this;
                return e || (e = new v(t,n,r,s,i)),
                e.state32.set(this.state32),
                e.pos = this.pos,
                e.posOut = this.posOut,
                e.finished = this.finished,
                e.rounds = i,
                e.suffix = n,
                e.outputLen = r,
                e.enableXOF = s,
                e.destroyed = this.destroyed,
                e
            }
        }
        t.Keccak = v;
        let b = (e, t, n) => (0,
        i.createHasher)( () => new v(t,e,n));
        t.sha3_224 = b(6, 144, 28),
        t.sha3_256 = b(6, 136, 32),
        t.sha3_384 = b(6, 104, 48),
        t.sha3_512 = b(6, 72, 64),
        t.keccak_224 = b(1, 144, 28),
        t.keccak_256 = b(1, 136, 32),
        t.keccak_384 = b(1, 104, 48),
        t.keccak_512 = b(1, 72, 64);
        let E = (e, t, n) => (0,
        i.createXOFer)( (r={}) => new v(t,e,void 0 === r.dkLen ? n : r.dkLen,!0));
        t.shake128 = E(31, 168, 16),
        t.shake256 = E(31, 136, 32)
    }
    ,
    55567: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SolanaSignMessage = void 0,
        t.SolanaSignMessage = "solana:signMessage"
    }
    ,
    59595: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SolanaSignIn = void 0,
        t.SolanaSignIn = "solana:signIn"
    }
    ,
    60096: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.Events = t.StandardEvents = void 0,
        t.StandardEvents = "standard:events",
        t.Events = t.StandardEvents
    }
    ,
    61542: (e, t, n) => {
        "use strict";
        n.d(t, {
            I: () => m
        });
        var r = n(26432);
        let i = {
            setVisible(e) {
                console.error(s("call", "setVisible"))
            },
            visible: !1
        };
        function s(e, t) {
            return `You have tried to  ${e} "${t}" on a WalletModalContext without providing one. Make sure to render a WalletModalProvider as an ancestor of the component that uses WalletModalContext`
        }
        Object.defineProperty(i, "visible", {
            get: () => (console.error(s("read", "visible")),
            !1)
        });
        let a = (0,
        r.createContext)(i);
        var o = n(72286)
          , l = n(27519)
          , c = n(64145);
        let d = ({id: e, children: t, expanded: n=!1}) => {
            let i = (0,
            r.useRef)(null)
              , s = (0,
            r.useRef)(!0)
              , a = () => {
                let e = i.current;
                e && requestAnimationFrame( () => {
                    e.style.height = e.scrollHeight + "px"
                }
                )
            }
              , o = () => {
                let e = i.current;
                e && requestAnimationFrame( () => {
                    e.style.height = e.offsetHeight + "px",
                    e.style.overflow = "hidden",
                    requestAnimationFrame( () => {
                        e.style.height = "0"
                    }
                    )
                }
                )
            }
            ;
            return (0,
            r.useLayoutEffect)( () => {
                n ? a() : o()
            }
            , [n]),
            (0,
            r.useLayoutEffect)( () => {
                let e = i.current;
                if (e)
                    return s.current && (t(),
                    s.current = !1),
                    e.addEventListener("transitionend", r),
                    () => e.removeEventListener("transitionend", r);
                function t() {
                    e && (e.style.overflow = n ? "initial" : "hidden",
                    n && (e.style.height = "auto"))
                }
                function r(n) {
                    e && n.target === e && "height" === n.propertyName && t()
                }
            }
            , [n]),
            r.createElement("div", {
                className: "wallet-adapter-collapse",
                id: e,
                ref: i,
                role: "region",
                style: {
                    height: 0,
                    transition: s.current ? void 0 : "height 250ms ease-out"
                }
            }, t)
        }
          , u = e => r.createElement("button", {
            className: `wallet-adapter-button ${e.className || ""}`,
            disabled: e.disabled,
            style: e.style,
            onClick: e.onClick,
            tabIndex: e.tabIndex || 0,
            type: "button"
        }, e.startIcon && r.createElement("i", {
            className: "wallet-adapter-button-start-icon"
        }, e.startIcon), e.children, e.endIcon && r.createElement("i", {
            className: "wallet-adapter-button-end-icon"
        }, e.endIcon))
          , h = ({wallet: e, ...t}) => e && r.createElement("img", {
            src: e.adapter.icon,
            alt: `${e.adapter.name} icon`,
            ...t
        })
          , p = ({handleClick: e, tabIndex: t, wallet: n}) => r.createElement("li", null, r.createElement(u, {
            onClick: e,
            startIcon: r.createElement(h, {
                wallet: n
            }),
            tabIndex: t
        }, n.adapter.name, n.readyState === o.Ok.Installed && r.createElement("span", null, "Detected")))
          , f = () => r.createElement("svg", {
            width: "97",
            height: "96",
            viewBox: "0 0 97 96",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg"
        }, r.createElement("circle", {
            cx: "48.5",
            cy: "48",
            r: "48",
            fill: "url(#paint0_linear_880_5115)",
            fillOpacity: "0.1"
        }), r.createElement("circle", {
            cx: "48.5",
            cy: "48",
            r: "47",
            stroke: "url(#paint1_linear_880_5115)",
            strokeOpacity: "0.4",
            strokeWidth: "2"
        }), r.createElement("g", {
            clipPath: "url(#clip0_880_5115)"
        }, r.createElement("path", {
            d: "M65.5769 28.1523H31.4231C27.6057 28.1523 24.5 31.258 24.5 35.0754V60.9215C24.5 64.7389 27.6057 67.8446 31.4231 67.8446H65.5769C69.3943 67.8446 72.5 64.7389 72.5 60.9215V35.0754C72.5 31.258 69.3943 28.1523 65.5769 28.1523ZM69.7308 52.1523H59.5769C57.2865 52.1523 55.4231 50.289 55.4231 47.9985C55.4231 45.708 57.2864 43.8446 59.5769 43.8446H69.7308V52.1523ZM69.7308 41.0754H59.5769C55.7595 41.0754 52.6539 44.1811 52.6539 47.9985C52.6539 51.8159 55.7595 54.9215 59.5769 54.9215H69.7308V60.9215C69.7308 63.2119 67.8674 65.0754 65.5769 65.0754H31.4231C29.1327 65.0754 27.2692 63.212 27.2692 60.9215V35.0754C27.2692 32.785 29.1326 30.9215 31.4231 30.9215H65.5769C67.8673 30.9215 69.7308 32.7849 69.7308 35.0754V41.0754Z",
            fill: "url(#paint2_linear_880_5115)"
        }), r.createElement("path", {
            d: "M61.4231 46.6172H59.577C58.8123 46.6172 58.1924 47.2371 58.1924 48.0018C58.1924 48.7665 58.8123 49.3863 59.577 49.3863H61.4231C62.1878 49.3863 62.8077 48.7664 62.8077 48.0018C62.8077 47.2371 62.1878 46.6172 61.4231 46.6172Z",
            fill: "url(#paint3_linear_880_5115)"
        })), r.createElement("defs", null, r.createElement("linearGradient", {
            id: "paint0_linear_880_5115",
            x1: "3.41664",
            y1: "98.0933",
            x2: "103.05",
            y2: "8.42498",
            gradientUnits: "userSpaceOnUse"
        }, r.createElement("stop", {
            stopColor: "#9945FF"
        }), r.createElement("stop", {
            offset: "0.14",
            stopColor: "#8A53F4"
        }), r.createElement("stop", {
            offset: "0.42",
            stopColor: "#6377D6"
        }), r.createElement("stop", {
            offset: "0.79",
            stopColor: "#24B0A7"
        }), r.createElement("stop", {
            offset: "0.99",
            stopColor: "#00D18C"
        }), r.createElement("stop", {
            offset: "1",
            stopColor: "#00D18C"
        })), r.createElement("linearGradient", {
            id: "paint1_linear_880_5115",
            x1: "3.41664",
            y1: "98.0933",
            x2: "103.05",
            y2: "8.42498",
            gradientUnits: "userSpaceOnUse"
        }, r.createElement("stop", {
            stopColor: "#9945FF"
        }), r.createElement("stop", {
            offset: "0.14",
            stopColor: "#8A53F4"
        }), r.createElement("stop", {
            offset: "0.42",
            stopColor: "#6377D6"
        }), r.createElement("stop", {
            offset: "0.79",
            stopColor: "#24B0A7"
        }), r.createElement("stop", {
            offset: "0.99",
            stopColor: "#00D18C"
        }), r.createElement("stop", {
            offset: "1",
            stopColor: "#00D18C"
        })), r.createElement("linearGradient", {
            id: "paint2_linear_880_5115",
            x1: "25.9583",
            y1: "68.7101",
            x2: "67.2337",
            y2: "23.7879",
            gradientUnits: "userSpaceOnUse"
        }, r.createElement("stop", {
            stopColor: "#9945FF"
        }), r.createElement("stop", {
            offset: "0.14",
            stopColor: "#8A53F4"
        }), r.createElement("stop", {
            offset: "0.42",
            stopColor: "#6377D6"
        }), r.createElement("stop", {
            offset: "0.79",
            stopColor: "#24B0A7"
        }), r.createElement("stop", {
            offset: "0.99",
            stopColor: "#00D18C"
        }), r.createElement("stop", {
            offset: "1",
            stopColor: "#00D18C"
        })), r.createElement("linearGradient", {
            id: "paint3_linear_880_5115",
            x1: "58.3326",
            y1: "49.4467",
            x2: "61.0002",
            y2: "45.4453",
            gradientUnits: "userSpaceOnUse"
        }, r.createElement("stop", {
            stopColor: "#9945FF"
        }), r.createElement("stop", {
            offset: "0.14",
            stopColor: "#8A53F4"
        }), r.createElement("stop", {
            offset: "0.42",
            stopColor: "#6377D6"
        }), r.createElement("stop", {
            offset: "0.79",
            stopColor: "#24B0A7"
        }), r.createElement("stop", {
            offset: "0.99",
            stopColor: "#00D18C"
        }), r.createElement("stop", {
            offset: "1",
            stopColor: "#00D18C"
        })), r.createElement("clipPath", {
            id: "clip0_880_5115"
        }, r.createElement("rect", {
            width: "48",
            height: "48",
            fill: "white",
            transform: "translate(24.5 24)"
        }))))
          , y = ({className: e="", container: t="body"}) => {
            let n = (0,
            r.useRef)(null)
              , {wallets: i, select: s} = (0,
            l.v)()
              , {setVisible: u} = (0,
            r.useContext)(a)
              , [h,y] = (0,
            r.useState)(!1)
              , [m,g] = (0,
            r.useState)(!1)
              , [w,_] = (0,
            r.useState)(null)
              , [v,b] = (0,
            r.useMemo)( () => {
                let e = []
                  , t = [];
                for (let n of i)
                    n.readyState === o.Ok.Installed ? e.push(n) : t.push(n);
                return e.length ? [e, t] : [t, []]
            }
            , [i])
              , E = (0,
            r.useCallback)( () => {
                g(!1),
                setTimeout( () => u(!1), 150)
            }
            , [u])
              , I = (0,
            r.useCallback)(e => {
                e.preventDefault(),
                E()
            }
            , [E])
              , k = (0,
            r.useCallback)( (e, t) => {
                s(t),
                I(e)
            }
            , [s, I])
              , M = (0,
            r.useCallback)( () => y(!h), [h])
              , S = (0,
            r.useCallback)(e => {
                let t = n.current;
                if (!t)
                    return;
                let r = t.querySelectorAll("button")
                  , i = r[0]
                  , s = r[r.length - 1];
                e.shiftKey ? document.activeElement === i && (s.focus(),
                e.preventDefault()) : document.activeElement === s && (i.focus(),
                e.preventDefault())
            }
            , [n]);
            return (0,
            r.useLayoutEffect)( () => {
                let e = e => {
                    "Escape" === e.key ? E() : "Tab" === e.key && S(e)
                }
                  , {overflow: t} = window.getComputedStyle(document.body);
                return setTimeout( () => g(!0), 0),
                document.body.style.overflow = "hidden",
                window.addEventListener("keydown", e, !1),
                () => {
                    document.body.style.overflow = t,
                    window.removeEventListener("keydown", e, !1)
                }
            }
            , [E, S]),
            (0,
            r.useLayoutEffect)( () => _(document.querySelector(t)), [t]),
            w && (0,
            c.createPortal)(r.createElement("div", {
                "aria-labelledby": "wallet-adapter-modal-title",
                "aria-modal": "true",
                className: `wallet-adapter-modal ${m && "wallet-adapter-modal-fade-in"} ${e}`,
                ref: n,
                role: "dialog"
            }, r.createElement("div", {
                className: "wallet-adapter-modal-container"
            }, r.createElement("div", {
                className: "wallet-adapter-modal-wrapper"
            }, r.createElement("button", {
                onClick: I,
                className: "wallet-adapter-modal-button-close"
            }, r.createElement("svg", {
                width: "14",
                height: "14"
            }, r.createElement("path", {
                d: "M14 12.461 8.3 6.772l5.234-5.233L12.006 0 6.772 5.234 1.54 0 0 1.539l5.234 5.233L0 12.006l1.539 1.528L6.772 8.3l5.69 5.7L14 12.461z"
            }))), v.length ? r.createElement(r.Fragment, null, r.createElement("h1", {
                className: "wallet-adapter-modal-title"
            }, "Connect a wallet on Solana to continue"), r.createElement("ul", {
                className: "wallet-adapter-modal-list"
            }, v.map(e => r.createElement(p, {
                key: e.adapter.name,
                handleClick: t => k(t, e.adapter.name),
                wallet: e
            })), b.length ? r.createElement(d, {
                expanded: h,
                id: "wallet-adapter-modal-collapse"
            }, b.map(e => r.createElement(p, {
                key: e.adapter.name,
                handleClick: t => k(t, e.adapter.name),
                tabIndex: h ? 0 : -1,
                wallet: e
            }))) : null), b.length ? r.createElement("button", {
                className: "wallet-adapter-modal-list-more",
                onClick: M,
                tabIndex: 0
            }, r.createElement("span", null, h ? "Less " : "More ", "options"), r.createElement("svg", {
                width: "13",
                height: "7",
                viewBox: "0 0 13 7",
                xmlns: "http://www.w3.org/2000/svg",
                className: `${h ? "wallet-adapter-modal-list-more-icon-rotate" : ""}`
            }, r.createElement("path", {
                d: "M0.71418 1.626L5.83323 6.26188C5.91574 6.33657 6.0181 6.39652 6.13327 6.43762C6.24844 6.47872 6.37371 6.5 6.50048 6.5C6.62725 6.5 6.75252 6.47872 6.8677 6.43762C6.98287 6.39652 7.08523 6.33657 7.16774 6.26188L12.2868 1.626C12.7753 1.1835 12.3703 0.5 11.6195 0.5H1.37997C0.629216 0.5 0.224175 1.1835 0.71418 1.626Z"
            }))) : null) : r.createElement(r.Fragment, null, r.createElement("h1", {
                className: "wallet-adapter-modal-title"
            }, "You'll need a wallet on Solana to continue"), r.createElement("div", {
                className: "wallet-adapter-modal-middle"
            }, r.createElement(f, null)), b.length ? r.createElement(r.Fragment, null, r.createElement("button", {
                className: "wallet-adapter-modal-list-more",
                onClick: M,
                tabIndex: 0
            }, r.createElement("span", null, h ? "Hide " : "Already have a wallet? View ", "options"), r.createElement("svg", {
                width: "13",
                height: "7",
                viewBox: "0 0 13 7",
                xmlns: "http://www.w3.org/2000/svg",
                className: `${h ? "wallet-adapter-modal-list-more-icon-rotate" : ""}`
            }, r.createElement("path", {
                d: "M0.71418 1.626L5.83323 6.26188C5.91574 6.33657 6.0181 6.39652 6.13327 6.43762C6.24844 6.47872 6.37371 6.5 6.50048 6.5C6.62725 6.5 6.75252 6.47872 6.8677 6.43762C6.98287 6.39652 7.08523 6.33657 7.16774 6.26188L12.2868 1.626C12.7753 1.1835 12.3703 0.5 11.6195 0.5H1.37997C0.629216 0.5 0.224175 1.1835 0.71418 1.626Z"
            }))), r.createElement(d, {
                expanded: h,
                id: "wallet-adapter-modal-collapse"
            }, r.createElement("ul", {
                className: "wallet-adapter-modal-list"
            }, b.map(e => r.createElement(p, {
                key: e.adapter.name,
                handleClick: t => k(t, e.adapter.name),
                tabIndex: h ? 0 : -1,
                wallet: e
            }))))) : null))), r.createElement("div", {
                className: "wallet-adapter-modal-overlay",
                onMouseDown: I
            })), w)
        }
          , m = ({children: e, ...t}) => {
            let[n,i] = (0,
            r.useState)(!1);
            return r.createElement(a.Provider, {
                value: {
                    visible: n,
                    setVisible: i
                }
            }, e, n && r.createElement(y, {
                ...t
            }))
        }
    }
    ,
    62257: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 0 1 2.25-2.25h13.5A2.25 2.25 0 0 1 21 7.5v11.25m-18 0A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75m-18 0v-7.5A2.25 2.25 0 0 1 5.25 9h13.5A2.25 2.25 0 0 1 21 11.25v7.5"
            }))
        })
    }
    ,
    68429: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.WalletWindowClosedError = t.WalletWindowBlockedError = t.WalletTimeoutError = t.WalletSignInError = t.WalletSignMessageError = t.WalletSignTransactionError = t.WalletSendTransactionError = t.WalletNotConnectedError = t.WalletKeypairError = t.WalletPublicKeyError = t.WalletAccountError = t.WalletDisconnectionError = t.WalletDisconnectedError = t.WalletConnectionError = t.WalletConfigError = t.WalletLoadError = t.WalletNotReadyError = t.WalletError = void 0;
        class n extends Error {
            constructor(e, t) {
                super(e),
                this.error = t
            }
        }
        t.WalletError = n;
        class r extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletNotReadyError"
            }
        }
        t.WalletNotReadyError = r;
        class i extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletLoadError"
            }
        }
        t.WalletLoadError = i;
        class s extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletConfigError"
            }
        }
        t.WalletConfigError = s;
        class a extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletConnectionError"
            }
        }
        t.WalletConnectionError = a;
        class o extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletDisconnectedError"
            }
        }
        t.WalletDisconnectedError = o;
        class l extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletDisconnectionError"
            }
        }
        t.WalletDisconnectionError = l;
        class c extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletAccountError"
            }
        }
        t.WalletAccountError = c;
        class d extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletPublicKeyError"
            }
        }
        t.WalletPublicKeyError = d;
        class u extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletKeypairError"
            }
        }
        t.WalletKeypairError = u;
        class h extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletNotConnectedError"
            }
        }
        t.WalletNotConnectedError = h;
        class p extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletSendTransactionError"
            }
        }
        t.WalletSendTransactionError = p;
        class f extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletSignTransactionError"
            }
        }
        t.WalletSignTransactionError = f;
        class y extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletSignMessageError"
            }
        }
        t.WalletSignMessageError = y;
        class m extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletSignInError"
            }
        }
        t.WalletSignInError = m;
        class g extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletTimeoutError"
            }
        }
        t.WalletTimeoutError = g;
        class w extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletWindowBlockedError"
            }
        }
        t.WalletWindowBlockedError = w;
        class _ extends n {
            constructor() {
                super(...arguments),
                this.name = "WalletWindowClosedError"
            }
        }
        t.WalletWindowClosedError = _
    }
    ,
    69335: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "M15 12H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
            }))
        })
    }
    ,
    69523: function(e, t, n) {
        "use strict";
        var r = this && this.__createBinding || (Object.create ? function(e, t, n, r) {
            void 0 === r && (r = n);
            var i = Object.getOwnPropertyDescriptor(t, n);
            (!i || ("get"in i ? !t.__esModule : i.writable || i.configurable)) && (i = {
                enumerable: !0,
                get: function() {
                    return t[n]
                }
            }),
            Object.defineProperty(e, r, i)
        }
        : function(e, t, n, r) {
            void 0 === r && (r = n),
            e[r] = t[n]
        }
        )
          , i = this && this.__exportStar || function(e, t) {
            for (var n in e)
                "default" === n || Object.prototype.hasOwnProperty.call(t, n) || r(t, e, n)
        }
        ;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        i(n(55037), t),
        i(n(59595), t),
        i(n(55567), t),
        i(n(41052), t),
        i(n(36803), t)
    },
    70410: (e, t) => {
        "use strict";
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.SOLANA_CHAINS = t.SOLANA_LOCALNET_CHAIN = t.SOLANA_TESTNET_CHAIN = t.SOLANA_DEVNET_CHAIN = t.SOLANA_MAINNET_CHAIN = void 0,
        t.isSolanaChain = function(e) {
            return t.SOLANA_CHAINS.includes(e)
        }
        ,
        t.SOLANA_MAINNET_CHAIN = "solana:mainnet",
        t.SOLANA_DEVNET_CHAIN = "solana:devnet",
        t.SOLANA_TESTNET_CHAIN = "solana:testnet",
        t.SOLANA_LOCALNET_CHAIN = "solana:localnet",
        t.SOLANA_CHAINS = [t.SOLANA_MAINNET_CHAIN, t.SOLANA_DEVNET_CHAIN, t.SOLANA_TESTNET_CHAIN, t.SOLANA_LOCALNET_CHAIN]
    }
    ,
    72008: (e, t, n) => {
        "use strict";
        var r;
        n.d(t, {
            B: () => r
        }),
        function(e) {
            e.Mainnet = "mainnet-beta",
            e.Testnet = "testnet",
            e.Devnet = "devnet"
        }(r || (r = {}))
    }
    ,
    73535: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => r
        });
        let r = (0,
        n(44074).A)("Ban", [["circle", {
            cx: "12",
            cy: "12",
            r: "10",
            key: "1mglay"
        }], ["path", {
            d: "m4.9 4.9 14.2 14.2",
            key: "1m5liu"
        }]])
    }
    ,
    74096: (e, t, n) => {
        "use strict";
        let r;
        n.r(t),
        n.d(t, {
            SolanaCloneAuthorization: () => p,
            SolanaMobileWalletAdapterError: () => a,
            SolanaMobileWalletAdapterErrorCode: () => s,
            SolanaMobileWalletAdapterProtocolError: () => l,
            SolanaMobileWalletAdapterProtocolErrorCode: () => o,
            SolanaSignInWithSolana: () => f,
            SolanaSignTransactions: () => h,
            startRemoteScenario: () => W,
            transact: () => P
        });
        let i = `(?:\\nURI: (?<uri>[^\\n]+))?(?:\\nVersion: (?<version>[^\\n]+))?(?:\\nChain ID: (?<chainId>[^\\n]+))?(?:\\nNonce: (?<nonce>[^\\n]+))?(?:\\nIssued At: (?<issuedAt>[^\\n]+))?(?:\\nExpiration Time: (?<expirationTime>[^\\n]+))?(?:\\nNot Before: (?<notBefore>[^\\n]+))?(?:\\nRequest ID: (?<requestId>[^\\n]+))?(?:\\nResources:(?<resources>(?:\\n- [^\\n]+)*))?`;
        RegExp(`^(?<domain>[^\\n]+?) wants you to sign in with your Solana account:\\n(?<address>[^\\n]+)(?:\\n|$)(?:\\n(?<statement>[\\S\\s]*?)(?:\\n|$))??${i}\\n*$`);
        let s = {
            ERROR_ASSOCIATION_PORT_OUT_OF_RANGE: "ERROR_ASSOCIATION_PORT_OUT_OF_RANGE",
            ERROR_REFLECTOR_ID_OUT_OF_RANGE: "ERROR_REFLECTOR_ID_OUT_OF_RANGE",
            ERROR_FORBIDDEN_WALLET_BASE_URL: "ERROR_FORBIDDEN_WALLET_BASE_URL",
            ERROR_SECURE_CONTEXT_REQUIRED: "ERROR_SECURE_CONTEXT_REQUIRED",
            ERROR_SESSION_CLOSED: "ERROR_SESSION_CLOSED",
            ERROR_SESSION_TIMEOUT: "ERROR_SESSION_TIMEOUT",
            ERROR_WALLET_NOT_FOUND: "ERROR_WALLET_NOT_FOUND",
            ERROR_INVALID_PROTOCOL_VERSION: "ERROR_INVALID_PROTOCOL_VERSION",
            ERROR_BROWSER_NOT_SUPPORTED: "ERROR_BROWSER_NOT_SUPPORTED"
        };
        class a extends Error {
            constructor(...e) {
                let[t,n,r] = e;
                super(n),
                this.code = t,
                this.data = r,
                this.name = "SolanaMobileWalletAdapterError"
            }
        }
        let o = {
            ERROR_AUTHORIZATION_FAILED: -1,
            ERROR_INVALID_PAYLOADS: -2,
            ERROR_NOT_SIGNED: -3,
            ERROR_NOT_SUBMITTED: -4,
            ERROR_TOO_MANY_PAYLOADS: -5,
            ERROR_ATTEST_ORIGIN_ANDROID: -100
        };
        class l extends Error {
            constructor(...e) {
                let[t,n,r,i] = e;
                super(r),
                this.code = n,
                this.data = i,
                this.jsonRpcMessageId = t,
                this.name = "SolanaMobileWalletAdapterProtocolError"
            }
        }
        function c(e, t, n, r) {
            return new (n || (n = Promise))(function(i, s) {
                function a(e) {
                    try {
                        l(r.next(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function o(e) {
                    try {
                        l(r.throw(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function l(e) {
                    var t;
                    e.done ? i(e.value) : ((t = e.value)instanceof n ? t : new n(function(e) {
                        e(t)
                    }
                    )).then(a, o)
                }
                l((r = r.apply(e, t || [])).next())
            }
            )
        }
        function d(e, t) {
            let n = window.btoa(String.fromCharCode.call(null, ...e));
            return t ? n.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "") : n
        }
        function u(e, t) {
            return c(this, void 0, void 0, function*() {
                let n = yield crypto.subtle.exportKey("raw", e)
                  , r = yield crypto.subtle.sign({
                    hash: "SHA-256",
                    name: "ECDSA"
                }, t, n)
                  , i = new Uint8Array(n.byteLength + r.byteLength);
                return i.set(new Uint8Array(n), 0),
                i.set(new Uint8Array(r), n.byteLength),
                i
            })
        }
        let h = "solana:signTransactions"
          , p = "solana:cloneAuthorization"
          , f = "solana:signInWithSolana";
        function y(e, t) {
            return new Proxy({},{
                get: (n, r) => "then" === r ? null : (null == n[r] && (n[r] = function(n) {
                    return c(this, void 0, void 0, function*() {
                        let {method: i, params: s} = function(e, t, n) {
                            let r = t
                              , i = e.toString().replace(/[A-Z]/g, e => `_${e.toLowerCase()}`).toLowerCase();
                            switch (e) {
                            case "authorize":
                                {
                                    let {chain: e} = r;
                                    if ("legacy" === n) {
                                        switch (e) {
                                        case "solana:testnet":
                                            e = "testnet";
                                            break;
                                        case "solana:devnet":
                                            e = "devnet";
                                            break;
                                        case "solana:mainnet":
                                            e = "mainnet-beta";
                                            break;
                                        default:
                                            e = r.cluster
                                        }
                                        r.cluster = e
                                    } else {
                                        switch (e) {
                                        case "testnet":
                                        case "devnet":
                                            e = `solana:${e}`;
                                            break;
                                        case "mainnet-beta":
                                            e = "solana:mainnet"
                                        }
                                        r.chain = e
                                    }
                                }
                            case "reauthorize":
                                {
                                    let {auth_token: e, identity: t} = r;
                                    e && ("legacy" === n ? (i = "reauthorize",
                                    r = {
                                        auth_token: e,
                                        identity: t
                                    }) : i = "authorize")
                                }
                            }
                            return {
                                method: i,
                                params: r
                            }
                        }(r, n, e)
                          , a = yield t(i, s);
                        return "authorize" === i && s.sign_in_payload && !a.sign_in_result && (a.sign_in_result = yield function(e, t, n) {
                            var r;
                            return c(this, void 0, void 0, function*() {
                                var i;
                                let s = null != (r = e.domain) ? r : window.location.host
                                  , a = t.accounts[0].address
                                  , o = (i = function(e) {
                                    let t = `${e.domain} wants you to sign in with your Solana account:
`;
                                    t += `${e.address}`,
                                    e.statement && (t += `

${e.statement}`);
                                    let n = [];
                                    if (e.uri && n.push(`URI: ${e.uri}`),
                                    e.version && n.push(`Version: ${e.version}`),
                                    e.chainId && n.push(`Chain ID: ${e.chainId}`),
                                    e.nonce && n.push(`Nonce: ${e.nonce}`),
                                    e.issuedAt && n.push(`Issued At: ${e.issuedAt}`),
                                    e.expirationTime && n.push(`Expiration Time: ${e.expirationTime}`),
                                    e.notBefore && n.push(`Not Before: ${e.notBefore}`),
                                    e.requestId && n.push(`Request ID: ${e.requestId}`),
                                    e.resources)
                                        for (let t of (n.push("Resources:"),
                                        e.resources))
                                            n.push(`- ${t}`);
                                    return n.length && (t += `

${n.join("\n")}`),
                                    t
                                }(Object.assign(Object.assign({}, e), {
                                    domain: s,
                                    address: a
                                })),
                                window.btoa(i))
                                  , l = yield n("sign_messages", {
                                    addresses: [a],
                                    payloads: [o]
                                });
                                return {
                                    address: a,
                                    signed_message: o,
                                    signature: l.signed_payloads[0].slice(o.length)
                                }
                            })
                        }(s.sign_in_payload, a, t)),
                        function(e, t, n) {
                            if ("getCapabilities" === e)
                                switch (n) {
                                case "legacy":
                                    {
                                        let e = [h];
                                        return !0 === t.supports_clone_authorization && e.push(p),
                                        Object.assign(Object.assign({}, t), {
                                            features: e
                                        })
                                    }
                                case "v1":
                                    return Object.assign(Object.assign({}, t), {
                                        supports_sign_and_send_transactions: !0,
                                        supports_clone_authorization: t.features.includes(p)
                                    })
                                }
                            return t
                        }(r, a, e)
                    })
                }
                ),
                n[r]),
                defineProperty: () => !1,
                deleteProperty: () => !1
            })
        }
        function m(e, t) {
            return c(this, void 0, void 0, function*() {
                let n = e.slice(0, 4)
                  , i = e.slice(4, 16)
                  , s = e.slice(16)
                  , a = yield crypto.subtle.decrypt(g(n, i), t, s);
                return (void 0 === r && (r = new TextDecoder("utf-8")),
                r).decode(a)
            })
        }
        function g(e, t) {
            return {
                additionalData: e,
                iv: t,
                name: "AES-GCM",
                tagLength: 128
            }
        }
        function w() {
            return c(this, void 0, void 0, function*() {
                return yield crypto.subtle.generateKey({
                    name: "ECDSA",
                    namedCurve: "P-256"
                }, !1, ["sign"])
            })
        }
        function _() {
            return c(this, void 0, void 0, function*() {
                return yield crypto.subtle.generateKey({
                    name: "ECDH",
                    namedCurve: "P-256"
                }, !1, ["deriveKey", "deriveBits"])
            })
        }
        function v(e) {
            let t = ""
              , n = new Uint8Array(e)
              , r = n.byteLength;
            for (let e = 0; e < r; e++)
                t += String.fromCharCode(n[e]);
            return window.btoa(t)
        }
        function b(e) {
            if (e < 49152 || e > 65535)
                throw new a(s.ERROR_ASSOCIATION_PORT_OUT_OF_RANGE,`Association port number must be between 49152 and 65535. ${e} given.`,{
                    port: e
                });
            return e
        }
        function E(e) {
            return e.replace(/[/+=]/g, e => ({
                "/": "_",
                "+": "-",
                "=": "."
            })[e])
        }
        function I(e) {
            return e.replace(/(^\/+|\/+$)/g, "").split("/")
        }
        function k(e, t) {
            let n = null;
            if (t) {
                try {
                    n = new URL(t)
                } catch (e) {}
                if ((null == n ? void 0 : n.protocol) !== "https:")
                    throw new a(s.ERROR_FORBIDDEN_WALLET_BASE_URL,"Base URLs supplied by wallets must be valid `https` URLs")
            }
            return n || (n = new URL("solana-wallet:/")),
            new URL(e.startsWith("/") ? e : [...I(n.pathname), ...I(e)].join("/"),n)
        }
        function M(e, t) {
            return c(this, void 0, void 0, function*() {
                return function(e, t, n) {
                    return c(this, void 0, void 0, function*() {
                        let r = function(e) {
                            if (e >= 0x100000000)
                                throw Error("Outbound sequence number overflow. The maximum sequence number is 32-bytes.");
                            let t = new ArrayBuffer(4);
                            return new DataView(t).setUint32(0, e, !1),
                            new Uint8Array(t)
                        }(t)
                          , i = new Uint8Array(12);
                        crypto.getRandomValues(i);
                        let s = yield crypto.subtle.encrypt(g(r, i), n, new TextEncoder().encode(e))
                          , a = new Uint8Array(r.byteLength + i.byteLength + s.byteLength);
                        return a.set(new Uint8Array(r), 0),
                        a.set(new Uint8Array(i), r.byteLength),
                        a.set(new Uint8Array(s), r.byteLength + i.byteLength),
                        a
                    })
                }(JSON.stringify(e), e.id, t)
            })
        }
        function S(e, t) {
            return c(this, void 0, void 0, function*() {
                let n = JSON.parse((yield m(e, t)));
                if (Object.hasOwnProperty.call(n, "error"))
                    throw new l(n.id,n.error.code,n.error.message);
                return n
            })
        }
        function A(e, t, n) {
            return c(this, void 0, void 0, function*() {
                let[r,i] = yield Promise.all([crypto.subtle.exportKey("raw", t), crypto.subtle.importKey("raw", e.slice(0, 65), {
                    name: "ECDH",
                    namedCurve: "P-256"
                }, !1, [])])
                  , s = yield crypto.subtle.deriveBits({
                    name: "ECDH",
                    public: i
                }, n, 256)
                  , a = yield crypto.subtle.importKey("raw", s, "HKDF", !1, ["deriveKey"]);
                return yield crypto.subtle.deriveKey({
                    name: "HKDF",
                    hash: "SHA-256",
                    salt: new Uint8Array(r),
                    info: new Uint8Array
                }, a, {
                    name: "AES-GCM",
                    length: 128
                }, !1, ["encrypt", "decrypt"])
            })
        }
        function T(e, t) {
            return c(this, void 0, void 0, function*() {
                let n = JSON.parse((yield m(e, t)))
                  , r = "legacy";
                if (Object.hasOwnProperty.call(n, "v"))
                    switch (n.v) {
                    case 1:
                    case "1":
                    case "v1":
                        r = "v1";
                        break;
                    case "legacy":
                        r = "legacy";
                        break;
                    default:
                        throw new a(s.ERROR_INVALID_PROTOCOL_VERSION,`Unknown/unsupported protocol version: ${n.v}`)
                    }
                return {
                    protocol_version: r
                }
            })
        }
        let C = {
            Firefox: 0,
            Other: 1
        }
          , O = null
          , N = {
            retryDelayScheduleMs: [150, 150, 200, 500, 500, 750, 750, 1e3],
            timeoutMs: 3e4
        }
          , L = "com.solana.mobilewalletadapter.v1"
          , x = "com.solana.mobilewalletadapter.v1.base64";
        function j() {
            if ("undefined" == typeof window || !0 !== window.isSecureContext)
                throw new a(s.ERROR_SECURE_CONTEXT_REQUIRED,"The mobile wallet adapter protocol must be used in a secure context (`https`).")
        }
        function D(e) {
            let t;
            try {
                t = new URL(e)
            } catch (e) {
                throw new a(s.ERROR_FORBIDDEN_WALLET_BASE_URL,"Invalid base URL supplied by wallet")
            }
            if ("https:" !== t.protocol)
                throw new a(s.ERROR_FORBIDDEN_WALLET_BASE_URL,"Base URLs supplied by wallets must be valid `https` URLs")
        }
        function R(e) {
            return new DataView(e).getUint32(0, !1)
        }
        function P(e, t) {
            return c(this, void 0, void 0, function*() {
                let n;
                j();
                let r = yield w()
                  , i = yield function(e, t) {
                    return c(this, void 0, void 0, function*() {
                        let n = b(49152 + Math.floor(16384 * Math.random()))
                          , r = yield function(e, t, n, r=["v1"]) {
                            return c(this, void 0, void 0, function*() {
                                let i = b(t)
                                  , s = v((yield crypto.subtle.exportKey("raw", e)))
                                  , a = k("v1/associate/local", n);
                                return a.searchParams.set("association", E(s)),
                                a.searchParams.set("port", `${i}`),
                                r.forEach(e => {
                                    a.searchParams.set("v", e)
                                }
                                ),
                                a
                            })
                        }(e, n, t);
                        return yield function(e) {
                            return c(this, void 0, void 0, function*() {
                                if ("https:" === e.protocol)
                                    window.location.assign(e);
                                else
                                    try {
                                        switch (-1 !== navigator.userAgent.indexOf("Firefox/") ? C.Firefox : C.Other) {
                                        case C.Firefox:
                                            null == O && ((O = document.createElement("iframe")).style.display = "none",
                                            document.body.appendChild(O)),
                                            O.contentWindow.location.href = e.toString();
                                            break;
                                        case C.Other:
                                            {
                                                let t = new Promise( (e, t) => {
                                                    function n() {
                                                        clearTimeout(i),
                                                        window.removeEventListener("blur", r)
                                                    }
                                                    function r() {
                                                        n(),
                                                        e()
                                                    }
                                                    window.addEventListener("blur", r);
                                                    let i = setTimeout( () => {
                                                        n(),
                                                        t()
                                                    }
                                                    , 3e3)
                                                }
                                                );
                                                window.location.assign(e),
                                                yield t
                                            }
                                        }
                                    } catch (e) {
                                        throw new a(s.ERROR_WALLET_NOT_FOUND,"Found no installed wallet that supports the mobile wallet protocol.")
                                    }
                            })
                        }(r),
                        n
                    })
                }(r.publicKey, null == t ? void 0 : t.baseUri)
                  , o = `ws://localhost:${i}/solana-wallet`
                  , d = ( () => {
                    let e = [...N.retryDelayScheduleMs];
                    return () => e.length > 1 ? e.shift() : e[0]
                }
                )()
                  , h = 1
                  , p = 0
                  , f = {
                    __type: "disconnected"
                };
                return new Promise( (t, i) => {
                    let m, g, w, v = {}, b = () => c(this, void 0, void 0, function*() {
                        if ("connecting" !== f.__type)
                            return void console.warn(`Expected adapter state to be \`connecting\` at the moment the websocket opens. Got \`${f.__type}\`.`);
                        m.removeEventListener("open", b);
                        let {associationKeypair: e} = f
                          , t = yield _();
                        m.send((yield u(t.publicKey, e.privateKey))),
                        f = {
                            __type: "hello_req_sent",
                            associationPublicKey: e.publicKey,
                            ecdhPrivateKey: t.privateKey
                        }
                    }), E = e => {
                        e.wasClean ? f = {
                            __type: "disconnected"
                        } : i(new a(s.ERROR_SESSION_CLOSED,`The wallet session dropped unexpectedly (${e.code}: ${e.reason}).`,{
                            closeEvent: e
                        })),
                        g()
                    }
                    , I = e => c(this, void 0, void 0, function*() {
                        g(),
                        Date.now() - n >= N.timeoutMs ? i(new a(s.ERROR_SESSION_TIMEOUT,`Failed to connect to the wallet websocket at ${o}.`)) : (yield new Promise(e => {
                            let t = d();
                            w = window.setTimeout(e, t)
                        }
                        ),
                        C())
                    }), k = n => c(this, void 0, void 0, function*() {
                        let s = yield n.data.arrayBuffer();
                        switch (f.__type) {
                        case "connecting":
                            if (0 !== s.byteLength)
                                throw Error("Encountered unexpected message while connecting");
                            let a = yield _();
                            m.send((yield u(a.publicKey, r.privateKey))),
                            f = {
                                __type: "hello_req_sent",
                                associationPublicKey: r.publicKey,
                                ecdhPrivateKey: a.privateKey
                            };
                            break;
                        case "connected":
                            try {
                                let e = s.slice(0, 4)
                                  , t = R(e);
                                if (t !== p + 1)
                                    throw Error("Encrypted message has invalid sequence number");
                                p = t;
                                let n = yield S(s, f.sharedSecret)
                                  , r = v[n.id];
                                delete v[n.id],
                                r.resolve(n.result)
                            } catch (e) {
                                if (e instanceof l) {
                                    let t = v[e.jsonRpcMessageId];
                                    delete v[e.jsonRpcMessageId],
                                    t.reject(e)
                                } else
                                    throw e
                            }
                            break;
                        case "hello_req_sent":
                            {
                                if (0 === s.byteLength) {
                                    let e = yield _();
                                    m.send((yield u(e.publicKey, r.privateKey))),
                                    f = {
                                        __type: "hello_req_sent",
                                        associationPublicKey: r.publicKey,
                                        ecdhPrivateKey: e.privateKey
                                    };
                                    break
                                }
                                let n = yield A(s, f.associationPublicKey, f.ecdhPrivateKey)
                                  , a = s.slice(65)
                                  , o = 0 !== a.byteLength ? yield c(this, void 0, void 0, function*() {
                                    let e = R(a.slice(0, 4));
                                    if (e !== p + 1)
                                        throw Error("Encrypted message has invalid sequence number");
                                    return p = e,
                                    T(a, n)
                                }) : {
                                    protocol_version: "legacy"
                                };
                                f = {
                                    __type: "connected",
                                    sharedSecret: n,
                                    sessionProperties: o
                                };
                                let l = y(o.protocol_version, (e, t) => c(this, void 0, void 0, function*() {
                                    let r = h++;
                                    return m.send((yield M({
                                        id: r,
                                        jsonrpc: "2.0",
                                        method: e,
                                        params: null != t ? t : {}
                                    }, n))),
                                    new Promise( (t, n) => {
                                        v[r] = {
                                            resolve(r) {
                                                switch (e) {
                                                case "authorize":
                                                case "reauthorize":
                                                    {
                                                        let {wallet_uri_base: e} = r;
                                                        if (null != e)
                                                            try {
                                                                D(e)
                                                            } catch (e) {
                                                                n(e);
                                                                return
                                                            }
                                                    }
                                                }
                                                t(r)
                                            },
                                            reject: n
                                        }
                                    }
                                    )
                                }));
                                try {
                                    t((yield e(l)))
                                } catch (e) {
                                    i(e)
                                } finally {
                                    g(),
                                    m.close()
                                }
                            }
                        }
                    }), C = () => {
                        g && g(),
                        f = {
                            __type: "connecting",
                            associationKeypair: r
                        },
                        void 0 === n && (n = Date.now()),
                        (m = new WebSocket(o,[L])).addEventListener("open", b),
                        m.addEventListener("close", E),
                        m.addEventListener("error", I),
                        m.addEventListener("message", k),
                        g = () => {
                            window.clearTimeout(w),
                            m.removeEventListener("open", b),
                            m.removeEventListener("close", E),
                            m.removeEventListener("error", I),
                            m.removeEventListener("message", k)
                        }
                    }
                    ;
                    C()
                }
                )
            })
        }
        function W(e) {
            return c(this, void 0, void 0, function*() {
                let t, n, r, i, o;
                j();
                let h = yield w()
                  , p = `wss://${null == e ? void 0 : e.remoteHostAuthority}/reflect`
                  , f = ( () => {
                    let e = [...N.retryDelayScheduleMs];
                    return () => e.length > 1 ? e.shift() : e[0]
                }
                )()
                  , m = 1
                  , g = 0
                  , b = {
                    __type: "disconnected"
                }
                  , I = e => c(this, void 0, void 0, function*() {
                    var t;
                    return "base64" != n ? yield e.data.arrayBuffer() : (t = yield e.data,
                    new Uint8Array(window.atob(t).split("").map(e => e.charCodeAt(0)))).buffer
                })
                  , C = yield new Promise( (o, l) => {
                    let u, y = () => c(this, void 0, void 0, function*() {
                        if ("connecting" !== b.__type)
                            return void console.warn(`Expected adapter state to be \`connecting\` at the moment the websocket opens. Got \`${b.__type}\`.`);
                        n = r.protocol.includes(x) ? "base64" : "binary",
                        r.removeEventListener("open", y)
                    }), m = e => {
                        e.wasClean ? b = {
                            __type: "disconnected"
                        } : l(new a(s.ERROR_SESSION_CLOSED,`The wallet session dropped unexpectedly (${e.code}: ${e.reason}).`,{
                            closeEvent: e
                        })),
                        i()
                    }
                    , g = e => c(this, void 0, void 0, function*() {
                        i(),
                        Date.now() - t >= N.timeoutMs ? l(new a(s.ERROR_SESSION_TIMEOUT,`Failed to connect to the wallet websocket at ${p}.`)) : (yield new Promise(e => {
                            let t = f();
                            u = window.setTimeout(e, t)
                        }
                        ),
                        _())
                    }), w = t => c(this, void 0, void 0, function*() {
                        let n = yield I(t);
                        if ("connecting" === b.__type) {
                            if (0 == n.byteLength)
                                throw Error("Encountered unexpected message while connecting");
                            let t = function(e) {
                                let {value: t, offset: n} = function(e) {
                                    var t, n = new Uint8Array(e), r = e.byteLength, i = 0, s = 0;
                                    do {
                                        if (s >= r || s > 10)
                                            throw RangeError("Failed to decode varint");
                                        i |= (127 & (t = n[s++])) << 7 * s
                                    } while (t >= 128);
                                    return {
                                        value: i,
                                        offset: s
                                    }
                                }(e);
                                return new Uint8Array(e.slice(n, n + t))
                            }(n);
                            b = {
                                __type: "reflector_id_received",
                                reflectorId: t
                            };
                            let i = yield function(e, t, n, r, i=["v1"]) {
                                return c(this, void 0, void 0, function*() {
                                    let s = v((yield crypto.subtle.exportKey("raw", e)))
                                      , a = k("v1/associate/remote", r);
                                    return a.searchParams.set("association", E(s)),
                                    a.searchParams.set("reflector", `${t}`),
                                    a.searchParams.set("id", `${d(n, !0)}`),
                                    i.forEach(e => {
                                        a.searchParams.set("v", e)
                                    }
                                    ),
                                    a
                                })
                            }(h.publicKey, e.remoteHostAuthority, t, null == e ? void 0 : e.baseUri);
                            r.removeEventListener("message", w),
                            o(i)
                        }
                    }), _ = () => {
                        i && i(),
                        b = {
                            __type: "connecting",
                            associationKeypair: h
                        },
                        void 0 === t && (t = Date.now()),
                        (r = new WebSocket(p,[L, x])).addEventListener("open", y),
                        r.addEventListener("close", m),
                        r.addEventListener("error", g),
                        r.addEventListener("message", w),
                        i = () => {
                            window.clearTimeout(u),
                            r.removeEventListener("open", y),
                            r.removeEventListener("close", m),
                            r.removeEventListener("error", g),
                            r.removeEventListener("message", w)
                        }
                    }
                    ;
                    _()
                }
                )
                  , O = !1;
                return {
                    associationUrl: C,
                    close: () => {
                        r.close(),
                        o()
                    }
                    ,
                    wallet: new Promise( (e, t) => {
                        let p = {}
                          , f = i => c(this, void 0, void 0, function*() {
                            let s = yield I(i);
                            switch (b.__type) {
                            case "reflector_id_received":
                                if (0 !== s.byteLength)
                                    throw Error("Encountered unexpected message while awaiting reflection");
                                let a = yield _()
                                  , o = yield u(a.publicKey, h.privateKey);
                                "base64" == n ? r.send(d(o)) : r.send(o),
                                b = {
                                    __type: "hello_req_sent",
                                    associationPublicKey: h.publicKey,
                                    ecdhPrivateKey: a.privateKey
                                };
                                break;
                            case "connected":
                                try {
                                    let e = s.slice(0, 4)
                                      , t = R(e);
                                    if (t !== g + 1)
                                        throw Error("Encrypted message has invalid sequence number");
                                    g = t;
                                    let n = yield S(s, b.sharedSecret)
                                      , r = p[n.id];
                                    delete p[n.id],
                                    r.resolve(n.result)
                                } catch (e) {
                                    if (e instanceof l) {
                                        let t = p[e.jsonRpcMessageId];
                                        delete p[e.jsonRpcMessageId],
                                        t.reject(e)
                                    } else
                                        throw e
                                }
                                break;
                            case "hello_req_sent":
                                {
                                    let i = yield A(s, b.associationPublicKey, b.ecdhPrivateKey)
                                      , a = s.slice(65)
                                      , o = 0 !== a.byteLength ? yield c(this, void 0, void 0, function*() {
                                        let e = R(a.slice(0, 4));
                                        if (e !== g + 1)
                                            throw Error("Encrypted message has invalid sequence number");
                                        return g = e,
                                        T(a, i)
                                    }) : {
                                        protocol_version: "legacy"
                                    };
                                    b = {
                                        __type: "connected",
                                        sharedSecret: i,
                                        sessionProperties: o
                                    };
                                    let l = y(o.protocol_version, (e, t) => c(this, void 0, void 0, function*() {
                                        let s = m++
                                          , a = yield M({
                                            id: s,
                                            jsonrpc: "2.0",
                                            method: e,
                                            params: null != t ? t : {}
                                        }, i);
                                        return "base64" == n ? r.send(d(a)) : r.send(a),
                                        new Promise( (t, n) => {
                                            p[s] = {
                                                resolve(r) {
                                                    switch (e) {
                                                    case "authorize":
                                                    case "reauthorize":
                                                        {
                                                            let {wallet_uri_base: e} = r;
                                                            if (null != e)
                                                                try {
                                                                    D(e)
                                                                } catch (e) {
                                                                    n(e);
                                                                    return
                                                                }
                                                        }
                                                    }
                                                    t(r)
                                                },
                                                reject: n
                                            }
                                        }
                                        )
                                    }));
                                    O = !0;
                                    try {
                                        e(l)
                                    } catch (e) {
                                        t(e)
                                    }
                                }
                            }
                        });
                        r.addEventListener("message", f),
                        o = () => {
                            r.removeEventListener("message", f),
                            i(),
                            O || t(new a(s.ERROR_SESSION_CLOSED,"The wallet session was closed before connection.",{
                                closeEvent: new CloseEvent("socket was closed before connection")
                            }))
                        }
                    }
                    )
                }
            })
        }
    }
    ,
    74192: function(e) {
        e.exports = function() {
            "use strict";
            function e(e) {
                return Number.isInteger(e) && e >= 0
            }
            function t(e) {
                this.name = "ArgumentError",
                this.message = e
            }
            return function(n, r) {
                if (r = r || {},
                "function" != typeof n)
                    throw new t("fetch must be a function");
                if ("object" != typeof r)
                    throw new t("defaults must be an object");
                if (void 0 !== r.retries && !e(r.retries))
                    throw new t("retries must be a positive integer");
                if (void 0 !== r.retryDelay && !e(r.retryDelay) && "function" != typeof r.retryDelay)
                    throw new t("retryDelay must be a positive integer or a function returning a positive integer");
                if (void 0 !== r.retryOn && !Array.isArray(r.retryOn) && "function" != typeof r.retryOn)
                    throw new t("retryOn property expects an array or function");
                return r = Object.assign({
                    retries: 3,
                    retryDelay: 1e3,
                    retryOn: []
                }, r),
                function(i, s) {
                    var a = r.retries
                      , o = r.retryDelay
                      , l = r.retryOn;
                    if (s && void 0 !== s.retries)
                        if (e(s.retries))
                            a = s.retries;
                        else
                            throw new t("retries must be a positive integer");
                    if (s && void 0 !== s.retryDelay)
                        if (e(s.retryDelay) || "function" == typeof s.retryDelay)
                            o = s.retryDelay;
                        else
                            throw new t("retryDelay must be a positive integer or a function returning a positive integer");
                    if (s && s.retryOn)
                        if (Array.isArray(s.retryOn) || "function" == typeof s.retryOn)
                            l = s.retryOn;
                        else
                            throw new t("retryOn property expects an array or function");
                    return new Promise(function(e, t) {
                        var r = function(r) {
                            n("undefined" != typeof Request && i instanceof Request ? i.clone() : i, s).then(function(n) {
                                if (Array.isArray(l) && -1 === l.indexOf(n.status))
                                    e(n);
                                else if ("function" == typeof l)
                                    try {
                                        return Promise.resolve(l(r, null, n)).then(function(t) {
                                            t ? c(r, null, n) : e(n)
                                        }).catch(t)
                                    } catch (e) {
                                        t(e)
                                    }
                                else
                                    r < a ? c(r, null, n) : e(n)
                            }).catch(function(e) {
                                if ("function" == typeof l)
                                    try {
                                        Promise.resolve(l(r, e, null)).then(function(n) {
                                            n ? c(r, e, null) : t(e)
                                        }).catch(function(e) {
                                            t(e)
                                        })
                                    } catch (e) {
                                        t(e)
                                    }
                                else
                                    r < a ? c(r, e, null) : t(e)
                            })
                        };
                        function c(e, t, n) {
                            setTimeout(function() {
                                r(++e)
                            }, "function" == typeof o ? o(e, t, n) : o)
                        }
                        r(0)
                    }
                    )
                }
            }
        }()
    },
    76928: e => {
        e.exports = {
            style: {
                fontFamily: "'Geist Mono', 'Geist Mono Fallback'",
                fontStyle: "normal"
            },
            className: "__className_9a8899",
            variable: "__variable_9a8899"
        }
    }
    ,
    77024: (e, t, n) => {
        var r = n(91015).Buffer;
        let i = n(46586)
          , s = n(38138)
          , a = {
            type: "object",
            properties: {
                types: {
                    type: "object",
                    additionalProperties: {
                        type: "array",
                        items: {
                            type: "object",
                            properties: {
                                name: {
                                    type: "string"
                                },
                                type: {
                                    type: "string"
                                }
                            },
                            required: ["name", "type"]
                        }
                    }
                },
                primaryType: {
                    type: "string"
                },
                domain: {
                    type: "object"
                },
                message: {
                    type: "object"
                }
            },
            required: ["types", "primaryType", "domain", "message"]
        }
          , o = {
            encodeData(e, t, n, a=!0) {
                let o = ["bytes32"]
                  , l = [this.hashType(e, n)];
                if (a) {
                    let c = (e, t, o) => {
                        if (void 0 !== n[t])
                            return ["bytes32", null == o ? "0x0000000000000000000000000000000000000000000000000000000000000000" : i.keccak(this.encodeData(t, o, n, a))];
                        if (void 0 === o)
                            throw Error(`missing value for field ${e} of type ${t}`);
                        if ("bytes" === t)
                            return ["bytes32", i.keccak(o)];
                        if ("string" === t)
                            return "string" == typeof o && (o = r.from(o, "utf8")),
                            ["bytes32", i.keccak(o)];
                        if (t.lastIndexOf("]") === t.length - 1) {
                            let n = t.slice(0, t.lastIndexOf("["))
                              , r = o.map(t => c(e, n, t));
                            return ["bytes32", i.keccak(s.rawEncode(r.map( ([e]) => e), r.map( ([,e]) => e)))]
                        }
                        return [t, o]
                    }
                    ;
                    for (let r of n[e]) {
                        let[e,n] = c(r.name, r.type, t[r.name]);
                        o.push(e),
                        l.push(n)
                    }
                } else
                    for (let s of n[e]) {
                        let e = t[s.name];
                        if (void 0 !== e)
                            if ("bytes" === s.type)
                                o.push("bytes32"),
                                e = i.keccak(e),
                                l.push(e);
                            else if ("string" === s.type)
                                o.push("bytes32"),
                                "string" == typeof e && (e = r.from(e, "utf8")),
                                e = i.keccak(e),
                                l.push(e);
                            else if (void 0 !== n[s.type])
                                o.push("bytes32"),
                                e = i.keccak(this.encodeData(s.type, e, n, a)),
                                l.push(e);
                            else if (s.type.lastIndexOf("]") === s.type.length - 1)
                                throw Error("Arrays currently unimplemented in encodeData");
                            else
                                o.push(s.type),
                                l.push(e)
                    }
                return s.rawEncode(o, l)
            },
            encodeType(e, t) {
                let n = ""
                  , r = this.findTypeDependencies(e, t).filter(t => t !== e);
                for (let i of r = [e].concat(r.sort())) {
                    if (!t[i])
                        throw Error("No type definition specified: " + i);
                    n += i + "(" + t[i].map( ({name: e, type: t}) => t + " " + e).join(",") + ")"
                }
                return n
            },
            findTypeDependencies(e, t, n=[]) {
                if (e = e.match(/^\w*/)[0],
                n.includes(e) || void 0 === t[e])
                    return n;
                for (let r of (n.push(e),
                t[e]))
                    for (let e of this.findTypeDependencies(r.type, t, n))
                        n.includes(e) || n.push(e);
                return n
            },
            hashStruct(e, t, n, r=!0) {
                return i.keccak(this.encodeData(e, t, n, r))
            },
            hashType(e, t) {
                return i.keccak(this.encodeType(e, t))
            },
            sanitizeData(e) {
                let t = {};
                for (let n in a.properties)
                    e[n] && (t[n] = e[n]);
                return t.types && (t.types = Object.assign({
                    EIP712Domain: []
                }, t.types)),
                t
            },
            hash(e, t=!0) {
                let n = this.sanitizeData(e)
                  , s = [r.from("1901", "hex")];
                return s.push(this.hashStruct("EIP712Domain", n.domain, n.types, t)),
                "EIP712Domain" !== n.primaryType && s.push(this.hashStruct(n.primaryType, n.message, n.types, t)),
                i.keccak(r.concat(s))
            }
        };
        e.exports = {
            TYPED_MESSAGE_SCHEMA: a,
            TypedDataUtils: o,
            hashForSignTypedDataLegacy: function(e) {
                return function(e) {
                    let t = Error("Expect argument to be non-empty array");
                    if ("object" != typeof e || !e.length)
                        throw t;
                    let n = e.map(function(e) {
                        return "bytes" === e.type ? i.toBuffer(e.value) : e.value
                    })
                      , r = e.map(function(e) {
                        return e.type
                    })
                      , a = e.map(function(e) {
                        if (!e.name)
                            throw t;
                        return e.type + " " + e.name
                    });
                    return s.soliditySHA3(["bytes32", "bytes32"], [s.soliditySHA3(Array(e.length).fill("string"), a), s.soliditySHA3(r, n)])
                }(e.data)
            },
            hashForSignTypedData_v3: function(e) {
                return o.hash(e.data, !1)
            },
            hashForSignTypedData_v4: function(e) {
                return o.hash(e.data)
            }
        }
    }
    ,
    79323: (e, t, n) => {
        e.exports = n(22482)("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
    }
    ,
    79482: (e, t, n) => {
        "use strict";
        function r() {
            for (var e, t, n = 0, r = ""; n < arguments.length; )
                (e = arguments[n++]) && (t = function e(t) {
                    var n, r, i = "";
                    if ("string" == typeof t || "number" == typeof t)
                        i += t;
                    else if ("object" == typeof t)
                        if (Array.isArray(t))
                            for (n = 0; n < t.length; n++)
                                t[n] && (r = e(t[n])) && (i && (i += " "),
                                i += r);
                        else
                            for (n in t)
                                t[n] && (i && (i += " "),
                                i += n);
                    return i
                }(e)) && (r && (r += " "),
                r += t);
            return r
        }
        n.d(t, {
            $: () => r
        })
    }
    ,
    81047: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "M10.5 1.5H8.25A2.25 2.25 0 0 0 6 3.75v16.5a2.25 2.25 0 0 0 2.25 2.25h7.5A2.25 2.25 0 0 0 18 20.25V3.75a2.25 2.25 0 0 0-2.25-2.25H13.5m-3 0V3h3V1.5m-3 0h3m-3 18.75h3"
            }))
        })
    }
    ,
    81564: (e, t, n) => {
        "use strict";
        function r() {
            let e = new Set
              , t = []
              , n = () => (function(e) {
                if ("undefined" == typeof window)
                    return;
                let t = t => e(t.detail);
                return window.addEventListener("eip6963:announceProvider", t),
                window.dispatchEvent(new CustomEvent("eip6963:requestProvider")),
                () => window.removeEventListener("eip6963:announceProvider", t)
            }
            )(n => {
                t.some( ({info: e}) => e.uuid === n.info.uuid) || (t = [...t, n],
                e.forEach(e => e(t, {
                    added: [n]
                })))
            }
            )
              , r = n();
            return {
                _listeners: () => e,
                clear() {
                    e.forEach(e => e([], {
                        removed: [...t]
                    })),
                    t = []
                },
                destroy() {
                    this.clear(),
                    e.clear(),
                    r?.()
                },
                findProvider: ({rdns: e}) => t.find(t => t.info.rdns === e),
                getProviders: () => t,
                reset() {
                    this.clear(),
                    r?.(),
                    r = n()
                },
                subscribe: (n, {emitImmediately: r}={}) => (e.add(n),
                r && n(t, {
                    added: t
                }),
                () => e.delete(n))
            }
        }
        n.d(t, {
            y: () => r
        })
    }
    ,
    81650: function(e, t, n) {
        "use strict";
        var r = this && this.__createBinding || (Object.create ? function(e, t, n, r) {
            void 0 === r && (r = n);
            var i = Object.getOwnPropertyDescriptor(t, n);
            (!i || ("get"in i ? !t.__esModule : i.writable || i.configurable)) && (i = {
                enumerable: !0,
                get: function() {
                    return t[n]
                }
            }),
            Object.defineProperty(e, r, i)
        }
        : function(e, t, n, r) {
            void 0 === r && (r = n),
            e[r] = t[n]
        }
        )
          , i = this && this.__exportStar || function(e, t) {
            for (var n in e)
                "default" === n || Object.prototype.hasOwnProperty.call(t, n) || r(t, e, n)
        }
        ;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        i(n(6117), t),
        i(n(68429), t),
        i(n(99551), t),
        i(n(6727), t),
        i(n(51366), t),
        i(n(93627), t)
    },
    81845: (e, t, n) => {
        "use strict";
        n.d(t, {
            lG: () => ea,
            Lj: () => es
        });
        var r = n(26432)
          , i = n(30924)
          , s = n(50748);
        function a(e, t, n, i) {
            let a = (0,
            s.Y)(n);
            (0,
            r.useEffect)( () => {
                function n(e) {
                    a.current(e)
                }
                return (e = null != e ? e : window).addEventListener(t, n, i),
                () => e.removeEventListener(t, n, i)
            }
            , [e, t, i])
        }
        var o = n(46145)
          , l = n(5049)
          , c = n(16531)
          , d = n(76603)
          , u = n(97186)
          , h = n(83627)
          , p = n(47086)
          , f = n(89478)
          , y = (e => (e[e.None = 1] = "None",
        e[e.Focusable = 2] = "Focusable",
        e[e.Hidden = 4] = "Hidden",
        e))(y || {});
        let m = (0,
        f.FX)(function(e, t) {
            var n;
            let {features: r=1, ...i} = e
              , s = {
                ref: t,
                "aria-hidden": (2 & r) == 2 || (null != (n = i["aria-hidden"]) ? n : void 0),
                hidden: (4 & r) == 4 || void 0,
                style: {
                    position: "fixed",
                    top: 1,
                    left: 1,
                    width: 1,
                    height: 0,
                    padding: 0,
                    margin: -1,
                    overflow: "hidden",
                    clip: "rect(0, 0, 0, 0)",
                    whiteSpace: "nowrap",
                    borderWidth: "0",
                    ...(4 & r) == 4 && (2 & r) != 2 && {
                        display: "none"
                    }
                }
            };
            return (0,
            f.Ci)()({
                ourProps: s,
                theirProps: i,
                slot: {},
                defaultTag: "span",
                name: "Hidden"
            })
        });
        var g = n(42138)
          , w = n(34765);
        let _ = (0,
        r.createContext)(null);
        function v(e) {
            let {children: t, node: n} = e
              , [i,s] = (0,
            r.useState)(null)
              , a = b(null != n ? n : i);
            return r.createElement(_.Provider, {
                value: a
            }, t, null === a && r.createElement(m, {
                features: y.Hidden,
                ref: e => {
                    var t, n;
                    if (e) {
                        for (let r of null != (n = null == (t = (0,
                        w.TW)(e)) ? void 0 : t.querySelectorAll("html > *, body > *")) ? n : [])
                            if (r !== document.body && r !== document.head && g.vq(r) && null != r && r.contains(e)) {
                                s(r);
                                break
                            }
                    }
                }
            }))
        }
        function b() {
            var e;
            let t = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : null;
            return null != (e = (0,
            r.useContext)(_)) ? e : t
        }
        var E = n(8573)
          , I = n(297)
          , k = n(56847)
          , M = n(21917);
        let S = (0,
        r.createContext)( () => {}
        );
        function A(e) {
            let {value: t, children: n} = e;
            return r.createElement(S.Provider, {
                value: t
            }, n)
        }
        var T = n(15377)
          , C = n(41161)
          , O = n(61445)
          , N = n(26935)
          , L = n(4515)
          , x = n(52116)
          , j = n(17394)
          , D = n(52718)
          , R = n(41603)
          , P = n(41290)
          , W = (e => (e[e.Forwards = 0] = "Forwards",
        e[e.Backwards = 1] = "Backwards",
        e))(W || {});
        function U(e, t) {
            let n = (0,
            r.useRef)([])
              , i = (0,
            l._)(e);
            (0,
            r.useEffect)( () => {
                let e = [...n.current];
                for (let[r,s] of t.entries())
                    if (n.current[r] !== s) {
                        let r = i(t, e);
                        return n.current = t,
                        r
                    }
            }
            , [i, ...t])
        }
        var z = n(17458);
        let q = [];
        !function(e) {
            function t() {
                "loading" !== document.readyState && (e(),
                document.removeEventListener("DOMContentLoaded", t))
            }
            "undefined" != typeof window && "undefined" != typeof document && (document.addEventListener("DOMContentLoaded", t),
            t())
        }( () => {
            function e(e) {
                if (!g.Lk(e.target) || e.target === document.body || q[0] === e.target)
                    return;
                let t = e.target;
                t = t.closest(z.Uo),
                q.unshift(null != t ? t : e.target),
                (q = q.filter(e => null != e && e.isConnected)).splice(10)
            }
            window.addEventListener("click", e, {
                capture: !0
            }),
            window.addEventListener("mousedown", e, {
                capture: !0
            }),
            window.addEventListener("focus", e, {
                capture: !0
            }),
            document.body.addEventListener("click", e, {
                capture: !0
            }),
            document.body.addEventListener("mousedown", e, {
                capture: !0
            }),
            document.body.addEventListener("focus", e, {
                capture: !0
            })
        }
        );
        var F = n(21020);
        function B(e) {
            if (!e)
                return new Set;
            if ("function" == typeof e)
                return new Set(e());
            let t = new Set;
            for (let n of e.current)
                g.vq(n.current) && t.add(n.current);
            return t
        }
        var H = (e => (e[e.None = 0] = "None",
        e[e.InitialFocus = 1] = "InitialFocus",
        e[e.TabLock = 2] = "TabLock",
        e[e.FocusLock = 4] = "FocusLock",
        e[e.RestoreFocus = 8] = "RestoreFocus",
        e[e.AutoFocus = 16] = "AutoFocus",
        e))(H || {});
        let K = Object.assign((0,
        f.FX)(function(e, t) {
            let n, i = (0,
            r.useRef)(null), s = (0,
            M.P)(i, t), {initialFocus: c, initialFocusFallback: d, containers: u, features: h=15, ..._} = e;
            (0,
            I.g)() || (h = 0);
            let v = (0,
            p.g)(i.current);
            !function(e, t) {
                let {ownerDocument: n} = t
                  , i = !!(8 & e)
                  , s = function() {
                    let e = !(arguments.length > 0) || void 0 === arguments[0] || arguments[0]
                      , t = (0,
                    r.useRef)(q.slice());
                    return U( (e, n) => {
                        let[r] = e
                          , [i] = n;
                        !0 === i && !1 === r && (0,
                        F._)( () => {
                            t.current.splice(0)
                        }
                        ),
                        !1 === i && !0 === r && (t.current = q.slice())
                    }
                    , [e, q, t]),
                    (0,
                    l._)( () => {
                        var e;
                        return null != (e = t.current.find(e => null != e && e.isConnected)) ? e : null
                    }
                    )
                }(i);
                U( () => {
                    i || (0,
                    w.X7)(null == n ? void 0 : n.body) && (0,
                    z.pW)(s())
                }
                , [i]),
                (0,
                R.X)( () => {
                    i && (0,
                    z.pW)(s())
                }
                )
            }(h, {
                ownerDocument: v
            });
            let b = function(e, t) {
                let {ownerDocument: n, container: i, initialFocus: s, initialFocusFallback: a} = t
                  , l = (0,
                r.useRef)(null)
                  , c = (0,
                o.S)(!!(1 & e), "focus-trap#initial-focus")
                  , d = (0,
                D.a)();
                return U( () => {
                    if (0 === e)
                        return;
                    if (!c) {
                        null != a && a.current && (0,
                        z.pW)(a.current);
                        return
                    }
                    let t = i.current;
                    t && (0,
                    F._)( () => {
                        if (!d.current)
                            return;
                        let r = null == n ? void 0 : n.activeElement;
                        if (null != s && s.current) {
                            if ((null == s ? void 0 : s.current) === r) {
                                l.current = r;
                                return
                            }
                        } else if (t.contains(r)) {
                            l.current = r;
                            return
                        }
                        if (null != s && s.current)
                            (0,
                            z.pW)(s.current);
                        else {
                            if (16 & e) {
                                if ((0,
                                z.CU)(t, z.BD.First | z.BD.AutoFocus) !== z.Me.Error)
                                    return
                            } else if ((0,
                            z.CU)(t, z.BD.First) !== z.Me.Error)
                                return;
                            if (null != a && a.current && ((0,
                            z.pW)(a.current),
                            (null == n ? void 0 : n.activeElement) === a.current))
                                return;
                            console.warn("There are no focusable elements inside the <FocusTrap />")
                        }
                        l.current = null == n ? void 0 : n.activeElement
                    }
                    )
                }
                , [a, c, e]),
                l
            }(h, {
                ownerDocument: v,
                container: i,
                initialFocus: c,
                initialFocusFallback: d
            });
            !function(e, t) {
                let {ownerDocument: n, container: r, containers: i, previousActiveElement: s} = t
                  , o = (0,
                D.a)()
                  , l = !!(4 & e);
                a(null == n ? void 0 : n.defaultView, "focus", e => {
                    if (!l || !o.current)
                        return;
                    let t = B(i);
                    g.sb(r.current) && t.add(r.current);
                    let n = s.current;
                    if (!n)
                        return;
                    let a = e.target;
                    g.sb(a) ? Q(t, a) ? (s.current = a,
                    (0,
                    z.pW)(a)) : (e.preventDefault(),
                    e.stopPropagation(),
                    (0,
                    z.pW)(n)) : (0,
                    z.pW)(s.current)
                }
                , !0)
            }(h, {
                ownerDocument: v,
                container: i,
                containers: u,
                previousActiveElement: b
            });
            let E = (n = (0,
            r.useRef)(0),
            (0,
            P.M)(!0, "keydown", e => {
                "Tab" === e.key && (n.current = +!!e.shiftKey)
            }
            , !0),
            n)
              , k = (0,
            l._)(e => {
                if (!g.sb(i.current))
                    return;
                let t = i.current;
                (0,
                L.Y)(E.current, {
                    [W.Forwards]: () => {
                        (0,
                        z.CU)(t, z.BD.First, {
                            skipElements: [e.relatedTarget, d]
                        })
                    }
                    ,
                    [W.Backwards]: () => {
                        (0,
                        z.CU)(t, z.BD.Last, {
                            skipElements: [e.relatedTarget, d]
                        })
                    }
                })
            }
            )
              , S = (0,
            o.S)(!!(2 & h), "focus-trap#tab-lock")
              , A = (0,
            j.L)()
              , T = (0,
            r.useRef)(!1)
              , C = (0,
            f.Ci)();
            return r.createElement(r.Fragment, null, S && r.createElement(m, {
                as: "button",
                type: "button",
                "data-headlessui-focus-guard": !0,
                onFocus: k,
                features: y.Focusable
            }), C({
                ourProps: {
                    ref: s,
                    onKeyDown(e) {
                        "Tab" == e.key && (T.current = !0,
                        A.requestAnimationFrame( () => {
                            T.current = !1
                        }
                        ))
                    },
                    onBlur(e) {
                        if (!(4 & h))
                            return;
                        let t = B(u);
                        g.sb(i.current) && t.add(i.current);
                        let n = e.relatedTarget;
                        g.Lk(n) && "true" !== n.dataset.headlessuiFocusGuard && (Q(t, n) || (T.current ? (0,
                        z.CU)(i.current, (0,
                        L.Y)(E.current, {
                            [W.Forwards]: () => z.BD.Next,
                            [W.Backwards]: () => z.BD.Previous
                        }) | z.BD.WrapAround, {
                            relativeTo: e.target
                        }) : g.Lk(e.target) && (0,
                        z.pW)(e.target)))
                    }
                },
                theirProps: _,
                defaultTag: "div",
                name: "FocusTrap"
            }), S && r.createElement(m, {
                as: "button",
                type: "button",
                "data-headlessui-focus-guard": !0,
                onFocus: k,
                features: y.Focusable
            }))
        }), {
            features: H
        });
        function Q(e, t) {
            for (let n of e)
                if (n.contains(t))
                    return !0;
            return !1
        }
        var V = n(41902)
          , Y = n(16248)
          , G = (e => (e[e.Open = 0] = "Open",
        e[e.Closed = 1] = "Closed",
        e))(G || {})
          , Z = (e => (e[e.SetTitleId = 0] = "SetTitleId",
        e))(Z || {});
        let $ = {
            0: (e, t) => e.titleId === t.id ? e : {
                ...e,
                titleId: t.id
            }
        }
          , J = (0,
        r.createContext)(null);
        function X(e) {
            let t = (0,
            r.useContext)(J);
            if (null === t) {
                let t = Error("<".concat(e, " /> is missing a parent <Dialog /> component."));
                throw Error.captureStackTrace && Error.captureStackTrace(t, X),
                t
            }
            return t
        }
        function ee(e, t) {
            return (0,
            L.Y)(t.type, $, e, t)
        }
        J.displayName = "DialogContext";
        let et = (0,
        f.FX)(function(e, t) {
            let n = (0,
            r.useId)()
              , {id: s="headlessui-dialog-".concat(n), open: y, onClose: m, initialFocus: _, role: v="dialog", autoFocus: S=!0, __demoMode: L=!1, unmount: j=!1, ...D} = e
              , R = (0,
            r.useRef)(!1);
            v = "dialog" === v || "alertdialog" === v ? v : (R.current || (R.current = !0,
            console.warn("Invalid role [".concat(v, "] passed to <Dialog />. Only `dialog` and and `alertdialog` are supported. Using `dialog` instead."))),
            "dialog");
            let P = (0,
            T.O_)();
            void 0 === y && null !== P && (y = (P & T.Uw.Open) === T.Uw.Open);
            let W = (0,
            r.useRef)(null)
              , U = (0,
            M.P)(W, t)
              , z = (0,
            p.g)(W.current)
              , q = +!y
              , [F,B] = (0,
            r.useReducer)(ee, {
                titleId: null,
                descriptionId: null,
                panelRef: (0,
                r.createRef)()
            })
              , Q = (0,
            l._)( () => m(!1))
              , Y = (0,
            l._)(e => B({
                type: 0,
                id: e
            }))
              , G = !!(0,
            I.g)() && 0 === q
              , [Z,$] = (0,
            V.k2)()
              , X = b()
              , {resolveContainers: et} = function() {
                let {defaultContainers: e=[], portals: t, mainTreeNode: n} = arguments.length > 0 && void 0 !== arguments[0] ? arguments[0] : {}
                  , r = (0,
                l._)( () => {
                    var r, i;
                    let s = (0,
                    w.TW)(n)
                      , a = [];
                    for (let t of e)
                        null !== t && (g.vq(t) ? a.push(t) : "current"in t && g.vq(t.current) && a.push(t.current));
                    if (null != t && t.current)
                        for (let e of t.current)
                            a.push(e);
                    for (let e of null != (r = null == s ? void 0 : s.querySelectorAll("html > *, body > *")) ? r : [])
                        e !== document.body && e !== document.head && g.vq(e) && "headlessui-portal-root" !== e.id && (n && (e.contains(n) || e.contains(null == (i = null == n ? void 0 : n.getRootNode()) ? void 0 : i.host)) || a.some(t => e.contains(t)) || a.push(e));
                    return a
                }
                );
                return {
                    resolveContainers: r,
                    contains: (0,
                    l._)(e => r().some(t => t.contains(e)))
                }
            }({
                mainTreeNode: X,
                portals: Z,
                defaultContainers: [{
                    get current() {
                        var ei;
                        return null != (ei = F.panelRef.current) ? ei : W.current
                    }
                }]
            })
              , es = null !== P && (P & T.Uw.Closing) === T.Uw.Closing;
            (0,
            c.v)(!L && !es && G, {
                allowed: (0,
                l._)( () => {
                    var e, t;
                    return [null != (t = null == (e = W.current) ? void 0 : e.closest("[data-headlessui-portal]")) ? t : null]
                }
                ),
                disallowed: (0,
                l._)( () => {
                    var e;
                    return [null != (e = null == X ? void 0 : X.closest("body > *:not(#headlessui-portal-root)")) ? e : null]
                }
                )
            });
            let ea = O.D.get(null);
            (0,
            d.s)( () => {
                if (G)
                    return ea.actions.push(s),
                    () => ea.actions.pop(s)
            }
            , [ea, s, G]);
            let eo = (0,
            N.y)(ea, (0,
            r.useCallback)(e => ea.selectors.isTop(e, s), [ea, s]));
            (0,
            h.j)(eo, et, e => {
                e.preventDefault(),
                Q()
            }
            ),
            function(e) {
                let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : "undefined" != typeof document ? document.defaultView : null
                  , n = arguments.length > 2 ? arguments[2] : void 0
                  , r = (0,
                o.S)(e, "escape");
                a(t, "keydown", e => {
                    r && (e.defaultPrevented || e.key === i.D.Escape && n(e))
                }
                )
            }(eo, null == z ? void 0 : z.defaultView, e => {
                e.preventDefault(),
                e.stopPropagation(),
                document.activeElement && "blur"in document.activeElement && "function" == typeof document.activeElement.blur && document.activeElement.blur(),
                Q()
            }
            ),
            (0,
            E.K)(!L && !es && G, z, et),
            (0,
            u.O)(G, W, Q);
            let[el,ec] = (0,
            x.rU)()
              , ed = (0,
            r.useMemo)( () => [{
                dialogState: q,
                close: Q,
                setTitleId: Y,
                unmount: j
            }, F], [q, Q, Y, j, F])
              , eu = (0,
            k._)({
                open: 0 === q
            })
              , eh = {
                ref: U,
                id: s,
                role: v,
                tabIndex: -1,
                "aria-modal": L ? void 0 : 0 === q || void 0,
                "aria-labelledby": F.titleId,
                "aria-describedby": el,
                unmount: j
            }
              , ep = !function() {
                var e;
                let[t] = (0,
                r.useState)( () => "undefined" != typeof window && "function" == typeof window.matchMedia ? window.matchMedia("(pointer: coarse)") : null)
                  , [n,i] = (0,
                r.useState)(null != (e = null == t ? void 0 : t.matches) && e);
                return (0,
                d.s)( () => {
                    if (t)
                        return t.addEventListener("change", e),
                        () => t.removeEventListener("change", e);
                    function e(e) {
                        i(e.matches)
                    }
                }
                , [t]),
                n
            }()
              , ef = H.None;
            G && !L && (ef |= H.RestoreFocus,
            ef |= H.TabLock,
            S && (ef |= H.AutoFocus),
            ep && (ef |= H.InitialFocus));
            let ey = (0,
            f.Ci)();
            return r.createElement(T.$x, null, r.createElement(C.a, {
                force: !0
            }, r.createElement(V.ZL, null, r.createElement(J.Provider, {
                value: ed
            }, r.createElement(V.Ee, {
                target: W
            }, r.createElement(C.a, {
                force: !1
            }, r.createElement(ec, {
                slot: eu
            }, r.createElement($, null, r.createElement(K, {
                initialFocus: _,
                initialFocusFallback: W,
                containers: et,
                features: ef
            }, r.createElement(A, {
                value: Q
            }, ey({
                ourProps: eh,
                theirProps: D,
                slot: eu,
                defaultTag: en,
                features: er,
                visible: 0 === q,
                name: "Dialog"
            })))))))))))
        })
          , en = "div"
          , er = f.Ac.RenderStrategy | f.Ac.Static
          , ei = (0,
        f.FX)(function(e, t) {
            let {transition: n=!1, open: i, ...s} = e
              , a = (0,
            T.O_)()
              , o = e.hasOwnProperty("open") || null !== a
              , l = e.hasOwnProperty("onClose");
            if (!o && !l)
                throw Error("You have to provide an `open` and an `onClose` prop to the `Dialog` component.");
            if (!o)
                throw Error("You provided an `onClose` prop to the `Dialog`, but forgot an `open` prop.");
            if (!l)
                throw Error("You provided an `open` prop to the `Dialog`, but forgot an `onClose` prop.");
            if (!a && "boolean" != typeof e.open)
                throw Error("You provided an `open` prop to the `Dialog`, but the value is not a boolean. Received: ".concat(e.open));
            if ("function" != typeof e.onClose)
                throw Error("You provided an `onClose` prop to the `Dialog`, but the value is not a function. Received: ".concat(e.onClose));
            return (void 0 !== i || n) && !s.static ? r.createElement(v, null, r.createElement(Y.e, {
                show: i,
                transition: n,
                unmount: s.unmount
            }, r.createElement(et, {
                ref: t,
                ...s
            }))) : r.createElement(v, null, r.createElement(et, {
                ref: t,
                open: i,
                ...s
            }))
        })
          , es = (0,
        f.FX)(function(e, t) {
            let n = (0,
            r.useId)()
              , {id: i="headlessui-dialog-panel-".concat(n), transition: s=!1, ...a} = e
              , [{dialogState: o, unmount: c},d] = X("Dialog.Panel")
              , u = (0,
            M.P)(t, d.panelRef)
              , h = (0,
            k._)({
                open: 0 === o
            })
              , p = (0,
            l._)(e => {
                e.stopPropagation()
            }
            )
              , y = s ? Y._ : r.Fragment
              , m = (0,
            f.Ci)();
            return r.createElement(y, {
                ...s ? {
                    unmount: c
                } : {}
            }, m({
                ourProps: {
                    ref: u,
                    id: i,
                    onClick: p
                },
                theirProps: a,
                slot: h,
                defaultTag: "div",
                name: "Dialog.Panel"
            }))
        })
          , ea = Object.assign(ei, {
            Panel: es,
            Title: ((0,
            f.FX)(function(e, t) {
                let {transition: n=!1, ...i} = e
                  , [{dialogState: s, unmount: a}] = X("Dialog.Backdrop")
                  , o = (0,
                k._)({
                    open: 0 === s
                })
                  , l = n ? Y._ : r.Fragment
                  , c = (0,
                f.Ci)();
                return r.createElement(l, {
                    ...n ? {
                        unmount: a
                    } : {}
                }, c({
                    ourProps: {
                        ref: t,
                        "aria-hidden": !0
                    },
                    theirProps: i,
                    slot: o,
                    defaultTag: "div",
                    name: "Dialog.Backdrop"
                }))
            }),
            (0,
            f.FX)(function(e, t) {
                let n = (0,
                r.useId)()
                  , {id: i="headlessui-dialog-title-".concat(n), ...s} = e
                  , [{dialogState: a, setTitleId: o}] = X("Dialog.Title")
                  , l = (0,
                M.P)(t);
                (0,
                r.useEffect)( () => (o(i),
                () => o(null)), [i, o]);
                let c = (0,
                k._)({
                    open: 0 === a
                });
                return (0,
                f.Ci)()({
                    ourProps: {
                        ref: l,
                        id: i
                    },
                    theirProps: s,
                    slot: c,
                    defaultTag: "h2",
                    name: "Dialog.Title"
                })
            })),
            Description: x.VY
        })
    }
    ,
    83597: (e, t, n) => {
        "use strict";
        n.d(t, {
            CE: () => r,
            g4: () => a,
            re: () => s,
            sE: () => i
        });
        let r = "solana:mainnet"
          , i = "solana:devnet"
          , s = "solana:testnet"
          , a = "solana:localnet"
    }
    ,
    87939: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => r
        });
        let r = (0,
        n(44074).A)("Trash2", [["path", {
            d: "M3 6h18",
            key: "d0wm0j"
        }], ["path", {
            d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6",
            key: "4alrt4"
        }], ["path", {
            d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2",
            key: "v07s0e"
        }], ["line", {
            x1: "10",
            x2: "10",
            y1: "11",
            y2: "17",
            key: "1uufr5"
        }], ["line", {
            x1: "14",
            x2: "14",
            y1: "11",
            y2: "17",
            key: "xtxkd"
        }]])
    }
    ,
    89037: (e, t, n) => {
        "use strict";
        function r(e) {
            return "version"in e
        }
        n.d(t, {
            Y: () => r
        })
    }
    ,
    91245: (e, t, n) => {
        "use strict";
        n.d(t, {
            h: () => o
        });
        var r = n(50901)
          , i = n(72286)
          , s = n(17972)
          , a = n(40476);
        class o extends r.DE {
            constructor(e={}) {
                super(),
                this.name = "Backpack",
                this.url = "https://backpack.app",
                this.icon = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAbvSURBVHgB7Z1dUtxGEMf/LZH3fU0V4PUJQg4QVj5BnBOAT2BzAsMJAicwPoHJCRDrAxifgLVxVV73ObDqdEtsjKn4C8+0NDv9e7AxprRC85uvnp4RYYW5qKpxCVTcYKsgfiDfGjMwIsZIvh7d/lkmzAiYy5fzhultyZhdlagf1vU5VhjCiiGFXq01zYSJdqWgx/hB5AHN5I/6iuilyFBjxVgZAdqCZ34ORoVIqAzSOhxsvq6PsSIkL4A281LwL2IW/F1UhLKgRz/X9QyJUyBhuuae31gWviLjiPF1wxeX29vPkTjJtgAftrd3GHSMnmHw4eZ0uodESVKAoRT+kpQlSE6Ats/XZv/ONK5vZHC49+B1fYjESG4MUDKfYmCFr0ic4fmHqtpCYiQlgA66QsztIzFi5j+RGMl0AXebfgn0aOTuvGG8owIarZsXOj3ronlRuEYnn84CJLo4Lgi/QL/H/LHmy/RwI6GA0RoS4acFHi8kGieFXS/QhmijFfQXmH3uPy5lSkoLbIkYlfyzhuM4juM4juM4juMMj6TzATQ4JH9tlRqFk8BM2aV9RWHB9K5kzK/KLui0KqliSQmgBa4BIS54cpMD0OeawFye3jk19JdKkWq62OAFkEIfrTXNUxBV1okf38Ot3MGjlFqHwQrQZvQ22Cfw7xjg6t8XkZaBGzpKIXdwcAJojZeCP5SC30HipJBEOigBZLn3qdzSPlKr8V9hyEmkgxCgj8zefuD9jen0AAOidwE0i6ZhfjXgRI+gDK016DUjqE3ubPhNLoWvaDLJouHToaSP9SbA0DJ7LekyiviNPgP0TC9dQM6FfxeZ7eyuT6cv0RPmAmjTx11uXx/MiegEDd425cfcwWV+H4O3+uiO+pTAVIA2uMN8av6QiWr5TQ++JVlTc/tEiF3jOMScZGC43kME0VSA95PJhWXhM+Gt1Phn98nStZa1r9mB2SDQPqefjhayfnDfFG2J5882z84eynVM5u3thlONhRhj0gLc5PRfwAw62JjW+wjE5Xa1L0VkshO4kXt/EPDev4ZJCyBRvlcwggjHG4EfYHc9OoIBBWy3mEUX4H1V7Ur7ZvILaT8qy7FRduleF9jXc4RggOUWs/gtANs0nYquvMXaMaTXlQHlE1ggayLvf5OKY0DUMYDWfmpsBjZa+9enOmiLy+VkcmqxaNW2ZgX9GnsLXNQWoGj4KYzQ2g8LyG5WUDR4hshEE6CN+AFmg5lFiRMYcI0uKRQGyIAwegWKJkBjYO8tzq12C7efQ7CK2I00MomIxOsCiCcwQhaW3sEQ6W7sPi/yIDqKAHp8m2nIF7COoc9ghQw4NU8SkYgiQCmLKXCCUSziPc84XYBh83/DSiWR3qUo2tT4ONdGYDTub73cSzD/PNt0rojdQHAByoXxw0E7XfoFhsjnRduD+DnWIkkXXACJl1cwRoMmf3cbRaOjLRzDXnKZVj9GBIILUJBtbVzyj9HAU19AgR6I9VzDtwCgMXpAo2Yxp0v/Ybi49ennJtIFEPMY/TCKHTvv+aTSUQzBgwrQ92YHbQVi3UN3GAVZhrf/jzECE1SAq/7n4yOJ074KPSBcJoii598vxgwrqAByg70HZJZbr0JJ0G5XZz5Z1e1rYccA5TAicqEk0O5ECl/3LvYys7mLTLHHCEzS7wz6Esv3+nyYTF58rwha63XAl8PG1aCnhesWq6EdOcKM3WvmXRHh+Gvv/tNVTJlJPC4a3RVEK72+sCSZ4+J/FBVhTUS43J7gJqFjrnl33A3sxtCa3nAWhX6bbAT4hJugCsNZ2TGA8224AJnjAmSOC5A5LkDmuACZ4wJkjguQOS5A5rgAmeMCZI4LkDkuQOa4AJnjAmSOC5A5LkDmuACZ4wJkjguQOWEFYJvz85xwBBWgKM1P68oKKsI/36ACdC9nsDlWPTsIJ5t1Hfw01OBjgI1p/YwLegIibw0CwESz9gUYZ2d/wHEcx3Ecx3Ecx3Ecx3HuS5QjfdrXxTHv3JzEkd2xKwHR9xPNuKGjzdf1MSIQXAA9XUsuuw8nKPpK3PWzs+AvrgwqgP1LojOjoEf3fRv6Zy+JgBSLOGfaOx1NE/6o+rCrgeT9fWp4SljmuACZ4wJkjguQOS5A5rgAmeMCZI4LkDkuQOa4AJnjAmSOC5A5LkDmuACZ4wJkjguQOS5A5rgAmeMCZI4LkDkuQOa4AJnj5wRmTlABqHQBohKhggUVYAEEP8fO+UiMgziDCvCwrnU3aw0nOATMQu8LVIIPAq+JdAerdwWBaQ/fjEBwAaQVmMnN7sEJCB3EqP3tlRGJy6qqmPkFMcZw7sucmfZiHQ6hRBNgSXdaCHbA7KeFfBvz9pxlxtl1gcN2XBWRfwHK959XFRG6AgAAAABJRU5ErkJggg==",
                this.supportedTransactionVersions = null,
                this._readyState = "undefined" == typeof window || "undefined" == typeof document ? i.Ok.Unsupported : i.Ok.NotDetected,
                this._disconnected = () => {
                    let e = this._wallet;
                    e && (e.off("disconnect", this._disconnected),
                    this._wallet = null,
                    this._publicKey = null,
                    this.emit("error", new s.PQ),
                    this.emit("disconnect"))
                }
                ,
                this._connecting = !1,
                this._wallet = null,
                this._publicKey = null,
                this._readyState !== i.Ok.Unsupported && (0,
                i.qG)( () => !!window.backpack?.isBackpack && (this._readyState = i.Ok.Installed,
                this.emit("readyStateChange", this._readyState),
                !0))
            }
            get publicKey() {
                return this._publicKey
            }
            get connecting() {
                return this._connecting
            }
            get connected() {
                return !!this._wallet?.isConnected
            }
            get readyState() {
                return this._readyState
            }
            async connect() {
                try {
                    let e;
                    if (this.connected || this.connecting)
                        return;
                    if (this._readyState !== i.Ok.Installed)
                        throw new s.AE;
                    this._connecting = !0;
                    let t = window.backpack;
                    try {
                        await t.connect()
                    } catch (e) {
                        throw new s.Y6(e?.message,e)
                    }
                    if (!t.publicKey)
                        throw new s.fk;
                    try {
                        e = new a.PublicKey(t.publicKey.toBytes())
                    } catch (e) {
                        throw new s.Kd(e?.message,e)
                    }
                    t.on("disconnect", this._disconnected),
                    this._wallet = t,
                    this._publicKey = e,
                    this.emit("connect", e)
                } catch (e) {
                    throw this.emit("error", e),
                    e
                } finally {
                    this._connecting = !1
                }
            }
            async disconnect() {
                let e = this._wallet;
                if (e) {
                    e.off("disconnect", this._disconnected),
                    this._wallet = null,
                    this._publicKey = null;
                    try {
                        await e.disconnect()
                    } catch (e) {
                        this.emit("error", new s.Y8(e?.message,e))
                    }
                }
                this.emit("disconnect")
            }
            async sendTransaction(e, t, n={}) {
                try {
                    let r = this._wallet;
                    if (!r)
                        throw new s.kW;
                    let {signers: i, ...a} = n;
                    try {
                        return await r.send(e, i, a, t, this.publicKey)
                    } catch (e) {
                        throw new s.UF(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signTransaction(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        return await t.signTransaction(e, this.publicKey)
                    } catch (e) {
                        throw new s.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signAllTransactions(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        return await t.signAllTransactions(e, this.publicKey)
                    } catch (e) {
                        throw new s.z4(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
            async signMessage(e) {
                try {
                    let t = this._wallet;
                    if (!t)
                        throw new s.kW;
                    try {
                        return await t.signMessage(e, this.publicKey)
                    } catch (e) {
                        throw new s.K3(e?.message,e)
                    }
                } catch (e) {
                    throw this.emit("error", e),
                    e
                }
            }
        }
    }
    ,
    91377: (e, t, n) => {
        "use strict";
        n.d(t, {
            A: () => i
        });
        var r = n(26432);
        let i = r.forwardRef(function(e, t) {
            let {title: n, titleId: i, ...s} = e;
            return r.createElement("svg", Object.assign({
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                viewBox: "0 0 24 24",
                strokeWidth: 1.5,
                stroke: "currentColor",
                "aria-hidden": "true",
                "data-slot": "icon",
                ref: t,
                "aria-labelledby": i
            }, s), n ? r.createElement("title", {
                id: i
            }, n) : null, r.createElement("path", {
                strokeLinecap: "round",
                strokeLinejoin: "round",
                d: "M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
            }))
        })
    }
    ,
    93627: (e, t) => {
        "use strict";
        var n;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.WalletAdapterNetwork = void 0,
        function(e) {
            e.Mainnet = "mainnet-beta",
            e.Testnet = "testnet",
            e.Devnet = "devnet"
        }(n || (t.WalletAdapterNetwork = n = {}))
    }
    ,
    94442: (e, t, n) => {
        "use strict";
        n.d(t, {
            S: () => a
        });
        var r = n(40476)
          , i = n(26432)
          , s = n(37358);
        let a = ({children: e, endpoint: t, config: n={
            commitment: "confirmed"
        }}) => {
            let a = (0,
            i.useMemo)( () => new r.Connection(t,n), [t, n]);
            return i.createElement(s.E.Provider, {
                value: {
                    connection: a
                }
            }, e)
        }
    }
    ,
    97680: (e, t, n) => {
        "use strict";
        n.d(t, {
            F: () => r
        });
        let r = "solana:signMessage"
    }
    ,
    99551: function(e, t, n) {
        "use strict";
        var r = this && this.__awaiter || function(e, t, n, r) {
            return new (n || (n = Promise))(function(i, s) {
                function a(e) {
                    try {
                        l(r.next(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function o(e) {
                    try {
                        l(r.throw(e))
                    } catch (e) {
                        s(e)
                    }
                }
                function l(e) {
                    var t;
                    e.done ? i(e.value) : ((t = e.value)instanceof n ? t : new n(function(e) {
                        e(t)
                    }
                    )).then(a, o)
                }
                l((r = r.apply(e, t || [])).next())
            }
            )
        }
          , i = this && this.__rest || function(e, t) {
            var n = {};
            for (var r in e)
                Object.prototype.hasOwnProperty.call(e, r) && 0 > t.indexOf(r) && (n[r] = e[r]);
            if (null != e && "function" == typeof Object.getOwnPropertySymbols)
                for (var i = 0, r = Object.getOwnPropertySymbols(e); i < r.length; i++)
                    0 > t.indexOf(r[i]) && Object.prototype.propertyIsEnumerable.call(e, r[i]) && (n[r[i]] = e[r[i]]);
            return n
        }
        ;
        Object.defineProperty(t, "__esModule", {
            value: !0
        }),
        t.BaseSignInMessageSignerWalletAdapter = t.BaseMessageSignerWalletAdapter = t.BaseSignerWalletAdapter = void 0;
        let s = n(6117)
          , a = n(68429)
          , o = n(51366);
        class l extends s.BaseWalletAdapter {
            sendTransaction(e, t) {
                return r(this, arguments, void 0, function*(e, t, n={}) {
                    let r = !0;
                    try {
                        if ((0,
                        o.isVersionedTransaction)(e)) {
                            if (!this.supportedTransactionVersions)
                                throw new a.WalletSendTransactionError("Sending versioned transactions isn't supported by this wallet");
                            if (!this.supportedTransactionVersions.has(e.version))
                                throw new a.WalletSendTransactionError(`Sending transaction version ${e.version} isn't supported by this wallet`);
                            try {
                                let r = (e = yield this.signTransaction(e)).serialize();
                                return yield t.sendRawTransaction(r, n)
                            } catch (e) {
                                if (e instanceof a.WalletSignTransactionError)
                                    throw r = !1,
                                    e;
                                throw new a.WalletSendTransactionError(null == e ? void 0 : e.message,e)
                            }
                        }
                        try {
                            let {signers: r} = n
                              , s = i(n, ["signers"]);
                            e = yield this.prepareTransaction(e, t, s),
                            (null == r ? void 0 : r.length) && e.partialSign(...r);
                            let a = (e = yield this.signTransaction(e)).serialize();
                            return yield t.sendRawTransaction(a, s)
                        } catch (e) {
                            if (e instanceof a.WalletSignTransactionError)
                                throw r = !1,
                                e;
                            throw new a.WalletSendTransactionError(null == e ? void 0 : e.message,e)
                        }
                    } catch (e) {
                        throw r && this.emit("error", e),
                        e
                    }
                })
            }
            signAllTransactions(e) {
                return r(this, void 0, void 0, function*() {
                    for (let t of e)
                        if ((0,
                        o.isVersionedTransaction)(t)) {
                            if (!this.supportedTransactionVersions)
                                throw new a.WalletSignTransactionError("Signing versioned transactions isn't supported by this wallet");
                            if (!this.supportedTransactionVersions.has(t.version))
                                throw new a.WalletSignTransactionError(`Signing transaction version ${t.version} isn't supported by this wallet`)
                        }
                    let t = [];
                    for (let n of e)
                        t.push((yield this.signTransaction(n)));
                    return t
                })
            }
        }
        t.BaseSignerWalletAdapter = l;
        class c extends l {
        }
        t.BaseMessageSignerWalletAdapter = c;
        class d extends c {
        }
        t.BaseSignInMessageSignerWalletAdapter = d
    }
}]);
