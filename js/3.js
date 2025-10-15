"use strict";
(self.webpackChunk_N_E = self.webpackChunk_N_E || []).push([[1233], {
    27373: (e, t, s) => {
        s.d(t, {
            N: () => d
        });
        var l = s(48876)
          , a = s(51816)
          , r = s(27442)
          , n = s(26432)
          , i = s(8626)
          , o = s(52630);
        let d = e => {
            let {children: t, trigger: s, icon: d, triggerLabel: c, className: m, scrollToFirstChild: x=!0, defaultOpen: u=!1, open: h, setOpen: p, ...g} = e
              , [j,b] = (0,
            n.useState)(!1)
              , f = null != h ? h : j
              , v = null != p ? p : b
              , y = (0,
            n.useRef)(null);
            return (0,
            n.useEffect)( () => {
                u && v(!0)
            }
            , [u, v]),
            (0,
            n.useLayoutEffect)( () => {
                let e;
                if (!u)
                    return f && x && y.current && (e = requestAnimationFrame( () => {
                        var e;
                        let t = null == (e = y.current) ? void 0 : e.firstElementChild;
                        t && t.scrollIntoView({
                            behavior: "smooth",
                            block: "nearest",
                            inline: "center"
                        })
                    }
                    )),
                    () => {
                        cancelAnimationFrame(e)
                    }
            }
            , [u, f, x]),
            (0,
            l.jsxs)("div", {
                className: (0,
                o.cn)("", m),
                ...g,
                children: [(0,
                l.jsxs)("button", {
                    className: "focus-ring group flex w-full cursor-pointer items-center justify-between gap-2",
                    onClick: () => {
                        v(e => !e),
                        !f && x && setTimeout( () => {
                            var e;
                            let t = null == (e = y.current) ? void 0 : e.firstElementChild;
                            t && t.scrollIntoView({
                                behavior: "smooth",
                                inline: "center"
                            })
                        }
                        , 180)
                    }
                    ,
                    children: [(0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-2",
                        children: [d ? (0,
                        l.jsx)("span", {
                            className: "text-brand",
                            children: d
                        }) : null, s || (0,
                        l.jsx)("h3", {
                            className: "text-heading-xxs font-brand leading-[1.5rem]",
                            children: c
                        })]
                    }), (0,
                    l.jsx)("span", {
                        className: "group-hover:bg-grey-800 grid size-8 place-items-center rounded-full transition-colors",
                        children: (0,
                        l.jsx)(i.zp, {
                            className: (0,
                            o.cn)("size-4 transition-transform", {
                                "rotate-180": f
                            })
                        })
                    })]
                }), (0,
                l.jsx)("div", {
                    className: "pt-0.5",
                    children: (0,
                    l.jsx)(a.N, {
                        initial: !1,
                        children: f && (0,
                        l.jsx)(r.P.div, {
                            ref: y,
                            animate: {
                                height: "auto"
                            },
                            className: "overflow-hidden",
                            exit: {
                                height: 0,
                                transition: {
                                    duration: .18
                                }
                            },
                            initial: {
                                height: 0
                            },
                            transition: {
                                duration: .25,
                                ease: [.26, .08, .25, 1]
                            },
                            children: (0,
                            l.jsx)("div", {
                                className: "py-1.5",
                                children: t
                            })
                        })
                    })
                })]
            })
        }
    }
    ,
    49146: (e, t, s) => {
        s.d(t, {
            A: () => r
        });
        var l = s(38915)
          , a = s(188);
        let r = (0,
        l.h)()(e => ({
            openChart: !1,
            setOpenChart: t => e({
                openChart: t
            }),
            tokenMarketData: void 0,
            setTokenMarketData: t => e({
                tokenMarketData: t
            })
        }), a.x)
    }
    ,
    51145: (e, t, s) => {
        s.d(t, {
            SwapMainView: () => tV
        });
        var l = s(48876)
          , a = s(5657)
          , r = s(26432)
          , n = s(78228)
          , i = s(52630)
          , o = s(90529)
          , d = s(85346)
          , c = s(33194)
          , m = s(59271);
        let x = e => {
            let {title: t, image: s, description: r, onClose: o, onMouseEnter: d, onMouseLeave: c} = e;
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("border-border-lowest flex justify-between rounded-xl border p-2"),
                onMouseEnter: d,
                onMouseLeave: c,
                children: [(0,
                l.jsxs)("div", {
                    className: "flex items-center gap-x-3",
                    children: [(0,
                    l.jsx)(a.default, {
                        alt: t,
                        className: "bg-bg-low-em h-20 w-20 rounded-md",
                        height: 80,
                        src: s,
                        width: 80
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex flex-col gap-y-1 self-start py-2",
                        children: [(0,
                        l.jsx)("p", {
                            className: "font-heading text-heading-xxs text-text-high-em",
                            children: t
                        }), (0,
                        l.jsx)("p", {
                            className: "font-body text-body-xs text-text-mid-em",
                            children: r
                        })]
                    })]
                }), (0,
                l.jsx)("button", {
                    className: "self-start px-1.5 py-2",
                    onClick: o,
                    children: (0,
                    l.jsx)(n.u, {
                        className: "text-icons size-4"
                    })
                })]
            })
        }
          , u = () => {
            let {campaigns: e} = (0,
            c.A)()
              , {walletAddress: t} = (0,
            o.z)()
              , [s,a] = (0,
            r.useState)([])
              , [n,u] = (0,
            r.useState)(0)
              , [h,p] = (0,
            r.useState)(!1)
              , [g,j] = (0,
            r.useState)(!1)
              , b = (0,
            r.useRef)(null)
              , f = (0,
            r.useRef)(null)
              , v = e.banner_campaigns.filter(e => "true" !== localStorage.getItem("".concat(e.campaign_id)) && !s.includes(e.campaign_id))
              , y = (0,
            r.useCallback)(async e => {
                if (a(t => [...t, e.campaign_id]),
                localStorage.setItem("".concat(e.campaign_id), "true"),
                t)
                    try {
                        await (0,
                        d.d_)(e, t)
                    } catch (e) {
                        m.R.warn("Failed to mark campaign as seen:", e)
                    }
                u(e => {
                    let t = v.length - 1;
                    return e >= t ? Math.max(0, t - 1) : e
                }
                )
            }
            , [t, v.length])
              , N = (0,
            r.useCallback)( () => {
                v.length <= 1 || u(e => {
                    let t = e + 1;
                    return t >= 2 * v.length ? 0 : t
                }
                )
            }
            , [v.length])
              , w = (0,
            r.useCallback)(e => {
                !(v.length <= 1) && (u(e),
                p(!1),
                f.current && clearInterval(f.current),
                v.length > 1 && (f.current = setInterval(N, 5e3)))
            }
            , [v.length, N]);
            (0,
            r.useEffect)( () => {
                if (v.length <= 1 || h) {
                    f.current && (clearInterval(f.current),
                    f.current = null);
                    return
                }
                return f.current = setInterval(N, 5e3),
                () => {
                    f.current && (clearInterval(f.current),
                    f.current = null)
                }
            }
            , [v.length, h, N]);
            let k = (0,
            r.useCallback)( () => {
                !(v.length <= 1) && n >= v.length && (b.current && clearTimeout(b.current),
                b.current = setTimeout( () => {
                    j(!0),
                    u(e => e % v.length),
                    requestAnimationFrame( () => {
                        j(!1)
                    }
                    )
                }
                , 3e3))
            }
            , [n, v.length]);
            (0,
            r.useEffect)( () => () => {
                b.current && clearTimeout(b.current),
                f.current && clearInterval(f.current)
            }
            , []);
            let C = (0,
            r.useMemo)( () => v.length > 0, [v]);
            return (0,
            l.jsx)("div", {
                className: (0,
                i.cn)("relative overflow-hidden transition-all duration-500 ease-in-out", C ? "animate-in slide-in-from-top-4 fade-in-0 mb-4" : "h-0 opacity-0"),
                tabIndex: C ? 0 : -1,
                children: C && (0,
                l.jsx)(l.Fragment, {
                    children: 1 === v.length ? (0,
                    l.jsx)(x, {
                        description: v[0].description,
                        image: v[0].image,
                        title: v[0].title,
                        onClose: () => y(v[0]),
                        onMouseEnter: () => p(!0),
                        onMouseLeave: () => p(!1)
                    }, v[0].campaign_id) : (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)("div", {
                            className: "overflow-hidden rounded-xl",
                            children: (0,
                            l.jsx)("div", {
                                className: "flex transition-transform duration-500 ease-in-out",
                                style: {
                                    transform: "translateX(-".concat(100 * n, "%)"),
                                    transition: g ? "none" : "transform 500ms ease-in-out"
                                },
                                onTransitionEnd: k,
                                children: [...v, ...v].map( (e, t) => {
                                    let s = Math.max(.3, 1 - .3 * Math.abs(t - n));
                                    return (0,
                                    l.jsx)("div", {
                                        className: "w-full flex-shrink-0",
                                        style: {
                                            opacity: s,
                                            transition: g ? "none" : "opacity 200ms ease-in-out"
                                        },
                                        children: (0,
                                        l.jsx)(x, {
                                            description: e.description,
                                            image: e.image,
                                            title: e.title,
                                            onClose: () => y(e),
                                            onMouseEnter: () => p(!0),
                                            onMouseLeave: () => p(!1)
                                        })
                                    }, "".concat(e.campaign_id, "-").concat(t))
                                }
                                )
                            })
                        }), (0,
                        l.jsx)("div", {
                            className: "mt-2 flex justify-end gap-1",
                            children: v.map( (e, t) => (0,
                            l.jsx)("button", {
                                "aria-label": "Go to campaign ".concat(t + 1),
                                className: (0,
                                i.cn)("h-1 w-5 rounded-full transition-colors duration-300 ease-in-out", t === n % v.length ? "bg-grey-400" : "bg-bg-high-em"),
                                onClick: () => t === n % v.length ? void 0 : w(t)
                            }, t))
                        })]
                    })
                })
            })
        }
        ;
        var h = s(38079)
          , p = s(93355)
          , g = s(93749)
          , j = s(8626)
          , b = s(36795)
          , f = s(49146);
        let v = () => {
            let {openChart: e, setOpenChart: t} = (0,
            f.A)();
            return b.m1 ? (0,
            l.jsx)(g.$, {
                "aria-label": "Chart toggle",
                className: (0,
                i.cn)("h-8 w-8 p-0", "flex items-center justify-center"),
                size: "sm",
                variant: "ghost",
                onClick: () => t(!e),
                children: (0,
                l.jsx)(j.R5, {})
            }) : (0,
            l.jsx)(r.Fragment, {})
        }
        ;
        var y = s(99188)
          , N = s(73861)
          , w = s(25535)
          , k = s(25465)
          , C = s(51092)
          , A = s(30369);
        function S(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {maxDigits: s=18, maxDecimals: l=6, onError: a} = t;
            if (0 === e)
                return "0";
            let r = e.toFixed(l);
            r.includes(".") && (r = r.replace(/\.?0+$/, ""));
            let[n,i=""] = r.split(".")
              , o = parseFloat(r)
              , d = new A.A(10).pow(-l).toNumber();
            if (o > 0 && o < d)
                return null == a || a("Amount too small", "Amount is smaller than the minimum precision of ".concat(l, " decimal places")),
                "0";
            let c = n.replace(/^0+(?!$)/, "").length;
            if (c > s)
                return null == a || a("Amount too large", "Amount exceeds the maximum supported ".concat(s, " digits")),
                "9".repeat(s);
            let m = s - c;
            if (m <= 0)
                return n;
            let x = i.slice(0, Math.min(l, m));
            return x.length > 0 ? "".concat(n, ".").concat(x) : n
        }
        var M = s(60647)
          , T = s(32641)
          , E = s(27373)
          , z = s(77284)
          , F = s(54206)
          , P = s(54248);
        let L = e => {
            let {className: t, ...s} = e
              , {connected: a} = (0,
            o.z)()
              , {openOrders: n, isLoadingOpenOrder: d, errorFetchingOpenOrders: c} = (0,
            p.A)()
              , [m,x] = (0,
            r.useState)(!1);
            return a ? (0,
            l.jsx)("div", {
                className: (0,
                i.cn)("", t),
                ...s,
                children: (0,
                l.jsx)(E.N, {
                    defaultOpen: n.length > 0,
                    icon: (0,
                    l.jsx)(j.JQ, {}),
                    triggerLabel: "Open orders",
                    children: (0,
                    l.jsx)("div", {
                        className: "flex w-full flex-col gap-y-2",
                        children: d ? (0,
                        l.jsx)("div", {
                            className: "flex items-center justify-center py-4",
                            children: (0,
                            l.jsx)("div", {
                                className: "text-text-low-em",
                                children: "Loading orders..."
                            })
                        }) : c ? (0,
                        l.jsx)("div", {
                            className: "flex items-center justify-center py-4",
                            children: (0,
                            l.jsx)("div", {
                                className: "text-text-low-em",
                                children: "Failed to load orders"
                            })
                        }) : (0,
                        l.jsxs)(l.Fragment, {
                            children: [(0,
                            l.jsx)(F.A, {
                                openOrders: n
                            }), (0,
                            l.jsxs)("div", {
                                className: "flex items-center gap-x-2 self-end",
                                children: [(0,
                                l.jsx)(z.Q, {
                                    size: "sm",
                                    variant: "tertiary",
                                    onClick: () => {
                                        m ? x(!1) : x(!0)
                                    }
                                    ,
                                    children: m ? "Yes, cancel all" : "Cancel all orders"
                                }), (0,
                                l.jsx)(P.A, {})]
                            })]
                        })
                    })
                })
            }) : (0,
            l.jsx)(r.Fragment, {})
        }
        ;
        var R = s(37358)
          , D = s(50273)
          , I = s(15334)
          , X = s(43106);
        let B = (e, t) => t <= 0 || e <= 0 ? 0 : parseFloat(((e - t) / t * 100).toFixed(2))
          , O = (e, t) => {
            if (e === X.PI.Never)
                return;
            let s = 0;
            switch (e) {
            case X.PI["1H"]:
                s = 3600;
                break;
            case X.PI["1D"]:
                s = 86400;
                break;
            case X.PI["7D"]:
                s = 604800;
                break;
            default:
                let l = e.match(/(\d+)H\s*(\d+)?M?/);
                l && (s = 60 * parseInt(l[1], 10) * 60 + 60 * (l[2] ? parseInt(l[2], 10) : 0))
            }
            if (0 !== s)
                return (t + 2 * s).toString()
        }
          , G = e => {
            let {hasError: t} = e
              , {sellValue: s, buyValue: a, buyToken: n, rateValue: i, expireValue: d, sellToken: c} = (0,
            p.A)()
              , {tokenBalances: x} = (0,
            I.A)()
              , {walletAddress: u} = (0,
            o.z)()
              , {connection: h} = (0,
            R.w)()
              , {executeLimitOrder: b, isExecuting: f, canExecute: v} = (0,
            T.NI)()
              , y = (0,
            r.useMemo)( () => {
                if (!c || !x)
                    return !1;
                let e = x.find(e => e.token.address === c.address);
                return !e || parseFloat((0,
                M.x)(s)) > e.balance
            }
            , [x, c, s])
              , N = (0,
            r.useMemo)( () => {
                let e = !s || !c || !n || !u || !i || f || !v || t || "0H 0M" === d || !d || y;
                return m.R.info("\uD83D\uDD0D Create Order Button State:", {
                    sellValue: s,
                    buyValue: a,
                    sellToken: null == c ? void 0 : c.symbol,
                    buyToken: null == n ? void 0 : n.symbol,
                    walletAddress: u ? "connected" : "not connected",
                    rateValue: i,
                    isExecuting: f,
                    canExecute: v,
                    isDisabled: e,
                    hasError: t ? "Rate error" : ""
                }),
                e
            }
            , [s, y, a, c, n, u, i, f, v, t, d])
              , k = async () => {
                if (!c || !n || !u || !s || !i)
                    return void m.R.error("Missing required data for limit order creation");
                try {
                    let e = (0,
                    M.x)(s)
                      , t = (0,
                    M.x)(a)
                      , l = (0,
                    M.x)(i);
                    if (!e || !t || !l)
                        return void m.R.error("Invalid sell amount, buy amount, or rate amount");
                    let r = parseFloat(e)
                      , o = parseFloat(t)
                      , x = parseFloat(l);
                    if (r <= 0 || o <= 0 || x <= 0)
                        return void m.R.error("Sell amount, buy amount, and rate must be greater than 0");
                    let p = new A.A(r).mul(new A.A(10).pow(c.decimals)).toNumber()
                      , g = new A.A(x).mul(new A.A(10).pow(c.decimals)).toNumber();
                    m.R.info("Creating limit order with parameters:", {
                        inputMint: c.address,
                        outputMint: n.address,
                        amount: p.toString(),
                        rate: x,
                        payer: u
                    }),
                    m.R.info("\uD83D\uDE80 Creating Limit Order:", {
                        inputMint: c.address,
                        outputMint: n.address,
                        amount: p.toString(),
                        price: {
                            base: g.toString(),
                            exponent: c.decimals
                        },
                        payer: u,
                        sellToken: c.symbol,
                        buyToken: n.symbol,
                        sellAmount: r,
                        buyAmount: o,
                        rate: x,
                        timestamp: new Date().toISOString()
                    });
                    let j = await (0,
                    D.kZ)(h, "confirmed")
                      , f = O(d, j);
                    m.R.info("Creating limit order with expiration", {
                        expireValue: d,
                        currentSlot: j,
                        expSlot: f,
                        sellToken: c.symbol,
                        buyToken: n.symbol,
                        sellAmount: r,
                        rate: x
                    });
                    let v = await b({
                        params: {
                            inputMint: c.address,
                            outputMint: n.address,
                            amount: p.toString(),
                            price: {
                                base: g.toString(),
                                exponent: c.decimals
                            },
                            timeInForce: 0,
                            expSlot: f
                        },
                        payer: u,
                        feeParams: {
                            microLamports: "5000"
                        }
                    });
                    v.success ? m.R.info("\uD83C\uDF89 Limit Order Executed Successfully:", {
                        signature: v.signature,
                        timestamp: new Date().toISOString()
                    }) : m.R.error("❌ Limit Order Execution Failed:", {
                        error: v.error,
                        timestamp: new Date().toISOString()
                    })
                } catch (e) {
                    m.R.error("Failed to create limit order:", e)
                }
            }
            ;
            return (0,
            l.jsx)(w.u, {
                children: (0,
                l.jsx)(g.$, {
                    className: "w-full",
                    disabled: N,
                    icon: f ? (0,
                    l.jsx)(j.Nl, {
                        className: "mr-2 size-4 animate-spin"
                    }) : void 0,
                    variant: "primary",
                    onClick: k,
                    children: y ? "Insufficient balance" : f ? "Creating Order" : "Create Order"
                })
            })
        }
        ;
        var U = s(26131)
          , V = s(75279);
        let q = e => {
            let {className: t, ...s} = e
              , {setExpireValue: a} = (0,
            p.A)()
              , [n,o] = (0,
            r.useState)("")
              , [d,c] = (0,
            r.useState)("")
              , [m,x] = (0,
            r.useState)(X.PI.Never);
            return (0,
            r.useEffect)( () => {
                if (m === X.PI.Custom) {
                    if (!n && !d)
                        return void a("0H 0M");
                    a("".concat(n || 0, "H ").concat(d || 0, "M"))
                }
            }
            , [n, d, m, a]),
            (0,
            l.jsxs)(l.Fragment, {
                children: [(0,
                l.jsxs)("div", {
                    className: (0,
                    i.cn)("flex items-center justify-between gap-x-2", t),
                    ...s,
                    children: [(0,
                    l.jsx)("p", {
                        className: "text-text-high-em text-body-m font-medium",
                        children: "Expiry"
                    }), (0,
                    l.jsx)(V.I, {
                        fullWidth: !0,
                        className: "w-fit gap-x-0.5 rounded-2xl border-none bg-transparent",
                        disableAll: !1,
                        indicatorClassName: "rounded-xl",
                        options: X.M5.map(e => ({
                            label: e,
                            value: e,
                            className: "[&_span]:rounded-full [&_span]:data-[active]:text-text-high-em [&_span]:text-text-mid-em [&_span]:font-medium [&_span]:text-body-s"
                        })),
                        title: "time-frame",
                        value: m,
                        onValueChange: e => {
                            o(""),
                            c(""),
                            x(e),
                            e !== X.PI.Custom ? a(e) : a("")
                        }
                    })]
                }), m === X.PI.Custom && (0,
                l.jsxs)("div", {
                    className: "flex w-full flex-col gap-y-2",
                    children: [(0,
                    l.jsx)(U.p, {
                        className: "text-body-s",
                        contentProps: {
                            className: "bg-bg-low-em h-10"
                        },
                        id: "custom-hours-input",
                        placeholder: "0",
                        rightContent: (0,
                        l.jsx)("span", {
                            className: "text-body-s text-text-mid-em",
                            children: "Hours"
                        }),
                        value: n,
                        onChange: e => {
                            let t = e.target.value;
                            /^[0-9]*$/.test(t) && o(t)
                        }
                    }), (0,
                    l.jsx)(U.p, {
                        className: "text-body-s",
                        contentProps: {
                            className: "bg-bg-low-em h-10"
                        },
                        id: "custom-minutes-input",
                        placeholder: "0",
                        rightContent: (0,
                        l.jsx)("span", {
                            className: "text-body-s text-text-mid-em",
                            children: "Minutes"
                        }),
                        value: d,
                        onChange: e => {
                            let t = e.target.value;
                            /^[0-9]*$/.test(t) && ("" === t ? c("") : c(String(Math.min(Number(t), 59))))
                        }
                    })]
                })]
            })
        }
        ;
        var H = s(19995)
          , _ = s(55036);
        function W(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {maxDecimals: s=6, maxDigits: l=18, min: a, max: r} = t;
            if ("" === e)
                return !0;
            let n = e;
            if (e.startsWith(".") && (n = "0" + e),
            "." === e || /^\.\d*0*$/.test(e) && 0 === parseFloat(e))
                return !0;
            if (!new RegExp(s > 0 ? "^(?!0\\d)\\d{0,".concat(l - s, "}(\\.?\\d{0,").concat(s, "})?$") : "^(?!0\\d)\\d{0,".concat(l, "}$")).test(n))
                return !1;
            if ("" !== e && "." !== e) {
                let e = parseFloat(n);
                if (void 0 !== a && e < a || void 0 !== r && e > r)
                    return !1
            }
            return !0
        }
        let Q = (0,
        r.forwardRef)( (e, t) => {
            let {className: s, value: a, onChange: n, maxDecimals: o=6, maxDigits: d=18, min: c, max: m, autoFocus: x, formatInput: u, ...h} = e
              , [p,g] = (0,
            r.useState)(a)
              , j = (0,
            r.useRef)(null)
              , b = (0,
            r.useRef)(!1);
            return (0,
            r.useEffect)( () => {
                g(a)
            }
            , [a]),
            (0,
            r.useEffect)( () => {
                b.current && t && "object" == typeof t && t.current && document.activeElement !== t.current && t.current.focus()
            }
            ),
            (0,
            r.useEffect)( () => () => {
                j.current && clearTimeout(j.current)
            }
            , []),
            (0,
            l.jsx)("input", {
                ref: t,
                autoComplete: "off",
                autoFocus: x,
                className: (0,
                i.cn)("placeholder:text-text-low-em caret-brand sm:text-heading-m w-full bg-transparent text-3xl font-medium outline-none", s),
                inputMode: "decimal",
                type: "text",
                value: p,
                onBlur: () => {
                    b.current = !1
                }
                ,
                onChange: n ? e => {
                    let t = e.target.value;
                    if (t.endsWith(",") && (t = t.slice(0, -1) + "."),
                    W(u ? (0,
                    M.x)(t) : t, {
                        maxDecimals: o,
                        maxDigits: d,
                        min: c,
                        max: m
                    }))
                        if (u) {
                            var s;
                            let[l,a] = Y(t, null != (s = e.target.selectionStart) ? s : 0);
                            null == n || n(l, e),
                            j.current && clearTimeout(j.current),
                            j.current = setTimeout( () => {
                                j.current = void 0
                            }
                            , 500),
                            g(l),
                            (0,
                            _.nextTick)( () => {
                                e.target.setSelectionRange(a, a)
                            }
                            )
                        } else
                            null == n || n(t, e),
                            g(t),
                            j.current && clearTimeout(j.current),
                            j.current = setTimeout( () => {
                                j.current = void 0
                            }
                            , 500)
                }
                : void 0,
                onFocus: () => {
                    b.current = !0
                }
                ,
                ...h
            })
        }
        );
        Q.displayName = "NumericInput";
        let J = e => {
            let t = e.replace(/[^0-9.]/g, "");
            if (!t)
                return "";
            let s = t.endsWith(".")
              , l = t.split(".")
              , a = l[0]
              , r = l[1];
            if (!a)
                return t;
            let n = Number(a).toLocaleString("en-US");
            return s && !r ? "".concat(n, ".") : void 0 !== r ? "".concat(n, ".").concat(r) : n
        }
          , Y = (e, t) => {
            let s = J(e)
              , l = e.slice(0, t)
              , a = (l.match(/\d/g) || []).length
              , r = l.includes(".")
              , n = 0
              , i = 0
              , o = !1;
            for (let e = 0; e < s.length; e++)
                if (/\d/.test(s[e]) ? n++ : "." === s[e] && (o = !0),
                n === a && (!r || o)) {
                    i = e + 1;
                    break
                }
            return [s, i]
        }
        ;
        var Z = s(27442)
          , $ = s(55796);
        let K = e => {
            let {selectedToken: t, onTokenChange: s} = e
              , {popularTokens: a} = (0,
            N.A)()
              , [n,i] = (0,
            r.useState)([]);
            (0,
            r.useEffect)( () => {
                0 !== a.length && i(a.filter(e => e.address !== (null == t ? void 0 : t.address)).slice(0, 3))
            }
            , [t, a]);
            let o = e => {
                s(e)
            }
            ;
            return (0,
            l.jsx)(Z.P.ul, {
                animate: "visible",
                className: "flex items-center gap-2",
                initial: "hidden",
                variants: ee,
                children: n.map(e => (0,
                l.jsx)(Z.P.li, {
                    layout: !0,
                    transition: {
                        layout: {
                            type: "spring",
                            stiffness: 300,
                            damping: 30
                        }
                    },
                    variants: et,
                    children: (0,
                    l.jsx)("button", {
                        className: "focus-ring hover:bg-grey-700 flex cursor-pointer rounded-full transition-all duration-150 hover:scale-110 active:scale-95",
                        onClick: () => o(e),
                        children: (0,
                        l.jsx)($.H, {
                            logoURI: e.logoURI,
                            size: 24,
                            symbol: e.symbol
                        })
                    })
                }, e.address))
            })
        }
          , ee = {
            hidden: {},
            visible: {
                transition: {
                    staggerChildren: .1
                }
            }
        }
          , et = {
            hidden: {
                opacity: 0,
                x: 20
            },
            visible: {
                opacity: 1,
                x: 0,
                transition: {
                    type: "spring",
                    stiffness: 80,
                    damping: 10
                }
            }
        };
        var es = s(6008);
        let el = e => {
            let {className: t, children: s, dotsClassName: a, ...r} = e;
            return (0,
            l.jsxs)("span", {
                className: (0,
                i.cn)("flex items-center", t),
                ...r,
                children: [s, (0,
                l.jsx)("span", {
                    className: (0,
                    i.cn)("loading-dots", a)
                })]
            })
        }
        ;
        var ea = s(15653)
          , er = s(51491)
          , en = s(1187);
        let ei = () => {
            let {tokenBalances: e} = (0,
            I.A)();
            return {
                getTokenBalance: t => {
                    if (!t || !e.length)
                        return 0;
                    let s = e.find(e => e.token.address === t);
                    return (null == s ? void 0 : s.balance) || 0
                }
                ,
                getTokenUsdValue: t => {
                    if (!t || !e.length)
                        return 0;
                    let s = e.find(e => e.token.address === t);
                    return (null == s ? void 0 : s.usdValue) || 0
                }
            }
        }
          , eo = (0,
        r.memo)(e => {
            let {token: t, isSelected: s, onClick: a} = e
              , {connected: n} = (0,
            o.z)()
              , {getTokenBalance: d} = ei()
              , {balanceLoading: c, balanceStale: x, balanceError: u, tokenBalances: h} = (0,
            I.A)()
              , {getTokenPrice: p} = (0,
            en.A)()
              , g = (0,
            r.useCallback)(e => {
                e.stopPropagation(),
                (0,
                er._)(t.address, {
                    successMessage: "Address copied to clipboard",
                    errorMessage: "Failed to copy address",
                    durationMs: 1e3
                })
            }
            , [t.address])
              , b = (0,
            r.useMemo)( () => {
                if (!n)
                    return 0;
                try {
                    return d(t.address)
                } catch (e) {
                    return m.R.warn("Error getting token balance for", t.symbol, ":", e),
                    0
                }
            }
            , [n, d, t.address, t.symbol])
              , f = (0,
            r.useMemo)( () => {
                if (!n || b <= 0)
                    return 0;
                let e = h.find(e => e.token.address === t.address);
                return e ? e.usdValue : b * (p(t.address) || 0)
            }
            , [n, b, t.address, h, p])
              , v = n && c
              , y = n && u && !c;
            return (0,
            l.jsxs)("li", {
                className: (0,
                i.cn)("hover:bg-bg-mid-em xs:flex-row xs:items-center flex flex-col justify-between gap-3 rounded-lg bg-transparent px-1 py-2 transition-[background-color] duration-150 ease-out will-change-[background-color] sm:px-3 sm:py-3", s && "bg-bg-mid-em"),
                onClick: a,
                children: [(0,
                l.jsxs)("div", {
                    className: "flex items-center gap-3",
                    children: [(0,
                    l.jsx)("div", {
                        className: "relative size-8 shrink-0",
                        children: (0,
                        l.jsx)($.H, {
                            logoURI: t.logoURI,
                            size: 32,
                            symbol: t.symbol
                        })
                    }), (0,
                    l.jsxs)("div", {
                        className: "min-w-0 flex-1",
                        children: [(0,
                        l.jsxs)("h3", {
                            className: "text-body-s sm:text-body-m flex items-center gap-1 font-medium",
                            children: [(0,
                            l.jsx)("span", {
                                className: "truncate",
                                children: t.name
                            }), (0,
                            l.jsx)(ea.Bc, {
                                children: (0,
                                l.jsxs)(ea.m_, {
                                    supportMobileTap: !1,
                                    children: [(0,
                                    l.jsx)(ea.k$, {
                                        asChild: !0,
                                        children: (0,
                                        l.jsx)("span", {
                                            className: "shrink-0",
                                            children: t.verified ? (0,
                                            l.jsx)(j.C1, {
                                                className: "text-success size-4"
                                            }) : (0,
                                            l.jsx)(j.eq, {
                                                className: "text-alert size-4"
                                            })
                                        })
                                    }), (0,
                                    l.jsx)(ea.ZI, {
                                        className: "min-w-fit",
                                        side: "top",
                                        sideOffset: 2,
                                        children: t.verified ? "Verified token" : "Unverified token"
                                    })]
                                })
                            })]
                        }), (0,
                        l.jsxs)("div", {
                            className: "text-body-xs sm:text-body-s text-text-low-em flex flex-wrap items-center gap-2",
                            children: [(0,
                            l.jsx)("span", {
                                className: "text-text-high-em max-w-[5rem] truncate",
                                children: t.symbol
                            }), (0,
                            l.jsx)("span", {
                                className: "bg-grey-600 size-1 shrink-0 rounded-full"
                            }), (0,
                            l.jsx)(ea.Bc, {
                                children: (0,
                                l.jsxs)(ea.m_, {
                                    supportMobileTap: !1,
                                    children: [(0,
                                    l.jsx)(ea.k$, {
                                        asChild: !0,
                                        children: (0,
                                        l.jsx)("button", {
                                            className: "hover:text-text-high-em max-w-[8rem] cursor-pointer truncate transition-colors duration-150",
                                            onClick: g,
                                            children: t.address
                                        })
                                    }), (0,
                                    l.jsx)(ea.ZI, {
                                        className: "min-w-fit",
                                        side: "top",
                                        sideOffset: 2,
                                        children: "Copy CA"
                                    })]
                                })
                            })]
                        })]
                    })]
                }), (0,
                l.jsxs)("div", {
                    className: "text-body-s text-text-high-em mr-2 flex shrink-0 flex-col items-end gap-1 sm:mr-0",
                    children: [(0,
                    l.jsxs)("div", {
                        className: "flex items-start gap-1",
                        children: [(0,
                        l.jsx)("span", {
                            className: "text-text-low-em",
                            children: "Balance:"
                        }), n ? v ? (0,
                        l.jsx)(j.Nl, {
                            className: "text-text-low-em size-3 animate-spin"
                        }) : y ? (0,
                        l.jsxs)("span", {
                            className: "text-alert flex items-center gap-1 text-sm",
                            children: ["Error", (0,
                            l.jsx)("button", {
                                className: "text-alert hover:text-alert-emphasized text-xs underline",
                                title: "Retry loading balance",
                                type: "button",
                                onClick: e => {
                                    e.stopPropagation(),
                                    window.location.reload()
                                }
                                ,
                                children: "Retry"
                            })]
                        }) : (0,
                        l.jsxs)("span", {
                            className: (0,
                            i.cn)(x && "text-warning"),
                            children: [(0,
                            C.A)({
                                number: b,
                                options: {
                                    maximumFractionDigits: 4,
                                    style: "decimal"
                                }
                            }), x && " ⚠️"]
                        }) : (0,
                        l.jsx)("span", {
                            className: "font-mono",
                            children: "-"
                        })]
                    }), n && !v && !y && b > 0 && (0,
                    l.jsxs)("div", {
                        className: "text-body-xs text-text-low-em",
                        children: ["$", (0,
                        C.A)({
                            number: f,
                            options: {
                                maximumFractionDigits: 2,
                                style: "decimal"
                            }
                        })]
                    })]
                })]
            })
        }
        , (e, t) => e.token.address === t.token.address && e.isSelected === t.isSelected);
        eo.displayName = "TokenItem";
        let ed = e => {
            let {onClearSearch: t} = e;
            return (0,
            l.jsxs)("div", {
                className: "flex flex-col items-center py-10 sm:py-20",
                children: [(0,
                l.jsx)("h4", {
                    className: "font-brand text-heading-xs sm:text-heading-s mb-2",
                    children: "No Tokens To Show"
                }), (0,
                l.jsx)("p", {
                    className: "text-body-s sm:text-body-m text-text-mid-em mb-4",
                    children: "We did not find any tokens matching your search.."
                }), (0,
                l.jsx)(g.$, {
                    size: "sm",
                    variant: "tertiary",
                    onClick: t,
                    children: "Clear"
                })]
            })
        }
          , ec = (0,
        r.forwardRef)( (e, t) => {
            let {className: s, selectedToken: a, ...r} = e;
            return (0,
            l.jsxs)("button", {
                ref: t,
                className: (0,
                i.cn)("focus-ring hover:bg-grey-900 text-text-high-em bg-grey-800 border-grey-700 flex cursor-pointer items-center gap-0.5 rounded-full border p-1.5 pr-2 transition-[background-color,_scale] active:enabled:scale-95", "disabled:bg-grey-700 disabled:text-text-disabled disabled:cursor-not-allowed", s),
                ...r,
                children: [(0,
                l.jsx)($.H, {
                    logoURI: a.logoURI,
                    size: 24,
                    symbol: a.symbol
                }), (0,
                l.jsx)("span", {
                    className: "text-body-m px-1 font-medium",
                    children: a.symbol
                }), (0,
                l.jsx)(j.zp, {
                    className: "size-4 shrink-0"
                })]
            })
        }
        );
        ec.displayName = "TokenSelectorTrigger";
        var em = s(92631)
          , ex = s(58019);
        let eu = e => {
            let {customTrigger: t, selectedToken: s, onTokenChange: a, ...n} = e
              , [o,d] = (0,
            r.useState)(!1)
              , [c,x] = (0,
            r.useState)("")
              , [u,h] = (0,
            r.useState)(15)
              , [p,g] = (0,
            r.useState)(!1)
              , b = (0,
            r.useRef)(null)
              , {popularTokens: f, allTokens: v} = (0,
            N.A)()
              , {tokenBalances: y} = (0,
            I.A)();
            (0,
            r.useEffect)( () => {
                h(15)
            }
            , [c]),
            (0,
            r.useEffect)( () => {
                if (o) {
                    g(!0),
                    h(15);
                    let e = setTimeout( () => {
                        g(!1)
                    }
                    , 50);
                    return () => clearTimeout(e)
                }
                g(!1),
                b.current && (clearTimeout(b.current),
                b.current = null)
            }
            , [o]),
            (0,
            r.useEffect)( () => () => {
                b.current && clearTimeout(b.current)
            }
            , []);
            let w = (0,
            em.d)(c, 300)
              , k = (0,
            ex.h)(w, o)
              , C = (0,
            r.useMemo)( () => {
                let e = c.toLowerCase().trim();
                return "" !== e && !(e.length < 2) && (c !== w || k.isLoading)
            }
            , [c, w, k.isLoading])
              , A = (0,
            r.useMemo)( () => {
                if (!o)
                    return [];
                let e = c.toLowerCase().trim()
                  , t = [...y.map(e => e.token).filter(e => !0 === e.verified), ...v].filter( (e, t, s) => t === s.findIndex(t => t.address === e.address))
                  , s = [];
                if ("" === e)
                    s = t;
                else {
                    let l = t.filter(t => t.name.toLowerCase().includes(e) || t.symbol.toLowerCase().includes(e) || t.address.toLowerCase().includes(e));
                    s = e.length < 2 ? l : k.data || []
                }
                let l = new Map(y.map(e => [e.token.address, e]))
                  , a = []
                  , r = [];
                return s.forEach(e => {
                    l.has(e.address) ? a.push(e) : r.push(e)
                }
                ),
                a.sort( (e, t) => {
                    let s = l.get(e.address)
                      , a = l.get(t.address);
                    return ((null == a ? void 0 : a.usdValue) || 0) - ((null == s ? void 0 : s.usdValue) || 0)
                }
                ),
                [...a, ...r]
            }
            , [o, c, v, k.data, y])
              , S = (0,
            r.useMemo)( () => o ? A.slice(0, u) : [], [o, A, u])
              , M = u < A.length
              , T = (0,
            r.useCallback)( () => {
                h(e => Math.min(e + 15, A.length))
            }
            , [A.length])
              , E = (0,
            r.useCallback)(e => {
                b.current && clearTimeout(b.current),
                b.current = setTimeout( () => {
                    try {
                        let t = e.currentTarget;
                        if (!t || !t.isConnected)
                            return;
                        let {scrollTop: s, scrollHeight: l, clientHeight: a} = t;
                        l - s - a < 100 && M && T()
                    } catch (e) {
                        m.R.debug("Scroll handler error (expected if component unmounted):", e)
                    }
                }
                , 100)
            }
            , [M, T])
              , z = () => {
                x("")
            }
            ;
            return (0,
            l.jsxs)(es.lG, {
                open: o,
                onOpenChange: e => {
                    d(e),
                    !e && (z(),
                    b.current && (clearTimeout(b.current),
                    b.current = null))
                }
                ,
                children: [(0,
                l.jsx)(es.zM, {
                    asChild: !0,
                    children: t || (0,
                    l.jsx)(ec, {
                        selectedToken: s,
                        ...n
                    })
                }), (0,
                l.jsxs)(es.Cf, {
                    className: "max-h-[calc(100dvh-2rem)] sm:max-h-[calc(80dvh-4rem)]",
                    children: [(0,
                    l.jsxs)(es.c7, {
                        children: [(0,
                        l.jsx)(es.L3, {
                            children: "Select token"
                        }), (0,
                        l.jsx)(U.p, {
                            autoFocus: !0,
                            id: "token-search",
                            leftContent: (0,
                            l.jsx)(j.WI, {}),
                            placeholder: "Search by name, symbol or address...",
                            value: c,
                            wrapperProps: {
                                className: "mt-3"
                            },
                            onChange: e => x(e.target.value)
                        }), (0,
                        l.jsx)(es.rr, {
                            className: "sr-only",
                            children: "Select a token to swap"
                        })]
                    }), (0,
                    l.jsxs)(es.R4, {
                        className: "max-h-[30.375rem] min-h-[30.375rem] pt-1.5 pb-6 sm:pt-2",
                        children: [!c && f.length > 0 && (0,
                        l.jsxs)("div", {
                            className: "mb-2 sm:mb-4",
                            children: [(0,
                            l.jsx)("h3", {
                                className: "text-body-m text-text-low-em mb-2",
                                children: "Popular"
                            }), (0,
                            l.jsx)("div", {
                                className: "grid gap-x-1 gap-y-1.5",
                                style: {
                                    gridTemplateColumns: "repeat(auto-fit, minmax(115px, 1fr))"
                                },
                                children: f.map(e => (0,
                                l.jsx)(es.HM, {
                                    asChild: !0,
                                    children: (0,
                                    l.jsxs)("button", {
                                        className: (0,
                                        i.cn)("focus-ring text-body-m hover:bg-grey-900 text-text-high-em border-grey-700 bg-grey-800 flex w-full items-center justify-center gap-0.5 rounded-full border px-4.5 py-1.5 font-medium transition-[scale,_background-color] active:scale-96"),
                                        title: e.symbol,
                                        type: "button",
                                        onClick: () => {
                                            a(e)
                                        }
                                        ,
                                        children: [(0,
                                        l.jsx)($.H, {
                                            logoURI: e.logoURI,
                                            size: 24,
                                            symbol: e.symbol
                                        }), (0,
                                        l.jsx)("span", {
                                            className: "truncate px-1",
                                            children: e.symbol
                                        })]
                                    })
                                }, e.symbol))
                            })]
                        }), (0,
                        l.jsx)("h3", {
                            className: "text-body-m text-text-low-em sm:mb-2",
                            children: c ? C ? (0,
                            l.jsx)(el, {
                                children: "Searching"
                            }) : "Search Results" : "Tokens"
                        }), p ? (0,
                        l.jsx)("div", {
                            className: (0,
                            i.cn)("thin-scrollbar space-y-1 overflow-y-auto will-change-scroll", "[&::-webkit-scrollbar-track]:bg-bg-low-em", "[&::-webkit-scrollbar-corner]:bg-bg-low-em", "[&::-webkit-scrollbar-thumb]:border-bg-low-em", "[scrollbar-color:var(--scrollbar-thumb)_var(--bg-bg-low-em)]"),
                            style: {
                                height: "350px",
                                width: "100%",
                                contain: "layout style paint"
                            },
                            children: Array.from({
                                length: 15
                            }, (e, t) => (0,
                            l.jsxs)("div", {
                                className: "flex animate-pulse items-center gap-3 rounded-lg p-2 will-change-auto",
                                children: [(0,
                                l.jsx)("div", {
                                    className: "bg-bg-high-em h-8 w-8 shrink-0 rounded-full"
                                }), (0,
                                l.jsxs)("div", {
                                    className: "min-w-0 flex-1",
                                    children: [(0,
                                    l.jsxs)("div", {
                                        className: "mb-1 flex items-center justify-between",
                                        children: [(0,
                                        l.jsx)("div", {
                                            className: "bg-bg-high-em h-4 w-20 rounded"
                                        }), (0,
                                        l.jsx)("div", {
                                            className: "bg-bg-high-em h-3 w-12 rounded"
                                        })]
                                    }), (0,
                                    l.jsx)("div", {
                                        className: "bg-bg-high-em h-3 w-32 rounded"
                                    })]
                                }), (0,
                                l.jsx)("div", {
                                    className: "bg-bg-high-em h-4 w-4 shrink-0 rounded-full"
                                })]
                            }, "skeleton-".concat(t)))
                        }) : C ? (0,
                        l.jsxs)("div", {
                            className: "flex flex-col items-center justify-center",
                            style: {
                                height: "350px"
                            },
                            children: [(0,
                            l.jsx)("div", {
                                className: "border-brand mb-3 h-8 w-8 animate-spin rounded-full border-b-2"
                            }), (0,
                            l.jsx)(el, {
                                className: "text-body-s text-text-mid-em",
                                children: "Searching tokens"
                            })]
                        }) : S.length > 0 ? (0,
                        l.jsx)(ea.Bc, {
                            children: (0,
                            l.jsxs)("ul", {
                                className: (0,
                                i.cn)("space-y-1 overflow-y-auto will-change-scroll", "[&::-webkit-scrollbar-track]:bg-bg-low-em", "[&::-webkit-scrollbar-corner]:bg-bg-low-em", "[&::-webkit-scrollbar-thumb]:border-bg-low-em", "[scrollbar-color:var(--scrollbar-thumb)_var(--bg-bg-low-em)]"),
                                style: {
                                    height: "350px",
                                    width: "100%",
                                    contain: "layout style paint"
                                },
                                onScroll: E,
                                children: [S.map(e => (0,
                                l.jsx)(es.HM, {
                                    asChild: !0,
                                    children: (0,
                                    l.jsx)(eo, {
                                        isSelected: s.symbol === e.symbol,
                                        token: e,
                                        onClick: () => {
                                            a(e)
                                        }
                                    })
                                }, e.address)), M && (0,
                                l.jsx)("li", {
                                    className: "py-4 text-center",
                                    children: (0,
                                    l.jsxs)("button", {
                                        className: "text-sm text-blue-400 transition-colors duration-150 hover:text-blue-300",
                                        onClick: T,
                                        children: ["Load more tokens... (", A.length - u, " remaining)"]
                                    })
                                })]
                            })
                        }) : (0,
                        l.jsx)(ed, {
                            onClearSearch: () => x("")
                        })]
                    })]
                })]
            })
        }
        ;
        var eh = s(47545)
          , ep = s(51816)
          , eg = s(76776);
        function ej(e) {
            let t = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : {}
              , {min: s=0, max: l=100} = t;
            if ("" === e)
                return !0;
            let a = e;
            if (e.startsWith(".") && (a = "0" + e),
            "." === e || /^\.\d*0*$/.test(e) && 0 === parseFloat(e))
                return !0;
            if (!/^(?!0\d)\d{0,3}(\.?\d{0,2})?$/.test(a))
                return !1;
            if ("" !== e && "." !== e) {
                let e = parseFloat(a);
                if (e < s || e > l)
                    return !1
            }
            return !0
        }
        var eb = s(55210);
        let ef = e => {
            let {className: t, changeAmount: s, currentBalance: a, selectedToken: n, ...o} = e
              , {receiveAmountLoading: d} = (0,
            eb.j)()
              , [c,m] = (0,
            r.useState)(!1)
              , [x,u] = (0,
            r.useState)(!1)
              , [h,p] = (0,
            r.useState)("")
              , b = e => {
                let t = a || 0
                  , l = parseFloat(e) / 100 * t;
                n && (0,
                k.cG)(n.address) && 100 === parseFloat(e) && (l = Math.min(l, t < .01 ? t : t - .01)),
                s(S(l, {
                    maxDecimals: null == n ? void 0 : n.decimals
                }).toString())
            }
            ;
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("flex items-center", t),
                ...o,
                children: [(0,
                l.jsx)(ep.N, {
                    initial: !1,
                    children: (0,
                    l.jsxs)(Z.P.div, {
                        animate: {
                            width: c ? "auto" : 0
                        },
                        className: "flex items-center gap-x-1 overflow-hidden",
                        exit: {
                            width: 0,
                            transition: {
                                duration: .18
                            }
                        },
                        initial: {
                            width: 0
                        },
                        transition: {
                            duration: .25,
                            ease: [.26, .08, .25, 1]
                        },
                        children: [ev.map( (e, t) => (0,
                        l.jsxs)("div", {
                            className: "flex items-center gap-x-1",
                            children: [(0,
                            l.jsxs)(z.Q, {
                                className: "text-body-xs sm:text-body-s !text-brand font-semibold",
                                disabled: !a || d,
                                size: "sm",
                                onClick: () => b(String(e)),
                                children: [e, "%"]
                            }), t < ev.length - 1 && (0,
                            l.jsx)("span", {
                                className: "bg-grey-600 size-1 rounded-full"
                            })]
                        }, t)), (0,
                        l.jsxs)(eg.AM, {
                            open: x,
                            onOpenChange: u,
                            children: [(0,
                            l.jsx)(eg.Wv, {
                                asChild: !0,
                                children: (0,
                                l.jsx)(g.$, {
                                    className: "border-grey-700 hover:bg-bg-high-em h-5 w-5 bg-[#302F2B] p-0",
                                    disabled: !a || d,
                                    size: "xs",
                                    variant: "tertiary",
                                    children: (0,
                                    l.jsx)(j.IE, {
                                        className: "size-2.5"
                                    })
                                })
                            }), (0,
                            l.jsxs)(eg.hl, {
                                className: "flex w-60 flex-col gap-y-4 p-4",
                                children: [(0,
                                l.jsxs)("div", {
                                    className: "flex w-full items-center justify-between",
                                    children: [(0,
                                    l.jsx)("h4", {
                                        className: "font-heading text-heading-xxs",
                                        children: "Swap Amount"
                                    }), (0,
                                    l.jsx)(eh.iN, {
                                        className: "hover:bg-grey-800 rounded-full p-1.5",
                                        onClick: () => {
                                            u(!1),
                                            p("")
                                        }
                                        ,
                                        children: (0,
                                        l.jsx)(j.uv, {
                                            className: "size-3"
                                        })
                                    })]
                                }), (0,
                                l.jsx)(U.p, {
                                    autoFocus: !0,
                                    id: "swap-amount",
                                    inputMode: "decimal",
                                    placeholder: "0.00",
                                    rightContent: (0,
                                    l.jsx)("span", {
                                        className: "text-body-s text-neutral-200",
                                        children: "%"
                                    }),
                                    type: "text",
                                    value: h,
                                    onChange: e => {
                                        let t = e.target.value;
                                        ej(t) && p(t)
                                    }
                                }), (0,
                                l.jsxs)("div", {
                                    className: "flex items-center justify-end",
                                    children: [(0,
                                    l.jsx)(g.$, {
                                        size: "sm",
                                        variant: "ghost",
                                        onClick: () => {
                                            "" !== h.trim() ? p("") : u(!1)
                                        }
                                        ,
                                        children: "Reset"
                                    }), (0,
                                    l.jsx)(g.$, {
                                        disabled: !h.trim(),
                                        size: "sm",
                                        variant: "primary",
                                        onClick: () => {
                                            b(h),
                                            u(!1)
                                        }
                                        ,
                                        children: "Apply"
                                    })]
                                })]
                            })]
                        }), (0,
                        l.jsx)(g.$, {
                            className: "border-grey-700 hover:bg-bg-high-em h-5 w-5 bg-[#302F2B] p-0",
                            size: "xs",
                            variant: "tertiary",
                            onClick: () => m(!1),
                            children: (0,
                            l.jsx)(j.uv, {
                                className: "size-2.5"
                            })
                        })]
                    })
                }), (0,
                l.jsx)(z.Q, {
                    className: (0,
                    i.cn)("text-body-xs sm:text-body-s !text-brand font-semibold", c ? "hidden" : "flex"),
                    disabled: !a,
                    size: "sm",
                    onClick: () => m(!0),
                    children: "%"
                })]
            })
        }
          , ev = [50, 25]
          , ey = (0,
        r.forwardRef)( (e, t) => {
            let {label: s, selectedToken: a, onChange: n, onBlur: d, onFocus: c, onTokenChange: x, isLoading: u, className: h, wrapperClassName: p, tokenSelectorClassName: g, formatInput: b=!0, slippagePercent: f, hidePrices: v, hideShortcuts: y, ...N} = e
              , {connected: w} = (0,
            o.z)()
              , A = (0,
            H.Ub)("(max-width: 600px)")
              , {getTokenBalance: T} = ei()
              , {balanceLoading: E, balanceStale: F, balanceError: P} = (0,
            I.A)()
              , {getTokenPrice: L} = (0,
            en.A)()
              , [R,D] = (0,
            r.useState)(!1)
              , X = (0,
            r.useMemo)( () => {
                if (!w || !(null == a ? void 0 : a.address))
                    return 0;
                try {
                    return T(a.address)
                } catch (e) {
                    return m.R.warn("Error getting token balance:", e),
                    0
                }
            }
            , [w, null == a ? void 0 : a.address, T])
              , B = (0,
            r.useMemo)( () => {
                if (!(null == a ? void 0 : a.address) || !N.value)
                    return 0;
                try {
                    let e = L(a.address)
                      , t = parseFloat((0,
                    M.x)(N.value)) || 0;
                    return e * t
                } catch (e) {
                    return m.R.warn("Error calculating USD value:", e),
                    0
                }
            }
            , [null == a ? void 0 : a.address, N.value, L])
              , O = N.readOnly
              , G = (0,
            r.useCallback)( () => {
                if (X > 0) {
                    let e = X;
                    a && (0,
                    k.cG)(a.address) && (e = X < .01 ? X : X - .01),
                    null == n || n(S(e, {
                        maxDecimals: null == a ? void 0 : a.decimals
                    }).toString())
                }
            }
            , [X, n, a]);
            if (!a)
                return (0,
                l.jsxs)("div", {
                    className: "border-border-lowest bg-bg-low-em flex flex-col gap-3 rounded-2xl border py-4 pr-4.5 pl-4 sm:pr-5 sm:pl-6",
                    children: [(0,
                    l.jsx)("label", {
                        className: "text-text-low-em text-body-s flex items-center gap-1 sm:text-base",
                        htmlFor: N.id,
                        children: s
                    }), (0,
                    l.jsx)("div", {
                        className: "flex items-center justify-center py-8",
                        children: (0,
                        l.jsx)(j.Nl, {
                            className: "text-text-low-em size-6 animate-spin"
                        })
                    })]
                });
            let U = w && E
              , V = w && P && !E;
            return (0,
            l.jsx)(l.Fragment, {
                children: (0,
                l.jsxs)("div", {
                    className: (0,
                    i.cn)("rounded-2xl border", "flex flex-col gap-3", "bg-bg-low-em px-5 py-4", R ? "border-brand" : "border-border-lowest", p),
                    children: [(0,
                    l.jsx)("label", {
                        className: "text-text-low-em text-body-s flex items-center gap-1",
                        htmlFor: N.id,
                        children: s
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex items-center justify-between gap-3",
                        children: [u ? (0,
                        l.jsx)("div", {
                            className: "flex size-10 items-center justify-start",
                            children: (0,
                            l.jsx)(j.Nl, {
                                className: "text-text-low-em size-6 animate-spin"
                            })
                        }) : (0,
                        l.jsx)(Q, {
                            ref: t,
                            className: (0,
                            i.cn)(O ? "truncate" : "", h),
                            formatInput: b,
                            placeholder: "0",
                            title: O ? N.value : void 0,
                            onBlur: e => {
                                D(!1),
                                null == d || d(e)
                            }
                            ,
                            onChange: n,
                            onFocus: e => {
                                e.target.disabled || e.target.readOnly || (D(!0),
                                null == c || c(e))
                            }
                            ,
                            ...N,
                            maxDecimals: 6
                        }), (0,
                        l.jsxs)("div", {
                            className: "flex items-center gap-2",
                            children: [y || O || A ? null : (0,
                            l.jsx)(K, {
                                selectedToken: a,
                                onTokenChange: x
                            }), (0,
                            l.jsx)(eu, {
                                className: (0,
                                i.cn)("shrink-0", g),
                                selectedToken: a,
                                onTokenChange: x
                            })]
                        })]
                    }), v ? (0,
                    l.jsx)(r.Fragment, {}) : (0,
                    l.jsx)(l.Fragment, {
                        children: (0,
                        l.jsxs)("div", {
                            className: "text-body-xs sm:text-body-s text-text-low-em flex items-center justify-between gap-2",
                            children: [(0,
                            l.jsx)("div", {
                                className: "flex min-w-0 flex-1 items-center gap-1",
                                children: u ? (0,
                                l.jsx)("div", {
                                    className: "flex size-5 items-center justify-start",
                                    children: (0,
                                    l.jsx)(j.Nl, {
                                        className: "text-text-low-em size-3 animate-spin"
                                    })
                                }) : (0,
                                l.jsxs)("div", {
                                    className: (0,
                                    i.cn)("truncate", "flex items-center gap-x-0.5", f < -5 ? "text-alert" : ""),
                                    children: [(0,
                                    l.jsxs)("p", {
                                        children: ["≈", " ", (0,
                                        C.A)({
                                            number: B,
                                            options: {
                                                style: "currency",
                                                currency: "USD",
                                                maximumFractionDigits: 2
                                            }
                                        })]
                                    }), f && f < 0 ? (0,
                                    l.jsxs)("span", {
                                        children: ["(", (0,
                                        C.A)({
                                            number: f,
                                            options: {
                                                maximumFractionDigits: 2
                                            }
                                        }), "%)"]
                                    }) : (0,
                                    l.jsx)(r.Fragment, {})]
                                })
                            }), (0,
                            l.jsxs)("div", {
                                className: "flex items-center gap-1",
                                children: [(0,
                                l.jsxs)("div", {
                                    className: "flex items-center gap-0.5",
                                    children: [(0,
                                    l.jsxs)("span", {
                                        children: ["Balance: ", w ? "" : "-"]
                                    }), w ? U ? (0,
                                    l.jsx)(j.Nl, {
                                        className: "text-text-low-em size-3 animate-spin"
                                    }) : V ? (0,
                                    l.jsxs)("span", {
                                        className: "text-alert flex items-center gap-1 text-sm",
                                        children: ["Error", (0,
                                        l.jsx)("button", {
                                            className: "text-alert hover:text-alert-emphasized text-xs underline",
                                            title: "Retry loading balance",
                                            type: "button",
                                            onClick: () => window.location.reload(),
                                            children: "Retry"
                                        })]
                                    }) : (0,
                                    l.jsxs)("span", {
                                        className: (0,
                                        i.cn)("text-text-high-em", F && "text-warning"),
                                        children: [(0,
                                        C.A)({
                                            number: X,
                                            options: {
                                                maximumFractionDigits: 4
                                            }
                                        }), F && " ⚠️"]
                                    }) : null]
                                }), O || !w ? null : (0,
                                l.jsxs)("div", {
                                    className: "flex items-center gap-1",
                                    children: [(0,
                                    l.jsx)("span", {
                                        className: "bg-grey-600 size-1 rounded-full"
                                    }), (0,
                                    l.jsx)(z.Q, {
                                        className: "text-body-xs sm:text-body-s !text-brand font-semibold",
                                        disabled: U || 0 === X,
                                        size: "sm",
                                        onClick: G,
                                        children: "Max"
                                    }), (0,
                                    l.jsx)("span", {
                                        className: "bg-grey-600 size-1 rounded-full"
                                    }), (0,
                                    l.jsx)(ef, {
                                        changeAmount: n,
                                        currentBalance: X,
                                        selectedToken: a
                                    })]
                                })]
                            })]
                        })
                    })]
                })
            })
        }
        );
        ey.displayName = "TokenInput";
        let eN = e => {
            let {rateErrorDirection: t, onSwitchToken: s} = e
              , {isLoadingToken: a} = (0,
            N.A)()
              , {updateUrl: n} = (0,
            y.A)()
              , {sellToken: i, buyToken: o, rateValue: d, buyValue: c, sellValue: m, setBuyValue: x, setSellValue: u, setBuyToken: h} = (0,
            p.A)()
              , [g,j] = (0,
            r.useState)(!1);
            return (0,
            r.useEffect)( () => {
                if (!m || !d || g || !(null == o ? void 0 : o.decimals))
                    return;
                let e = parseFloat((0,
                M.x)(m))
                  , s = parseFloat((0,
                M.x)(d));
                if (0 === e || 0 === s)
                    return;
                let l = e / s;
                t === eF.LowerThanMarket && (l = e * s),
                x((0,
                C.A)({
                    number: l,
                    fixedDecimals: 6
                }))
            }
            , [m, d, g, null == o ? void 0 : o.decimals, t, x]),
            (0,
            l.jsx)(ey, {
                hidePrices: !0,
                hideShortcuts: !0,
                id: "limit-buy-input",
                isLoading: a,
                label: "Buy",
                selectedToken: o,
                value: c,
                onBlur: () => j(!1),
                onChange: e => {
                    if (x(e),
                    d && g) {
                        let s = parseFloat((0,
                        M.x)(e))
                          , l = parseFloat((0,
                        M.x)(d));
                        if ("" === e && u(""),
                        s >= 0 && l > 0) {
                            let e = (0,
                            C.A)({
                                number: s * l
                            });
                            t === eF.LowerThanMarket && (e = (0,
                            C.A)({
                                number: s / l
                            })),
                            0 === parseFloat(e) ? u("") : u(e)
                        }
                    }
                }
                ,
                onFocus: e => {
                    e.target.disabled || e.target.readOnly || j(!0)
                }
                ,
                onTokenChange: e => {
                    if ([i.address, o.address].includes(e.address))
                        return void s();
                    h(e),
                    n(o.address, e.address)
                }
            })
        }
          , ew = e => {
            let {tokenPrice: t, isLoading: s, className: a, onUseMarketPrice: r, ...n} = e;
            return s ? (0,
            l.jsx)(j.Nl, {
                className: "text-text-lowest-em size-3 animate-spin"
            }) : (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("flex items-center gap-x-1", a),
                ...n,
                children: [(0,
                l.jsxs)("p", {
                    className: "text-body-s text-text-lowest-em flex",
                    children: ["Market:", (0,
                    l.jsxs)("span", {
                        className: "text-text-high-em ml-0.5",
                        children: ["~", (0,
                        C.A)({
                            number: t,
                            options: {
                                maximumFractionDigits: 2
                            }
                        })]
                    })]
                }), (0,
                l.jsx)(z.Q, {
                    className: "text-brand",
                    size: "sm",
                    onClick: r,
                    children: "Use"
                })]
            })
        }
          , ek = e => {
            let {basePrice: t, className: s, exchangeRate: a, buyTokenSymbol: n, sellTokenSymbol: o, rateErrorDirection: d, onSwitchToken: c, ...m} = e
              , {sellToken: x, buyToken: u, rateValue: h, setRateValue: g} = (0,
            p.A)()
              , [b,f] = (0,
            r.useState)(!1)
              , v = (k.xc.includes((null == x ? void 0 : x.address) || "") || k.xc.includes((null == u ? void 0 : u.address) || "")) && (null == x ? void 0 : x.address) !== k.wV && (null == u ? void 0 : u.address) !== k.wV
              , y = (null == x ? void 0 : x.address) === k.wV || (null == u ? void 0 : u.address) === k.wV;
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("rounded-2xl border", "flex flex-col gap-3", "bg-bg-low-em px-5 py-4", b ? "border-brand" : "border-border-lowest", s),
                ...m,
                children: [(0,
                l.jsxs)("div", {
                    className: "flex items-center justify-between",
                    children: [(0,
                    l.jsx)("p", {
                        className: "text-body-s text-text-low-em",
                        children: "".concat(d === eF.HigherThanMarket ? "Buy when 1" : "Sell  when 1", " ").concat(v || y ? o : n, "  is worth")
                    }), (0,
                    l.jsx)("button", {
                        onClick: c,
                        children: (0,
                        l.jsx)(j.A$, {
                            className: "text-icons rotate-90"
                        })
                    })]
                }), (0,
                l.jsxs)("div", {
                    className: "flex items-center justify-between",
                    children: [(0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-x-1",
                        children: [(0,
                        l.jsx)(Q, {
                            ref: _.ref,
                            className: (0,
                            i.cn)("placeholder:text-text-mid-em", "text-heading-xs md:text-heading-xs", "field-sizing-content w-fit"),
                            formatInput: !0,
                            maxDecimals: 6,
                            placeholder: "0",
                            value: h,
                            onBlur: () => f(!1),
                            onChange: e => g(e),
                            onFocus: e => {
                                e.target.disabled || e.target.readOnly || f(!0)
                            }
                        }), (0,
                        l.jsx)("p", {
                            className: "text-heading-xs text-text-mid-em font-medium",
                            children: v || y ? n : o
                        })]
                    }), (0,
                    l.jsx)(ew, {
                        isLoading: !t,
                        tokenPrice: t,
                        onUseMarketPrice: () => {
                            g((0,
                            C.A)({
                                number: a,
                                options: {
                                    maximumFractionDigits: 6
                                }
                            }))
                        }
                    })]
                })]
            })
        }
        ;
        var eC = s(59305);
        let eA = e => {
            let {onSwitchToken: t} = e
              , s = (0,
            r.useRef)(null)
              , {sellToken: a, buyToken: n, sellValue: i, setBuyValue: o, setSellValue: d, setSellToken: c} = (0,
            p.A)()
              , {view: m} = (0,
            eC.u)()
              , {isLoadingToken: x} = (0,
            N.A)()
              , {updateUrl: u} = (0,
            y.A)();
            return (0,
            r.useEffect)( () => {
                "limit" === m && (null == s ? void 0 : s.current) && setTimeout( () => {
                    var e;
                    return null == s || null == (e = s.current) ? void 0 : e.focus()
                }
                , 0)
            }
            , [m]),
            (0,
            l.jsx)(ey, {
                ref: s,
                autoFocus: !0,
                disabled: x,
                id: "limit-sell-input",
                label: "Sell",
                selectedToken: a,
                tokenSelectorClassName: "bg-grey-700 border-grey-600 hover:bg-bg-mid-em",
                value: i,
                wrapperClassName: "bg-bg-mid-em",
                onChange: e => {
                    "" === e && o(""),
                    d(e)
                }
                ,
                onTokenChange: e => {
                    if ([a.address, n.address].includes(e.address))
                        return void t();
                    c(e),
                    u(e.address, n.address)
                }
            })
        }
          , eS = e => {
            let {className: t, label: s, rightContent: a, ...r} = e;
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("w-full", "flex items-center justify-between", t),
                ...r,
                children: [(0,
                l.jsx)("div", {
                    className: "text-text-mid-em text-body-s",
                    children: s
                }), (0,
                l.jsx)("div", {
                    className: "text-body-s text-text-high-em flex items-center gap-x-0.5 font-medium",
                    children: a
                })]
            })
        }
          , eM = () => {
            let {buyValue: e, sellValue: t, buyToken: s, sellToken: a, expireValue: n, rateValue: i} = (0,
            p.A)();
            return e && 0 !== parseFloat(e) && t && 0 !== parseFloat(t) && i ? (0,
            l.jsxs)("div", {
                className: "flex w-full flex-col gap-y-2",
                children: [(0,
                l.jsx)("p", {
                    className: "text-body-m text-text-high-em font-medium",
                    children: "Summary"
                }), (0,
                l.jsx)(eS, {
                    label: "You sell",
                    rightContent: (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)("p", {
                            children: t
                        }), (0,
                        l.jsx)("span", {
                            className: "text-text-mid-em",
                            children: null == a ? void 0 : a.symbol
                        })]
                    })
                }), (0,
                l.jsx)(eS, {
                    label: "You buy",
                    rightContent: (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)("p", {
                            children: e
                        }), (0,
                        l.jsx)("span", {
                            className: "text-text-mid-em",
                            children: null == s ? void 0 : s.symbol
                        })]
                    })
                }), (0,
                l.jsx)(eS, {
                    label: "Trade value",
                    rightContent: (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)("span", {
                            className: "text-text-mid-em",
                            children: "≈"
                        }), (0,
                        l.jsx)("p", {
                            children: (0,
                            C.A)({
                                number: parseFloat((0,
                                M.x)(i)),
                                options: {
                                    currency: "USD",
                                    style: "currency",
                                    maximumFractionDigits: 4
                                }
                            })
                        })]
                    })
                }), (0,
                l.jsx)(eS, {
                    label: (0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-x-1",
                        children: [(0,
                        l.jsx)("p", {
                            children: "Limit order account rent"
                        }), (0,
                        l.jsx)(ea.Bc, {
                            children: (0,
                            l.jsxs)(ea.m_, {
                                children: [(0,
                                l.jsx)(ea.k$, {
                                    asChild: !0,
                                    children: (0,
                                    l.jsx)(j.ee, {
                                        className: "text-text-mid-em hover:text-text-high-em"
                                    })
                                }), (0,
                                l.jsx)(ea.ZI, {
                                    className: "min-w-[240px]",
                                    side: "top",
                                    sideOffset: 2,
                                    children: (0,
                                    l.jsx)("p", {
                                        className: "text-body-s",
                                        children: "A small deposit of ~0.004 SOL will be taken as rent, this will be returned after the limit order is filled/closed."
                                    })
                                })]
                            })
                        })]
                    }),
                    rightContent: (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)("p", {
                            children: "~0.004"
                        }), (0,
                        l.jsx)("span", {
                            className: "text-text-mid-em",
                            children: "SOL"
                        })]
                    })
                }), (0,
                l.jsx)(eS, {
                    label: "Expiry",
                    rightContent: n
                }), (0,
                l.jsx)(eS, {
                    label: "Platform fee",
                    rightContent: (0,
                    l.jsxs)("p", {
                        className: "text-brand",
                        children: [eT, "%"]
                    })
                })]
            }) : (0,
            l.jsx)(r.Fragment, {})
        }
          , eT = .1
          , eE = e => {
            let {className: t, rateErrorDirection: s, ...a} = e
              , {setView: r} = (0,
            eC.u)()
              , {buyToken: n, setSellValue: o, setBuyValue: d, setRateValue: c} = (0,
            p.A)();
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("w-full p-4", "bg-alert/20 rounded-lg text-[#F34C68]", t),
                ...a,
                children: [(0,
                l.jsx)("p", {
                    className: "text-body-m font-medium",
                    children: s === eF.LowerThanMarket ? "Limit price lower than market price" : "Limit price higher than market price"
                }), (0,
                l.jsxs)("div", {
                    className: "text-body-s mb-1",
                    children: [s === eF.LowerThanMarket ? "Since you are buying ".concat(null == n ? void 0 : n.symbol, " at a lower rate, we'd recommend using Instant swap.") : "Since you are buying ".concat(null == n ? void 0 : n.symbol, " at a higher rate, we'd recommend using Instant swap."), (0,
                    l.jsx)(z.Q, {
                        size: "sm",
                        variant: "primary",
                        onClick: () => {
                            r("instant"),
                            d(""),
                            o(""),
                            c("")
                        }
                        ,
                        children: "Go to Instant swap."
                    })]
                })]
            })
        }
          , ez = () => {
            let {sellToken: e, buyToken: t, buyValue: s, sellValue: a, rateValue: n, setRateValue: i, setBuyToken: o, setSellToken: d, setBuyValue: c, setSellValue: m} = (0,
            p.A)()
              , {openChart: x} = (0,
            f.A)()
              , {updateUrl: u} = (0,
            y.A)()
              , {price: h} = (0,
            T.YQ)((null == e ? void 0 : e.address) || "", {
                enabled: !!(null == e ? void 0 : e.address),
                refetchInterval: 15e3
            })
              , {price: b} = (0,
            T.YQ)((null == t ? void 0 : t.address) || "", {
                enabled: !!(null == t ? void 0 : t.address),
                refetchInterval: 15e3
            })
              , [v,N] = (0,
            r.useState)(!1)
              , [A,E,z,F,P] = (0,
            r.useMemo)( () => {
                if (!e || !t)
                    return [0, "", "", 1, h];
                let s = e.address === k.wV
                  , l = t.address === k.wV
                  , a = k.xc.includes(e.address || "")
                  , r = k.xc.includes(t.address || "")
                  , n = e.symbol || ""
                  , i = t.symbol || "";
                if (!b || !h)
                    return [0, n, i, 1, h];
                let o = h / b;
                return s && !r ? [b / h, i, n, 0, b / h] : l && !a ? [h / b, n, i, 1, h / b] : a && !l ? [b, i, n, 0, b] : r && !s ? [h, n, i, 1, h] : r && s ? [h, i, n, 1, h] : a && l ? [b, n, i, 0, b] : [1 / o, n, i, 0, 1 / o]
            }
            , [b, h, e, t])
              , R = (0,
            r.useMemo)( () => {
                if (!n || !A)
                    return 0;
                let e = (0,
                M.x)(n);
                return B(e ? parseFloat(e) : 0, A)
            }
            , [A, n])
              , D = (0,
            r.useMemo)( () => !!a && !!R && (0 === F && R > 0 || 1 === F && R < 0 || void 0), [R, a, F])
              , I = () => {
                d(t),
                o(e);
                let l = parseFloat((0,
                M.x)(s))
                  , r = !s || isNaN(l) ? "" : S(l, {
                    maxDecimals: 6
                }).toString()
                  , n = parseFloat((0,
                M.x)(a));
                c(!a || isNaN(n) ? "" : S(n, {
                    maxDecimals: 6
                }).toString()),
                m(r),
                t && e && u(t.address, e.address)
            }
            ;
            return (0,
            r.useEffect)( () => {
                e && t && A && i((0,
                C.A)({
                    number: A,
                    options: {
                        maximumFractionDigits: null == e ? void 0 : e.decimals
                    }
                }))
            }
            , [e, A, t, i, a]),
            (0,
            l.jsxs)("div", {
                className: "flex flex-col gap-2",
                children: [(0,
                l.jsxs)("div", {
                    className: "relative flex w-full flex-col gap-y-2",
                    children: [(0,
                    l.jsx)(eA, {
                        onSwitchToken: I
                    }), (0,
                    l.jsx)(g.$, {
                        "aria-label": "Switch tokens",
                        className: "z-elevated absolute top-1/2 left-1/2 mt-4 flex size-9 -translate-x-1/2 -translate-y-1/2",
                        icon: v ? (0,
                        l.jsx)(j.A$, {}) : (0,
                        l.jsx)(j.HK, {}),
                        variant: "tertiary",
                        onClick: I,
                        onMouseEnter: () => N(!0),
                        onMouseLeave: () => N(!1)
                    }), (0,
                    l.jsx)(eN, {
                        rateErrorDirection: F,
                        onSwitchToken: I
                    })]
                }), (0,
                l.jsx)(ek, {
                    basePrice: P,
                    buyTokenSymbol: z,
                    exchangeRate: A,
                    rateErrorDirection: F,
                    sellTokenSymbol: E,
                    onSwitchToken: I
                }), D && (0,
                l.jsx)(eE, {
                    rateErrorDirection: F
                }), (0,
                l.jsx)(q, {}), (0,
                l.jsx)(w.u, {
                    className: "w-full",
                    children: (0,
                    l.jsx)(G, {
                        hasError: D
                    })
                }), (0,
                l.jsx)(eM, {}), !x && (0,
                l.jsx)(L, {})]
            })
        }
        ;
        var eF = function(e) {
            return e[e.HigherThanMarket = 0] = "HigherThanMarket",
            e[e.LowerThanMarket = 1] = "LowerThanMarket",
            e
        }({});
        let eP = e => {
            let {className: t, onBackTo: s, backToLabel: a="Instant", primaryText: r, description: n, ...o} = e;
            return (0,
            l.jsxs)("div", {
                className: (0,
                i.cn)("py-15 text-center sm:py-20", t),
                ...o,
                children: [(0,
                l.jsx)("h2", {
                    className: "text-heading-s font-brand mb-2",
                    children: r
                }), (0,
                l.jsx)("p", {
                    className: "text-body-m mx-auto max-w-[16rem] text-neutral-200",
                    children: n
                }), (0,
                l.jsxs)(g.$, {
                    className: "mt-6 sm:mt-8",
                    size: "sm",
                    variant: "tertiary",
                    onClick: s,
                    children: ["Back to ", a]
                })]
            })
        }
          , eL = e => {
            let {isComingSoon: t} = e
              , {setView: s} = (0,
            eC.u)()
              , {sellToken: a, buyToken: n, setSellToken: i, setBuyToken: o} = (0,
            p.A)()
              , {allTokens: d, isLoadingToken: c} = (0,
            N.A)()
              , {parseTokensFromSearchParams: m} = (0,
            y.A)()
              , x = (0,
            r.useCallback)(e => d.find(t => t.address === e) || null, [d]);
            return ((0,
            r.useEffect)( () => {
                if (0 === d.length || c)
                    return;
                let e = null
                  , t = null
                  , {sellTokenAddress: s, receiveTokenAddress: l} = m();
                s && (e = x(s)),
                l && (t = x(l)),
                !e && d.length > 0 && (e = x("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")),
                !t && d.length > 0 && (t = x("So11111111111111111111111111111111111111112")),
                a || n || (i(e),
                o(t))
            }
            , [d, m, x, a, n, c, i, o]),
            t || !b.m1) ? (0,
            l.jsx)(eP, {
                description: "We’ll make sure you’ll hear about the release when it happens.",
                primaryText: "Limit orders coming soon",
                onBackTo: () => s("instant")
            }) : (0,
            l.jsx)(ez, {})
        }
          , eR = e => {
            let {isComingSoon: t} = e
              , {setView: s} = (0,
            eC.u)();
            return t ? (0,
            l.jsx)(eP, {
                description: "We’ll make sure you’ll hear about the release when it happens.",
                primaryText: "DCA coming soon",
                onBackTo: () => s("instant")
            }) : (0,
            l.jsx)("div", {
                children: (0,
                l.jsx)("h1", {
                    children: "DCA View"
                })
            })
        }
        ;
        var eD = s(82554)
          , eI = s(3242)
          , eX = s(5464);
        let eB = ["XsHtf5RpxsQ7jeJ9ivNewouZKJHbPxhPoEy6yYvULr7", "XswbinNKyPmzTa5CskMbCPvMW6G5CMnZXZEeQSSQoie", "Xs5UJzmCRQ8DWZjskExdSQDnbE6iLkRu2jjrRAB1JSU", "XsCPL9dNWBMvFtTmwcCA5v3xWPSMEBCszbQdiLLq6aN", "Xs3eBt7uRfJX8QUs4suhyU8p2M6DoUDrJyWBa8LLZsg", "XsaQTCgebC2KPbf27KUhdv5JFvHhQ4GDAPURwrEhAzb", "XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp", "XsPdAVBi8Zc1xvv53k4JcMrQaEDTgkGqKYeh7AYgPHV", "Xs3ZFkPYT2BN7qBMqf1j1bfTeTm1rFzEFSsQ1z3wAKU", "XswsQk4duEQmCbGzfqUUWYmi7pV7xpJ9eEmLHXCaEQP", "Xs6B6zawENwAbWVi7w92rjazLuAr5Az59qgWKcNb45x", "XsgSaSvNSqLTtFuyWPBhK9196Xb9Bbdyjj4fH3cPJGo", "XsNNMt7WTNA2sV3jrb1NNfNgapxRF5i4i6GcnTRRHts", "XsueG8BtpquVJX9LVLLEGuViXUungE6WmK5YZ3p3bd1", "Xsr3pdLQyXvDJBFgpR5nexCEZwXvigb8wbPYp4YoNFf", "XsaBXg8dU5cPM6ehmVctMkVqoiRG2ZjMo1cyBJ3AykQ", "Xs7ZdzSHLU9ftNJsii5fCeJhoRWSC32SQGzGQtePxNu", "XsvKCaNsxg2GN8jjUmq71qukMJr7Q1c5R2Mk9P8kcS8", "Xs7xXqkcK7K8urEqGg52SECi79dRp2cEKKuYjUePYDw", "Xseo8tgCZfkHxWS9xbFYeKFyMSbWEvZGFV1Gh53GtCV", "Xs2yquAgsHByNzx68WJC55WHjHBvG9JsMB7CWjTLyPy", "Xsnuv4omNoHozR6EEW5mXkw8Nrny5rB3jVfLqi6gKMH", "XsaHND8sHyfMfsWPj6kSdd5VwvCayZvjYgKmmcNL5qh", "Xsf9mBktVB9BSU5kf4nHxPq5hCBJ2j2ui3ecFGxPRGc", "Xsv9hRk1z5ystj9MhnA7Lq4vjSsLwzL2nxrwmwtD3re", "XsgaUyp4jd1fNBCxgtTKkW64xnnhQcvgaxzsbAq5ZD1", "XszjVtyhowGjSC5odCqBpW1CtXXwXjYokymrk7fGKD3", "XsRbLZthfABAPAfumWNEJhPyiKDW6TvDVeAeW7oKqA2", "XshPgPdXFRWB8tP1j82rebb2Q9rPgGX37RuqzohmArM", "XspwhyYPdWVM8XBHZnpS9hgyag9MKjLRyE3tVfmCbSr", "XsGVi5eo1Dh2zUpic4qACcjuWGjNv8GCt3dm5XcX6Dn", "XsMAqkcKsUewDrzVkait4e5u4y8REgtyS7jWgCpLV2C", "XsSr8anD1hkvNMu8XQiVcmiaTP7XGvYu7Q58LdmtE8Z", "XsuxRGDzbLjnJ72v74b7p9VY6N66uYgTCyfwwRjVCJA", "XsApJFV9MAktqnAc6jqzsHVujxkGm9xcSUffaBoYLKC", "XsqE9cRRpzxcGKDXj1BJ7Xmg4GRhZoyY1KpmGSxAWT2", "XsDgw22qRLTv5Uwuzn6T63cW69exG41T6gwQhEK22u2", "XsnQnU7AdbRZYe2akqqpibDdXjkieGFfSkbkjX1Sd1X", "Xsa62P5mvPszXL1krVUnU5ar38bBSVcWAB6fmPCo5Zu", "XspzcW1PRtgf6Wj92HCiZdjzKCyFekVD8P5Ueh3dRMX", "XsP7xzNPvEHS1m6qfanPUGjNmdnmsLKEoNAnHjdxxyZ", "Xs8S1uUs1zvS2p7iwtsG3b6fkhpvmwz4GYU3gWAmWHZ", "XsEH7wWfJJu2ZT3UCFeVfALnVA6CP5ur7Ee11KmzVpL", "XsfAzPzYrYjd4Dpa9BU3cusBsvWfVB9gBcyGC87S57n", "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh", "XsjFwUPiLofddX5cWFHW35GCbXcSu1BCUGfxoQAQjeL", "XsoBhf2ufR8fTyNSjqfU71DYGaE6Z3SUGAidpzriAA4", "Xsv99frTRUeornyvCfvhnDesQDWuvns1M852Pez91vF", "XsAtbqkAP1HJxy7hFDeq7ok6yM43DQ9mQ1Rh861X8rw", "Xsba6tUnSjDae2VcopDB6FGGDaxRrewFCDa5hKn5vT3", "XsYdjDjNUygZ7yGKfQaB6TxLh2gC6RRjzLtLAGJrhzV", "XsvNBAYkrDRNhA7wPHQfX3ZUXZyZLdnCQDfHZ56bzpg", "XsczbcQ3zfcgAEt9qHQES8pxKAVG5rujPSHQEXi4kaN", "XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W", "XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB", "Xs8drBWy3Sd5QY3aifG9kt9KFs2K3PGZmx7jWrsrk57", "XsjQP3iMAaQ3kQScQKthQpx9ALRbjKAjQtHg6TFomoc", "XszvaiXGPwvk2nwb3o9C1CX4K6zH8sez11E6uyup6fe", "XsssYEQjzxBCFgvYFFNuhJFBeHNdLWYeUSP8F45cDr9", "XsqgsbXwWogGJsNcVZ3TyVouy2MbTkfCFhCGGGcQZ2p", "Xs151QeqTCiuKtinzfRATnUESM2xTU6V9Wy8Vy538ci", "PresTj4Yc2bAR197Er7wz4UUKSfqt6FryBEdAriBoQB", "Pren1FvFX6J3E4kXhJuCiAD5aDmGEb7qJRncwA8Lkhw", "PreweJYECqtQwBtpxHL171nL2K6umo692gTm7Q3rpgF", "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh", "PreC1KtJ1sBPPqaeeqL6Qb15GTLCYVvyYEwxhdfTwfx", "ALTP6gug9wv5mFtx2tSU1YYZ1NrEc2chDdMPoJA8f8pu", "FJug3z58gssSTDhVNkTse5fP8GRZzuidf9SRtfB2RhDe", "5fKr9joRHpioriGmMgRVFdmZge8EVUTbrWyxDVdSrcuG", "AVw2QGVkXJPRPRjLAceXVoLqU5DVtJ53mdgMXp14yGit", "B8GKqTDGYc7F6udTHjYeazZ4dFCRkrwK2mBQNS4igqTv"]
          , eO = () => {
            let {data: e, isLoading: t, isFetching: s, isError: l, refetch: a} = (0,
            eI.s)();
            return {
                isTokenRestricted: (0,
                r.useCallback)(a => !t && !s && !l && !!e && !!e.isRegionRestricted && !!a && eB.includes(a), [e, t, s, l]),
                checkTokenRestrictedWithRefetch: (0,
                r.useCallback)(async e => {
                    if (!e)
                        return m.R.debug("No mintAddress provided"),
                        !1;
                    if (!eB.includes(e))
                        return m.R.debug("Token not in restricted list, skipping refetch", e),
                        !1;
                    let {data: t} = await a();
                    return t ? (m.R.debug("isRegionRestricted", t.isRegionRestricted),
                    !!t.isRegionRestricted || (m.R.debug("Region not restricted, token allowed"),
                    !1)) : (m.R.debug("No fresh data available"),
                    !1)
                }
                , [a]),
                isGeoBlockingLoading: s || t,
                isGeoBlockingError: l
            }
        }
        ;
        var eG = s(82945)
          , eU = s(76013)
          , eV = s(30925);
        let eq = () => {
            var e;
            let t = function(e) {
                let t = (0,
                eU.jE)()
                  , [s,l] = (0,
                r.useState)( () => {
                    let s = t.getQueryCache().find({
                        queryKey: e
                    });
                    return s ? {
                        data: s.state.data,
                        status: s.state.status,
                        error: s.state.error,
                        isLoading: "pending" === s.state.status,
                        isError: "error" === s.state.status,
                        isSuccess: "success" === s.state.status,
                        isFetching: "fetching" === s.state.fetchStatus
                    } : {
                        data: void 0,
                        status: "pending",
                        error: void 0,
                        isLoading: !0,
                        isError: !1,
                        isSuccess: !1,
                        isFetching: !1
                    }
                }
                );
                return (0,
                r.useEffect)( () => {
                    let s = new eV.$(t,{
                        queryKey: e,
                        enabled: !1
                    }).subscribe(e => {
                        l({
                            data: e.data,
                            status: e.status,
                            error: e.error,
                            isLoading: "pending" === e.status,
                            isError: "error" === e.status,
                            isSuccess: "success" === e.status,
                            isFetching: "fetching" === e.fetchStatus
                        })
                    }
                    );
                    return () => s()
                }
                , [t, e]),
                s
            }(eG.l.alerts.jitoStatus())
              , s = null != (e = t.data) ? e : "unknown";
            return {
                jitoStatus: s,
                isJitoActive: "active" === s,
                isJitoInactive: "inactive" === s,
                isJitoUnknown: "unknown" === s,
                isLoading: t.isLoading,
                isError: t.isError,
                lastUpdatedTimestamp: new Date
            }
        }
        ;
        var eH = s(20956)
          , e_ = s(41313)
          , eW = s(998)
          , eQ = s(31001)
          , eJ = s(19646)
          , eY = s(16694);
        let eZ = e => {
            let {transactions: t, className: s, ...a} = e
              , {connected: n} = (0,
            o.z)();
            return n ? (0,
            l.jsx)("div", {
                className: (0,
                i.cn)("", s),
                ...a,
                children: (0,
                l.jsx)(E.N, {
                    defaultOpen: t.length > 0,
                    icon: (0,
                    l.jsx)(j.JQ, {}),
                    triggerLabel: "Transaction History",
                    children: (0,
                    l.jsx)(eY.A, {
                        hasMore: !1,
                        showEndOfHistory: !1,
                        transactions: t.length > 3 ? t.slice(0, 3) : t
                    })
                })
            }) : (0,
            l.jsx)(r.Fragment, {})
        }
          , e$ = (0,
        s(86741).tv)({
            slots: {
                base: ["flex", "items-center", "border", "border-transparent"],
                icon: ["shrink-0"],
                content: ["flex-1"],
                title: ["font-medium"],
                description: ["text-sm"],
                button: ["shrink-0"]
            },
            variants: {
                size: {
                    md: {
                        base: "gap-4 px-4 py-4 rounded-lg",
                        icon: "h-5 w-5",
                        title: "text-body-l",
                        description: "text-body-s"
                    },
                    sm: {
                        base: "gap-2 px-2 py-2.5 rounded-md",
                        icon: "h-4 w-4",
                        title: "text-body-s",
                        description: "text-body-s"
                    },
                    xs: {
                        base: "gap-1 px-2 py-1 rounded",
                        icon: "h-3 w-3",
                        title: "text-body-s",
                        description: "text-body-s"
                    }
                },
                variant: {
                    info: {
                        base: "text-brand bg-brand-soft",
                        icon: "",
                        title: "",
                        description: ""
                    },
                    neutral: {
                        base: "text-brand border-grey-700 bg-grey-800",
                        icon: "text-text-mid-em",
                        title: "text-grey-000",
                        description: "text-text-mid-em"
                    },
                    alert: {
                        base: "text-alert bg-alert/20",
                        icon: "",
                        title: "",
                        description: ""
                    }
                },
                layout: {
                    horizontal: {
                        content: "flex items-center"
                    },
                    vertical: {
                        content: "flex flex-col"
                    }
                }
            },
            compoundSlots: [{
                slots: ["description"],
                size: "md",
                layout: "vertical",
                className: "mt-1"
            }, {
                slots: ["description"],
                size: "md",
                layout: "horizontal",
                className: "ml-2"
            }, {
                slots: ["description"],
                size: "sm",
                layout: "vertical",
                className: "mt-0.5"
            }, {
                slots: ["description"],
                size: "sm",
                layout: "horizontal",
                className: "ml-1"
            }, {
                slots: ["description"],
                size: "xs",
                layout: "vertical",
                className: "mt-0.5"
            }, {
                slots: ["description"],
                size: "xs",
                layout: "horizontal",
                className: "ml-1"
            }],
            defaultVariants: {
                size: "md",
                variant: "info",
                layout: "vertical"
            }
        })
          , eK = e => e
          , e0 = (0,
        r.createContext)(void 0)
          , e1 = () => {
            let e = (0,
            r.useContext)(e0);
            if (void 0 === e)
                throw Error("useAlert was used outside of its Provider");
            return e
        }
          , e2 = e => {
            let {children: t, ...s} = e
              , a = eK(s);
            return (0,
            l.jsx)(e0.Provider, {
                value: a,
                children: t
            })
        }
        ;
        var e3 = s(54094);
        function e5(e) {
            let {className: t, variant: s="info", size: a="md", layout: r="vertical", ...n} = e
              , {base: i} = e$({
                variant: s,
                size: a,
                layout: r
            });
            return (0,
            l.jsx)(e2, {
                layout: r,
                size: a,
                variant: s,
                children: (0,
                l.jsx)("div", {
                    className: i({
                        className: t
                    }),
                    ...n
                })
            })
        }
        function e6(e) {
            let {icon: t} = e
              , {variant: s, size: a, layout: r} = e1()
              , {icon: n} = e$({
                variant: s,
                size: a,
                layout: r
            })
              , i = {
                info: (0,
                l.jsx)(j.ee, {}),
                neutral: (0,
                l.jsx)(j.ee, {}),
                alert: (0,
                l.jsx)(j.eq, {})
            }
              , o = null != t ? t : i[s];
            return (0,
            l.jsx)(e => {
                let {icon: t} = e;
                return (0,
                e3.O)({
                    element: t,
                    themeStyle: n
                })
            }
            , {
                icon: o
            })
        }
        function e4(e) {
            let {children: t} = e
              , {variant: s, size: a, layout: r} = e1()
              , {content: n} = e$({
                variant: s,
                size: a,
                layout: r
            });
            return (0,
            l.jsx)("div", {
                className: n(),
                "data-slot": "alert-content",
                children: t
            })
        }
        function e9(e) {
            let {className: t, ...s} = e
              , {variant: a, size: r, layout: n} = e1()
              , {title: i} = e$({
                variant: a,
                size: r,
                layout: n
            });
            return (0,
            l.jsx)("div", {
                className: i({
                    className: t
                }),
                "data-slot": "alert-title",
                ...s
            })
        }
        function e8(e) {
            let {className: t, ...s} = e
              , {variant: a, size: r, layout: n} = e1()
              , {description: i} = e$({
                variant: a,
                size: r,
                layout: n
            });
            return (0,
            l.jsx)("div", {
                className: i({
                    className: t
                }),
                "data-slot": "alert-description",
                ...s
            })
        }
        var e7 = s(35958);
        let te = e => {
            let {tokenSymbol: t} = e
              , s = (0,
            H.Ub)("(max-width: 768px)");
            return (0,
            l.jsxs)(e5, {
                className: "mb-3",
                size: s ? "sm" : "md",
                variant: "alert",
                children: [(0,
                l.jsx)(e6, {
                    icon: (0,
                    l.jsx)(j.eq, {
                        className: "size-4"
                    })
                }), (0,
                l.jsxs)(e4, {
                    children: [(0,
                    l.jsxs)(e9, {
                        children: [t, " is unavailable in your region"]
                    }), (0,
                    l.jsx)(e8, {
                        children: "Trading for this token is restricted due to regional regulations."
                    })]
                })]
            })
        }
          , tt = e => {
            let {tokenSymbol: t} = e;
            return (0,
            l.jsx)(e7.Y, {
                children: (0,
                l.jsx)(te, {
                    tokenSymbol: t
                })
            })
        }
        ;
        var ts = s(2642);
        let tl = e => {
            let {sellAmount: t, receiveAmount: s, sellToken: a, receiveToken: n, isSwapDisabled: i, receiveAmountLoading: o} = e
              , d = (0,
            H.Ub)("(max-width: 768px)")
              , c = (0,
            ts.Y)((null == a ? void 0 : a.address) || "", {
                enabled: !!(null == a ? void 0 : a.address)
            })
              , m = (0,
            ts.Y)((null == n ? void 0 : n.address) || "", {
                enabled: !!(null == n ? void 0 : n.address)
            });
            return (0,
            r.useMemo)( () => {
                if (i || o || !t || !s || !a || !n || c.isLoading || m.isLoading)
                    return !1;
                try {
                    let e = parseFloat((0,
                    M.x)(t)) || 0
                      , l = parseFloat((0,
                    M.x)(s)) || 0;
                    if (e <= 0 || l <= 0)
                        return !1;
                    let a = c.price
                      , r = m.price;
                    if (!c.hasPrice || !m.hasPrice)
                        return !1;
                    let n = e * a
                      , i = l * r;
                    if (n <= 0 || i <= 0)
                        return !1;
                    return (i - n) / n * 100 < -5
                } catch (e) {
                    return !1
                }
            }
            , [i, o, t, s, a, n, c.isLoading, c.hasPrice, c.price, m.isLoading, m.hasPrice, m.price]) ? (0,
            l.jsxs)(e5, {
                className: "mb-3",
                size: d ? "sm" : "md",
                variant: "alert",
                children: [(0,
                l.jsx)(e6, {
                    icon: (0,
                    l.jsx)(j.eq, {
                        className: "size-4"
                    })
                }), (0,
                l.jsxs)(e4, {
                    children: [(0,
                    l.jsx)(e9, {
                        children: "Low output value detected"
                    }), (0,
                    l.jsx)(e8, {
                        children: "The output value is significantly lower than the input value. Please proceed with caution."
                    })]
                })]
            }) : null
        }
          , ta = e => (0,
        l.jsx)(e7.Y, {
            children: (0,
            l.jsx)(tl, {
                ...e
            })
        })
          , tr = () => {
            let {isJitoInactive: e} = eq()
              , {settings: t} = (0,
            e_.t0)()
              , s = (0,
            H.Ub)("(max-width: 768px)");
            return e && "mev-protect" === t.txFeeSettings.broadcastMode && "auto" === t.txFeeSettings.feeMode ? (0,
            l.jsx)(e5, {
                className: "mb-3",
                size: s ? "sm" : "md",
                variant: "alert",
                children: (0,
                l.jsxs)(e4, {
                    children: [(0,
                    l.jsx)(e9, {
                        children: "MEV fee estimation currently unavailable"
                    }), (0,
                    l.jsx)(e8, {
                        children: "Set a custom tip or use an alternate broadcast method."
                    })]
                })
            }) : null
        }
          , tn = () => (0,
        l.jsx)(e7.Y, {
            children: (0,
            l.jsx)(tr, {})
        });
        var ti = s(53833)
          , to = s(65908)
          , td = s(65314);
        let tc = {
            Jupiter: "/images/sources/jupiter.png",
            Hashflow: "/images/sources/hashflow.png",
            Titan: "/images/sources/titan.png",
            "Pyth Express Relay": "/images/sources/per.png",
            DFlow: "/images/sources/dflow.svg",
            OKX: "/images/sources/okx.svg",
            Metis: "/images/sources/jupiter.png"
        }
          , tm = {
            Jupiter: "M\xe9tis algorithm"
        }
          , tx = ["Titan", "Jupiter", "Pyth Express Relay", "OKX"]
          , tu = () => {
            var e;
            let {quotes: t, isStreaming: s, isLoading: n, venueInfo: o} = (0,
            eH.FF)()
              , {receiveToken: d} = (0,
            eb.j)()
              , {getTokenPrice: c} = (0,
            en.A)()
              , {isExecuting: m} = (0,
            eQ.A)()
              , x = (0,
            r.useRef)(null)
              , u = (0,
            r.useRef)(null)
              , h = (0,
            r.useMemo)( () => {
                if (m && x.current)
                    return x.current;
                if (!(null == t ? void 0 : t.quotes) || 0 === Object.keys(t.quotes).length)
                    return [];
                let e = e => {
                    if (!d)
                        return e;
                    let t = new A.A(e).div(new A.A(10).pow(d.decimals)).toNumber()
                      , s = c(d.address);
                    return new A.A(t).mul(s).toNumber()
                }
                  , s = Object.entries(t.quotes).map(t => {
                    let[s,l] = t
                      , a = l.outAmount
                      , r = e(a)
                      , n = Math.round(100 * r) / 100
                      , i = s;
                    return "Okx" === s && (i = "OKX"),
                    ("PythExpressRelay" === s || "Pyth" === s) && (i = "Pyth Express Relay"),
                    "Metis" === s && (i = "Jupiter"),
                    {
                        originalProvider: s,
                        provider: i,
                        route: l,
                        price: a,
                        usdValue: r,
                        roundedUSD: n
                    }
                }
                ).filter(Boolean).sort( (e, t) => t.price !== e.price ? t.price - e.price : "Titan" === e.provider && "Titan" !== t.provider ? -1 : "Titan" === t.provider && "Titan" !== e.provider ? 1 : e.provider.localeCompare(t.provider))
                  , l = Object.keys(t.quotes)
                  , a = s.length < 3 ? tx.filter(e => !(l.includes(e) || "Jupiter" === e && l.includes("Metis"))) : []
                  , r = [...s, ...null == a ? void 0 : a.map(e => ({
                    originalProvider: e,
                    provider: e,
                    route: null,
                    price: null,
                    usdValue: null,
                    roundedUSD: null
                }))].slice(0, 3);
                return !m && s.length > 0 && (x.current = r),
                r
            }
            , [t, d, c, m])
              , p = (0,
            r.useMemo)( () => m && u.current ? u.current : (!m && o && (u.current = o),
            o), [o, m])
              , g = (0,
            r.useMemo)( () => h.length > 0 ? h[0] : null, [h])
              , b = (0,
            r.useCallback)(e => (0,
            C.A)({
                number: e,
                options: {
                    style: "currency",
                    currency: "USD",
                    maximumFractionDigits: 2,
                    minimumFractionDigits: 2
                }
            }), [])
              , f = (e, t) => {
                if (!e || !t || !t.roundedUSD)
                    return null;
                let s = e.roundedUSD - t.roundedUSD;
                if (s <= 0)
                    return null;
                let l = (0,
                C.A)({
                    number: s,
                    options: {
                        style: "currency",
                        currency: "USD",
                        maximumFractionDigits: 2,
                        minimumFractionDigits: 2
                    }
                });
                return "+".concat(l)
            }
              , v = (0,
            r.useMemo)( () => {
                if (0 === h.length)
                    return {
                        badge: "w-16",
                        price: "w-20",
                        outperformance: "w-24"
                    };
                let e = h.reduce( (e, t) => {
                    let s = b(t.usdValue)
                      , l = f(g, h[1]);
                    return {
                        price: Math.max(e.price, s.length),
                        outperformance: Math.max(e.outperformance, (null == l ? void 0 : l.length) || 0)
                    }
                }
                , {
                    price: 0,
                    outperformance: 0
                })
                  , t = e => e <= 6 ? "w-15" : e <= 8 ? "w-18" : e <= 10 ? "w-22" : e <= 12 ? "w-26" : e <= 14 ? "w-30" : "w-34";
                return {
                    badge: "w-18",
                    price: t(e.price),
                    outperformance: t(e.outperformance)
                }
            }
            , [g, h]);
            return (n || s) && 0 === h.length ? (0,
            l.jsxs)("div", {
                className: "mt-5",
                children: [(0,
                l.jsx)("div", {
                    className: "mb-3 flex items-center justify-between gap-2 px-1.5",
                    children: (0,
                    l.jsxs)("h3", {
                        className: "text-heading-xxs font-brand flex items-center gap-2",
                        children: [(0,
                        l.jsx)(j.gq, {
                            className: "text-brand size-4"
                        }), "Quotes"]
                    })
                }), (0,
                l.jsx)("div", {
                    className: "bg-bg-low-em border-border-lowest relative w-full overflow-hidden rounded-3xl border p-4 pb-3.5",
                    children: (0,
                    l.jsxs)("div", {
                        className: "flex flex-col items-center justify-center py-8",
                        children: [(0,
                        l.jsx)(j.Nl, {
                            className: "text-brand mb-2 size-8 animate-spin"
                        }), (0,
                        l.jsx)("p", {
                            className: "text-text-low-em",
                            children: "Finding best quotes..."
                        })]
                    })
                })]
            }) : 0 === h.length ? (0,
            l.jsxs)("div", {
                className: "mt-5",
                children: [(0,
                l.jsx)("div", {
                    className: "mb-3 flex items-center justify-between gap-2 px-1.5",
                    children: (0,
                    l.jsxs)("h3", {
                        className: "text-heading-xxs font-brand flex items-center gap-2",
                        children: [(0,
                        l.jsx)(j.gq, {
                            className: "text-brand size-4"
                        }), "Quotes"]
                    })
                }), (0,
                l.jsx)("div", {
                    className: "bg-bg-low-em border-border-lowest relative w-full overflow-hidden rounded-3xl border p-4 pb-3.5",
                    children: (0,
                    l.jsx)("div", {
                        className: "flex flex-col items-center justify-center py-8",
                        children: (0,
                        l.jsx)("p", {
                            className: "text-text-low-em",
                            children: "No quotes available"
                        })
                    })
                })]
            }) : (0,
            l.jsxs)("div", {
                className: "mt-5",
                children: [(0,
                l.jsxs)("div", {
                    className: "mb-3 flex flex-wrap items-center justify-between gap-2 px-1.5",
                    children: [(0,
                    l.jsxs)("h3", {
                        className: "text-heading-xxs font-brand flex items-center gap-2",
                        children: [(0,
                        l.jsx)(j.gq, {
                            className: "text-brand size-4"
                        }), "Quotes"]
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-2",
                        children: [(0,
                        l.jsxs)("div", {
                            className: "flex items-center gap-1 overflow-hidden",
                            children: [(0,
                            l.jsx)(ea.Bc, {
                                children: (0,
                                l.jsxs)(ea.m_, {
                                    children: [(0,
                                    l.jsx)(ea.k$, {
                                        asChild: !0,
                                        children: (0,
                                        l.jsxs)(to.E, {
                                            className: "gap-0.5",
                                            size: "sm",
                                            variant: "neutral-emphasized",
                                            children: [(null == p ? void 0 : p.count) || 0, " ", (0,
                                            l.jsx)(j.hF, {
                                                className: "text-neutral-icons size-3 group-hover:text-neutral-200"
                                            })]
                                        })
                                    }), (0,
                                    l.jsx)(ea.ZI, {
                                        className: "text-body-s max-w-fit min-w-fit text-balance",
                                        side: "top",
                                        children: "Markets"
                                    })]
                                })
                            }), (0,
                            l.jsxs)("div", {
                                className: "text-text-mid-em text-body-s flex items-center gap-0.5",
                                children: [(0,
                                l.jsx)("span", {
                                    className: "text-text-lowest-em",
                                    children: "via"
                                }), (0,
                                l.jsx)(ea.Bc, {
                                    children: (0,
                                    l.jsxs)(ea.m_, {
                                        children: [(0,
                                        l.jsx)(ea.k$, {
                                            asChild: !0,
                                            className: (0,
                                            i.cn)(!(null == p ? void 0 : p.formatted) && "pointer-events-none"),
                                            children: (0,
                                            l.jsx)("span", {
                                                children: n || s ? "Finding routes..." : (null == p ? void 0 : p.formatted) || "No routes"
                                            })
                                        }), (0,
                                        l.jsx)(ea.ZI, {
                                            className: "text-body-s max-w-fit min-w-fit text-balance",
                                            side: "top",
                                            children: null == p || null == (e = p.venues) ? void 0 : e.join(", ")
                                        })]
                                    })
                                })]
                            })]
                        }), (0,
                        l.jsx)(ea.Bc, {
                            children: (0,
                            l.jsxs)(ea.m_, {
                                children: [(0,
                                l.jsx)(ea.k$, {
                                    asChild: !0,
                                    children: (0,
                                    l.jsx)(j.ee, {
                                        className: "size-4 shrink-0"
                                    })
                                }), (0,
                                l.jsx)(ea.ZI, {
                                    className: "text-body-s text-balance",
                                    side: "top",
                                    children: "Quotes from major routers on Solana are simulated on the same block to find the best executable prices."
                                })]
                            })
                        })]
                    })]
                }), h.length > 0 && (0,
                l.jsx)("div", {
                    className: "bg-bg-low-em border-border-lowest relative w-full overflow-hidden rounded-2xl border p-2",
                    children: (0,
                    l.jsxs)("table", {
                        className: "text-body-s w-full table-fixed caption-bottom whitespace-nowrap",
                        children: [(0,
                        l.jsxs)("colgroup", {
                            children: [(0,
                            l.jsx)("col", {
                                className: "w-auto"
                            }), (0,
                            l.jsx)("col", {
                                className: v.outperformance
                            }), (0,
                            l.jsx)("col", {
                                className: v.price
                            })]
                        }), (0,
                        l.jsx)(Z.P.tbody, {
                            layout: !0,
                            layoutRoot: !0,
                            children: h.map( (e, t) => {
                                let {provider: s, originalProvider: r} = e
                                  , n = 0 === t
                                  , o = f(g, h[1])
                                  , d = tc[s] || "/images/sources/titan.png";
                                return (0,
                                l.jsxs)(Z.P.tr, {
                                    className: (0,
                                    i.cn)("[&_td:first-child]:rounded-l-[.625rem] [&_td:last-child]:rounded-r-[.625rem]", "rounded-[.625rem] outline", n ? "text-text-high-em" : "text-text-low-em", n ? "bg-brand-soft/80 outline-brand-opacity-30" : "bg-transparent outline-transparent", "px-2 py-1 transition-[outline,background-color] duration-200"),
                                    layout: "position",
                                    children: [(0,
                                    l.jsx)("td", {
                                        className: "p-2 align-middle whitespace-nowrap",
                                        children: (0,
                                        l.jsxs)("div", {
                                            className: "flex items-center gap-2",
                                            children: [(0,
                                            l.jsx)(a.default, {
                                                alt: s,
                                                className: "h-6 w-auto object-contain ".concat("Titan" === s ? "pl-0.75" : "Pyth Express Relay" === s ? "pl-1" : "p-0.5"),
                                                height: 24,
                                                loader: td.f,
                                                src: d,
                                                width: 24
                                            }), (0,
                                            l.jsx)("span", {
                                                className: "text-text-high-em truncate",
                                                children: s
                                            }), tm[s] && (0,
                                            l.jsx)(ea.Bc, {
                                                children: (0,
                                                l.jsxs)(ea.m_, {
                                                    children: [(0,
                                                    l.jsx)(ea.k$, {
                                                        asChild: !0,
                                                        children: (0,
                                                        l.jsx)(j.ee, {
                                                            className: "size-4"
                                                        })
                                                    }), (0,
                                                    l.jsx)(ea.ZI, {
                                                        className: "min-w-fit",
                                                        side: "top",
                                                        sideOffset: 4,
                                                        children: tm[s]
                                                    })]
                                                })
                                            }), n && (0,
                                            l.jsx)(to.E, {
                                                size: "sm",
                                                variant: "primary-emphasized",
                                                children: "Best Price"
                                            })]
                                        })
                                    }), (0,
                                    l.jsx)(Z.P.td, {
                                        className: "text-success p-1 py-2 align-middle font-medium whitespace-nowrap",
                                        layout: "position",
                                        children: (0,
                                        l.jsx)("div", {
                                            className: (0,
                                            i.cn)("text-right tabular-nums", n && o ? "opacity-100" : "opacity-0"),
                                            children: null != o ? o : "\xa0".repeat(b(e.usdValue).length)
                                        })
                                    }), (0,
                                    l.jsx)(Z.P.td, {
                                        className: "p-2 text-right align-middle whitespace-nowrap",
                                        layout: "position",
                                        children: e.usdValue ? b(e.usdValue) : ""
                                    })]
                                }, r || s)
                            }
                            )
                        })]
                    })
                })]
            })
        }
        ;
        var th = s(93739);
        let tp = e => {
            let {className: t} = e
              , {appConfig: s} = (0,
            th.A)()
              , a = (0,
            H.Ub)("(max-width: 768px)");
            if (!b.m1 || a || !(null == s ? void 0 : s.QUOTE_STREAM_ENDPOINT))
                return null;
            let r = s.QUOTE_STREAM_ENDPOINT
              , n = "DEV";
            r === s.APOLLO_ENDPOINT_PROD ? n = "PROD" : r === s.APOLLO_ENDPOINT_DEV2 && (n = "DEV2");
            let i = e => {
                let t = new URL(window.location.href);
                return t.searchParams.set("apollo", e),
                t.toString()
            }
              , o = [{
                key: "prod",
                label: "PROD",
                url: i("prod")
            }, {
                key: "dev",
                label: "DEV",
                url: i("dev")
            }, {
                key: "dev2",
                label: "DEV2",
                url: i("dev2")
            }];
            return (0,
            l.jsxs)(eg.AM, {
                children: [(0,
                l.jsx)(eg.Wv, {
                    asChild: !0,
                    children: (0,
                    l.jsxs)("button", {
                        "aria-label": "Apollo endpoint selector",
                        className: "text-text-low-em hover:text-text-high-em flex items-center justify-center gap-1 pr-2 transition-colors duration-150 ".concat(t || ""),
                        type: "button",
                        children: [(0,
                        l.jsx)(j.ee, {
                            height: "14",
                            width: "14"
                        }), (0,
                        l.jsxs)("span", {
                            className: "text-xs",
                            children: ["Apollo: ", n]
                        }), (0,
                        l.jsx)(j.zp, {
                            height: "12",
                            width: "12"
                        })]
                    })
                }), (0,
                l.jsx)(eg.hl, {
                    align: "end",
                    className: "w-64 p-3",
                    side: "top",
                    children: (0,
                    l.jsxs)("div", {
                        className: "space-y-3",
                        children: [(0,
                        l.jsx)("div", {
                            className: "text-text-high-em text-sm font-semibold",
                            children: "Apollo Endpoint"
                        }), (0,
                        l.jsxs)("div", {
                            className: "space-y-2",
                            children: [(0,
                            l.jsxs)("div", {
                                className: "bg-grey-800 rounded-md p-2",
                                children: [(0,
                                l.jsx)("div", {
                                    className: "text-text-low-em text-xs",
                                    children: "Current Endpoint"
                                }), (0,
                                l.jsx)("div", {
                                    className: "text-text-high-em font-mono text-xs break-all",
                                    children: r
                                })]
                            }), (0,
                            l.jsxs)("div", {
                                className: "space-y-1",
                                children: [(0,
                                l.jsx)("div", {
                                    className: "text-text-low-em text-xs font-medium",
                                    children: "Switch to:"
                                }), o.map(e => (0,
                                l.jsxs)("a", {
                                    className: "hover:bg-grey-800 flex items-center justify-between rounded-md px-3 py-2 text-sm transition-all duration-150 ".concat(e.label === n ? "bg-grey-800 text-text-high-em font-medium" : "text-text-mid-em hover:text-text-high-em"),
                                    href: e.url,
                                    children: [(0,
                                    l.jsx)("span", {
                                        children: e.label
                                    }), e.label === n && (0,
                                    l.jsx)("span", {
                                        className: "text-brand text-xs",
                                        children: "●"
                                    })]
                                }, e.key))]
                            })]
                        })]
                    })
                })]
            })
        }
        ;
        var tg = s(47337);
        let tj = e => {
            let {className: t} = e
              , {ammMap: s, isLoading: a, excludedAmmIds: n, setExcludedAmmIds: i, setIncludedAmmIds: o} = (0,
            tg.A)()
              , {triggerSave: d} = (0,
            e_.t0)()
              , {enabledCount: c, totalCount: m, hasDisabledAmms: x} = (0,
            r.useMemo)( () => {
                if (a || 0 === Object.keys(s).length)
                    return {
                        enabledCount: 0,
                        totalCount: 0,
                        hasDisabledAmms: !1
                    };
                let e = Object.keys(s).length;
                return {
                    enabledCount: e - n.length,
                    totalCount: e,
                    hasDisabledAmms: n.length > 0
                }
            }
            , [s, n, a]);
            return a || 0 === m || !x ? null : (0,
            l.jsxs)("div", {
                className: "flex items-center justify-end gap-1 ".concat(t || ""),
                children: [(0,
                l.jsxs)("span", {
                    children: [c, "/", m, " AMMs are enabled."]
                }), (0,
                l.jsx)(z.Q, {
                    className: "text-body-xs hover:text-text-high-em font-normal",
                    onClick: () => {
                        i([]),
                        o(Object.keys(s)),
                        d()
                    }
                    ,
                    children: "Enable All"
                })]
            })
        }
        ;
        var tb = s(31283);
        let tf = e => {
            let {isLoading: t, receiveAmount: s, receiveToken: a, sellAmount: n, sellToken: i, onFlip: o} = e
              , [d,c] = (0,
            r.useState)(!1)
              , {} = (0,
            T.YQ)((null == i ? void 0 : i.address) || "", {
                enabled: !!(null == i ? void 0 : i.address)
            })
              , {} = (0,
            T.YQ)((null == a ? void 0 : a.address) || "", {
                enabled: !!(null == a ? void 0 : a.address)
            })
              , m = (0,
            r.useMemo)( () => {
                if (!s || !n || !n || !i || t)
                    return null;
                let e = Number(n.replace(/[^0-9.]/g, ""));
                return Number(s.replace(/[^0-9.]/g, "")) / e
            }
            , [s, n, i, t]);
            return i && a ? (0,
            l.jsx)("div", {
                className: "text-text-low-em text-body-xs py-2 pl-2 sm:mt-1",
                children: (0,
                l.jsxs)("div", {
                    className: "flex items-center justify-between",
                    children: [(0,
                    l.jsx)("button", {
                        "aria-label": "Flip exchange rate",
                        className: "hover:text-text-high-em flex cursor-pointer items-center gap-1 transition-colors duration-150",
                        disabled: t || !m,
                        type: "button",
                        onClick: () => {
                            c(!d),
                            null == o || o()
                        }
                        ,
                        children: (0,
                        l.jsx)(tb.A, {
                            inTokenAddress: i.address,
                            inTokenSymbol: i.symbol,
                            isFlipped: d,
                            outTokenAddress: a.address,
                            outTokenSymbol: a.symbol,
                            rate: m,
                            showPlaceholder: t || !m,
                            usingCustomFormat: !0
                        })
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-1",
                        children: [(0,
                        l.jsx)(tp, {}), (0,
                        l.jsx)(tj, {})]
                    })]
                })
            }) : null
        }
          , tv = e => {
            let {tokenSymbol: t, tokenType: s="buy"} = e
              , a = (0,
            H.Ub)("(max-width: 768px)");
            return (0,
            l.jsxs)(e5, {
                className: "mb-3",
                size: a ? "sm" : "md",
                variant: "alert",
                children: [(0,
                l.jsx)(e6, {
                    icon: (0,
                    l.jsx)(j.eq, {
                        className: "size-4"
                    })
                }), (0,
                l.jsxs)(e4, {
                    children: [(0,
                    l.jsxs)(e9, {
                        children: ["sell" === s ? "Sell" : "Buy", " token ", t, " is unverified"]
                    }), (0,
                    l.jsx)(e8, {
                        children: "Double check token address before proceeding."
                    })]
                })]
            })
        }
          , ty = e => {
            let {tokenSymbol: t, tokenType: s="buy"} = e;
            return (0,
            l.jsx)(e7.Y, {
                children: (0,
                l.jsx)(tv, {
                    tokenSymbol: t,
                    tokenType: s
                })
            })
        }
          , tN = () => {
            let {view: e} = (0,
            eC.u)()
              , {isLoadingToken: t} = (0,
            N.A)()
              , s = (0,
            H.Ub)("(max-width: 1023px)")
              , {openChart: a} = (0,
            f.A)()
              , {balanceLoading: n} = (0,
            I.A)()
              , {stopStream: i} = (0,
            eH.FF)()
              , {isTokenRestricted: o, checkTokenRestrictedWithRefetch: d} = eO()
              , {isJitoInactive: c} = eq()
              , {settings: x} = (0,
            e_.t0)()
              , {history: u} = (0,
            eW.q)()
              , {getTokenPrice: h} = (0,
            en.A)()
              , {executeSwap: p, isExecuting: b, canExecute: v, executionState: y, hasSufficientBalance: k} = (0,
            eX.v)()
              , {refetch: C, isRefetching: A} = (0,
            eI.s)()
              , {isRefreshing: S, hasNoRoutes: T, sellToken: E, receiveToken: z, receiveAmount: F, receiveAmountLoading: P, setSellAmount: L, sellAmount: R, onTokenSelect: D, onSwitchTokens: X, sellInputRef: B} = (0,
            eb.j)()
              , {quoteStreamId: O, setIsTradeGeoBlocked: G, setIsExecuting: U} = (0,
            eQ.A)()
              , [V,q] = (0,
            r.useState)("")
              , [_,W] = (0,
            r.useState)(!1)
              , {displaySlippage: Q, displayMevShield: J, displayMevShieldLabel: Y, loading: Z} = (0,
            ti.iD)()
              , $ = E && !1 === E.verified
              , K = z && !1 === z.verified
              , ee = o(null == E ? void 0 : E.address)
              , et = o(null == z ? void 0 : z.address)
              , es = ee || et
              , ea = c && "mev-protect" === x.txFeeSettings.broadcastMode && "auto" === x.txFeeSettings.feeMode
              , er = !v || b || S || t || _ && n || ea || es || !R || 0 === parseFloat(R) || P || !F || F && !k
              , ei = R && 0 !== parseFloat(R) && !P && F
              , eo = async () => {
                if (!v)
                    return;
                let e = await d(null == E ? void 0 : E.address)
                  , t = await d(null == z ? void 0 : z.address);
                if (e || t)
                    return void G(!0);
                try {
                    q(F),
                    (await p()).success && (W(!0),
                    setTimeout( () => W(!1), 5e3)),
                    q("")
                } catch (e) {
                    q(""),
                    m.R.error("Unexpected swap error:", e)
                }
            }
              , ed = (0,
            r.useMemo)( () => {
                if (!E || !z)
                    return 0;
                let e = h(E.address)
                  , t = parseFloat((0,
                M.x)(R)) || 0
                  , s = h(z.address)
                  , l = parseFloat((0,
                M.x)(F)) || 0
                  , a = (0,
                eJ.tf)(s * l, e * t);
                return a >= -.01 && a <= .01 ? 0 : a
            }
            , [h, R, E, z, F]);
            return (0,
            r.useEffect)( () => {
                G(es),
                U(b)
            }
            , [es, b, G, U]),
            (0,
            r.useEffect)( () => {
                (0,
                eD.A)(O) || O >= 0 && !R && i()
            }
            , [R, O, i]),
            (0,
            r.useEffect)( () => {
                "instant" === e && B.current && setTimeout( () => B.current.focus(), 0)
            }
            , [e, B]),
            (0,
            l.jsxs)(l.Fragment, {
                children: [(0,
                l.jsxs)("div", {
                    className: "flex flex-col gap-1",
                    children: [(0,
                    l.jsx)(ey, {
                        ref: B,
                        autoFocus: !0,
                        disabled: t,
                        id: "sell-token-input",
                        label: "Sell",
                        selectedToken: E,
                        tokenSelectorClassName: "bg-grey-700 border-grey-600 hover:bg-bg-mid-em",
                        value: R,
                        wrapperClassName: "bg-bg-mid-em",
                        onChange: L,
                        onTokenChange: e => D("sell", e)
                    }), (0,
                    l.jsx)("div", {
                        className: "relative w-full",
                        children: (0,
                        l.jsx)(g.$, {
                            "aria-label": "Switch tokens",
                            className: "z-elevated absolute top-1/2 left-1/2 flex size-9 -translate-x-1/2 -translate-y-1/2",
                            disabled: P,
                            icon: (0,
                            l.jsx)(j.A$, {}),
                            variant: "tertiary",
                            onClick: X
                        })
                    }), (0,
                    l.jsx)(ey, {
                        readOnly: !0,
                        id: "receive-token-input",
                        isLoading: P,
                        label: "Buy",
                        selectedToken: z,
                        slippagePercent: ed || 0,
                        value: V || F,
                        onTokenChange: e => D("receive", e)
                    }), (0,
                    l.jsx)(tf, {
                        isLoading: P,
                        receiveAmount: V || F,
                        receiveToken: z,
                        sellAmount: R,
                        sellToken: E
                    })]
                }), (0,
                l.jsxs)("div", {
                    className: "flex flex-col gap-2 pt-2 pb-4",
                    children: [$ && (0,
                    l.jsx)(ty, {
                        tokenSymbol: null == E ? void 0 : E.symbol,
                        tokenType: "sell"
                    }), K && (0,
                    l.jsx)(ty, {
                        tokenSymbol: null == z ? void 0 : z.symbol,
                        tokenType: "buy"
                    }), ee && E ? (0,
                    l.jsx)(tt, {
                        tokenSymbol: E.symbol
                    }) : et && z ? (0,
                    l.jsx)(tt, {
                        tokenSymbol: z.symbol
                    }) : null, (0,
                    l.jsx)(ta, {
                        isSwapDisabled: er,
                        receiveAmount: F,
                        receiveAmountLoading: P,
                        receiveToken: z,
                        sellAmount: R,
                        sellToken: E
                    }), (0,
                    l.jsx)(tn, {})]
                }), (0,
                l.jsx)(w.u, {
                    className: "w-full",
                    children: (0,
                    l.jsx)(g.$, {
                        className: "w-full",
                        disabled: er,
                        icon: b ? (0,
                        l.jsx)(j.Nl, {
                            className: "mr-2 size-4 animate-spin"
                        }) : void 0,
                        variant: "primary",
                        onClick: eo,
                        children: b ? (0,
                        l.jsxs)(el, {
                            children: ["building" === y && "Building transaction", "signing" === y && "Sign in wallet", "sending" === y && "Sending transaction", "confirming" === y && "Confirming"]
                        }) : _ && n ? "Loading balances..." : R && 0 !== parseFloat(R) ? T ? "No routes found" : P || !F ? "Loading" : k ? "Swap" : "Insufficient balance" : "Swap"
                    })
                }), ei && (0,
                l.jsx)(tu, {}), a && !s ? (0,
                l.jsx)(r.Fragment, {}) : (0,
                l.jsx)(eZ, {
                    className: "mt-4 sm:mt-5",
                    transactions: (null == u ? void 0 : u.swaps) || []
                })]
            })
        }
        ;
        var tw = s(91975)
          , tk = s(80032);
        let tC = e => {
            let {open: t, setOpen: s} = e;
            return (0,
            l.jsx)(es.lG, {
                open: t,
                onOpenChange: s,
                children: (0,
                l.jsxs)(es.Cf, {
                    children: [(0,
                    l.jsx)(es.c7, {
                        children: (0,
                        l.jsx)(es.L3, {
                            children: "Priority lane"
                        })
                    }), (0,
                    l.jsxs)(es.R4, {
                        className: "text-body-m flex w-full flex-col gap-y-4",
                        children: [(0,
                        l.jsx)("p", {
                            children: "Trade like the pros. Priority Lane gives Titan users institutional-grade execution like the pros at Galaxy. If speed and precision matter to you, this is your edge."
                        }), (0,
                        l.jsx)("p", {
                            children: "We route your swaps through Galaxy’s enterprise-grade propagation engine, unlocking sub-slot execution speeds with over 50% of trades filling in zero slots. Every millisecond saved means better prices, tighter spreads, and less slippage."
                        }), (0,
                        l.jsx)("p", {
                            children: "Priority Lane is only available for our VIP users. As a special promotion, new VIPs will get 25 free transactions via Titan’s Priority Lane - covering both gas fees and costs associated with Galaxy transaction propagation - giving faster, more reliable execution."
                        }), (0,
                        l.jsx)("p", {
                            children: "Simply toggle the Priority Lane button during your swap, and your swap will be sent through Galaxy’s high-performance infrastructure for guaranteed landing and optimal fills all at ZERO cost."
                        }), (0,
                        l.jsx)("p", {
                            className: "text-brand",
                            children: "This is\xa0the fastest way to trade on Solana— and it’s only for Titan VIPs."
                        })]
                    })]
                })
            })
        }
          , tA = () => {
            let {walletVipStatus: e, sponsoredTransactionStatus: t, priorityLaneEnabled: s, setPriorityLaneEnabled: a} = (0,
            tk.Ay)()
              , {isPrimeMode: n} = (0,
            ti.iD)()
              , [o,d] = (0,
            r.useState)(!1);
            if (!(null == t ? void 0 : t.exists) || !n || !(null == e ? void 0 : e.isVip))
                return null;
            let c = t.exists && t.count > 0;
            return (0,
            l.jsxs)(l.Fragment, {
                children: [(0,
                l.jsx)(ea.Bc, {
                    children: (0,
                    l.jsxs)(ea.m_, {
                        supportMobileTap: !1,
                        children: [(0,
                        l.jsx)(ea.k$, {
                            asChild: !0,
                            children: (0,
                            l.jsx)(tw.A, {
                                checked: s,
                                className: (0,
                                i.cn)("h-7 w-11.5", s ? "[&_button]:!bg-brand" : "[&_button]:!bg-bg-high-em"),
                                customBackgroundContent: (0,
                                l.jsxs)("div", {
                                    className: "flex h-full w-full items-center justify-center gap-x-2",
                                    children: [(0,
                                    l.jsx)(j.C2, {
                                        className: "text-brand-soft"
                                    }), (0,
                                    l.jsx)(j.C2, {
                                        className: "text-gray-500"
                                    })]
                                }),
                                disabled: !(null == e ? void 0 : e.isVip) || t.isLoading || !n,
                                id: "priority-lane",
                                thumbContent: c && (0,
                                l.jsx)("span", {
                                    className: (0,
                                    i.cn)("text-body-xxs font-medium", s ? "text-brand" : "text-bg-high-em"),
                                    children: t.count || 0
                                }),
                                thumbProps: {
                                    className: "data-[state=checked]:translate-x-[22px] relative z-20"
                                },
                                onCheckedChange: a
                            })
                        }), (0,
                        l.jsx)(ea.ZI, {
                            className: "sm:max-w-[26rem]",
                            side: "bottom",
                            children: (0,
                            l.jsxs)("div", {
                                className: "flex w-full flex-col gap-y-2",
                                children: [(0,
                                l.jsxs)("div", {
                                    className: "flex w-full items-center justify-between",
                                    children: [(0,
                                    l.jsx)("p", {
                                        className: "font-heading text-body-s",
                                        children: "Priority Lane"
                                    }), (0,
                                    l.jsxs)("span", {
                                        className: (0,
                                        i.cn)("rounded-md", "text-body-xs px-1.5 py-1", s ? "bg-brand-soft text-brand" : "bg-bg-mid-em text-text-high-em"),
                                        children: [s ? "Enabled" : "Disabled", " "]
                                    })]
                                }), (0,
                                l.jsx)("p", {
                                    className: "text-body-xs",
                                    children: "Priority Lane routes trades through Galaxy’s institutional-grade infrastructure for even better speed and precision."
                                }), (0,
                                l.jsx)("p", {
                                    className: "text-body-xs mt-1",
                                    children: "Only available to VIPs with limited sponsored transactions, after which fees apply."
                                }), (null == e ? void 0 : e.isVip) && t.exists && (0,
                                l.jsxs)("p", {
                                    className: "text-text-mid-em",
                                    children: ["Sponsored Transactions:", " ", (0,
                                    l.jsx)("span", {
                                        className: "text-text-high-em",
                                        children: t.isLoading ? "Loading..." : t.count || 0
                                    }), t.error && (0,
                                    l.jsx)("span", {
                                        className: "ml-2 text-red-500",
                                        children: "(Error loading)"
                                    })]
                                }), (0,
                                l.jsx)(z.Q, {
                                    className: "w-fit font-semibold",
                                    onClick: () => d(!0),
                                    children: (0,
                                    l.jsx)("p", {
                                        className: "text-body-xs text-brand font-semibold",
                                        children: "Learn more"
                                    })
                                })]
                            })
                        })]
                    })
                }), (0,
                l.jsx)(tC, {
                    open: o,
                    setOpen: d
                })]
            })
        }
          , tS = () => {
            let {isEmpty: e, setReceiveAmount: t, bestQuote: s, receiveToken: l} = (0,
            eb.j)();
            return (0,
            r.useEffect)( () => {
                if (!s || e) {
                    e && t("");
                    return
                }
                if (l) {
                    let e = s.route.outAmount
                      , a = l.decimals;
                    t((0,
                    C.A)({
                        number: new A.A(e).div(new A.A(10).pow(a)).toNumber()
                    }))
                }
            }
            , [s, e, l, t]),
            {}
        }
          , tM = (0,
        r.createContext)(void 0)
          , tT = e => {
            let {children: t} = e
              , s = tS();
            return (0,
            l.jsx)(tM.Provider, {
                value: s,
                children: t
            })
        }
          , tE = () => {
            let {isRefreshing: e, handleRefresh: t} = (0,
            eb.j)()
              , {connected: s} = (0,
            o.z)()
              , {view: a} = (0,
            eC.u)()
              , {sellAmount: r, receiveAmountLoading: n} = (0,
            eb.j)();
            return (0,
            l.jsx)(g.$, {
                "aria-label": "Refresh quote",
                className: (0,
                i.cn)("h-8 w-8", "flex items-center justify-center"),
                disabled: e || !s || !r || n,
                icon: n ? (0,
                l.jsx)(j.Nl, {
                    className: "size-4 animate-spin"
                }) : (0,
                l.jsx)(j.fN, {
                    className: "size-4"
                }),
                size: "sm",
                variant: "ghost",
                onClick: t
            }, a)
        }
        ;
        var tz = s(41339)
          , tF = s(43141);
        let tP = e => {
            let t = {};
            return Object.entries(tF.Wd).forEach(e => {
                let[s,l] = e;
                t[l] = s
            }
            ),
            t[e] || "faster"
        }
          , tL = e => tF.Wd[e] || "75"
          , tR = e => {
            let {liquiditySourcesState: t, onToggle: s} = e
              , [a,n] = (0,
            r.useState)(!0);
            return (0,
            r.useEffect)( () => {
                Object.keys(t).length && (Object.values(t).some(e => !e.isActive) ? n(!1) : n(!0))
            }
            , [t]),
            (0,
            l.jsxs)("div", {
                className: "flex flex-col gap-4",
                children: [(0,
                l.jsxs)("div", {
                    children: [(0,
                    l.jsx)(tB, {
                        icon: (0,
                        l.jsx)(j.Gg, {}),
                        children: "What is Prime?"
                    }), (0,
                    l.jsxs)("p", {
                        className: "text-body-s sm:text-body-m text-text-mid-em py-2",
                        children: ["Prime automatically optimizes your swap settings — including slippage and transaction landing — to deliver the best execution through Titan’s meta-aggregator.", (0,
                        l.jsxs)("span", {
                            className: "text-brand mt-3 flex items-center gap-x-2",
                            children: [(0,
                            l.jsx)(j.ee, {
                                className: "size-4"
                            }), (0,
                            l.jsx)("span", {
                                className: "text-body-s",
                                children: "All with zero swap fees."
                            })]
                        })]
                    })]
                }), (0,
                l.jsx)(tI, {
                    isToggleAll: a,
                    liquiditySourcesState: t,
                    onToggle: s
                })]
            })
        }
          , tD = () => {
            let {loading: e} = (0,
            e_.t0)()
              , {connected: t, connect: s} = (0,
            o.z)()
              , {baseTokenSlippage: a, onBaseTokenSlippageChange: n, broadCastMode: d, feeMode: c, onFeeModeChange: m, onBroadCastModeChange: x, priorityFee: u, onPriorityFeeChange: h, onMevSpeedChange: p, resetAll: f, isSettingsModalOpen: v, setIsSettingsModalOpen: y, stableTokenSlippage: N, onStableTokenSlippageChange: w, maxCap: A, onMaxCapChange: S, mevTip: M, onMevTipChange: T, priorityExactFee: E, onPriorityExactFeeChange: z, setIsPrimeMode: F, isPrimeMode: P, mevTipPercentile: L} = (0,
            ti.iD)()
              , {ammMap: R, excludedAmmIds: D, setExcludedAmmIds: I, includedAmmIds: X, setIncludedAmmIds: B, isLoading: O} = (0,
            tg.A)()
              , {excludedQuoteProviders: G, setExcludedQuoteProviders: q} = (0,
            eQ.A)()
              , {isJitoInactive: H} = eq()
              , {getTokenPrice: _} = (0,
            en.A)()
              , Q = _(k.wV)
              , J = e => {
                let t = parseFloat(e);
                return isNaN(t) || !Q || Q <= 0 ? "0" : (0,
                C.A)({
                    number: t * Q,
                    fixedDecimals: 2
                })
            }
              , [Y,Z] = (0,
            r.useState)( () => R)
              , [$,K] = (0,
            r.useState)(A)
              , [ee,et] = (0,
            r.useState)(M)
              , [ea,er] = (0,
            r.useState)(E)
              , [ei,eo] = (0,
            r.useState)(a)
              , [ed,ec] = (0,
            r.useState)(N)
              , [em,ex] = (0,
            r.useState)(P)
              , [eu,eh] = (0,
            r.useState)(d)
              , [ep,eg] = (0,
            r.useState)(c)
              , [eb,ef] = (0,
            r.useState)(u)
              , [ev,ey] = (0,
            r.useState)(L)
              , [eN,ew] = (0,
            r.useState)(!0)
              , [ek,eC] = (0,
            r.useState)(!1)
              , [eA,eS] = (0,
            r.useState)(G);
            (0,
            r.useEffect)( () => {
                Object.keys(R).length > 0 && Z(R)
            }
            , [R]),
            (0,
            r.useEffect)( () => {
                Object.keys(Y).length && (Object.values(Y).some(e => e.isExcluded) ? ew(!1) : ew(!0))
            }
            , [Y]),
            (0,
            r.useEffect)( () => {
                v && (Z(R),
                eo(a),
                ec(N),
                K(A),
                et(M),
                er(E),
                eh(d),
                eg(c),
                ef(u),
                ey(L),
                ex(P),
                eS(G))
            }
            , [v, R, a, N, A, M, E, d, c, u, L, P, D, X, G]),
            (0,
            r.useEffect)( () => {
                v || (Z(R),
                eo(a),
                ec(N),
                K(A),
                et(M),
                er(E),
                eh(d),
                eg(c),
                ef(u),
                ey(L),
                ex(P),
                eS(G))
            }
            , [R, a, N, A, M, E, d, c, u, L, P, v, G]);
            let eM = (0,
            r.useMemo)( () => O || 0 === Object.keys(Y).length ? [] : Object.entries(Y).map(e => {
                let[t,s] = e;
                return {
                    programId: t,
                    label: s.label,
                    image: "/images/sources/titan.png",
                    isActive: !s.isExcluded
                }
            }
            ), [Y, O])
              , eT = e => {
                if ("select-all" === e) {
                    let e = {
                        ...R
                    };
                    Object.keys(e).forEach(t => {
                        e[t] = {
                            ...e[t],
                            isExcluded: !!eN
                        }
                    }
                    ),
                    Z(e);
                    return
                }
                Z(t => {
                    let s = {
                        ...t
                    }
                      , l = s[e];
                    return l && (s[e] = {
                        ...l,
                        isExcluded: !l.isExcluded
                    }),
                    s
                }
                )
            }
            ;
            return (0,
            l.jsxs)(es.lG, {
                open: v,
                onOpenChange: e => {
                    e || (Z(R),
                    eo(a),
                    ec(N),
                    K(A),
                    et(M),
                    er(E),
                    eh(d),
                    eg(c),
                    ef(u),
                    ex(P),
                    eS(G)),
                    y(e)
                }
                ,
                children: [(0,
                l.jsx)(es.zM, {
                    asChild: !0,
                    children: (0,
                    l.jsx)("button", {
                        className: "focus-ring transition-all hover:cursor-pointer active:scale-95 disabled:cursor-default",
                        disabled: e,
                        onClick: t ? void 0 : e => {
                            e.preventDefault(),
                            s()
                        }
                        ,
                        onMouseEnter: () => eC(!0),
                        onMouseLeave: () => eC(!1),
                        onTouchEnd: () => eC(!1),
                        onTouchStart: () => eC(!0),
                        children: (0,
                        l.jsx)(tz.v, {
                            className: (0,
                            i.cn)("pr-3", {
                                "bg-grey-800": e,
                                "!border-brand": !e && em && ek
                            }),
                            isAnimated: !e && em && !ek,
                            leadingIcon: e ? (0,
                            l.jsx)(j.Nl, {
                                className: "size-3 animate-spin"
                            }) : em ? (0,
                            l.jsx)(j.Gg, {}) : (0,
                            l.jsx)(j.IE, {}),
                            trailingIcon: em && !e ? (0,
                            l.jsx)(j.IE, {}) : (0,
                            l.jsx)(l.Fragment, {}),
                            variant: e ? "default" : em ? "brand" : "default",
                            children: e ? (0,
                            l.jsx)(el, {
                                className: "text-body-xs sm:text-body-s",
                                children: "Loading"
                            }) : em ? "Prime" : "Manual"
                        })
                    })
                }), (0,
                l.jsxs)(es.Cf, {
                    preventDefaultDomBehavior: !0,
                    children: [(0,
                    l.jsx)(es.c7, {
                        closeBtnProps: {
                            className: "hidden"
                        },
                        children: (0,
                        l.jsxs)(es.L3, {
                            children: ["Swap Settings", (0,
                            l.jsx)(es.rr, {
                                className: "sr-only",
                                children: "Settings related to the swap"
                            })]
                        })
                    }), (0,
                    l.jsxs)(es.R4, {
                        children: [(0,
                        l.jsx)(V.I, {
                            fullWidth: !0,
                            className: "mb-4",
                            options: [{
                                label: "Prime",
                                value: "prime",
                                iconLeft: (0,
                                l.jsx)(j.Gg, {})
                            }, {
                                label: "Manual",
                                value: "manual"
                            }],
                            title: "swap-type",
                            value: em ? "prime" : "manual",
                            onValueChange: e => ex("prime" === e)
                        }), em ? (0,
                        l.jsx)(tR, {
                            liquiditySourcesState: eM,
                            onToggle: eT
                        }) : null, em && b.m1 ? (0,
                        l.jsx)(tO, {
                            excludedProviders: eA,
                            onChange: eS
                        }) : null, em ? null : (0,
                        l.jsxs)("div", {
                            className: "flex flex-col gap-4",
                            children: [(0,
                            l.jsxs)("div", {
                                children: [(0,
                                l.jsx)(tB, {
                                    icon: (0,
                                    l.jsx)(j.gq, {}),
                                    children: "Max Slippage"
                                }), (0,
                                l.jsx)(tG, {
                                    title: "Base tokens",
                                    tooltip: "Set max slippage tolerance for all swaps excluding stablecoins and liquid staked tokens.",
                                    children: (0,
                                    l.jsx)(U.p, {
                                        id: "base-tokens",
                                        inputMode: "decimal",
                                        placeholder: "0.0",
                                        rightContent: (0,
                                        l.jsx)("span", {
                                            className: "text-body-s",
                                            children: "%"
                                        }),
                                        value: ei,
                                        wrapperProps: {
                                            className: "max-w-[6.2rem]"
                                        },
                                        onChange: e => {
                                            let t = e.target.value;
                                            ("" === t || ej(t)) && eo(t)
                                        }
                                    })
                                }), (0,
                                l.jsx)(tG, {
                                    title: "Stable / LST",
                                    tooltip: "Set max slippage tolerance for stablecoin pairs or liquid staked token / SOL pairs.",
                                    children: (0,
                                    l.jsx)(U.p, {
                                        id: "stable-tokens",
                                        inputMode: "decimal",
                                        placeholder: "0.0",
                                        rightContent: (0,
                                        l.jsx)("span", {
                                            className: "text-body-s",
                                            children: "%"
                                        }),
                                        value: ed,
                                        wrapperProps: {
                                            className: "max-w-[6.2rem]"
                                        },
                                        onChange: e => {
                                            let t = e.target.value;
                                            ("" === t || ej(t)) && ec(t)
                                        }
                                    })
                                })]
                            }), (0,
                            l.jsxs)("div", {
                                children: [(0,
                                l.jsx)(tB, {
                                    icon: (0,
                                    l.jsx)(j.rI, {}),
                                    children: "Transaction Fees"
                                }), (0,
                                l.jsx)(tG, {
                                    title: "Broadcast Mode",
                                    children: (0,
                                    l.jsx)(V.I, {
                                        options: ti.pj,
                                        title: "broadcast-mode",
                                        value: eu,
                                        onValueChange: e => eh(e)
                                    })
                                }), (0,
                                l.jsx)(tG, {
                                    title: "Fee Mode",
                                    children: (0,
                                    l.jsx)(V.I, {
                                        options: ti.wn.map(e => ({
                                            ...e,
                                            disabled: "auto" === e.value && H && "mev-shield" === eu
                                        })),
                                        title: "fee-mode",
                                        value: ep,
                                        onValueChange: e => eg(e)
                                    })
                                }), H && "mev-shield" === eu && (0,
                                l.jsx)("div", {
                                    className: "flex flex-row justify-end pb-2",
                                    children: (0,
                                    l.jsxs)("div", {
                                        className: "flex flex-row items-center gap-2 px-2 py-0 pr-0",
                                        children: [(0,
                                        l.jsx)(j.ee, {
                                            className: "m-0 size-4 p-0 text-red-900"
                                        }), (0,
                                        l.jsx)("span", {
                                            className: "text-xs text-red-900",
                                            children: "Auto fee estimation unavailable"
                                        })]
                                    })
                                }), "auto" === ep && (0,
                                l.jsx)(tG, {
                                    title: "Speed",
                                    children: (0,
                                    l.jsx)(V.I, {
                                        options: ti.fk,
                                        title: "speed",
                                        value: "mev-shield" === eu ? tP(ev) : eb,
                                        onValueChange: e => {
                                            "mev-shield" === eu ? ey(tL(e)) : ef(e)
                                        }
                                    })
                                }), "priority-fee" === eu && "auto" === ep && (0,
                                l.jsx)(tG, {
                                    title: "Set Max Cap",
                                    children: (0,
                                    l.jsxs)("div", {
                                        className: "flex items-center gap-2",
                                        children: [(0,
                                        l.jsx)(U.p, {
                                            id: "max-cap",
                                            inputMode: "decimal",
                                            placeholder: "0.0",
                                            rightContent: (0,
                                            l.jsx)("span", {
                                                className: "text-body-s",
                                                children: "SOL"
                                            }),
                                            value: $,
                                            wrapperProps: {
                                                className: "max-w-[8.2rem] xs:max-w-[10.2rem]"
                                            },
                                            onChange: e => {
                                                let t = e.target.value;
                                                ("" === t || W(t)) && K(t)
                                            }
                                        }), (0,
                                        l.jsxs)("div", {
                                            className: "text-body-xs flex items-center gap-1",
                                            children: [(0,
                                            l.jsx)("span", {
                                                className: "text-text-lowest-em",
                                                children: "≈"
                                            }), (0,
                                            l.jsxs)("span", {
                                                className: "text-text-low-em",
                                                children: [J($), " USDC"]
                                            })]
                                        })]
                                    })
                                }), "priority-fee" === eu && "custom" === ep && (0,
                                l.jsxs)(l.Fragment, {
                                    children: [(0,
                                    l.jsx)(tG, {
                                        title: "Set Priority Fee",
                                        children: (0,
                                        l.jsxs)("div", {
                                            className: "flex items-center gap-2",
                                            children: [(0,
                                            l.jsx)(U.p, {
                                                id: "priority-fee",
                                                inputMode: "decimal",
                                                placeholder: "0.0",
                                                rightContent: (0,
                                                l.jsx)("span", {
                                                    className: "text-body-s",
                                                    children: "SOL"
                                                }),
                                                value: ea,
                                                wrapperProps: {
                                                    className: "max-w-[8.2rem] xs:max-w-[10.2rem]"
                                                },
                                                onChange: e => {
                                                    let t = e.target.value;
                                                    ("" === t || W(t)) && er(t)
                                                }
                                            }), (0,
                                            l.jsxs)("div", {
                                                className: "text-body-xs flex items-center gap-1",
                                                children: [(0,
                                                l.jsx)("span", {
                                                    className: "text-text-lowest-em",
                                                    children: "≈"
                                                }), (0,
                                                l.jsxs)("span", {
                                                    className: "text-text-low-em",
                                                    children: [J(ea), " USDC"]
                                                })]
                                            })]
                                        })
                                    }), (0,
                                    l.jsx)(tG, {
                                        title: "Set Max Cap",
                                        children: (0,
                                        l.jsxs)("div", {
                                            className: "flex items-center gap-2",
                                            children: [(0,
                                            l.jsx)(U.p, {
                                                id: "max-cap-custom",
                                                inputMode: "decimal",
                                                placeholder: "0.0",
                                                rightContent: (0,
                                                l.jsx)("span", {
                                                    className: "text-body-s",
                                                    children: "SOL"
                                                }),
                                                value: $,
                                                wrapperProps: {
                                                    className: "max-w-[8.2rem] xs:max-w-[10.2rem]"
                                                },
                                                onChange: e => {
                                                    let t = e.target.value;
                                                    ("" === t || W(t)) && K(t)
                                                }
                                            }), (0,
                                            l.jsxs)("div", {
                                                className: "text-body-xs flex items-center gap-1",
                                                children: [(0,
                                                l.jsx)("span", {
                                                    className: "text-text-lowest-em",
                                                    children: "≈"
                                                }), (0,
                                                l.jsxs)("span", {
                                                    className: "text-text-low-em",
                                                    children: [J($), " USDC"]
                                                })]
                                            })]
                                        })
                                    })]
                                }), "mev-shield" === eu && "auto" === ep && null, "mev-shield" === eu && "custom" === ep && (0,
                                l.jsx)(tG, {
                                    title: "Set MEV Tip",
                                    children: (0,
                                    l.jsxs)("div", {
                                        className: "flex items-center gap-2",
                                        children: [(0,
                                        l.jsx)(U.p, {
                                            id: "mev-tip",
                                            inputMode: "decimal",
                                            placeholder: "0.0",
                                            rightContent: (0,
                                            l.jsx)("span", {
                                                className: "text-body-s",
                                                children: "SOL"
                                            }),
                                            value: ee,
                                            wrapperProps: {
                                                className: "max-w-[8.2rem] xs:max-w-[10.2rem]"
                                            },
                                            onChange: e => {
                                                let t = e.target.value;
                                                ("" === t || W(t)) && et(t)
                                            }
                                        }), (0,
                                        l.jsxs)("div", {
                                            className: "text-body-xs flex items-center gap-1",
                                            children: [(0,
                                            l.jsx)("span", {
                                                className: "text-text-lowest-em",
                                                children: "≈"
                                            }), (0,
                                            l.jsxs)("span", {
                                                className: "text-text-low-em",
                                                children: [J(ee), " USDC"]
                                            })]
                                        })]
                                    })
                                })]
                            }), (0,
                            l.jsx)(tI, {
                                isLoading: O,
                                isToggleAll: eN,
                                liquiditySourcesState: eM,
                                onToggle: eT
                            }), b.m1 && (0,
                            l.jsx)(tO, {
                                excludedProviders: eA,
                                onChange: eS
                            }), (0,
                            l.jsx)(tU, {})]
                        })]
                    }), (0,
                    l.jsxs)(es.Es, {
                        children: [(0,
                        l.jsx)(g.$, {
                            size: "md",
                            variant: "ghost",
                            onClick: () => {
                                f(),
                                eo("1.5"),
                                ec("0.1"),
                                K("0.001"),
                                et("0"),
                                er("0"),
                                eh("priority-fee"),
                                eg("auto"),
                                ef("faster"),
                                ex(!0),
                                eS([]);
                                let e = {
                                    ...R
                                };
                                Object.keys(e).forEach(t => {
                                    e[t] = {
                                        ...e[t],
                                        isExcluded: !1
                                    }
                                }
                                ),
                                Z(e),
                                I([]),
                                B(Object.keys(R)),
                                q([]),
                                y(!1)
                            }
                            ,
                            children: "Reset all"
                        }), (0,
                        l.jsx)(g.$, {
                            size: "md",
                            variant: "primary",
                            onClick: () => {
                                n(ei),
                                w(ed),
                                S($),
                                T(ee),
                                z(ea),
                                x(eu),
                                m(ep),
                                "mev-shield" === eu && "auto" === ep ? p(tP(ev)) : h(eb),
                                I(Object.entries(Y).filter(e => {
                                    let[,t] = e;
                                    return t.isExcluded
                                }
                                ).map(e => {
                                    let[t] = e;
                                    return t
                                }
                                )),
                                B(Object.entries(Y).filter(e => {
                                    let[,t] = e;
                                    return !t.isExcluded
                                }
                                ).map(e => {
                                    let[t] = e;
                                    return t
                                }
                                )),
                                F(em),
                                q(eA),
                                y(!1)
                            }
                            ,
                            children: "Save"
                        })]
                    })]
                })]
            })
        }
          , tI = e => {
            let {liquiditySourcesState: t, onToggle: s, isToggleAll: a, isLoading: n=!1} = e
              , [o,d] = (0,
            r.useState)(!1)
              , c = (0,
            r.useRef)(null);
            return (0,
            r.useLayoutEffect)( () => {
                let e;
                return o && c.current && (e = requestAnimationFrame( () => {
                    let e = document.getElementById("liquidity-sources-list-item-0");
                    e && e.scrollIntoView({
                        behavior: "smooth",
                        block: "nearest",
                        inline: "center"
                    })
                }
                )),
                () => {
                    cancelAnimationFrame(e)
                }
            }
            , [o]),
            (0,
            l.jsxs)("div", {
                children: [(0,
                l.jsxs)("button", {
                    className: "focus-ring group group flex w-full cursor-pointer items-center justify-between gap-2",
                    onClick: () => {
                        d(e => !e),
                        o || setTimeout( () => {
                            let e = document.getElementById("liquidity-sources-list-item-0");
                            e && e.scrollIntoView({
                                behavior: "smooth",
                                inline: "center"
                            })
                        }
                        , 180)
                    }
                    ,
                    children: [(0,
                    l.jsx)(tB, {
                        icon: (0,
                        l.jsx)(j.Y$, {}),
                        children: "Manage Liquidity Sources"
                    }), (0,
                    l.jsx)("span", {
                        className: "group-hover:bg-grey-800 grid size-8 place-items-center rounded-full transition-colors",
                        children: (0,
                        l.jsx)(j.zp, {
                            className: (0,
                            i.cn)("size-4 transition-transform", {
                                "rotate-180": o
                            })
                        })
                    })]
                }), (0,
                l.jsx)("div", {
                    className: "pt-2",
                    children: (0,
                    l.jsx)(ep.N, {
                        initial: !1,
                        children: o && (0,
                        l.jsx)(Z.P.ul, {
                            ref: c,
                            animate: {
                                height: "auto"
                            },
                            className: "overflow-hidden",
                            exit: {
                                height: 0,
                                transition: {
                                    duration: .18
                                }
                            },
                            initial: {
                                height: 0
                            },
                            transition: {
                                duration: .25,
                                ease: [.26, .08, .25, 1]
                            },
                            children: n ? (0,
                            l.jsx)("li", {
                                className: "flex items-center justify-center py-4",
                                children: (0,
                                l.jsxs)("div", {
                                    className: "text-text-low-em flex items-center gap-2",
                                    children: [(0,
                                    l.jsx)(j.Nl, {
                                        className: "size-4 animate-spin"
                                    }), (0,
                                    l.jsx)("span", {
                                        className: "text-body-s",
                                        children: "Loading liquidity sources..."
                                    })]
                                })
                            }) : 0 === t.length ? (0,
                            l.jsx)("li", {
                                className: "flex items-center justify-center py-4",
                                children: (0,
                                l.jsx)("span", {
                                    className: "text-body-s text-text-low-em",
                                    children: "No liquidity sources available"
                                })
                            }) : (0,
                            l.jsxs)(l.Fragment, {
                                children: [(0,
                                l.jsx)("div", {
                                    className: "border-grey-600 mb-2 border-b pb-2.5",
                                    children: (0,
                                    l.jsx)(tX, {
                                        index: "select-all",
                                        source: {
                                            programId: "select-all",
                                            label: "Toggle All",
                                            image: "",
                                            isActive: a
                                        },
                                        onToggle: s
                                    })
                                }), t.map( (e, t) => (0,
                                l.jsx)(tX, {
                                    index: String(t),
                                    source: e,
                                    onToggle: s
                                }, t))]
                            })
                        })
                    })
                })]
            })
        }
          , tX = e => {
            let {index: t, source: s, onToggle: a} = e;
            return (0,
            l.jsxs)("li", {
                className: "flex items-center justify-between py-2 pr-1",
                id: "liquidity-sources-list-item-".concat(t),
                children: [(0,
                l.jsx)("label", {
                    className: "flex flex-1 cursor-pointer items-center gap-3 pr-2",
                    htmlFor: s.label,
                    children: (0,
                    l.jsx)("span", {
                        className: "text-body-s font-medium",
                        children: s.label
                    })
                }), (0,
                l.jsx)(tw.A, {
                    checked: s.isActive,
                    id: s.label,
                    onCheckedChange: () => a(s.programId)
                })]
            }, s.label)
        }
          , tB = e => {
            let {icon: t, children: s} = e;
            return (0,
            l.jsxs)("h3", {
                className: "text-heading-xxs font-brand mb-1 flex items-center gap-2 leading-[1.5rem]",
                children: [(0,
                l.jsx)("span", {
                    className: "text-brand",
                    children: t
                }), s]
            })
        }
          , tO = e => {
            let {excludedProviders: t, onChange: s} = e
              , a = (0,
            r.useMemo)( () => ["Titan", "Jupiter", "Hashflow", "DFlow", "OKX", "Pyth"], [])
              , n = e => t.includes(e)
              , o = e => {
                s(n(e) ? t.filter(t => t !== e) : [...t, e])
            }
              , [d,c] = (0,
            r.useState)(!1)
              , m = (0,
            r.useRef)(null);
            return (0,
            r.useLayoutEffect)( () => {
                let e;
                return d && m.current && (e = requestAnimationFrame( () => {
                    let e = document.getElementById("provider-filter-item-0");
                    e && e.scrollIntoView({
                        behavior: "smooth",
                        inline: "center"
                    })
                }
                )),
                () => cancelAnimationFrame(e)
            }
            , [d]),
            (0,
            l.jsxs)("div", {
                className: "mt-2",
                children: [(0,
                l.jsxs)("button", {
                    className: "focus-ring group group flex w-full cursor-pointer items-center justify-between gap-2",
                    onClick: () => {
                        c(e => !e),
                        d || setTimeout( () => {
                            let e = document.getElementById("provider-filter-item-0");
                            e && e.scrollIntoView({
                                behavior: "smooth",
                                inline: "center"
                            })
                        }
                        , 180)
                    }
                    ,
                    children: [(0,
                    l.jsx)(tB, {
                        icon: (0,
                        l.jsx)(j.ee, {}),
                        children: "Provider Filters"
                    }), (0,
                    l.jsx)("span", {
                        className: "group-hover:bg-grey-800 grid size-8 place-items-center rounded-full transition-colors",
                        children: (0,
                        l.jsx)(j.zp, {
                            className: (0,
                            i.cn)("size-4 transition-transform", {
                                "rotate-180": d
                            })
                        })
                    })]
                }), (0,
                l.jsx)("div", {
                    className: "pt-2",
                    children: (0,
                    l.jsx)(ep.N, {
                        initial: !1,
                        children: d && (0,
                        l.jsxs)(Z.P.ul, {
                            ref: m,
                            animate: {
                                height: "auto"
                            },
                            className: "overflow-hidden",
                            exit: {
                                height: 0,
                                transition: {
                                    duration: .18
                                }
                            },
                            initial: {
                                height: 0
                            },
                            transition: {
                                duration: .25,
                                ease: [.26, .08, .25, 1]
                            },
                            children: [(0,
                            l.jsx)("li", {
                                className: "text-body-xs text-text-low-em mb-1 px-0 pt-0",
                                children: "Hide quotes from selected providers (internal only)"
                            }), a.map( (e, t) => (0,
                            l.jsxs)("li", {
                                className: "flex items-center justify-between py-1 pr-1",
                                id: "provider-filter-item-".concat(t),
                                children: [(0,
                                l.jsx)("label", {
                                    className: "flex flex-1 cursor-pointer items-center gap-3 pr-2",
                                    htmlFor: "provider-filter-".concat(t),
                                    children: (0,
                                    l.jsx)("span", {
                                        className: "text-body-s font-medium",
                                        children: e
                                    })
                                }), (0,
                                l.jsx)(tw.A, {
                                    checked: !n(e),
                                    id: "provider-filter-".concat(t),
                                    onCheckedChange: () => o(e)
                                })]
                            }, e))]
                        })
                    })
                })]
            })
        }
          , tG = e => {
            let {title: t, tooltip: s, children: a} = e;
            return (0,
            l.jsxs)("div", {
                className: "flex items-center justify-between gap-2 py-2",
                children: [(0,
                l.jsxs)("div", {
                    className: "text-body-s sm:text-body-m flex items-center gap-2 font-medium",
                    onFocusCapture: e => {
                        e.stopPropagation()
                    }
                    ,
                    children: [t, s && (0,
                    l.jsx)(ea.Bc, {
                        children: (0,
                        l.jsxs)(ea.m_, {
                            children: [(0,
                            l.jsx)(ea.k$, {
                                onPointerDown: e => e.preventDefault(),
                                children: (0,
                                l.jsx)(j.ee, {
                                    className: "size-4"
                                })
                            }), (0,
                            l.jsx)(ea.ZI, {
                                className: "max-w-[13.5rem]",
                                children: s
                            })]
                        })
                    })]
                }), a]
            })
        }
          , tU = () => {
            let {jitoStatus: e, isJitoActive: t, isJitoInactive: s, lastUpdatedTimestamp: a} = eq();
            return (0,
            l.jsxs)("div", {
                className: "border-grey-600 mt-2 border-t pt-3",
                children: [(0,
                l.jsxs)("div", {
                    className: "flex items-center justify-between",
                    children: [(0,
                    l.jsxs)("div", {
                        className: "text-body-xs text-text-low-em flex items-center gap-2",
                        children: [(0,
                        l.jsx)("span", {
                            children: "MEV Protection"
                        }), (0,
                        l.jsx)(ea.Bc, {
                            children: (0,
                            l.jsxs)(ea.m_, {
                                children: [(0,
                                l.jsx)(ea.k$, {
                                    onPointerDown: e => e.preventDefault(),
                                    children: (0,
                                    l.jsx)(j.ee, {
                                        className: "size-3"
                                    })
                                }), (0,
                                l.jsx)(ea.ZI, {
                                    className: "max-w-[13.5rem]",
                                    children: "Real-time status of Jito MEV protection infrastructure. When inactive, auto fee mode may be disabled."
                                })]
                            })
                        })]
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-2",
                        children: [(0,
                        l.jsx)("div", {
                            className: (0,
                            i.cn)("h-1.5 w-1.5 rounded-full transition-colors duration-200", t ? "bg-green-500" : s ? "bg-red-500" : "bg-yellow-500"),
                            title: "MEV Protection: ".concat(e)
                        }), (0,
                        l.jsx)("span", {
                            className: "text-body-xs text-text-med-em",
                            children: t ? "Active" : s ? "Inactive" : "Unknown"
                        })]
                    })]
                }), a && (0,
                l.jsx)("div", {
                    className: "mt-1 flex justify-end",
                    children: (0,
                    l.jsxs)("span", {
                        className: "text-body-xs text-text-lowest-em",
                        children: ["Updated ", a.toLocaleTimeString()]
                    })
                })]
            })
        }
          , tV = () => {
            let {view: e, setView: t} = (0,
            eC.u)()
              , {isRefreshing: s, setSellAmount: a, setReceiveAmount: r} = (0,
            eb.j)()
              , {setBuyValue: n, setSellValue: o, setRateValue: d} = (0,
            p.A)()
              , c = () => {
                a(""),
                r(""),
                n(""),
                o(""),
                d("")
            }
            ;
            return (0,
            l.jsxs)("section", {
                className: "mx-auto w-full max-w-[33.75rem]",
                children: [(0,
                l.jsx)(u, {}), (0,
                l.jsxs)(h.tU, {
                    size: "sm",
                    value: e,
                    onValueChange: e => {
                        t(e),
                        c()
                    }
                    ,
                    children: [(0,
                    l.jsxs)("div", {
                        className: "flex flex-wrap items-center justify-between gap-3",
                        children: [(0,
                        l.jsxs)(h.j7, {
                            children: [(0,
                            l.jsx)(h.Xi, {
                                className: (0,
                                i.cn)("disabled:cursor-not-allowed", "disabled:pointer-events-auto", "disabled:text-text-mid-em", "disabled:data-[state=active]:text-text-high-em", "disabled:data-[state=active]:bg-bg-mid-em"),
                                disabled: s,
                                title: s ? "Refreshing please wait..." : "Instant",
                                value: "instant",
                                children: (0,
                                l.jsx)("span", {
                                    className: "text-body-s xs:text-body-m",
                                    children: "Instant"
                                })
                            }), (0,
                            l.jsx)(h.Xi, {
                                className: (0,
                                i.cn)("disabled:cursor-not-allowed", "disabled:pointer-events-auto", "disabled:text-text-mid-em", "disabled:data-[state=active]:text-text-high-em", "disabled:data-[state=active]:bg-bg-mid-em"),
                                disabled: s,
                                title: s ? "Refreshing please wait..." : "Limit",
                                value: "limit",
                                children: (0,
                                l.jsx)("span", {
                                    className: "text-body-s xs:text-body-m",
                                    children: "Limit"
                                })
                            })]
                        }), (0,
                        l.jsxs)("div", {
                            className: "xs:gap-2 flex items-center gap-1",
                            children: ["instant" === e && (0,
                            l.jsxs)(l.Fragment, {
                                children: [(0,
                                l.jsx)(tD, {}), (0,
                                l.jsx)(tA, {})]
                            }), (0,
                            l.jsx)(v, {}), "instant" === e && (0,
                            l.jsx)(tE, {})]
                        })]
                    }), (0,
                    l.jsxs)("div", {
                        children: [(0,
                        l.jsx)(h.av, {
                            forceMount: !0,
                            value: "instant",
                            children: "instant" === e && (0,
                            l.jsx)(tT, {
                                children: (0,
                                l.jsx)(tN, {})
                            })
                        }), (0,
                        l.jsx)(h.av, {
                            forceMount: !0,
                            value: "limit",
                            children: "limit" === e && (0,
                            l.jsx)(eL, {})
                        }), (0,
                        l.jsx)(h.av, {
                            forceMount: !0,
                            value: "dca",
                            children: "dca" === e && (0,
                            l.jsx)(eR, {
                                isComingSoon: !0
                            })
                        })]
                    })]
                })]
            })
        }
    }
    ,
    54206: (e, t, s) => {
        s.d(t, {
            A: () => b
        });
        var l = s(48876)
          , a = s(81177)
          , r = s(30369)
          , n = s(26432)
          , i = s(93749)
          , o = s(75412)
          , d = s(90933)
          , c = s(8626)
          , m = s(77284)
          , x = s(15653)
          , u = s(52630)
          , h = s(51092)
          , p = s(65993)
          , g = s(73861)
          , j = s(52905);
        let b = e => {
            let {openOrders: t} = e
              , {allTokens: s} = (0,
            g.A)()
              , {isDeleting: b, selectedDeletedOrder: y, setSelectedDeletedOrder: N, handleDeleteOrder: w} = (0,
            p.A)()
              , [k,C] = (0,
            n.useState)(null);
            if (!t.length)
                return (0,
                l.jsx)(n.Fragment, {});
            let A = t.sort( (e, t) => t.createdAt - e.createdAt).slice(0, 4);
            return (0,
            l.jsx)(o._, {
                colGroup: (0,
                l.jsx)(v, {}),
                columns: f,
                headerHasBackground: !1,
                thProps: {
                    className: "text-body-xs px-2 pr-0 py-0 text-text-lowest-em"
                },
                wrapperProps: {
                    className: "min-w-[320px]"
                },
                children: A.map( (e, t) => {
                    let n, o, p = k === (null == e ? void 0 : e.address), g = s.find(t => t.address === e.inputMint), f = s.find(t => t.address === e.outputMint), v = (null == g ? void 0 : g.decimals) ? (0,
                    h.A)({
                        number: new r.A(e.amount).div(new r.A(10).pow(null == g ? void 0 : g.decimals)).toNumber()
                    }) : e.amount, A = (null == g ? void 0 : g.decimals) ? (0,
                    h.A)({
                        number: new r.A(e.amountFilled).div(new r.A(10).pow(null == g ? void 0 : g.decimals)).toNumber()
                    }) : e.amountFilled, S = Number.MAX_SAFE_INTEGER, M = e.expiresAt >= S - 1e6;
                    M ? (n = "Never",
                    o = "expires") : (n = (0,
                    a.GP)(new Date(1e3 * e.expiresAt), "MMM dd"),
                    o = (0,
                    a.GP)(new Date(1e3 * e.expiresAt), "HH:mm:ss"));
                    let T = "".concat(A, "/").concat(v);
                    return (0,
                    l.jsxs)(d.A, {
                        className: (0,
                        u.cn)("relative overflow-hidden", "h-10 [&_td]:py-2 [&_td]:pl-2", "text-body-xxs md:text-body-xs rounded-lg font-medium", k && k === (null == e ? void 0 : e.address) && "bg-bg-low-em"),
                        children: [(0,
                        l.jsx)("td", {
                            className: "rounded-tl-lg rounded-bl-lg",
                            children: (0,
                            l.jsx)(j.A, {
                                buyTokenLogoURI: null == f ? void 0 : f.logoURI,
                                buyTokenSymbol: null == f ? void 0 : f.symbol,
                                sellTokenLogoURI: null == g ? void 0 : g.logoURI,
                                sellTokenSymbol: null == g ? void 0 : g.symbol
                            })
                        }), (0,
                        l.jsx)("td", {
                            children: (0,
                            l.jsx)("p", {
                                children: (0,
                                h.A)({
                                    number: null == e ? void 0 : e.limitPrice,
                                    options: {
                                        maximumFractionDigits: 4
                                    }
                                })
                            })
                        }), (0,
                        l.jsx)("td", {
                            children: (0,
                            l.jsx)("p", {
                                children: T
                            })
                        }), (0,
                        l.jsx)("td", {
                            children: (0,
                            l.jsxs)("div", {
                                className: "flex flex-col gap-x-1 sm:flex-row sm:items-center",
                                children: [(0,
                                l.jsx)("span", {
                                    children: n
                                }), (0,
                                l.jsx)("span", {
                                    className: M ? "text-text-high-em hidden" : "text-text-low-em",
                                    children: o
                                })]
                            })
                        }), (0,
                        l.jsx)("td", {
                            className: "rounded-tr-lg rounded-br-lg",
                            children: (0,
                            l.jsxs)(m.Q, {
                                className: "relative z-10 pr-3",
                                disabled: b,
                                size: "xs",
                                onClick: () => N((null == e ? void 0 : e.address) || ""),
                                children: [(0,
                                l.jsx)(c.uv, {
                                    className: "text-text-high-em size-4"
                                }), " "]
                            })
                        }), (0,
                        l.jsx)("td", {
                            className: "absolute top-0 left-0 h-full w-full !p-0",
                            children: (0,
                            l.jsx)(x.Bc, {
                                delayDuration: 100,
                                children: (0,
                                l.jsx)(x.m_, {
                                    open: p,
                                    onOpenChange: t => C(t ? e.address : k === e.address ? null : k),
                                    children: (0,
                                    l.jsx)(x.k$, {
                                        asChild: !0,
                                        children: (0,
                                        l.jsx)("button", {
                                            className: "absolute top-0 left-0 h-full w-full",
                                            children: (0,
                                            l.jsx)(x.ZI, {
                                                className: "flex max-w-[131px] min-w-fit items-center justify-center",
                                                side: "top",
                                                sideOffset: -5,
                                                children: (0,
                                                l.jsx)("a", {
                                                    className: "text-body-s hover:underline",
                                                    href: "https://solscan.io/account/".concat(e.address),
                                                    target: "_blank",
                                                    children: "Open in Solscan"
                                                })
                                            })
                                        })
                                    })
                                })
                            })
                        }), y && y === (null == e ? void 0 : e.address) && (0,
                        l.jsxs)("td", {
                            className: (0,
                            u.cn)("h-full w-full !p-0", "absolute top-0 left-0 z-30"),
                            children: [(0,
                            l.jsx)("div", {
                                className: "absolute inset-0 h-full w-full bg-black/1 backdrop-blur-[2px]"
                            }), (0,
                            l.jsxs)("div", {
                                className: "relative z-30 flex h-full w-full items-center justify-end gap-x-1",
                                children: [(0,
                                l.jsx)(i.$, {
                                    disabled: b,
                                    size: "xs",
                                    variant: "tertiary",
                                    onClick: () => w(),
                                    children: b ? "Canceling..." : "Yes, cancel"
                                }), (0,
                                l.jsx)(i.$, {
                                    disabled: b,
                                    size: "xs",
                                    variant: "secondary",
                                    onClick: () => N(""),
                                    children: "No, keep it"
                                })]
                            })]
                        })]
                    }, t)
                }
                )
            })
        }
          , f = [{
            label: "Transaction",
            key: "transaction"
        }, {
            label: "Limit price",
            key: "limitPrice"
        }, {
            label: "Amount",
            key: "amount"
        }, {
            label: "Expiry",
            key: "expiry"
        }, {
            label: "",
            key: "action"
        }]
          , v = () => (0,
        l.jsxs)("colgroup", {
            children: [(0,
            l.jsx)("col", {
                className: "w-[30%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[23%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[23%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[23%]"
            }), (0,
            l.jsx)("col", {
                className: "w-4"
            })]
        })
    }
    ,
    54248: (e, t, s) => {
        s.d(t, {
            A: () => D
        });
        var l = s(48876)
          , a = s(26432)
          , r = s(6008)
          , n = s(38079)
          , i = s(77284)
          , o = s(81177)
          , d = s(30369)
          , c = s(93749)
          , m = s(75412)
          , x = s(90933)
          , u = s(8626)
          , h = s(5379)
          , p = s(52630)
          , g = s(51092)
          , j = s(65993)
          , b = s(93355)
          , f = s(73861)
          , v = s(52905)
          , y = s(32606)
          , N = s(40401);
        let w = () => {
            let {allTokens: e} = (0,
            f.A)()
              , {openOrders: t, expiredOrders: s} = (0,
            b.A)()
              , {isDeleting: r, selectedDeletedOrder: n, setSelectedDeletedOrder: i, handleDeleteOrder: w} = (0,
            j.A)()
              , [A,S] = (0,
            a.useState)(null)
              , [M,T] = (0,
            a.useState)(1)
              , E = (M - 1) * h.re
              , z = (0,
            a.useMemo)( () => [...t, ...s], [t, s])
              , F = (0,
            a.useMemo)( () => 0 === z.length ? [] : A ? z.sort( (e, t) => (null == A ? void 0 : A.key) && (null == A ? void 0 : A.direction) === "desc" ? (t[null == A ? void 0 : A.key] || 0) - (e[null == A ? void 0 : A.key] || 0) : (e[null == A ? void 0 : A.key] || 0) - (t[null == A ? void 0 : A.key] || 0)) : z, [A, z])
              , P = (0,
            a.useMemo)( () => F && (null == F ? void 0 : F.slice(E, E + h.re)) || [], [F, E]);
            return (0,
            l.jsxs)("div", {
                className: "flex w-full flex-col gap-y-5",
                children: [(0,
                l.jsx)(m._, {
                    colGroup: (0,
                    l.jsx)(C, {}),
                    columns: k,
                    headerHasBackground: !1,
                    thProps: {
                        className: "text-body-xs sm:text-body-s px-2 pr-0 py-0 text-text-lowest-em [&_div_button]:text-text-lowest-em"
                    },
                    wrapperProps: {
                        className: "min-w-[560px]"
                    },
                    onSort: (e, t) => {
                        S({
                            key: e,
                            direction: t
                        })
                    }
                    ,
                    children: P.map( (t, s) => {
                        let a = e.find(e => e.address === t.inputMint)
                          , m = e.find(e => e.address === t.outputMint)
                          , h = (null == a ? void 0 : a.decimals) ? (0,
                        g.A)({
                            number: new d.A(t.amount).div(new d.A(10).pow(null == a ? void 0 : a.decimals)).toNumber()
                        }) : t.amount
                          , j = (null == a ? void 0 : a.decimals) ? (0,
                        g.A)({
                            number: new d.A(t.amountFilled).div(new d.A(10).pow(null == a ? void 0 : a.decimals)).toNumber()
                        }) : t.amountFilled
                          , {label: b, color: f, background: N} = (0,
                        y.C)(t.status, t.expiresAt);
                        return (0,
                        l.jsxs)(x.A, {
                            className: (0,
                            p.cn)("text-body-s font-medium", "relative overflow-hidden", "h-15.5 [&_td]:py-2 [&_td]:pl-2"),
                            children: [(0,
                            l.jsx)("td", {
                                children: (0,
                                l.jsx)(v.A, {
                                    buyTokenLogoURI: null == m ? void 0 : m.logoURI,
                                    buyTokenSymbol: null == m ? void 0 : m.symbol,
                                    className: "text-body-s",
                                    sellTokenLogoURI: null == a ? void 0 : a.logoURI,
                                    sellTokenSymbol: null == a ? void 0 : a.symbol,
                                    size: 24
                                })
                            }), (0,
                            l.jsx)("td", {
                                children: (0,
                                l.jsx)("p", {
                                    children: (0,
                                    g.A)({
                                        number: null == t ? void 0 : t.limitPrice,
                                        options: {
                                            maximumFractionDigits: 2
                                        }
                                    })
                                })
                            }), (0,
                            l.jsx)("td", {
                                children: (0,
                                l.jsxs)("p", {
                                    children: [j, "/", h]
                                })
                            }), (0,
                            l.jsx)("td", {
                                children: (0,
                                l.jsxs)("div", {
                                    className: "flex flex-col gap-x-1 sm:flex-row sm:items-center",
                                    children: [(0,
                                    l.jsx)("span", {
                                        children: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "Never" : (0,
                                        o.GP)(new Date(1e3 * t.expiresAt), "MMM dd")
                                    }), (0,
                                    l.jsx)("span", {
                                        className: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "text-text-mid-em" : "text-text-low-em",
                                        children: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "expires" : (0,
                                        o.GP)(new Date(1e3 * t.expiresAt), "HH:mm:ss")
                                    })]
                                })
                            }), (0,
                            l.jsx)("td", {
                                children: (0,
                                l.jsxs)("div", {
                                    className: "flex w-full items-center justify-end gap-x-1",
                                    children: [(0,
                                    l.jsx)("span", {
                                        className: (0,
                                        p.cn)("px-2 py-0.5", "text-body-s rounded-md", f, N),
                                        children: b
                                    }), (0,
                                    l.jsx)(c.$, {
                                        className: "relative z-10 h-8 w-8 p-0",
                                        disabled: r,
                                        size: "xs",
                                        variant: "ghost",
                                        onClick: () => {
                                            i(n === t.address ? "" : t.address)
                                        }
                                        ,
                                        children: (0,
                                        l.jsx)(u.uv, {
                                            className: "text-text-high-em size-4"
                                        })
                                    })]
                                })
                            }), n && n === (null == t ? void 0 : t.address) && (0,
                            l.jsxs)("td", {
                                className: (0,
                                p.cn)("h-full w-full !p-0", "absolute top-0 left-0 z-30"),
                                children: [(0,
                                l.jsx)("div", {
                                    className: "absolute inset-0 h-full w-full bg-black/1 backdrop-blur-[2px]"
                                }), (0,
                                l.jsxs)("div", {
                                    className: "relative z-30 flex h-full w-full items-center justify-end gap-x-1",
                                    children: [(0,
                                    l.jsx)(c.$, {
                                        disabled: r,
                                        size: "xs",
                                        variant: "tertiary",
                                        onClick: () => w(),
                                        children: r ? "Canceling..." : "Yes, cancel"
                                    }), (0,
                                    l.jsx)(c.$, {
                                        disabled: r,
                                        size: "xs",
                                        variant: "secondary",
                                        onClick: () => i(""),
                                        children: "No, keep it"
                                    })]
                                })]
                            })]
                        }, s)
                    }
                    )
                }), (0,
                l.jsx)(N.d, {
                    className: "self-end",
                    page: M,
                    pageSize: h.re,
                    total: F.length,
                    onPageChange: e => T(e)
                })]
            })
        }
          , k = [{
            label: "Transaction",
            key: "transaction"
        }, {
            label: "Limit price",
            key: "limitPrice",
            sortable: !0
        }, {
            label: "Amount",
            key: "amount",
            sortable: !0
        }, {
            label: "Time",
            key: "time",
            sortable: !0
        }, {
            label: "Status",
            key: "status"
        }]
          , C = () => (0,
        l.jsxs)("colgroup", {
            children: [(0,
            l.jsx)("col", {
                className: "w-[25%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[25%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[30%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[10%]"
            })]
        })
          , A = e => {
            let {label: t, contentData: s, className: a, ...r} = e;
            return (0,
            l.jsxs)("div", {
                className: (0,
                p.cn)("flex flex-col gap-y-1", "text-body-xs text-text-low-em", a),
                ...r,
                children: [s || (0,
                l.jsx)("p", {
                    children: "-"
                }), (0,
                l.jsx)("p", {
                    children: t
                })]
            })
        }
        ;
        var S = s(51816)
          , M = s(27442);
        let T = e => {
            let {open: t, children: s} = e;
            return (0,
            l.jsx)(S.N, {
                children: (0,
                l.jsx)("tr", {
                    className: "relative overflow-hidden",
                    children: (0,
                    l.jsx)("td", {
                        className: "p-0",
                        colSpan: 6,
                        children: (0,
                        l.jsx)(M.P.div, {
                            animate: {
                                height: t ? "auto" : 0
                            },
                            className: "w-full overflow-hidden",
                            exit: {
                                height: 0,
                                transition: {
                                    duration: .18
                                }
                            },
                            initial: {
                                height: 0
                            },
                            transition: {
                                duration: .25,
                                ease: [.26, .08, .25, 1]
                            },
                            children: (0,
                            l.jsx)("div", {
                                className: "w-full",
                                children: s
                            })
                        })
                    })
                })
            })
        }
        ;
        var E = s(70734)
          , z = s(19646);
        let F = e => {
            let {txHash: t, className: s, ...a} = e;
            if (t)
                return (0,
                l.jsxs)("div", {
                    className: (0,
                    p.cn)("flex items-center gap-x-1", s),
                    ...a,
                    children: [(0,
                    l.jsx)("p", {
                        className: "font-medium text-white",
                        children: (0,
                        z.lV)(t)
                    }), (0,
                    l.jsx)(E.i, {
                        className: "text-text-low-em",
                        copyContent: t,
                        iconClassName: "size-4"
                    })]
                })
        }
          , P = () => {
            let {allTokens: e} = (0,
            f.A)()
              , {openOrders: t, isLoadingOpenOrder: s, errorFetchingOpenOrders: r} = (0,
            b.A)()
              , {isDeleting: n, selectedDeletedOrder: i, setSelectedDeletedOrder: y, handleDeleteOrder: w} = (0,
            j.A)()
              , [k,C] = (0,
            a.useState)(null)
              , [S,M] = (0,
            a.useState)(null)
              , [E,z] = (0,
            a.useState)(1)
              , P = (E - 1) * h.re
              , D = (0,
            a.useMemo)( () => 0 === t.length ? [] : k ? t.sort( (e, t) => {
                let s, l;
                return ((null == k ? void 0 : k.key) === "expiry" ? (s = e.expiresAt,
                l = t.expiresAt) : (s = e[null == k ? void 0 : k.key] || 0,
                l = t[null == k ? void 0 : k.key] || 0),
                (null == k ? void 0 : k.direction) === "desc") ? l - s : s - l
            }
            ) : t, [t, k])
              , I = (0,
            a.useMemo)( () => D && (null == D ? void 0 : D.slice(P, P + h.re)) || [], [D, P]);
            return s ? (0,
            l.jsx)("div", {
                className: "flex items-center justify-center py-8",
                children: (0,
                l.jsx)("div", {
                    className: "text-text-low-em",
                    children: "Loading orders..."
                })
            }) : r ? (0,
            l.jsx)("div", {
                className: "flex items-center justify-center py-8",
                children: (0,
                l.jsx)("div", {
                    className: "text-text-low-em",
                    children: "Failed to load orders"
                })
            }) : 0 === D.length ? (0,
            l.jsx)("div", {
                className: "flex items-center justify-center py-8",
                children: (0,
                l.jsx)("div", {
                    className: "text-text-low-em",
                    children: "No open orders found"
                })
            }) : (0,
            l.jsxs)("div", {
                className: "flex w-full flex-col gap-y-5",
                children: [(0,
                l.jsx)(m._, {
                    colGroup: (0,
                    l.jsx)(R, {}),
                    columns: L,
                    headerHasBackground: !1,
                    thProps: {
                        className: "text-body-xs sm:text-body-s px-2 pr-0 py-0 text-text-lowest-em [&_div_button]:text-text-lowest-em"
                    },
                    wrapperProps: {
                        className: "min-w-[600px]"
                    },
                    onSort: (e, t) => {
                        C({
                            key: e,
                            direction: t
                        })
                    }
                    ,
                    children: I.map( (t, s) => {
                        let r = e.find(e => e.address === t.inputMint)
                          , m = e.find(e => e.address === t.outputMint)
                          , h = (null == r ? void 0 : r.decimals) ? (0,
                        g.A)({
                            number: new d.A(t.amount).div(new d.A(10).pow(null == r ? void 0 : r.decimals)).toNumber()
                        }) : t.amount
                          , j = (null == r ? void 0 : r.decimals) ? (0,
                        g.A)({
                            number: new d.A(t.amountFilled).div(new d.A(10).pow(null == r ? void 0 : r.decimals)).toNumber()
                        }) : t.amountFilled;
                        return (0,
                        l.jsxs)(a.Fragment, {
                            children: [(0,
                            l.jsxs)(x.A, {
                                className: (0,
                                p.cn)("text-body-s font-medium", "relative overflow-hidden", "h-15.5 [&_td]:py-2 [&_td]:pl-2", S === s && "border-transparent"),
                                children: [(0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsx)(v.A, {
                                        buyTokenLogoURI: null == m ? void 0 : m.logoURI,
                                        buyTokenSymbol: null == m ? void 0 : m.symbol,
                                        className: "text-body-s",
                                        sellTokenLogoURI: null == r ? void 0 : r.logoURI,
                                        sellTokenSymbol: null == r ? void 0 : r.symbol,
                                        size: 24
                                    })
                                }), (0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsx)("p", {
                                        children: (0,
                                        g.A)({
                                            number: null == t ? void 0 : t.limitPrice,
                                            options: {
                                                maximumFractionDigits: 2
                                            }
                                        })
                                    })
                                }), (0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsx)("p", {
                                        children: "".concat(j, "/").concat(h)
                                    })
                                }), (0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsxs)("div", {
                                        className: "flex flex-col gap-x-1 sm:flex-row sm:items-center",
                                        children: [(0,
                                        l.jsx)("span", {
                                            children: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "Never" : (0,
                                            o.GP)(new Date(1e3 * t.expiresAt), "MMM dd")
                                        }), (0,
                                        l.jsx)("span", {
                                            className: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "text-text-mid-em" : "text-text-low-em",
                                            children: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "expires" : (0,
                                            o.GP)(new Date(1e3 * t.expiresAt), "HH:mm:ss")
                                        })]
                                    })
                                }), (0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsx)("div", {
                                        className: "flex justify-end",
                                        children: (0,
                                        l.jsx)(c.$, {
                                            className: "relative z-10 h-8 w-8 p-0",
                                            disabled: n,
                                            size: "xs",
                                            variant: "ghost",
                                            onClick: () => {
                                                M(null),
                                                y(i === t.address ? "" : t.address)
                                            }
                                            ,
                                            children: (0,
                                            l.jsx)(u.uv, {
                                                className: "text-text-high-em size-4"
                                            })
                                        })
                                    })
                                }), (0,
                                l.jsx)("td", {
                                    children: (0,
                                    l.jsx)(c.$, {
                                        className: "h-8 w-8 p-0",
                                        variant: "ghost",
                                        onClick: () => M(S === s ? null : s),
                                        children: (0,
                                        l.jsx)(u.zp, {
                                            className: (0,
                                            p.cn)("size-4 transition-transform", S === s && "rotate-180")
                                        })
                                    })
                                }), i && i === (null == t ? void 0 : t.address) && (0,
                                l.jsxs)("td", {
                                    className: (0,
                                    p.cn)("h-full w-full !p-0", "absolute top-0 left-0 z-30"),
                                    children: [(0,
                                    l.jsx)("div", {
                                        className: "absolute inset-0 h-full w-full bg-black/1 backdrop-blur-[2px]"
                                    }), (0,
                                    l.jsxs)("div", {
                                        className: "relative z-30 flex h-full w-full items-center justify-end gap-x-1",
                                        children: [(0,
                                        l.jsx)(c.$, {
                                            disabled: n,
                                            size: "xs",
                                            variant: "tertiary",
                                            onClick: () => w(),
                                            children: n ? "Canceling..." : "Yes, cancel"
                                        }), (0,
                                        l.jsx)(c.$, {
                                            disabled: n,
                                            size: "xs",
                                            variant: "secondary",
                                            onClick: () => y(""),
                                            children: "No, keep it"
                                        })]
                                    })]
                                })]
                            }), (0,
                            l.jsx)(T, {
                                open: S === s,
                                children: (0,
                                l.jsxs)("div", {
                                    className: (0,
                                    p.cn)("bg-bg-mid-em w-full", "rounded-lg px-2 py-3", "flex flex-col gap-y-4"),
                                    children: [(0,
                                    l.jsxs)("div", {
                                        className: "grid grid-cols-4",
                                        children: [(0,
                                        l.jsx)(A, {
                                            contentData: t.timeInForce,
                                            label: "Time in Force"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: (0,
                                            o.GP)(new Date(1e3 * t.createdAt), "MMM dd, yyyy HH:mm:ss"),
                                            label: "Order creation"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: (0,
                                            o.GP)(new Date(1e3 * t.createdAt), "MMM dd, yyyy HH:mm:ss"),
                                            label: "Last order open date time"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: t.expiresAt >= Number.MAX_SAFE_INTEGER - 1e6 ? "Never expires" : (0,
                                            o.GP)(new Date(1e3 * t.expiresAt), "MMM dd, yyyy HH:mm:ss"),
                                            label: "Order close time"
                                        })]
                                    }), (0,
                                    l.jsxs)("div", {
                                        className: "grid grid-cols-4",
                                        children: [(0,
                                        l.jsx)(A, {
                                            contentData: (0,
                                            l.jsx)(F, {
                                                txHash: t.address
                                            }),
                                            label: "Order Address"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: (0,
                                            o.GP)(new Date(1e3 * t.createdAt), "MMM dd, yyyy HH:mm:ss"),
                                            label: "Created At"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: t.status,
                                            label: "Status"
                                        }), (0,
                                        l.jsx)(A, {
                                            contentData: "".concat(j, "/").concat(h),
                                            label: "Amount Filled"
                                        })]
                                    })]
                                })
                            })]
                        }, s)
                    }
                    )
                }), (0,
                l.jsx)(N.d, {
                    className: "self-end",
                    page: E,
                    pageSize: h.re,
                    total: D.length,
                    onPageChange: e => z(e)
                })]
            })
        }
          , L = [{
            label: "Transaction",
            key: "transaction"
        }, {
            label: "Limit price",
            key: "limitPrice",
            sortable: !0
        }, {
            label: "Amount",
            key: "amount",
            sortable: !0
        }, {
            label: "Expiry",
            key: "expiry",
            sortable: !0
        }, {
            label: "",
            key: "close"
        }, {
            label: "",
            key: "expand"
        }]
          , R = () => (0,
        l.jsxs)("colgroup", {
            children: [(0,
            l.jsx)("col", {
                className: "w-[35%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[15%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[15%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[10%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[5%]"
            })]
        })
          , D = () => {
            let[e,t] = (0,
            a.useState)(!1)
              , [s,o] = (0,
            a.useState)("OpenOrder");
            return (0,
            l.jsxs)(r.lG, {
                open: e,
                onOpenChange: t,
                children: [(0,
                l.jsx)(r.zM, {
                    asChild: !0,
                    children: (0,
                    l.jsx)(i.Q, {
                        size: "sm",
                        children: "See more"
                    })
                }), (0,
                l.jsxs)(r.Cf, {
                    className: "overflow-x-hidden md:max-w-[720px]",
                    children: [(0,
                    l.jsx)(r.c7, {
                        children: (0,
                        l.jsx)(r.L3, {
                            children: "Limit Orders"
                        })
                    }), (0,
                    l.jsx)(r.R4, {
                        className: "max-w-full overflow-hidden px-2 sm:px-4",
                        children: (0,
                        l.jsxs)(n.tU, {
                            value: s,
                            children: [(0,
                            l.jsxs)(n.j7, {
                                className: "border-border-lowest rounded-full border p-1",
                                children: [(0,
                                l.jsx)(n.Xi, {
                                    className: "flex w-full items-center justify-center py-1.5",
                                    value: "OpenOrder",
                                    onClick: () => o("OpenOrder"),
                                    children: (0,
                                    l.jsx)("p", {
                                        className: "text-body-s font-medium",
                                        children: "Open Orders"
                                    })
                                }), (0,
                                l.jsx)(n.Xi, {
                                    className: "flex w-full items-center justify-center py-1.5",
                                    value: "AllOrder",
                                    onClick: () => o("AllOrder"),
                                    children: (0,
                                    l.jsx)("p", {
                                        className: "text-body-s font-medium",
                                        children: "Order History"
                                    })
                                })]
                            }), (0,
                            l.jsx)(n.av, {
                                value: "OpenOrder",
                                children: (0,
                                l.jsx)(P, {})
                            }), (0,
                            l.jsx)(n.av, {
                                value: "AllOrder",
                                children: (0,
                                l.jsx)(w, {})
                            })]
                        })
                    })]
                })]
            })
        }
        ;
        var I = function(e) {
            return e.OpenOrder = "OpenOrder",
            e.AllOrder = "AllOrder",
            e
        }(I || {})
    }
    ,
    59305: (e, t, s) => {
        s.d(t, {
            default: () => o,
            u: () => i
        });
        var l = s(48876)
          , a = s(26432);
        let r = () => {
            let[e,t] = (0,
            a.useState)("instant");
            return (0,
            a.useMemo)( () => ({
                view: e,
                setView: t
            }), [e, t])
        }
          , n = (0,
        a.createContext)(void 0)
          , i = () => {
            let e = (0,
            a.useContext)(n);
            if (void 0 === e)
                throw Error("useSwapView was used outside of its Provider");
            return e
        }
          , o = e => {
            let {children: t} = e
              , s = r();
            return (0,
            l.jsx)(n.Provider, {
                value: s,
                children: t
            })
        }
    }
    ,
    60741: (e, t, s) => {
        s.d(t, {
            $g: () => d,
            o6: () => n,
            sG: () => r
        });
        var l = s(15845);
        let a = {
            1: "1m",
            3: "3m",
            5: "5m",
            15: "15m",
            30: "30m",
            60: "1H",
            120: "2H",
            240: "4H",
            "1D": "1D",
            "1W": "1W"
        };
        function r(e) {
            return e && a[e] ? a[e] : a[0]
        }
        function n(e) {
            let t, s = arguments.length > 1 && void 0 !== arguments[1] ? arguments[1] : "1D";
            if (!e)
                return;
            let l = s.slice(-1);
            switch (!0) {
            case "W" === l:
                t = 6048e5 + e.time;
                break;
            case "D" === l:
                t = 864e5 + e.time;
                break;
            default:
                t = 6e4 + e.time
            }
            return t
        }
        let i = {
            4: "₄",
            5: "₅",
            6: "₆",
            7: "₇",
            8: "₈",
            9: "₉",
            10: "₁₀",
            11: "₁₁",
            12: "₁₂",
            13: "₁₃",
            14: "₁₄",
            15: "₁₅"
        }
          , o = e => {
            if (!e)
                return 8;
            switch (!0) {
            case 1e-11 > Math.abs(+e):
                return 16;
            case 1e-9 > Math.abs(+e):
                return 14;
            case 1e-7 > Math.abs(+e):
                return 12;
            case 1e-5 > Math.abs(+e):
                return 10;
            case .05 > Math.abs(+e):
                return 6;
            case 1 > Math.abs(+e):
                return 4;
            case 20 > Math.abs(+e):
                return 3;
            default:
                return 2
            }
        }
          , d = function(e, t) {
            let s = !(arguments.length > 2) || void 0 === arguments[2] || arguments[2];
            if (!e)
                return e;
            t || (t = o(+e));
            let a = new l.A(e).toFormat(t);
            if (a.match(/^0\.[0]+$/g) && (a = a.replace(/\.[0]+$/g, "")),
            s && a.match(/\.0{4,15}[1-9]+/g)) {
                let e = a.match(/\.0{4,15}/g)[0].slice(1);
                a = a.replace(/\.0{4,15}/g, ".0".concat(i[e.length]))
            }
            return a
        }
    }
    ,
    75279: (e, t, s) => {
        s.d(t, {
            I: () => i
        });
        var l = s(48876)
          , a = s(27442);
        let r = (0,
        s(86741).tv)({
            slots: {
                base: ["w-fit", "flex", "items-center", "rounded-full", "border", "border-border-lowest", "bg-bg-low-em", "gap-1", "p-1"],
                option: ["relative", "flex", "items-center", "justify-center", "rounded-full", "font-medium", "whitespace-nowrap", "transition-all", "focus-ring", "text-text-mid-em enabled:hover:text-text-high-em", "bg-transparent", "transition-[background-color,_scale]", "px-3", "py-2", "text-body-s", "enabled:hover:bg-bg-low-em", "enabled:active:scale-[0.98]", "disabled:opacity-50", "data-[active]:disabled:opacity-100", "data-[active]:text-text-high-em"],
                indicator: ["bg-bg-high-em", "absolute", "inset-0", "rounded-full"],
                icon: ["shrink-0"]
            },
            variants: {
                fullWidth: {
                    true: {
                        base: "w-full",
                        option: "flex-1"
                    }
                }
            },
            defaultVariants: {
                fullWidth: !1
            }
        });
        var n = s(54094);
        function i(e) {
            let {options: t, value: s, onValueChange: a, className: n, title: i, disableAll: d, fullWidth: c, indicatorClassName: m} = e
              , {base: x} = r({
                fullWidth: c
            });
            return (0,
            l.jsx)("div", {
                className: x({
                    className: n
                }),
                children: t.map(e => (0,
                l.jsx)(o, {
                    disableAll: d,
                    fullWidth: c,
                    indicatorClassName: m,
                    isActive: s === e.value,
                    title: i,
                    value: s,
                    onValueChange: a,
                    ...e
                }, e.value))
            })
        }
        let o = e => {
            let {label: t, value: s, iconLeft: i, iconRight: o, isActive: d, onValueChange: c, className: m, disableAll: x, disabled: u, title: h, fullWidth: p, indicatorClassName: g, ...j} = e
              , {option: b, icon: f, indicator: v} = r({
                fullWidth: p
            })
              , y = e => {
                let {icon: t} = e;
                return (0,
                n.O)({
                    element: t,
                    themeStyle: f
                })
            }
            ;
            return (0,
            l.jsxs)(a.P.button, {
                layout: !0,
                layoutRoot: !0,
                className: b({
                    className: m,
                    fullWidth: p
                }),
                "data-active": d ? "" : void 0,
                disabled: u || d || x,
                onClick: e => {
                    var t;
                    u || (null == c || c(s),
                    null == (t = j.onClick) || t.call(j, e))
                }
                ,
                ...j,
                children: [d && (0,
                l.jsx)(a.P.span, {
                    className: v({
                        className: g
                    }),
                    layoutId: "segmented-control-option-active-indicator-".concat(h),
                    style: {
                        originY: "0px"
                    },
                    transition: {
                        type: "spring",
                        bounce: .2,
                        duration: .6
                    }
                }), (0,
                l.jsxs)("span", {
                    className: "relative z-10 flex items-center justify-center gap-x-1.5",
                    children: [i && (0,
                    l.jsx)(y, {
                        icon: i
                    }), t, o && (0,
                    l.jsx)(y, {
                        icon: o
                    })]
                })]
            })
        }
    }
    ,
    77049: (e, t, s) => {
        s.d(t, {
            default: () => Q
        });
        var l = s(48876)
          , a = s(19995)
          , r = s(51816)
          , n = s(27442)
          , i = s(26432)
          , o = s(36795)
          , d = s(52630)
          , c = s(55436)
          , m = s(87492)
          , x = s(49146);
        let u = e => {
            let {setTokenMarketData: t} = (0,
            x.A)()
              , s = (0,
            c.I)({
                queryKey: ["birdeye", "token_overview"],
                queryFn: async () => await (0,
                m.n)(e),
                enabled: !!e && o.m1,
                staleTime: 12e4,
                gcTime: 3e5
            });
            return (0,
            i.useEffect)( () => {
                s.data && t(s.data)
            }
            , [s.data, t]),
            s
        }
        ;
        var h = s(93355)
          , p = s(31001)
          , g = s(90529)
          , j = s(27373)
          , b = s(8626)
          , f = s(77284)
          , v = s(54206)
          , y = s(54248);
        let N = () => {
            let {openOrders: e, isLoadingOpenOrder: t, errorFetchingOpenOrders: s} = (0,
            h.A)()
              , [a,r] = (0,
            i.useState)(!1);
            return (0,
            l.jsx)(j.N, {
                defaultOpen: e.length > 0,
                icon: (0,
                l.jsx)(b.JQ, {}),
                triggerLabel: "Transaction History",
                children: (0,
                l.jsx)("div", {
                    className: "flex w-full flex-col gap-y-2",
                    children: t ? (0,
                    l.jsx)("div", {
                        className: "flex items-center justify-center py-4",
                        children: (0,
                        l.jsx)("div", {
                            className: "text-text-low-em",
                            children: "Loading orders..."
                        })
                    }) : s ? (0,
                    l.jsx)("div", {
                        className: "flex items-center justify-center py-4",
                        children: (0,
                        l.jsx)("div", {
                            className: "text-text-low-em",
                            children: "Failed to load orders"
                        })
                    }) : (0,
                    l.jsxs)(l.Fragment, {
                        children: [(0,
                        l.jsx)(v.A, {
                            openOrders: e
                        }), (0,
                        l.jsxs)("div", {
                            className: "flex items-center gap-x-2 self-end",
                            children: [(0,
                            l.jsx)(f.Q, {
                                size: "sm",
                                variant: "tertiary",
                                onClick: () => {
                                    a ? r(!1) : r(!0)
                                }
                                ,
                                children: a ? "Yes, cancel all" : "Cancel all orders"
                            }), (0,
                            l.jsx)(y.A, {})]
                        })]
                    })
                })
            })
        }
        ;
        var w = s(998)
          , k = s(41291);
        let C = () => {
            var e;
            let {history: t} = (0,
            w.q)();
            return (0,
            l.jsx)(j.N, {
                defaultOpen: (null == t || null == (e = t.swaps) ? void 0 : e.length) > 0,
                icon: (0,
                l.jsx)(b.JQ, {}),
                triggerLabel: "Transaction History",
                children: (0,
                l.jsx)(k.A, {
                    customColGroup: (0,
                    l.jsx)(A, {}),
                    wrapperClassName: "min-w-[780px]"
                })
            })
        }
          , A = () => (0,
        l.jsxs)("colgroup", {
            children: [(0,
            l.jsx)("col", {
                className: "w-[10%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[5%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            l.jsx)("col", {
                className: "w-[5%]"
            })]
        });
        var S = s(59305);
        let M = () => {
            let {view: e} = (0,
            S.u)()
              , {connected: t, walletAddress: s} = (0,
            g.z)();
            return t && s ? (0,
            l.jsxs)("div", {
                className: "w-full",
                children: ["instant" === e && (0,
                l.jsx)(C, {}), "limit" === e && (0,
                l.jsx)(N, {})]
            }) : (0,
            l.jsx)(i.Fragment, {})
        }
        ;
        var T = s(98860)
          , E = s(5657)
          , z = s(93749)
          , F = s(70734);
        let P = e => {
            let {...t} = e;
            return (0,
            l.jsx)("svg", {
                fill: "none",
                height: "16",
                viewBox: "0 0 16 16",
                width: "16",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                l.jsx)("path", {
                    d: "M7.99967 1.33331C9.99967 2.66665 10.6148 5.528 10.6663 7.99998C10.6148 10.472 9.99968 13.3333 7.99968 14.6666M7.99967 1.33331C5.99967 2.66665 5.38451 5.528 5.33301 7.99998C5.38451 10.472 5.99968 13.3333 7.99968 14.6666M7.99967 1.33331C4.31778 1.33331 1.33301 4.31808 1.33301 7.99998M7.99967 1.33331C11.6816 1.33331 14.6663 4.31808 14.6663 7.99998M7.99968 14.6666C11.6816 14.6666 14.6663 11.6819 14.6663 7.99998M7.99968 14.6666C4.31778 14.6666 1.33301 11.6819 1.33301 7.99998M14.6663 7.99998C13.333 9.99998 10.4717 10.6151 7.99967 10.6666C5.5277 10.6151 2.66634 9.99998 1.33301 7.99998M14.6663 7.99998C13.333 5.99998 10.4717 5.38481 7.99967 5.33331C5.5277 5.38481 2.66634 5.99998 1.33301 7.99998",
                    stroke: "currentColor",
                    strokeLinecap: "round",
                    strokeLinejoin: "round",
                    strokeWidth: "1.2"
                })
            })
        }
          , L = e => {
            let {...t} = e;
            return (0,
            l.jsx)("svg", {
                fill: "none",
                height: "16",
                viewBox: "0 0 16 16",
                width: "16",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                l.jsx)("path", {
                    d: "M7.52169 2.30223C7.67534 1.99094 7.75217 1.83529 7.85647 1.78556C7.94722 1.7423 8.05264 1.7423 8.14339 1.78556C8.24769 1.83529 8.32452 1.99094 8.47817 2.30223L9.93596 5.25554C9.98132 5.34744 10.004 5.39339 10.0372 5.42907C10.0665 5.46066 10.1017 5.48625 10.1408 5.50443C10.185 5.52497 10.2357 5.53238 10.3371 5.5472L13.5979 6.02382C13.9413 6.07401 14.113 6.09911 14.1924 6.18298C14.2616 6.25595 14.2941 6.35622 14.2809 6.45587C14.2658 6.57041 14.1415 6.69147 13.8929 6.93361L11.5342 9.23097C11.4607 9.30258 11.4239 9.33839 11.4002 9.381C11.3792 9.41872 11.3657 9.46016 11.3605 9.50302C11.3547 9.55143 11.3633 9.60202 11.3807 9.70318L11.9372 12.9481C11.9959 13.2903 12.0253 13.4615 11.9701 13.563C11.9221 13.6514 11.8368 13.7133 11.738 13.7317C11.6243 13.7527 11.4707 13.6719 11.1633 13.5103L8.24817 11.9772C8.15734 11.9295 8.11193 11.9056 8.06408 11.8962C8.02172 11.8879 7.97814 11.8879 7.93578 11.8962C7.88793 11.9056 7.84252 11.9295 7.75169 11.9772L4.83653 13.5103C4.5292 13.6719 4.37553 13.7527 4.2619 13.7317C4.16304 13.7133 4.07773 13.6514 4.02974 13.563C3.97459 13.4615 4.00394 13.2903 4.06264 12.9481L4.61918 9.70318C4.63653 9.60202 4.6452 9.55143 4.63933 9.50302C4.63413 9.46016 4.62066 9.41872 4.59966 9.381C4.57593 9.33839 4.53917 9.30258 4.46564 9.23097L2.10696 6.93361C1.85836 6.69147 1.73406 6.57041 1.71894 6.45587C1.70578 6.35622 1.73829 6.25595 1.80742 6.18298C1.88688 6.09911 2.05857 6.07401 2.40195 6.02382L5.66279 5.5472C5.7642 5.53238 5.81491 5.52497 5.85906 5.50443C5.89816 5.48625 5.93336 5.46066 5.96271 5.42907C5.99586 5.39339 6.01854 5.34744 6.0639 5.25554L7.52169 2.30223Z",
                    stroke: "currentColor",
                    strokeLinecap: "round",
                    strokeLinejoin: "round",
                    strokeWidth: "1.2"
                })
            })
        }
        ;
        var R = s(55796)
          , D = s(65314)
          , I = s(19646);
        let X = e => {
            var t, s, a, r;
            let {className: n, selectedToken: i, ...o} = e
              , {tokenMarketData: c} = (0,
            x.A)();
            return (0,
            l.jsxs)("div", {
                className: (0,
                d.cn)("flex w-full justify-between", n),
                ...o,
                children: [(0,
                l.jsxs)("div", {
                    className: "flex gap-x-2 lg:items-center",
                    children: [(0,
                    l.jsx)(R.H, {
                        logoURI: null == i ? void 0 : i.logoURI,
                        size: 40,
                        symbol: null == i ? void 0 : i.symbol
                    }), (0,
                    l.jsxs)("div", {
                        className: "flex flex-col gap-x-1 font-medium lg:flex-row",
                        children: [(0,
                        l.jsxs)("div", {
                            className: "flex items-center gap-x-1",
                            children: [(0,
                            l.jsx)("p", {
                                className: "text-body-xl lg:text-body-xxl",
                                children: null == i ? void 0 : i.symbol
                            }), (0,
                            l.jsx)(b.C1, {
                                className: "text-success flex size-3.5 lg:hidden"
                            })]
                        }), (0,
                        l.jsx)("p", {
                            className: "text-body-m lg:text-body-xxl text-text-mid-em",
                            children: null == i ? void 0 : i.name
                        })]
                    }), (0,
                    l.jsx)(b.C1, {
                        className: "text-success hidden size-3.5 lg:flex"
                    })]
                }), (0,
                l.jsxs)("div", {
                    className: "flex items-center gap-x-1",
                    children: [(0,
                    l.jsxs)("div", {
                        className: "flex items-center gap-x-0.5",
                        children: [(0,
                        l.jsx)("p", {
                            className: "text-body-s text-text-high-em font-medium",
                            children: (0,
                            I.lV)(null == i ? void 0 : i.address)
                        }), (0,
                        l.jsx)(F.i, {
                            copiedIconClassName: "h-3.5 w-3.5 text-text-high-em",
                            copyContent: (null == i ? void 0 : i.address) || "",
                            copyIconClassName: "h-3.5 w-3.5 text-text-high-em"
                        })]
                    }), (0,
                    l.jsx)(z.$, {
                        as: "a",
                        className: "h-8 w-8 p-0",
                        href: "https://solscan.io/token/".concat(null == i ? void 0 : i.address),
                        target: "_blank",
                        variant: "ghost",
                        children: (0,
                        l.jsx)(E.default, {
                            alt: "View on Solscan",
                            className: "mx-4 size-4",
                            height: 16,
                            loader: D.A,
                            loading: "lazy",
                            src: "/images/solscan-icon.png",
                            width: 16
                        })
                    }), (0,
                    l.jsx)(z.$, {
                        as: "a",
                        className: "h-8 w-8 p-0",
                        href: (null == c || null == (t = c.extensions) ? void 0 : t.twitter) ? null == c || null == (s = c.extensions) ? void 0 : s.twitter : "#",
                        target: "_blank",
                        variant: "ghost",
                        children: (0,
                        l.jsx)(b.B0, {
                            className: "size-4"
                        })
                    }), (0,
                    l.jsx)(z.$, {
                        className: "h-8 w-8 p-0",
                        variant: "ghost",
                        children: (0,
                        l.jsx)(L, {})
                    }), (0,
                    l.jsx)(z.$, {
                        as: "a",
                        className: "h-8 w-8 p-0",
                        href: (null == c || null == (a = c.extensions) ? void 0 : a.website) ? null == c || null == (r = c.extensions) ? void 0 : r.website : "#",
                        target: "_blank",
                        variant: "ghost",
                        children: (0,
                        l.jsx)(P, {})
                    })]
                })]
            })
        }
        ;
        var B = s(51092)
          , O = s(32641);
        let G = e => {
            let {className: t, selectedToken: s, ...a} = e
              , {price: r} = (0,
            O.YQ)(null == s ? void 0 : s.address, {
                enabled: !!(null == s ? void 0 : s.address)
            })
              , {tokenMarketData: n} = (0,
            x.A)();
            return (0,
            l.jsxs)("div", {
                className: (0,
                d.cn)("flex w-full flex-col justify-between md:flex-row", t),
                ...a,
                children: [(0,
                l.jsxs)("div", {
                    children: [(0,
                    l.jsx)("p", {
                        className: "font-heading text-heading-m",
                        children: (0,
                        B.A)({
                            number: r,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2,
                                minimumFractionDigits: 2
                            }
                        })
                    }), (0,
                    l.jsx)("p", {
                        className: (0,
                        d.cn)("text-body-s font-medium", (null == n ? void 0 : n.priceChange24hPercent) > 0 ? "text-success" : (null == n ? void 0 : n.priceChange24hPercent) < 0 ? "text-alert" : ""),
                        children: (null == n ? void 0 : n.priceChange24hPercent) ? "".concat((null == n ? void 0 : n.priceChange24hPercent) > 0 ? "+" : "").concat(null == n ? void 0 : n.priceChange24hPercent.toFixed(2), "%") : "-"
                    })]
                }), (0,
                l.jsxs)("div", {
                    className: "flex items-center gap-x-6",
                    children: [(0,
                    l.jsx)(U, {
                        label: "Market cap",
                        value: (null == n ? void 0 : n.marketCap) ? (0,
                        B.v)(n.marketCap) : "-"
                    }), (0,
                    l.jsx)(U, {
                        label: "FDV",
                        value: (null == n ? void 0 : n.fdv) ? (0,
                        B.v)(null == n ? void 0 : n.fdv) : "-"
                    }), (0,
                    l.jsx)(U, {
                        label: "24H Volume",
                        value: (null == n ? void 0 : n.vHistory24hUSD) ? (0,
                        B.v)(null == n ? void 0 : n.vHistory24hUSD) : "-"
                    })]
                })]
            })
        }
          , U = e => {
            let {label: t, value: s} = e;
            return (0,
            l.jsxs)("div", {
                className: "flex w-full flex-col md:w-fit",
                children: [(0,
                l.jsx)("p", {
                    className: "text-body-m text-text-high-em font-medium",
                    children: s
                }), (0,
                l.jsx)("p", {
                    className: "text-body-s text-text-mid-em",
                    children: t
                })]
            })
        }
        ;
        var V = s(33187)
          , q = s(7903)
          , H = s(27829);
        let _ = (0,
        V.default)( () => s.e(1905).then(s.bind(s, 61905)).then(e => e.TVChartContainer), {
            loadableGenerated: {
                webpack: () => [61905]
            },
            ssr: !1
        })
          , W = e => {
            let {tokenAddress: t, tokenSymbol: s} = e
              , [a,r] = (0,
            i.useState)(!1);
            return t && s ? (0,
            l.jsxs)(l.Fragment, {
                children: [(0,
                l.jsx)(q.default, {
                    src: "/static/datafeeds/udf/dist/bundle.js",
                    strategy: "lazyOnload",
                    onReady: () => {
                        r(!0)
                    }
                }), a ? (0,
                l.jsx)(_, {
                    address: t,
                    symbol: s
                }) : (0,
                l.jsx)(H.A, {
                    className: "border-border-lowest relative h-[338px] w-full overflow-hidden rounded-2xl border p-1"
                })]
            }) : (0,
            l.jsx)(H.A, {
                className: "border-border-lowest relative h-[338px] w-full overflow-hidden rounded-2xl border p-1"
            })
        }
          , Q = () => {
            let {view: e} = (0,
            S.u)()
              , {openChart: t} = (0,
            x.A)()
              , {buyToken: s} = (0,
            h.A)()
              , {selectedBuyToken: c} = (0,
            p.A)()
              , m = (0,
            a.Ub)("(max-width: 1023px)")
              , {} = u(null == c ? void 0 : c.address)
              , g = (0,
            i.useMemo)( () => "instant" === e ? c : s, [e, s, c]);
            return ((0,
            i.useEffect)( () => {
                o.m1 && (t || (0,
                T.c)())
            }
            , [t]),
            o.m1) ? (0,
            l.jsx)(r.N, {
                children: (0,
                l.jsx)(n.P.div, {
                    animate: {
                        width: t ? "100%" : 0
                    },
                    className: (0,
                    d.cn)("overflow-hidden", "max-w-[33.75rem] lg:max-w-none", t && "mr-0 mb-6 lg:mr-10 lg:mb-6", t && "mx-auto lg:mr-10 lg:ml-0"),
                    exit: {
                        width: 0,
                        transition: {
                            duration: .5
                        }
                    },
                    initial: {
                        width: 0
                    },
                    transition: {
                        duration: .5,
                        ease: [.26, .08, .25, 1]
                    },
                    children: (0,
                    l.jsx)("div", {
                        className: "flex flex-col gap-y-4",
                        children: t ? (0,
                        l.jsxs)(l.Fragment, {
                            children: [(0,
                            l.jsx)(X, {
                                selectedToken: g
                            }), (0,
                            l.jsx)(G, {
                                selectedToken: g
                            }), (0,
                            l.jsx)(W, {
                                tokenAddress: null == g ? void 0 : g.address,
                                tokenSymbol: null == g ? void 0 : g.symbol
                            }), !m && (0,
                            l.jsx)(M, {})]
                        }) : (0,
                        l.jsx)(i.Fragment, {})
                    })
                })
            }) : (0,
            l.jsx)(i.Fragment, {})
        }
    }
    ,
    77284: (e, t, s) => {
        s.d(t, {
            Q: () => c
        });
        var l = s(48876)
          , a = s(15721)
          , r = s.n(a)
          , n = s(26432)
          , i = s(86741);
        let o = (0,
        i.tv)({
            slots: {
                base: []
            },
            variants: {
                size: {
                    lg: {
                        base: ["text-body-m"]
                    },
                    sm: {
                        base: ["text-body-s"]
                    },
                    xs: {
                        base: ["text-body-xs"]
                    }
                }
            }
        })
          , d = (0,
        i.tv)({
            extend: o,
            slots: {
                base: ["font-semibold", "enabled:active:scale-95", "disabled:text-text-disabled", "data-[disabled]:text-text-disabled", "enabled:cursor-pointer", "enabled:hover:underline", "enabled:underline-offset-2", "border", "border-transparent"]
            },
            variants: {
                variant: {
                    primary: {
                        base: ["text-brand"]
                    },
                    secondary: {
                        base: ["text-text-high-em"]
                    },
                    tertiary: {
                        base: ["text-text-low-em"]
                    }
                },
                disabled: {
                    true: {
                        base: ["cursor-not-allowed"]
                    }
                }
            },
            defaultVariants: {
                variant: "primary",
                size: "lg"
            }
        })
          , c = (0,
        n.forwardRef)( (e, t) => {
            let {children: s, size: a, variant: i, disabled: o, ...c} = e
              , m = (0,
            n.useCallback)(e => {
                let {children: s, ...a} = e;
                if ("a" === a.as) {
                    let {external: e, href: n, as: i, ...o} = a;
                    if (e) {
                        let e = {
                            target: "_blank",
                            rel: "noopener",
                            href: n,
                            ...o
                        };
                        return (0,
                        l.jsx)("a", {
                            ref: t,
                            ...e,
                            children: s
                        })
                    }
                    return (0,
                    l.jsx)(r(), {
                        ref: t,
                        ...o,
                        href: n,
                        children: s
                    })
                }
                {
                    let {as: e, ...r} = a;
                    return (0,
                    l.jsx)("button", {
                        ref: t,
                        ...r,
                        children: s
                    })
                }
            }
            , [])
              , {base: x} = d({
                size: a,
                disabled: o,
                variant: i
            });
            return (0,
            l.jsx)(m, {
                ...c,
                className: x({
                    className: c.className
                }),
                "data-disabled": o,
                disabled: o,
                children: s
            })
        }
        );
        c.displayName = "TextButton"
    }
    ,
    87492: (e, t, s) => {
        async function l(e) {
            let t = await fetch("/api/birdeye?endpoint=defi/token_overview&address=".concat(e, "&frames=24h"), {
                method: "GET",
                headers: {
                    "Content-Type": "application/json"
                }
            });
            if (!t.ok)
                throw Error((await t.json().catch( () => ({
                    error: "Unknown error"
                }))).error || "HTTP error! status: ".concat(t.status));
            let s = await t.json();
            return null == s ? void 0 : s.data
        }
        async function a(e) {
            let t = await fetch("/api/birdeye?endpoint=defi/ohlcv&".concat(e), {
                method: "GET",
                headers: {
                    "Content-Type": "application/json"
                }
            });
            if (!t.ok)
                throw Error((await t.json().catch( () => ({
                    error: "Unknown error"
                }))).error || "HTTP error! status: ".concat(t.status));
            return t.json()
        }
        s.d(t, {
            G: () => a,
            n: () => l
        })
    }
    ,
    98860: (e, t, s) => {
        s.d(t, {
            c: () => d,
            g: () => o
        });
        var l = s(59271)
          , a = s(60741);
        let r = null
          , n = null;
        async function i() {
            if (r && r.readyState === WebSocket.OPEN)
                return r;
            let e = await fetch("/api/birdeye/advanced-chart-socket")
              , {socketUrl: t} = await e.json();
            return (r = new WebSocket(t)).addEventListener("open", () => {
                l.R.info("[Birdeye socket] Connected")
            }
            ),
            r.addEventListener("message", e => {
                let t, s = JSON.parse(e.data);
                if ("PRICE_DATA" !== s.type || !n || n.ticker !== s.data.address)
                    return;
                let l = 1e3 * s.data.unixTime
                  , r = n.lastBar
                  , i = n.resolution
                  , o = (0,
                a.o6)(r, i);
                t = l >= o ? {
                    time: o,
                    open: s.data.o,
                    high: s.data.h,
                    low: s.data.l,
                    close: s.data.c,
                    volume: s.data.v
                } : {
                    ...r,
                    high: Math.max(r.high, s.data.h),
                    low: Math.min(r.low, s.data.l),
                    close: s.data.c,
                    volume: s.data.v
                },
                n.lastBar = t,
                n.callback(t)
            }
            ),
            r
        }
        async function o(e, t, s, l, r, o) {
            let d = await i();
            n = {
                resolution: t,
                lastBar: o,
                ticker: e.ticker,
                callback: s
            };
            let c = {
                type: "SUBSCRIBE_PRICE",
                data: {
                    queryType: "simple",
                    mc: !1,
                    scaled: !1,
                    chartType: (0,
                    a.sG)(t),
                    address: null == e ? void 0 : e.ticker,
                    currency: "usd"
                }
            };
            d.readyState === WebSocket.OPEN ? d.send(JSON.stringify(c)) : d.addEventListener("open", () => d.send(JSON.stringify(c)), {
                once: !0
            })
        }
        async function d() {
            r && r.readyState === WebSocket.OPEN && r.send(JSON.stringify({
                type: "UNSUBSCRIBE_PRICE"
            }))
        }
    }
}]);
