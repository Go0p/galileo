(self.webpackChunk_N_E = self.webpackChunk_N_E || []).push([[7177], {
    3306: (e, t, s) => {
        "use strict";
        s.r(t),
        s.d(t, {
            Header: () => eT
        });
        var a = s(48876)
          , i = s(26432)
          , n = s(15721)
          , l = s.n(n)
          , r = s(18586)
          , o = s(25535)
          , c = s(5379);
        let d = [{
            href: c.R7.SWAP,
            label: "Swap"
        }, {
            href: c.R7.PROFILE,
            label: "Profile"
        }, {
            href: c.R7.LEADERBOARD,
            label: "Leaderboard"
        }];
        var u = s(93749)
          , m = s(30369)
          , h = s(75412)
          , x = s(90933)
          , p = s(8626)
          , b = s(52630)
          , g = s(51092)
          , f = s(65993)
          , v = s(93355)
          , y = s(73861)
          , w = s(52905)
          , j = s(32606);
        let k = () => {
            let {allTokens: e} = (0,
            y.A)()
              , {isDeleting: t, selectedDeletedOrder: s, setSelectedDeletedOrder: n, handleDeleteOrder: l} = (0,
            f.A)()
              , {openOrders: r, expiredOrders: o} = (0,
            v.A)()
              , c = (0,
            i.useMemo)( () => [...r, ...o], [r, o]);
            return c.length ? (0,
            a.jsx)(h._, {
                colGroup: (0,
                a.jsx)(A, {}),
                columns: N,
                headerHasBackground: !1,
                thProps: {
                    className: "text-body-xs sm:text-body-s !pl-2 !pr-0 text-text-lowest-em [&_div_button]:text-text-lowest-em bg-bg-mid-em"
                },
                wrapperProps: {
                    className: "min-w-full"
                },
                children: c.map( (i, r) => {
                    let o = e.find(e => e.address === i.inputMint)
                      , c = e.find(e => e.address === i.outputMint)
                      , {label: d, color: h, background: f} = (0,
                    j.C)(i.status, i.expiresAt)
                      , v = (null == o ? void 0 : o.decimals) ? (0,
                    g.A)({
                        number: new m.A(i.amount).div(new m.A(10).pow(null == o ? void 0 : o.decimals)).toNumber()
                    }) : i.amount
                      , y = (null == o ? void 0 : o.decimals) ? (0,
                    g.A)({
                        number: new m.A(i.amountFilled).div(new m.A(10).pow(null == o ? void 0 : o.decimals)).toNumber()
                    }) : i.amountFilled;
                    return (0,
                    a.jsxs)(x.A, {
                        className: (0,
                        b.cn)("text-body-xs font-medium", "relative overflow-hidden", "h-12 [&_td]:py-2 [&_td]:pl-[2px] sm:[&_td]:pl-2"),
                        children: [(0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsx)(w.A, {
                                buyTokenLogoURI: null == c ? void 0 : c.logoURI,
                                buyTokenSymbol: null == c ? void 0 : c.symbol,
                                className: "text-body-xs",
                                sellTokenLogoURI: null == o ? void 0 : o.logoURI,
                                sellTokenSymbol: null == o ? void 0 : o.symbol,
                                size: 24
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsx)("p", {
                                children: (0,
                                g.A)({
                                    number: null == i ? void 0 : i.limitPrice,
                                    options: {
                                        maximumFractionDigits: 2
                                    }
                                })
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsxs)("p", {
                                children: [y, "/", v]
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsxs)("div", {
                                className: "flex w-full items-center justify-end gap-x-1",
                                children: [(0,
                                a.jsx)("span", {
                                    className: (0,
                                    b.cn)("px-2 py-0.5", "text-body-s rounded-md", h, f),
                                    children: d
                                }), (0,
                                a.jsx)(u.$, {
                                    className: "relative z-10 h-8 w-8 p-0",
                                    disabled: t,
                                    size: "xs",
                                    variant: "ghost",
                                    onClick: () => {
                                        n(s === i.address ? "" : i.address)
                                    }
                                    ,
                                    children: (0,
                                    a.jsx)(p.uv, {
                                        className: "text-text-high-em size-4"
                                    })
                                })]
                            })
                        }), s && s === (null == i ? void 0 : i.address) && (0,
                        a.jsxs)("td", {
                            className: (0,
                            b.cn)("h-full w-full !p-0", "absolute top-0 left-0 z-30"),
                            children: [(0,
                            a.jsx)("div", {
                                className: "absolute inset-0 h-full w-full bg-black/1 backdrop-blur-[2px]"
                            }), (0,
                            a.jsxs)("div", {
                                className: "relative z-30 flex h-full w-full items-center justify-end gap-x-1",
                                children: [(0,
                                a.jsx)(u.$, {
                                    disabled: t,
                                    size: "xs",
                                    variant: "tertiary",
                                    onClick: () => l(),
                                    children: t ? "Canceling..." : "Yes, cancel"
                                }), (0,
                                a.jsx)(u.$, {
                                    disabled: t,
                                    size: "xs",
                                    variant: "secondary",
                                    onClick: () => n(""),
                                    children: "No, keep it"
                                })]
                            })]
                        })]
                    }, r)
                }
                )
            }) : (0,
            a.jsxs)("div", {
                className: "flex h-full flex-col items-center justify-center gap-y-2",
                children: [(0,
                a.jsx)("h3", {
                    className: "font-brand text-2xl leading-[1.5rem]",
                    children: "No limit orders"
                }), (0,
                a.jsx)("p", {
                    className: "text-text-mid-em max-w-[295px] text-center",
                    children: "Looks like you have no order history."
                })]
            })
        }
          , N = [{
            label: "Transaction",
            key: "transaction"
        }, {
            label: "Limit price",
            key: "limitPrice"
        }, {
            label: "Amount",
            key: "amount"
        }, {
            label: "Status",
            key: "status"
        }]
          , A = () => (0,
        a.jsxs)("colgroup", {
            children: [(0,
            a.jsx)("col", {
                className: "w-[35%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[30%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[15%]"
            })]
        });
        var S = s(38079);
        let C = () => {
            let[e,t] = (0,
            i.useState)("Orders");
            return (0,
            a.jsxs)(S.tU, {
                className: "h-full",
                size: "sm",
                value: e,
                onValueChange: e => t(e),
                children: [(0,
                a.jsx)(S.j7, {
                    children: (0,
                    a.jsx)(S.Xi, {
                        value: "Orders",
                        children: (0,
                        a.jsx)("h3", {
                            className: "text-body-s font-medium",
                            children: "Orders"
                        })
                    })
                }), (0,
                a.jsx)(S.av, {
                    className: (0,
                    b.cn)("max-h-full flex-1 overflow-y-auto", "[&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent"),
                    value: "Orders",
                    children: (0,
                    a.jsx)(k, {})
                })]
            })
        }
        ;
        var R = s(76776)
          , W = s(36795)
          , _ = s(90529);
        let O = () => {
            let {connected: e, walletAddress: t} = (0,
            _.z)()
              , [s,n] = (0,
            i.useState)(!1);
            return W.m1 && e && t ? (0,
            a.jsxs)(R.AM, {
                open: s,
                onOpenChange: n,
                children: [(0,
                a.jsx)(R.Wv, {
                    asChild: !0,
                    children: (0,
                    a.jsx)(u.$, {
                        className: "h-10 w-10",
                        size: "sm",
                        variant: "ghost",
                        children: (0,
                        a.jsxs)("div", {
                            className: "relative",
                            children: [(0,
                            a.jsx)(p.XF, {}), (0,
                            a.jsx)("div", {
                                className: "absolute top-0 right-[0.5px]",
                                children: (0,
                                a.jsxs)("span", {
                                    className: "relative flex size-2",
                                    children: [(0,
                                    a.jsx)("span", {
                                        className: "bg-brand/80 absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
                                    }), (0,
                                    a.jsx)("span", {
                                        className: "bg-brand absolute top-1/2 left-1/2 inline-flex size-1 -translate-x-1/2 -translate-y-1/2 rounded-full"
                                    })]
                                })
                            })]
                        })
                    })
                }), (0,
                a.jsx)(R.hl, {
                    align: "start",
                    className: (0,
                    b.cn)("h-[370px] max-h-[370px] min-h-auto max-w-[95vw] min-w-[95vw]", "relative mr-2.5 overflow-hidden p-3 sm:min-w-[31.25rem]"),
                    children: (0,
                    a.jsx)(C, {})
                })]
            }) : (0,
            a.jsx)(i.Fragment, {})
        }
        ;
        var T = s(47545)
          , U = s(5657)
          , E = s(6264)
          , P = s(94187)
          , D = s(65314);
        let L = e => {
            let {isMobile: t} = e
              , {walletStats: s, referralStats: i, hasDataError: n, hasBadgeError: l} = (0,
            P.Ay)();
            return (0,
            a.jsxs)("div", {
                className: (0,
                b.cn)("thin-scrollbar space-y-2 overflow-y-auto", "[&::-webkit-scrollbar-track]:bg-bg-low-em", "[&::-webkit-scrollbar-corner]:bg-bg-low-em", "[&::-webkit-scrollbar-thumb]:border-bg-low-em", "[scrollbar-color:var(--scrollbar-thumb)_var(--bg-bg-low-em)]"),
                children: [(0,
                a.jsxs)("div", {
                    children: [(0,
                    a.jsx)(M, {
                        hasError: !!n,
                        Icon: p.Ee,
                        children: "Trading Edge"
                    }), (0,
                    a.jsx)(I, {
                        label: "Total volume",
                        children: (0,
                        g.A)({
                            number: s.total_volume_usd,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2
                            }
                        })
                    }), (0,
                    a.jsx)(I, {
                        label: "Total trades",
                        children: s.total_trades
                    }), (0,
                    a.jsx)("div", {
                        className: "border-border-lowest my-1 w-full border-b"
                    }), (0,
                    a.jsx)(I, {
                        label: "Total outperformance",
                        children: (0,
                        g.A)({
                            number: s.total_outperformance_usd,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2
                            }
                        })
                    }), (0,
                    a.jsx)(I, {
                        label: "Total fee savings",
                        children: (0,
                        g.A)({
                            number: s.total_fee_savings_usd,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2
                            }
                        })
                    }), (0,
                    a.jsx)("div", {
                        className: "border-border-lowest my-1 w-full border-b"
                    }), (0,
                    a.jsx)(I, {
                        label: "Total trading edge",
                        children: (0,
                        a.jsx)("p", {
                            className: "text-brand",
                            children: (0,
                            g.A)({
                                number: s.total_edge_usd,
                                options: {
                                    currency: "USD",
                                    style: "currency",
                                    maximumFractionDigits: 2
                                }
                            })
                        })
                    })]
                }), (0,
                a.jsxs)("div", {
                    children: [(0,
                    a.jsx)(M, {
                        hasError: !!n,
                        Icon: p.Fo,
                        children: "Referral Stats"
                    }), (0,
                    a.jsx)(I, {
                        label: "USD volume",
                        children: (0,
                        g.A)({
                            number: i.total_referred_volume_usd || 0,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2,
                                minimumFractionDigits: 2
                            }
                        })
                    }), (0,
                    a.jsx)(I, {
                        label: "Referrals",
                        children: i.total_referrals
                    })]
                }), (0,
                a.jsxs)("div", {
                    children: [(0,
                    a.jsx)(M, {
                        hasError: !!l,
                        Icon: p.f2,
                        children: "Badges"
                    }), (0,
                    a.jsx)(z, {
                        isMobile: t
                    })]
                })]
            })
        }
          , M = e => {
            let {Icon: t, children: s, hasError: i} = e;
            return (0,
            a.jsxs)("div", {
                className: "font-brand text-heading-xxs mb-1 flex items-center gap-2",
                children: [(0,
                a.jsx)(t, {
                    className: "text-brand size-4"
                }), s, i && (0,
                a.jsx)(p.ee, {
                    className: "text-text-low-em size-3"
                })]
            })
        }
          , I = e => {
            let {label: t, children: s} = e;
            return (0,
            a.jsxs)("div", {
                className: "flex items-center justify-between gap-2 py-1",
                children: [(0,
                a.jsx)("span", {
                    className: "text-body-s sm:text-body-m text-text-mid-em",
                    children: t
                }), (0,
                a.jsx)("span", {
                    className: "text-text-high-em text-body-s sm:text-body-m font-medium",
                    children: s
                })]
            })
        }
          , z = e => {
            let {isMobile: t} = e
              , {badges: s} = (0,
            P.Ay)();
            return (0,
            a.jsx)("div", {
                className: "mt-3 grid grid-cols-[repeat(auto-fit,minmax(5rem,5rem))] justify-center gap-3",
                children: s.map( (e, s) => {
                    let i = e.description;
                    return (0,
                    a.jsx)(E.A, {
                        badge: e,
                        description: i,
                        index: s,
                        isMobile: t,
                        children: (0,
                        a.jsx)("div", {
                            className: "transition-transform duration-200 ease-out group-hover:scale-105",
                            children: e.has_badge ? (0,
                            a.jsx)(U.default, {
                                alt: "".concat(e.name, " badge"),
                                className: "object-contain",
                                height: 48,
                                loader: D.A,
                                loading: "lazy",
                                src: e.icon,
                                width: 48
                            }) : (0,
                            a.jsxs)("div", {
                                className: "relative",
                                children: [(0,
                                a.jsx)(U.default, {
                                    alt: "".concat(e.name, " badge (locked)"),
                                    className: "object-contain opacity-40",
                                    height: 48,
                                    loader: D.A,
                                    loading: "lazy",
                                    src: e.icon,
                                    width: 48
                                }), (0,
                                a.jsx)(p.XA, {
                                    className: "text-brand absolute inset-0 m-auto size-4"
                                })]
                            })
                        })
                    }, e.name)
                }
                )
            })
        }
        ;
        var F = s(62500);
        let B = e => {
            let {isMobile: t} = e
              , {connected: s} = (0,
            _.z)()
              , {refreshAll: i} = (0,
            F.A)()
              , {walletStats: n} = (0,
            P.Ay)();
            return s ? (0,
            a.jsxs)(R.AM, {
                onOpenChange: e => {
                    e && i()
                }
                ,
                children: [(0,
                a.jsx)(R.Wv, {
                    asChild: !0,
                    children: (0,
                    a.jsx)(u.$, {
                        className: "!bg-grey-900 hover:!bg-grey-800 border-transparent",
                        iconLeft: (0,
                        a.jsx)(p.Ee, {
                            className: (0,
                            b.cn)("text-brand h-3.5 w-3.5 md:h-4 md:w-4")
                        }),
                        size: "md",
                        variant: "tertiary",
                        children: (0,
                        g.A)({
                            number: n.total_edge_usd,
                            options: {
                                currency: "USD",
                                style: "currency",
                                maximumFractionDigits: 2
                            }
                        })
                    })
                }), (0,
                a.jsxs)(R.hl, {
                    align: t ? "start" : "end",
                    className: (0,
                    b.cn)("relative min-w-[90vw] sm:min-w-[26.25rem]", t ? "mr-5 md:hidden" : "hidden md:block"),
                    children: [(0,
                    a.jsx)(T.iN, {
                        className: "text-text-mid-em active:text-text-high-em z-elevated absolute top-[1.375rem] right-5 transition-all active:scale-75 sm:hidden",
                        children: (0,
                        a.jsx)(p.uv, {
                            className: "size-4"
                        })
                    }), (0,
                    a.jsx)(L, {})]
                })]
            }) : null
        }
        ;
        var q = s(41339)
          , V = s(25465)
          , Q = s(32641);
        let K = e => {
            let {className: t, ...s} = e
              , {price: n, isLoading: l} = (0,
            Q.YQ)(V.wV, {
                enabled: !0,
                refetchInterval: 15e3
            })
              , r = (0,
            i.useRef)(!1);
            return l || !(n > 0) || r.current || (r.current = !0),
            (0,
            a.jsx)(q.v, {
                className: (0,
                b.cn)("h-10 px-0 hover:bg-transparent", t),
                leadingIcon: (0,
                a.jsx)(p.Jq, {}),
                ...s,
                children: l && !r.current ? (0,
                a.jsx)(p.Nl, {
                    className: "text-text-low-em size-4 animate-spin"
                }) : (0,
                g.A)({
                    number: n,
                    options: {
                        currency: "USD",
                        style: "currency",
                        maximumFractionDigits: 2,
                        minimumFractionDigits: 2
                    }
                })
            })
        }
        ;
        var G = s(998)
          , X = s(16694);
        let $ = e => {
            let {onCloseDrawer: t} = e
              , {history: s} = (0,
            G.q)();
            return (0,
            a.jsx)("div", {
                className: "mt-3 w-full",
                children: (null == s ? void 0 : s.swaps.length) ? (0,
                a.jsx)(i.Suspense, {
                    fallback: (0,
                    a.jsx)("div", {
                        className: "mt-20 flex flex-col items-center justify-between gap-y-2",
                        children: (0,
                        a.jsxs)("div", {
                            className: "flex items-center gap-2",
                            children: [(0,
                            a.jsx)(p.Nl, {
                                className: "text-text-low-em size-4 animate-spin"
                            }), (0,
                            a.jsx)("span", {
                                className: "text-text-mid-em",
                                children: "Loading transactions..."
                            })]
                        })
                    }),
                    children: (0,
                    a.jsx)(X.A, {
                        hasMore: s.hasMore,
                        transactions: s.swaps
                    })
                }) : (0,
                a.jsxs)("div", {
                    className: "mt-20 flex flex-col items-center justify-between gap-y-2",
                    children: [(0,
                    a.jsx)("h3", {
                        className: "font-brand text-2xl leading-[1.5rem]",
                        children: "No activity"
                    }), (0,
                    a.jsx)("p", {
                        className: "text-text-mid-em max-w-[295px] text-center",
                        children: "Once you send, receive, or swap tokens, your activity will appear here."
                    }), (0,
                    a.jsxs)("button", {
                        className: "mt-4 flex items-center gap-x-1 font-medium text-white",
                        onClick: t,
                        children: [(0,
                        a.jsx)(p.Oy, {}), (0,
                        a.jsx)("p", {
                            children: "Swap"
                        })]
                    })]
                })
            })
        }
        ;
        var J = s(55796)
          , H = s(15653)
          , Z = s(51491);
        let Y = (0,
        i.memo)(e => {
            let {token: t, isActive: s} = e
              , n = (0,
            i.useCallback)( () => {
                (0,
                Z._)(t.address, {
                    successMessage: "Address copied to clipboard",
                    errorMessage: "Failed to copy address",
                    durationMs: 1e3
                })
            }
            , [t.address]);
            return (0,
            a.jsxs)("li", {
                className: (0,
                b.cn)("px-1 py-2 sm:px-3 sm:py-3", "sm:hover:bg-bg-mid-em bg-transparent", "flex flex-row items-center justify-between gap-3", "rounded-lg transition-[background-color] duration-150 ease-out will-change-[background-color]", s && "bg-bg-mid-em"),
                children: [(0,
                a.jsxs)("div", {
                    className: "flex items-center gap-3",
                    children: [(0,
                    a.jsx)(J.H, {
                        logoURI: t.logoURI,
                        size: 32,
                        symbol: t.symbol
                    }), (0,
                    a.jsxs)("div", {
                        children: [(0,
                        a.jsxs)("h3", {
                            className: "text-body-s sm:text-body-m flex items-center gap-1 font-medium",
                            children: [(0,
                            a.jsx)("span", {
                                className: "truncate",
                                children: t.symbol
                            }), (0,
                            a.jsx)(H.Bc, {
                                children: (0,
                                a.jsxs)(H.m_, {
                                    supportMobileTap: !1,
                                    children: [(0,
                                    a.jsx)(H.k$, {
                                        asChild: !0,
                                        children: (0,
                                        a.jsx)("span", {
                                            className: "shrink-0",
                                            children: t.verified ? (0,
                                            a.jsx)(p.C1, {
                                                className: "text-success size-4"
                                            }) : (0,
                                            a.jsx)(p.eq, {
                                                className: "text-alert size-4"
                                            })
                                        })
                                    }), (0,
                                    a.jsx)(H.ZI, {
                                        className: "min-w-fit",
                                        side: "top",
                                        sideOffset: 2,
                                        children: t.verified ? "Verified token" : "Unverified token"
                                    })]
                                })
                            })]
                        }), (0,
                        a.jsxs)("span", {
                            className: "text-body-xs sm:text-body-s text-text-low-em flex flex-wrap items-center gap-2",
                            children: [(0,
                            a.jsxs)("span", {
                                className: "flex items-center gap-1",
                                children: ["Balance:", " ", (0,
                                a.jsx)("span", {
                                    className: "text-text-high-em break-all",
                                    children: (0,
                                    g.A)({
                                        number: t.balance || 0,
                                        options: {
                                            maximumFractionDigits: 4,
                                            style: "decimal"
                                        }
                                    })
                                })]
                            }), (0,
                            a.jsx)("span", {
                                className: "bg-grey-600 size-1 rounded-full"
                            }), (0,
                            a.jsx)(H.Bc, {
                                children: (0,
                                a.jsxs)(H.m_, {
                                    supportMobileTap: !1,
                                    children: [(0,
                                    a.jsx)(H.k$, {
                                        asChild: !0,
                                        children: (0,
                                        a.jsx)("button", {
                                            className: "hover:text-text-high-em max-w-[8rem] cursor-pointer truncate transition-colors duration-150",
                                            onClick: n,
                                            children: "".concat(t.address.slice(0, 4), "...").concat(t.address.slice(-4))
                                        })
                                    }), (0,
                                    a.jsx)(H.ZI, {
                                        className: "min-w-fit",
                                        side: "top",
                                        sideOffset: 2,
                                        children: "Copy CA"
                                    })]
                                })
                            })]
                        })]
                    })]
                }), t.verified && (0,
                a.jsx)("data", {
                    className: "text-body-s text-text-high-em",
                    value: t.priceInUSD || 0,
                    children: (0,
                    g.A)({
                        number: t.priceInUSD || 0,
                        options: {
                            currency: "USD",
                            style: "currency",
                            maximumFractionDigits: 2,
                            minimumFractionDigits: 2
                        }
                    })
                })]
            })
        }
        , (e, t) => e.token.address === t.token.address && e.token.balance === t.token.balance && e.token.priceInUSD === t.token.priceInUSD && e.isActive === t.isActive);
        Y.displayName = "AssetItem";
        let ee = e => {
            let {address: t} = e;
            return (0,
            a.jsx)(u.$, {
                size: "md",
                onClick: () => {
                    (0,
                    Z._)(t, {
                        successMessage: "Wallet address copied to clipboard",
                        errorMessage: "Failed to copy wallet address",
                        durationMs: 1e3
                    })
                }
                ,
                children: "Copy Wallet Address"
            })
        }
          , et = e => {
            let {address: t} = e;
            return (0,
            a.jsxs)("div", {
                className: "flex flex-col items-center py-10 sm:py-20",
                children: [(0,
                a.jsx)("h4", {
                    className: "font-brand text-heading-xs sm:text-heading-s mb-2",
                    children: "No assets"
                }), (0,
                a.jsxs)("p", {
                    className: "text-body-s sm:text-body-m text-text-mid-em mb-6",
                    children: ["Looks like your wallet’s empty. ", (0,
                    a.jsx)("br", {}), " Add some SOL or tokens to begin."]
                }), (0,
                a.jsx)(ee, {
                    address: t
                })]
            })
        }
        ;
        var es = s(15334);
        let ea = () => {
            let {tokenBalances: e, balanceLoading: t, balanceError: s} = (0,
            es.A)()
              , {connected: n, publicKey: l} = (0,
            _.z)()
              , r = (0,
            i.useMemo)( () => {
                if (!l)
                    return null;
                let e = l.toBase58();
                return {
                    full: e,
                    short: "".concat(e.slice(0, 4), "...").concat(e.slice(Math.max(0, e.length - 4)))
                }
            }
            , [l])
              , o = (0,
            i.useMemo)( () => n && 0 !== e.length ? e.filter(e => e.balance > 0).map(e => ({
                ...e.token,
                balance: e.balance,
                priceInUSD: e.usdValue
            })).sort( (e, t) => e.verified !== t.verified ? t.verified ? 1 : -1 : (t.priceInUSD || 0) - (e.priceInUSD || 0)) : [], [n, e]);
            return (0,
            a.jsx)(a.Fragment, {
                children: t && 0 === o.length ? (0,
                a.jsx)("div", {
                    className: "mt-6 space-y-3",
                    children: [void 0, void 0, void 0].map( (e, t) => (0,
                    a.jsxs)("div", {
                        className: "flex animate-pulse items-center justify-between",
                        children: [(0,
                        a.jsxs)("div", {
                            className: "flex items-center gap-3",
                            children: [(0,
                            a.jsx)("div", {
                                className: "size-8 rounded-full bg-gray-700"
                            }), (0,
                            a.jsxs)("div", {
                                className: "space-y-1",
                                children: [(0,
                                a.jsx)("div", {
                                    className: "h-4 w-20 rounded bg-gray-700"
                                }), (0,
                                a.jsx)("div", {
                                    className: "h-3 w-16 rounded bg-gray-600"
                                })]
                            })]
                        }), (0,
                        a.jsxs)("div", {
                            className: "space-y-1 text-right",
                            children: [(0,
                            a.jsx)("div", {
                                className: "h-4 w-16 rounded bg-gray-700"
                            }), (0,
                            a.jsx)("div", {
                                className: "h-3 w-12 rounded bg-gray-600"
                            })]
                        })]
                    }, t))
                }) : s && 0 === o.length ? (0,
                a.jsxs)("div", {
                    className: "mt-6 space-y-3 text-center",
                    children: [(0,
                    a.jsx)("div", {
                        className: "text-alert",
                        children: "Failed to load assets"
                    }), (0,
                    a.jsx)("button", {
                        className: "text-alert hover:text-alert-emphasized text-sm underline",
                        type: "button",
                        onClick: () => window.location.reload(),
                        children: "Retry loading assets"
                    })]
                }) : 0 === o.length ? (0,
                a.jsx)(et, {
                    address: null == r ? void 0 : r.full
                }) : (0,
                a.jsx)("ul", {
                    className: "mt-3 flex flex-col gap-1.5 sm:gap-1",
                    children: o.map(e => (0,
                    a.jsx)(Y, {
                        token: e
                    }, e.address))
                })
            })
        }
        ;
        var ei = s(51816)
          , en = s(27442)
          , el = s(23145)
          , er = s(70734)
          , eo = s(37998)
          , ec = s(35849)
          , ed = s(15867)
          , eu = s(33194)
          , em = s(77729)
          , eh = s(19646)
          , ex = s(59271);
        let ep = e => {
            let {setIsOpen: t} = e
              , s = (0,
            r.useRouter)()
              , {campaigns: n} = (0,
            eu.A)()
              , {handleResetStore: l} = (0,
            ec.A)()
              , {profile: o, profileLoading: d} = (0,
            em.Ay)()
              , {disconnect: m, walletAddress: h} = (0,
            _.z)()
              , {balanceStale: x} = (0,
            es.A)()
              , {errorLoadingBalance: f, portfolioLoading: v, portfolioMetrics: y} = (0,
            ed.A)()
              , [w,j] = (0,
            i.useState)(!1)
              , k = () => {
                if (null == o ? void 0 : o.editable)
                    return void (0,
                    eo.o)({
                        title: "",
                        description: "Please set your username in Profile.",
                        variant: "alert",
                        hideCloseBtn: !0,
                        buttons: [{
                            children: "Go to Profile",
                            onClick: async () => {
                                s.push(c.R7.PROFILE)
                            }
                            ,
                            variant: "secondary"
                        }]
                    });
                if (!o.username)
                    return;
                let e = "".concat(window.location.host, "/@").concat(o.username);
                (0,
                Z._)(e, {
                    successMessage: "Referral code copied to clipboard",
                    errorMessage: "Failed to copy username",
                    durationMs: 1e3
                })
            }
              , N = async () => {
                try {
                    n.toast_campaigns.forEach(e => {
                        var t;
                        (null == (t = e.triggers) ? void 0 : t.includes("connected")) && el.oR.dismiss(e.campaign_id)
                    }
                    ),
                    await m(),
                    l(),
                    t(!1)
                } catch (e) {
                    ex.R.error("Error disconnecting wallet:", e),
                    l(),
                    t(!1)
                }
            }
            ;
            return (0,
            a.jsxs)(a.Fragment, {
                children: [(0,
                a.jsxs)("div", {
                    className: "flex justify-between gap-3",
                    children: [(0,
                    a.jsxs)("section", {
                        className: "flex items-center gap-x-3",
                        children: [(0,
                        a.jsx)("div", {
                            className: "bg-brand/10 text-brand grid size-10 place-items-center rounded-full md:size-12",
                            children: (0,
                            a.jsx)(p.r4, {})
                        }), (0,
                        a.jsxs)("div", {
                            className: "flex flex-col gap-y-1",
                            children: [(0,
                            a.jsx)("div", {
                                children: (0,
                                a.jsx)("p", {
                                    className: "font-heading text-heading-xs text-text-high-em max-w-25 truncate",
                                    children: (null == o ? void 0 : o.editable) || !(null == o ? void 0 : o.username) ? (0,
                                    eh.lV)(h || "") : o.username
                                })
                            }), (0,
                            a.jsxs)("span", {
                                className: "text-text-mid-em text-body-s relative flex items-center gap-x-1 font-semibold",
                                children: [(0,
                                eh.lV)(h || ""), (0,
                                a.jsx)(er.i, {
                                    className: "text-text-mid-em after:absolute after:inset-0",
                                    copyContent: h
                                })]
                            })]
                        })]
                    }), (0,
                    a.jsxs)("div", {
                        className: "flex items-center gap-2",
                        children: [(0,
                        a.jsx)(u.$, {
                            className: "text-body-s md:hover:bg-grey-800 p-0 hover:bg-transparent md:px-3 md:py-2",
                            disabled: d,
                            size: "md",
                            variant: "ghost",
                            onClick: () => {
                                k(),
                                t(!1)
                            }
                            ,
                            children: d ? (0,
                            a.jsx)(p.Nl, {
                                className: "text-text-low-em size-4 animate-spin"
                            }) : (0,
                            a.jsxs)("div", {
                                className: "flex items-center gap-x-1",
                                children: [(0,
                                a.jsx)(p.Td, {
                                    className: "size-3"
                                }), (0,
                                a.jsx)("p", {
                                    className: (0,
                                    b.cn)(w ? "hidden md:flex" : "flex"),
                                    children: "Copy referral"
                                })]
                            })
                        }), (0,
                        a.jsx)(ei.N, {
                            children: (0,
                            a.jsxs)(en.P.button, {
                                className: (0,
                                b.cn)("relative overflow-hidden", "flex items-center justify-center", "border-grey-700 rounded-full border", "bg-grey-800 max-h-10 min-h-10 min-w-10 px-2.5 py-2"),
                                onClick: () => {
                                    j(!w),
                                    w && N()
                                }
                                ,
                                children: [(0,
                                a.jsx)(p.W1, {}), (0,
                                a.jsx)(en.P.p, {
                                    animate: {
                                        width: w ? "auto" : 0,
                                        visibility: w ? "visible" : "hidden"
                                    },
                                    className: (0,
                                    b.cn)("text-body-s md:text-body-m font-medium", w ? "ml-1" : ""),
                                    initial: {
                                        width: 0,
                                        visibility: "hidden"
                                    },
                                    transition: {
                                        duration: .2
                                    },
                                    children: "Disconnect"
                                })]
                            })
                        })]
                    })]
                }), (0,
                a.jsx)("section", {
                    className: "mt-3 sm:mt-6",
                    children: v ? (0,
                    a.jsxs)("div", {
                        className: "flex items-center gap-2",
                        children: [(0,
                        a.jsx)(p.Nl, {
                            className: "text-text-low-em size-4 animate-spin"
                        }), (0,
                        a.jsx)("span", {
                            className: "text-heading-m sm:text-heading-l font-brand",
                            children: "Loading..."
                        })]
                    }) : f ? (0,
                    a.jsxs)("div", {
                        className: "flex flex-col gap-2",
                        children: [(0,
                        a.jsx)("div", {
                            className: "text-heading-m sm:text-heading-l font-brand text-alert",
                            children: "Error loading portfolio"
                        }), (0,
                        a.jsx)("button", {
                            className: "text-alert hover:text-alert-emphasized text-sm underline",
                            type: "button",
                            onClick: () => window.location.reload(),
                            children: "Retry loading balances"
                        })]
                    }) : (0,
                    a.jsxs)(a.Fragment, {
                        children: [(0,
                        a.jsxs)("data", {
                            className: (0,
                            b.cn)("text-heading-m sm:text-heading-l font-brand", x && "text-warning"),
                            value: y.totalUsdValue,
                            children: [(0,
                            g.A)({
                                number: y.totalUsdValue,
                                options: {
                                    currency: "USD",
                                    style: "currency",
                                    maximumFractionDigits: 2,
                                    minimumFractionDigits: 2
                                }
                            }), x && " ⚠️"]
                        }), (0,
                        a.jsx)("div", {
                            className: "mt-1 flex items-center gap-2",
                            children: (0,
                            a.jsxs)("div", {
                                className: "text-text-low-em text-body-xs sm:text-body-s flex items-center gap-1",
                                children: [(0,
                                a.jsx)("span", {
                                    children: "≈"
                                }), (0,
                                a.jsx)("span", {
                                    children: (0,
                                    g.A)({
                                        number: y.totalSolValue,
                                        options: {
                                            maximumFractionDigits: 4
                                        }
                                    })
                                }), (0,
                                a.jsx)("span", {
                                    children: "SOL"
                                })]
                            })
                        })]
                    })
                })]
            })
        }
        ;
        var eb = s(81177);
        let eg = () => {
            let {allTokens: e} = (0,
            y.A)()
              , {isDeleting: t, selectedDeletedOrder: s, setSelectedDeletedOrder: n, handleDeleteOrder: l} = (0,
            f.A)()
              , {openOrders: r, expiredOrders: o} = (0,
            v.A)()
              , [c,d] = (0,
            i.useState)(null)
              , k = (0,
            i.useMemo)( () => [...r, ...o], [r, o]);
            return k.length ? (0,
            a.jsx)(h._, {
                colGroup: (0,
                a.jsx)(ev, {}),
                columns: ef,
                headerHasBackground: !1,
                thProps: {
                    className: "text-body-xs sm:text-body-s px-0 sm:px-2 pr-0 py-0 text-text-lowest-em [&_div_button]:text-text-lowest-em"
                },
                wrapperProps: {
                    className: "min-w-full mt-3"
                },
                children: k.map( (i, r) => {
                    let o, h, f = c === (null == i ? void 0 : i.address), v = e.find(e => e.address === i.inputMint), y = e.find(e => e.address === i.outputMint), {label: k, color: N, background: A} = (0,
                    j.C)(i.status, i.expiresAt), [S,C] = [(0,
                    eb.GP)(new Date(1e3 * i.createdAt), "MMM dd"), (0,
                    eb.GP)(new Date(1e3 * i.createdAt), "HH:mm:ss")], R = (null == v ? void 0 : v.decimals) ? (0,
                    g.A)({
                        number: new m.A(i.amount).div(new m.A(10).pow(null == v ? void 0 : v.decimals)).toNumber()
                    }) : i.amount, W = (null == v ? void 0 : v.decimals) ? (0,
                    g.A)({
                        number: new m.A(i.amountFilled).div(new m.A(10).pow(null == v ? void 0 : v.decimals)).toNumber()
                    }) : i.amountFilled, _ = Number.MAX_SAFE_INTEGER;
                    return i.expiresAt >= _ - 1e6 ? (o = "Never",
                    h = "expires") : (o = (0,
                    eb.GP)(new Date(1e3 * i.expiresAt), "MMM dd"),
                    h = (0,
                    eb.GP)(new Date(1e3 * i.expiresAt), "HH:mm:ss")),
                    (0,
                    a.jsxs)(x.A, {
                        className: (0,
                        b.cn)("border-none", "text-body-xs font-medium", "relative overflow-hidden", "h-12 [&_td]:py-2 [&_td]:pl-0 sm:[&_td]:pl-2"),
                        children: [(0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsx)(w.A, {
                                buyTokenLogoURI: null == y ? void 0 : y.logoURI,
                                buyTokenSymbol: null == y ? void 0 : y.symbol,
                                className: "text-body-xs",
                                logoClassName: "hidden sm:flex",
                                sellTokenLogoURI: null == v ? void 0 : v.logoURI,
                                sellTokenSymbol: null == v ? void 0 : v.symbol,
                                size: 24
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsx)("p", {
                                children: (0,
                                g.A)({
                                    number: null == i ? void 0 : i.limitPrice,
                                    options: {
                                        maximumFractionDigits: 2
                                    }
                                })
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsxs)("div", {
                                className: "flex flex-col gap-x-0.5 sm:flex-row sm:items-center",
                                children: [(0,
                                a.jsx)("p", {
                                    children: S
                                }), (0,
                                a.jsx)("p", {
                                    className: "text-text-low-em",
                                    children: C
                                })]
                            })
                        }), (0,
                        a.jsx)("td", {
                            children: (0,
                            a.jsxs)("div", {
                                className: "flex w-full items-center justify-end gap-x-1",
                                children: [(0,
                                a.jsx)("span", {
                                    className: (0,
                                    b.cn)("px-2 py-0.5", "text-body-s rounded-md", N, A),
                                    children: k
                                }), (0,
                                a.jsx)(u.$, {
                                    className: "relative z-10 h-8 w-8 p-0",
                                    disabled: t,
                                    size: "xs",
                                    variant: "ghost",
                                    onClick: () => {
                                        n(s === i.address ? "" : i.address)
                                    }
                                    ,
                                    children: (0,
                                    a.jsx)(p.uv, {
                                        className: "text-text-high-em size-4"
                                    })
                                })]
                            })
                        }), (0,
                        a.jsx)("td", {
                            className: "absolute top-0 left-0 h-full w-full !p-0",
                            children: (0,
                            a.jsx)(H.Bc, {
                                delayDuration: 100,
                                children: (0,
                                a.jsx)(H.m_, {
                                    open: f,
                                    onOpenChange: e => d(e ? i.address : c === i.address ? null : c),
                                    children: (0,
                                    a.jsx)(H.k$, {
                                        asChild: !0,
                                        children: (0,
                                        a.jsx)("button", {
                                            className: "absolute top-0 left-0 h-full w-full",
                                            children: (0,
                                            a.jsx)(H.ZI, {
                                                className: "flex max-w-[131px] min-w-fit items-center justify-center",
                                                side: "top",
                                                sideOffset: -5,
                                                children: (0,
                                                a.jsxs)("div", {
                                                    className: "flex flex-col",
                                                    children: [(0,
                                                    a.jsx)("h4", {
                                                        className: "font-heading text-body-s",
                                                        children: "Order Status"
                                                    }), (0,
                                                    a.jsxs)("p", {
                                                        className: "text-text-low-em text-body-xs",
                                                        children: ["Filled:", " ", (0,
                                                        a.jsxs)("span", {
                                                            className: "text-text-high-em",
                                                            children: [W, "/", R]
                                                        })]
                                                    }), (0,
                                                    a.jsxs)("p", {
                                                        className: "text-text-low-em text-body-xs",
                                                        children: ["Valid until:", " ", (0,
                                                        a.jsx)("span", {
                                                            className: "text-text-high-em",
                                                            children: "Expired" === k ? k : "".concat(o, " ").concat(h, " ")
                                                        })]
                                                    }), (0,
                                                    a.jsx)("a", {
                                                        className: "text-body-xs text-brand mt-1.5 font-semibold hover:underline",
                                                        href: "https://solscan.io/account/".concat(i.address),
                                                        target: "_blank",
                                                        children: "Open in Solscan"
                                                    })]
                                                })
                                            })
                                        })
                                    })
                                })
                            })
                        }), s && s === (null == i ? void 0 : i.address) && (0,
                        a.jsxs)("td", {
                            className: (0,
                            b.cn)("h-full w-full !p-0", "absolute top-0 left-0 z-30"),
                            children: [(0,
                            a.jsx)("div", {
                                className: "absolute inset-0 h-full w-full bg-black/1 backdrop-blur-[2px]"
                            }), (0,
                            a.jsxs)("div", {
                                className: "relative z-30 flex h-full w-full items-center justify-end gap-x-1",
                                children: [(0,
                                a.jsx)(u.$, {
                                    disabled: t,
                                    size: "xs",
                                    variant: "tertiary",
                                    onClick: () => l(),
                                    children: t ? "Canceling..." : "Yes, cancel"
                                }), (0,
                                a.jsx)(u.$, {
                                    disabled: t,
                                    size: "xs",
                                    variant: "secondary",
                                    onClick: () => n(""),
                                    children: "No, keep it"
                                })]
                            })]
                        })]
                    }, r)
                }
                )
            }) : (0,
            a.jsxs)("div", {
                className: "mt-20 flex flex-col items-center justify-between gap-y-2",
                children: [(0,
                a.jsx)("h3", {
                    className: "font-brand text-2xl leading-[1.5rem]",
                    children: "No limit orders"
                }), (0,
                a.jsx)("p", {
                    className: "text-text-mid-em max-w-[295px] text-center",
                    children: "Looks like you have no order history."
                })]
            })
        }
          , ef = [{
            label: "Transaction",
            key: "transaction"
        }, {
            label: "Limit price",
            key: "limitPrice"
        }, {
            label: "Created",
            key: "time"
        }, {
            label: "Status",
            key: "status"
        }]
          , ev = () => (0,
        a.jsxs)("colgroup", {
            children: [(0,
            a.jsx)("col", {
                className: "w-[35%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[20%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[25%]"
            }), (0,
            a.jsx)("col", {
                className: "w-[20%]"
            })]
        });
        var ey = s(58458)
          , ew = s(80032)
          , ej = s(84249);
        let ek = () => {
            let {connected: e, wallet: t, walletAddress: s} = (0,
            _.z)()
              , {profile: n} = (0,
            em.Ay)()
              , l = (0,
            em.Ab)()
              , {portfolioLoading: r} = (0,
            ed.A)()
              , {walletVipStatus: o} = (0,
            ew.Ay)()
              , [c,d] = (0,
            i.useState)(!1)
              , [m,h] = (0,
            i.useState)("Assets")
              , x = (0,
            i.useMemo)( () => {
                var s, a;
                return e && t ? {
                    name: null == (s = t.adapter) ? void 0 : s.name,
                    icon: null == (a = t.adapter) ? void 0 : a.icon
                } : null
            }
            , [e, t]);
            return e && s ? (0,
            a.jsx)(a.Fragment, {
                children: (0,
                a.jsxs)(ey._s, {
                    open: c,
                    onOpenChange: d,
                    children: [(0,
                    a.jsx)(ey.Uz, {
                        asChild: !0,
                        children: (0,
                        a.jsxs)(u.$, {
                            className: (0,
                            b.cn)("text-body-m font-medium normal-case", "hover:!bg-grey-800 !bg-grey-900 border-transparent", o.isVip ? "px-2" : "pr-3 pl-2"),
                            size: "md",
                            variant: "tertiary",
                            children: [(null == x ? void 0 : x.icon) ? (0,
                            a.jsx)(U.default, {
                                alt: x.name,
                                className: "size-4 rounded-full object-contain md:size-5",
                                height: 20,
                                loader: D.f,
                                src: x.icon,
                                width: 20
                            }) : (0,
                            a.jsx)("div", {
                                className: "bg-brand/20 flex size-4 items-center justify-center rounded-full md:size-5",
                                children: (0,
                                a.jsx)("span", {
                                    className: "text-brand text-xs font-bold",
                                    children: "T"
                                })
                            }), !l && (null == n ? void 0 : n.username) ? n.username : (0,
                            eh.lV)(s), o.isVip && (0,
                            a.jsx)(ej.A, {})]
                        })
                    }), (0,
                    a.jsxs)(ey.zj, {
                        customClose: (0,
                        a.jsx)(u.$, {
                            className: "mr-3 h-10 w-10",
                            size: "md",
                            variant: "ghost",
                            onClick: () => d(!1),
                            children: (0,
                            a.jsx)(p.SJ, {})
                        }),
                        children: [(0,
                        a.jsx)(ey.BE, {
                            className: "z-10 items-start",
                            isShowCloseBtn: !1,
                            children: (0,
                            a.jsx)(ep, {
                                setIsOpen: d
                            })
                        }), (0,
                        a.jsx)(ey.ys, {
                            className: "flex-1",
                            children: (0,
                            a.jsxs)(S.tU, {
                                className: "h-full gap-0",
                                defaultValue: "Assets",
                                value: m,
                                children: [(0,
                                a.jsxs)(S.j7, {
                                    children: [(0,
                                    a.jsx)(S.Xi, {
                                        disabled: r,
                                        value: "Assets",
                                        onClick: () => h("Assets"),
                                        children: (0,
                                        a.jsx)("h3", {
                                            className: "text-body-m font-medium",
                                            children: "Assets"
                                        })
                                    }), (0,
                                    a.jsx)(S.Xi, {
                                        disabled: r,
                                        value: "Activity",
                                        onClick: () => h("Activity"),
                                        children: (0,
                                        a.jsx)("h3", {
                                            className: "text-body-m font-medium",
                                            children: "Activity"
                                        })
                                    }), W.m1 && (0,
                                    a.jsx)(S.Xi, {
                                        disabled: r,
                                        value: "Orders",
                                        onClick: () => h("Orders"),
                                        children: (0,
                                        a.jsx)("h3", {
                                            className: "text-body-m font-medium",
                                            children: "Orders"
                                        })
                                    })]
                                }), (0,
                                a.jsx)(S.av, {
                                    className: "overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                                    value: "Assets",
                                    children: (0,
                                    a.jsx)(ea, {})
                                }), (0,
                                a.jsx)(S.av, {
                                    className: "overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                                    value: "Activity",
                                    children: (0,
                                    a.jsx)($, {
                                        onCloseDrawer: () => d(!1)
                                    })
                                }), W.m1 && (0,
                                a.jsx)(S.av, {
                                    className: "overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                                    value: "Orders",
                                    children: (0,
                                    a.jsx)(eg, {})
                                })]
                            })
                        })]
                    })]
                })
            }) : (0,
            a.jsx)(i.Fragment, {})
        }
        ;
        var eN = function(e) {
            return e.Assets = "Assets",
            e.Activity = "Activity",
            e.Orders = "Orders",
            e
        }({});
        let eA = () => {
            let e = (0,
            r.usePathname)()
              , t = e === c.R7.ROOT;
            return (0,
            a.jsxs)(a.Fragment, {
                children: [(0,
                a.jsxs)("nav", {
                    className: "flex items-center gap-10",
                    children: [(0,
                    a.jsx)(l(), {
                        href: c.R7.ROOT,
                        children: (0,
                        a.jsx)(p.Mw, {
                            className: "h-5 w-20 text-[#FFFDF7] sm:h-6 sm:w-24 lg:h-7 lg:w-32"
                        })
                    }), (0,
                    a.jsx)("div", {
                        className: "hidden h-[var(--header-height)] items-center gap-2 md:flex",
                        children: d.map(t => {
                            let s = e === t.href;
                            return (0,
                            a.jsx)(l(), {
                                className: (0,
                                b.cn)("px-4", "text-body-m font-body font-medium", "text-text-mid-em hover:text-text-high-em", s ? "text-text-high-em" : ""),
                                href: t.href,
                                children: t.label
                            }, t.href)
                        }
                        )
                    })]
                }), (0,
                a.jsxs)("div", {
                    className: "hidden items-center gap-2 md:flex",
                    children: [(0,
                    a.jsx)(K, {}), !t && (0,
                    a.jsx)(O, {}), (0,
                    a.jsx)(o.u, {
                        className: "text-body-m px-4 font-medium",
                        size: "md"
                    }), !t && (0,
                    a.jsxs)(a.Fragment, {
                        children: [(0,
                        a.jsx)(B, {}), (0,
                        a.jsx)(ek, {})]
                    })]
                })]
            })
        }
          , eS = e => {
            let {item: t, setOpen: s, className: i, ...n} = e
              , o = (0,
            r.usePathname)() === t.href;
            return (0,
            a.jsx)(l(), {
                className: (0,
                b.cn)("text-body-m group text-text-mid-em flex h-[90%] items-center justify-center py-2 font-medium", i),
                onClick: () => null == s ? void 0 : s(!1),
                ...n,
                children: (0,
                a.jsx)("span", {
                    className: (0,
                    b.cn)("rounded-full px-4 py-2 transition-colors duration-200", o ? "bg-bg-mid-em text-text-high-em" : "group-hover:bg-bg-mid-em/50"),
                    children: t.label
                })
            })
        }
          , eC = () => {
            let {profile: e} = (0,
            em.Ay)()
              , {walletVipStatus: t} = (0,
            ew.Ay)()
              , {portfolioLoading: s} = (0,
            ed.A)()
              , n = (0,
            em.Ab)()
              , {connected: l, wallet: r, walletAddress: o} = (0,
            _.z)()
              , [c,d] = (0,
            i.useState)(!1)
              , [m,h] = (0,
            i.useState)(eN.Assets)
              , x = (0,
            i.useMemo)( () => {
                var e, t;
                return l && r ? {
                    name: null == (e = r.adapter) ? void 0 : e.name,
                    icon: null == (t = r.adapter) ? void 0 : t.icon
                } : null
            }
            , [l, r]);
            return l && o ? (0,
            a.jsxs)(R.AM, {
                open: c,
                onOpenChange: d,
                children: [(0,
                a.jsx)(R.Wv, {
                    asChild: !0,
                    children: (0,
                    a.jsxs)(u.$, {
                        className: (0,
                        b.cn)("text-body-m font-medium normal-case", "hover:!bg-grey-800 !bg-grey-900 border-transparent", t.isVip ? "px-2" : "pr-3 pl-2"),
                        size: "md",
                        variant: "tertiary",
                        children: [(null == x ? void 0 : x.icon) ? (0,
                        a.jsx)(U.default, {
                            alt: x.name,
                            className: "size-4 rounded-full object-contain md:size-5",
                            height: 20,
                            loader: D.A,
                            src: x.icon,
                            width: 20
                        }) : (0,
                        a.jsx)("div", {
                            className: "bg-brand/20 flex size-4 items-center justify-center rounded-full md:size-5",
                            children: (0,
                            a.jsx)("span", {
                                className: "text-brand text-xs font-bold",
                                children: "T"
                            })
                        }), (0,
                        a.jsx)("p", {
                            className: (0,
                            b.cn)(!n && (null == e ? void 0 : e.username) ? "max-w-20 truncate" : ""),
                            children: !n && (null == e ? void 0 : e.username) ? e.username : (0,
                            eh.lV)(o)
                        }), t.isVip && (0,
                        a.jsx)(ej.A, {})]
                    })
                }), (0,
                a.jsxs)(R.hl, {
                    align: "start",
                    className: (0,
                    b.cn)("max-h-[80vh] max-w-[95vw] min-w-[95vw]", "relative mr-2.5 overflow-hidden p-3 sm:min-w-[26.25rem]"),
                    children: [(0,
                    a.jsx)(ep, {
                        setIsOpen: d
                    }), (0,
                    a.jsxs)(S.tU, {
                        className: "mt-4 h-full gap-0",
                        defaultValue: eN.Assets,
                        value: m,
                        children: [(0,
                        a.jsxs)(S.j7, {
                            children: [(0,
                            a.jsx)(S.Xi, {
                                disabled: s,
                                value: eN.Assets,
                                onClick: () => h(eN.Assets),
                                children: (0,
                                a.jsx)("h3", {
                                    className: "text-body-m font-medium",
                                    children: "Assets"
                                })
                            }), (0,
                            a.jsx)(S.Xi, {
                                disabled: s,
                                value: eN.Activity,
                                onClick: () => h(eN.Activity),
                                children: (0,
                                a.jsx)("h3", {
                                    className: "text-body-m font-medium",
                                    children: "Activity"
                                })
                            }), W.m1 && (0,
                            a.jsx)(S.Xi, {
                                disabled: s,
                                value: eN.Orders,
                                onClick: () => h(eN.Orders),
                                children: (0,
                                a.jsx)("h3", {
                                    className: "text-body-m font-medium",
                                    children: "Orders"
                                })
                            })]
                        }), (0,
                        a.jsx)(S.av, {
                            className: "max-h-[300px] overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                            value: eN.Assets,
                            children: (0,
                            a.jsx)(ea, {})
                        }), (0,
                        a.jsx)(S.av, {
                            className: "max-h-[300px] overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                            value: eN.Activity,
                            children: (0,
                            a.jsx)($, {
                                onCloseDrawer: () => d(!1)
                            })
                        }), W.m1 && (0,
                        a.jsx)(S.av, {
                            className: "max-h-[300px] overflow-x-hidden overflow-y-auto [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                            value: eN.Orders,
                            children: (0,
                            a.jsx)(eg, {})
                        })]
                    })]
                })]
            }) : (0,
            a.jsx)(i.Fragment, {})
        }
        ;
        var eR = s(93284);
        let eW = () => {
            let e = (0,
            r.usePathname)();
            return (0,
            a.jsx)("nav", {
                className: "mobile-nav flex items-center gap-1 md:hidden",
                children: "/" !== e && (0,
                a.jsxs)(a.Fragment, {
                    children: [(0,
                    a.jsx)(eC, {}), (0,
                    a.jsx)(o.u, {
                        className: "text-body-m px-4 font-medium",
                        size: "md"
                    }), (0,
                    a.jsx)(O, {}), (0,
                    a.jsx)(e_, {})]
                })
            })
        }
          , e_ = () => {
            let {isConnected: e} = (0,
            eR.j)()
              , [t,s] = (0,
            i.useState)(!1);
            return (0,
            a.jsx)(a.Fragment, {
                children: (0,
                a.jsxs)(R.AM, {
                    open: t,
                    onOpenChange: s,
                    children: [(0,
                    a.jsx)(R.Wv, {
                        asChild: !0,
                        children: (0,
                        a.jsx)("button", {
                            "aria-label": "Open menu",
                            className: "flex size-7 items-center justify-center text-neutral-50",
                            onClick: () => s(!t),
                            children: t ? (0,
                            a.jsx)(p.uv, {
                                className: "mobile-nav-open size-5"
                            }) : (0,
                            a.jsx)(p.ZB, {
                                className: "size-5"
                            })
                        })
                    }), (0,
                    a.jsx)(R.hl, {
                        align: "start",
                        className: "relative mr-5 max-h-[90vh] min-w-[90vw] overflow-y-scroll sm:min-w-[26.25rem] [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-track]:bg-transparent",
                        children: (0,
                        a.jsxs)("div", {
                            className: "flex w-full flex-col gap-y-4",
                            children: [(0,
                            a.jsx)("div", {
                                className: "flex w-full flex-col items-center justify-center gap-y-4",
                                children: d.map(e => (0,
                                a.jsx)(eS, {
                                    className: "w-full py-0 [&_span]:w-full",
                                    href: e.href,
                                    item: e,
                                    setOpen: s
                                }, e.href))
                            }), e && (0,
                            a.jsx)(L, {})]
                        })
                    })]
                })
            })
        }
        ;
        var eO = s(1187);
        let eT = () => {
            let {setPortfolioLoading: e, setPortfolioMetrics: t, setErrorLoadingBalance: s} = (0,
            ed.A)()
              , {getTokenPrice: n} = (0,
            eO.A)()
              , {tokenBalances: l, totalUsdValue: r, balanceLoading: o, balanceError: c} = (0,
            es.A)()
              , d = (0,
            i.useCallback)( () => {
                if (0 === l.length)
                    return;
                let e = r / (n(V.wV) || 1);
                return {
                    totalUsdValue: r,
                    totalSolValue: e
                }
            }
            , [l, r]);
            return (0,
            i.useEffect)( () => {
                e(o && 0 === l.length)
            }
            , [o, e, l.length]),
            (0,
            i.useEffect)( () => {
                s(c && 0 === l.length)
            }
            , [c, s, l.length]),
            (0,
            i.useEffect)( () => {
                let e = d();
                e && t(e)
            }
            , [t, d]),
            (0,
            a.jsxs)("header", {
                className: "z-header sticky top-0 container flex h-18 max-w-full items-center justify-between lg:h-[var(--header-height)]",
                children: [(0,
                a.jsx)("div", {
                    className: (0,
                    b.cn)("bg-background -z-elevated pointer-events-none absolute inset-0 backdrop-blur-md", "group-[&:has(.mobile-nav-open)]/body:opacity-0 group-[&:has(.mobile-nav-open)]/body:transition-opacity group-[&:has(.mobile-nav-open)]/body:delay-200")
                }), (0,
                a.jsx)(eA, {}), (0,
                a.jsx)(eW, {})]
            })
        }
    }
    ,
    6063: (e, t, s) => {
        "use strict";
        s.d(t, {
            default: () => l
        });
        var a = s(48876)
          , i = s(26432)
          , n = s(93739);
        let l = e => {
            let {config: t} = e
              , {setAppConfig: s} = (0,
            n.A)();
            return (0,
            i.useEffect)( () => {
                s(t)
            }
            , [t, s]),
            (0,
            a.jsx)(a.Fragment, {})
        }
    }
    ,
    6722: (e, t, s) => {
        "use strict";
        s.d(t, {
            default: () => u
        });
        var a = s(48876)
          , i = s(76013)
          , n = s(26432)
          , l = s(82945)
          , r = s(25465)
          , o = s(32641)
          , c = s(3242)
          , d = s(59271);
        let u = () => {
            (0,
            c.s)(),
            (0,
            o.Mk)();
            let e = (0,
            i.jE)()
              , t = (0,
            n.useRef)(Date.now());
            return (0,
            o.gK)(r.Tt, {
                staleTime: 3e4,
                gcTime: 3e5
            }),
            (0,
            n.useEffect)( () => {
                let s = setInterval( () => {
                    Date.now() - t.current > 6e5 && (d.R.info("Price cache cleared due to inactivity"),
                    e.invalidateQueries({
                        queryKey: l.l.prices.all
                    }))
                }
                , 6e4);
                return () => clearInterval(s)
            }
            , [e]),
            (0,
            a.jsx)(n.Fragment, {})
        }
    }
    ,
    37962: (e, t, s) => {
        Promise.resolve().then(s.t.bind(s, 20638, 23)),
        Promise.resolve().then(s.t.bind(s, 76928, 23)),
        Promise.resolve().then(s.t.bind(s, 37082, 23)),
        Promise.resolve().then(s.t.bind(s, 7464, 23)),
        Promise.resolve().then(s.bind(s, 3306)),
        Promise.resolve().then(s.bind(s, 27550)),
        Promise.resolve().then(s.bind(s, 91447)),
        Promise.resolve().then(s.bind(s, 6063)),
        Promise.resolve().then(s.bind(s, 6722)),
        Promise.resolve().then(s.bind(s, 97267)),
        Promise.resolve().then(s.bind(s, 72784)),
        Promise.resolve().then(s.bind(s, 77143)),
        Promise.resolve().then(s.bind(s, 85438)),
        Promise.resolve().then(s.t.bind(s, 89063, 23)),
        Promise.resolve().then(s.bind(s, 36759))
    }
    ,
    72784: (e, t, s) => {
        "use strict";
        s.d(t, {
            default: () => A
        });
        var a = s(48876)
          , i = s(19995)
          , n = s(26432)
          , l = s(14234);
        let r = e => {
            let {campaignInfoId: t, open: s, onOpenChange: i} = e;
            return "vip" === t ? (0,
            a.jsx)(l.A, {
                open: s,
                onOpenChange: i
            }) : null
        }
        ;
        var o = s(5657)
          , c = s(93749);
        let d = e => {
            let {...t} = e;
            return (0,
            a.jsx)("svg", {
                fill: "none",
                height: "16",
                viewBox: "0 0 16 16",
                width: "16",
                xmlns: "http://www.w3.org/2000/svg",
                ...t,
                children: (0,
                a.jsx)("path", {
                    d: "M8.00033 10.0001L6.00033 8.00008M8.00033 10.0001C8.93156 9.64591 9.82492 9.19923 10.667 8.66675M8.00033 10.0001V13.3334C8.00033 13.3334 10.0203 12.9667 10.667 12.0001C11.387 10.9201 10.667 8.66675 10.667 8.66675M6.00033 8.00008C6.35509 7.0797 6.80179 6.19746 7.33366 5.36675C8.11045 4.12474 9.19207 3.10212 10.4757 2.39614C11.7592 1.69017 13.2021 1.32433 14.667 1.33342C14.667 3.14675 14.147 6.33342 10.667 8.66675M6.00033 8.00008H2.66699C2.66699 8.00008 3.03366 5.98008 4.00033 5.33342C5.08033 4.61342 7.33366 5.33342 7.33366 5.33342M3.00033 11.0001C2.00033 11.8401 1.66699 14.3334 1.66699 14.3334C1.66699 14.3334 4.16033 14.0001 5.00033 13.0001C5.47366 12.4401 5.46699 11.5801 4.94033 11.0601C4.6812 10.8128 4.33985 10.6698 3.98181 10.6588C3.62376 10.6477 3.27424 10.7692 3.00033 11.0001Z",
                    stroke: "#F2D364",
                    strokeLinecap: "round",
                    strokeLinejoin: "round",
                    strokeWidth: "1.33"
                })
            })
        }
        ;
        var u = s(6008)
          , m = s(27829)
          , h = s(52630);
        let x = e => {
            let {open: t, title: s, label: i, imgSrc: n, description: l, onOpenChange: r, campaignInfoId: x, onOpenCampaignInfo: p} = e
              , b = () => {
                x && p && p(x)
            }
            ;
            return (0,
            a.jsx)(a.Fragment, {
                children: (0,
                a.jsx)(u.lG, {
                    open: t,
                    onOpenChange: r,
                    children: (0,
                    a.jsxs)(u.Cf, {
                        preventDefaultDomBehavior: !0,
                        className: "md:max-w-[770px]",
                        children: [(0,
                        a.jsx)(u.c7, {
                            className: "py-0 sm:py-0",
                            showCloseButton: !1,
                            children: (0,
                            a.jsx)(u.L3, {
                                className: "hidden",
                                children: s
                            })
                        }), (0,
                        a.jsxs)(u.R4, {
                            className: (0,
                            h.cn)("relative !p-3", "flex flex-col gap-y-2", "max-h-full min-h-[564px] w-full md:max-h-none md:min-h-[492px]"),
                            children: [(0,
                            a.jsxs)("div", {
                                className: "relative h-[344px] w-full rounded-xl",
                                children: [(0,
                                a.jsxs)("span", {
                                    className: (0,
                                    h.cn)("px-1.5 py-1", "text-brand", "absolute top-3 left-3 z-10 md:right-3 md:left-auto", "rounded-md bg-[#272213]", "flex items-center gap-x-1.5"),
                                    children: [(0,
                                    a.jsx)(d, {}), (0,
                                    a.jsx)("span", {
                                        className: "text-body-s",
                                        children: "Product update"
                                    })]
                                }), n ? (0,
                                a.jsx)(o.default, {
                                    fill: !0,
                                    priority: !0,
                                    alt: "Notification Banner",
                                    className: "rounded-xl object-cover object-center",
                                    src: n
                                }) : (0,
                                a.jsx)(m.A, {
                                    className: "bg-bg-mid-em h-full w-full rounded-xl"
                                })]
                            }), i && (0,
                            a.jsx)("h3", {
                                className: "font-heading text-heading-s text-text-high-em",
                                children: i
                            }), l && (0,
                            a.jsx)("p", {
                                className: "text-body-m text-text-mid-em",
                                children: l
                            })]
                        }), (0,
                        a.jsxs)("div", {
                            className: "flex w-full items-center justify-end gap-x-2 p-5 md:p-6",
                            children: [x && (0,
                            a.jsx)(c.$, {
                                size: "md",
                                variant: "ghost",
                                onClick: () => {
                                    b(),
                                    r(!1)
                                }
                                ,
                                children: "Read more"
                            }), (0,
                            a.jsx)(c.$, {
                                size: "md",
                                onClick: () => r(!1),
                                children: "Continue to dApp"
                            })]
                        })]
                    })
                })
            })
        }
        ;
        var p = s(23145);
        let b = e => {
            let {callBack: t, title: s, image: i, campaignId: n} = e;
            return (0,
            a.jsx)(a.Fragment, {
                children: (0,
                a.jsxs)(c.$, {
                    className: (0,
                    h.cn)("normal-case", "relative mx-auto", "flex gap-x-2 overflow-hidden", "bg-bg-low-em hover:bg-bg-mid-em py-2"),
                    size: "md",
                    variant: "tertiary",
                    onClick: () => {
                        t instanceof Function && t(),
                        p.oR.dismiss("".concat(n))
                    }
                    ,
                    children: [(0,
                    a.jsx)(o.default, {
                        alt: "Campaign",
                        className: "absolute -top-3 left-0 h-18 w-10",
                        height: 80,
                        src: i,
                        width: 40
                    }), (0,
                    a.jsx)("p", {
                        className: "text-body-s text-text-high-em ml-6",
                        children: s
                    })]
                })
            })
        }
        ;
        var g = s(90529)
          , f = s(85346)
          , v = s(55436)
          , y = s(82945)
          , w = s(33194)
          , j = s(77729)
          , k = s(31001)
          , N = s(80032);
        let A = () => {
            let e = (0,
            i.Ub)("(max-width: 768px)")
              , {walletAddress: t, connected: s} = (0,
            g.z)()
              , {walletVipStatus: l, sponsoredTransactionStatus: o} = (0,
            N.Ay)()
              , {profile: c} = (0,
            j.Ay)()
              , {sellInputRef: d} = (0,
            k.A)()
              , {campaigns: u} = (0,
            w.A)()
              , {openedCampaigns: m, setOpenedCampaigns: h} = function(e) {
                let[t,s] = (0,
                n.useState)(new Set);
                return (0,
                n.useEffect)( () => {
                    if (!e)
                        return;
                    let t = [];
                    if (localStorage.getItem("".concat(e, "-has_open_toast"))) {
                        let a = "vip_toast_".concat(e)
                          , i = "not_vip_toast_".concat(e);
                        t.push(a, i),
                        s(e => new Set([...e, a, i])),
                        localStorage.setItem(a, "true"),
                        localStorage.setItem(i, "true")
                    }
                    if (localStorage.getItem("".concat(e, "-has_open_vip_announcement"))) {
                        let a = "vip_modal_".concat(e);
                        t.push(a),
                        s(e => new Set([...e, a])),
                        localStorage.setItem(a, "true")
                    }
                    if (localStorage.getItem("".concat(e, "-has_open_running_out"))) {
                        let a = "no_sponsored_tx_".concat(e);
                        t.push(a),
                        s(e => new Set([...e, a])),
                        localStorage.setItem(a, "true")
                    }
                    t.length > 0 && (0,
                    f.aO)(e, t)
                }
                , [e]),
                {
                    openedCampaigns: t,
                    setOpenedCampaigns: s
                }
            }(t)
              , [A,S] = (0,
            n.useState)({
                open: !1
            })
              , C = (0,
            n.useMemo)( () => ({
                connected: s && !!t,
                is_vip: (null == l ? void 0 : l.isVip) && !!(null == c ? void 0 : c.exists),
                not_vip: !(null == l ? void 0 : l.isVip) && !!(null == c ? void 0 : c.exists),
                no_sponsored_tx: o.exists && 0 === o.count && !o.isLoading
            }), [s, t, l, c, o.exists, o.count, o.isLoading]);
            !function(e, t) {
                let {setCampaigns: s, setIsLoading: a, setError: l} = (0,
                w.A)()
                  , r = (0,
                i.d7)({
                    walletAddress: e,
                    userTriggers: t
                }, 3e3)
                  , o = (0,
                v.I)({
                    queryKey: y.l.campaigns.list(r.walletAddress, r.userTriggers),
                    queryFn: async () => {
                        let e = await (0,
                        f.ME)(r.walletAddress, r.userTriggers);
                        return s(e),
                        e
                    }
                    ,
                    enabled: !0,
                    staleTime: 0,
                    gcTime: 0,
                    refetchOnWindowFocus: !0,
                    refetchOnMount: !0,
                    retry: (e, t) => !(t.message.includes("400") || t.message.includes("403")) && e < 3,
                    retryDelay: e => Math.min(1e3 * 2 ** e, 3e4)
                });
                (0,
                n.useEffect)( () => {
                    a(o.isLoading)
                }
                , [o.isLoading, a]),
                (0,
                n.useEffect)( () => {
                    o.error ? l(o.error.message) : l(null)
                }
                , [o.error, l])
            }(t, C);
            let R = (0,
            n.useCallback)(e => {
                S({
                    open: !0,
                    campaignInfoId: e
                })
            }
            , [])
              , W = (0,
            n.useCallback)( () => {
                S({
                    open: !1
                })
            }
            , [])
              , _ = (0,
            n.useCallback)(e => {
                h(t => new Set([...t, e.campaign_id])),
                localStorage.setItem("".concat(e.campaign_id), "true"),
                t && (0,
                f.d_)(e, t),
                e.campaign_info_id && R(e.campaign_info_id)
            }
            , [t, h, R]);
            return (0,
            n.useEffect)( () => {
                (null == u ? void 0 : u.success) && u.toast_campaigns.length > 0 && u.toast_campaigns.forEach(e => {
                    "true" === localStorage.getItem("".concat(e.campaign_id)) || m.has(e.campaign_id) || function(e, t, s, i, n) {
                        p.oR.custom( () => (0,
                        a.jsx)(b, {
                            callBack: () => e(),
                            campaignId: t,
                            image: i,
                            title: s
                        }), {
                            duration: 1 / 0,
                            position: n,
                            id: "".concat(t)
                        })
                    }( () => _(e), e.campaign_id, e.title, e.image, e.toast_position || "top-center")
                }
                )
            }
            , [null == u ? void 0 : u.success, null == u ? void 0 : u.toast_campaigns, _, m]),
            (0,
            a.jsxs)(a.Fragment, {
                children: [(null == u ? void 0 : u.success) && u.modal_campaigns.map(s => {
                    let i = !localStorage.getItem("".concat(s.campaign_id)) && !m.has(s.campaign_id);
                    return (0,
                    a.jsx)(x, {
                        campaignInfoId: s.campaign_info_id,
                        description: s.description,
                        imgSrc: e ? s.mobile_image : s.image,
                        label: s.title,
                        open: i,
                        title: s.title,
                        onOpenCampaignInfo: R,
                        onOpenChange: e => {
                            !1 === e && (localStorage.setItem("".concat(s.campaign_id), "true"),
                            h(e => new Set([...e, s.campaign_id])),
                            t && (0,
                            f.d_)(s, t))
                        }
                    }, s.campaign_id)
                }
                ), A.campaignInfoId && (0,
                a.jsx)(r, {
                    campaignInfoId: A.campaignInfoId,
                    open: A.open,
                    onOpenChange: W
                })]
            })
        }
    }
    ,
    77143: (e, t, s) => {
        "use strict";
        s.d(t, {
            QueryProvider: () => r
        });
        var a = s(48876)
          , i = s(47960)
          , n = s(76013)
          , l = s(26432);
        let r = e => {
            let {children: t} = e
              , [s] = (0,
            l.useState)( () => new i.E({
                defaultOptions: {
                    queries: {
                        staleTime: 3e5,
                        gcTime: 6e5,
                        retry: 3,
                        retryDelay: e => Math.min(1e3 * 2 ** e, 3e4),
                        refetchOnWindowFocus: !0,
                        refetchOnReconnect: !0
                    },
                    mutations: {
                        retry: 1
                    }
                }
            }));
            return (0,
            a.jsxs)(n.Ht, {
                client: s,
                children: [t, !1]
            })
        }
    }
    ,
    85438: (e, t, s) => {
        "use strict";
        s.d(t, {
            WalletConnectionProvider: () => ee
        });
        var a = s(48876)
          , i = s(91245)
          , n = s(72008)
          , l = s(94442)
          , r = s(3148)
          , o = s(61542)
          , c = s(19687)
          , d = s(2874)
          , u = s(26432)
          , m = s(76013)
          , h = s(32641)
          , x = s(93284)
          , p = s(40476)
          , b = s(10985)
          , g = s(59271)
          , f = s(91015).Buffer;
        class v {
            static getInstance() {
                return v.instance || (v.instance = new v),
                v.instance
            }
            initialize(e, t) {
                this.apiBaseUrl = e.apiBaseUrl || this.apiBaseUrl,
                this.rpcWssUrl = e.rpcWssUrl || this.rpcWssUrl,
                this.queryClient = t,
                this.rpcWssUrl || g.R.warn("[LimitOrderWebSocket] RPC WSS URL is not set - configure RPC_WSS_URL environment variable")
            }
            startMonitoring(e) {
                this.queryClient && e && (this.stopMonitoring(),
                this.walletAddress = e,
                this.reconnectAttempts = 0,
                this.connect())
            }
            stopMonitoring() {
                this.cleanupConnection(),
                this.walletAddress = null,
                this.reconnectAttempts = 0,
                this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null)
            }
            connect() {
                this.walletAddress && this.connectViaWebSocket()
            }
            connectViaWebSocket() {
                if (this.walletAddress && !this.isConnectingWs && (!this.socket || this.socket.readyState !== WebSocket.OPEN)) {
                    if (!this.rpcWssUrl)
                        return void g.R.warn("[LimitOrderWebSocket] RPC WSS URL missing; cannot connect");
                    try {
                        this.isConnectingWs = !0,
                        this.socket = new WebSocket(this.rpcWssUrl),
                        this.socket.onopen = () => {
                            this.isConnectingWs = !1,
                            this.reconnectAttempts = 0,
                            g.R.info("[LimitOrderWebSocket] WebSocket connection established"),
                            this.walletAddress && (g.R.info("[LimitOrderWebSocket] Starting subscriptions for wallet: ".concat(this.walletAddress)),
                            this.subscribeToLimitOrders(this.walletAddress))
                        }
                        ,
                        this.socket.onmessage = async e => {
                            try {
                                let S = JSON.parse(e.data);
                                if ((null == S ? void 0 : S.result) && "number" == typeof S.result && S.id) {
                                    this.subscriptions.set(S.id, S.result),
                                    g.R.info("[LimitOrderWebSocket] Subscription confirmed - ID: ".concat(S.id, ", Subscription ID: ").concat(S.result));
                                    return
                                }
                                if ((null == S ? void 0 : S.method) === "programNotification") {
                                    var t, s, a, i, n, l, r, o, c, d, u, m, h, x, p, b, f, v, y, w, j, k, N, A;
                                    g.R.debug("[LimitOrderWebSocket] Program notification received: ".concat(JSON.stringify(S.params))),
                                    g.R.info("[LimitOrderWebSocket] Program notification details:", {
                                        subscription: null == (t = S.params) ? void 0 : t.subscription,
                                        hasResult: !!(null == (s = S.params) ? void 0 : s.result),
                                        hasValue: !!(null == (i = S.params) || null == (a = i.result) ? void 0 : a.value),
                                        hasData: !!(null == (r = S.params) || null == (l = r.result) || null == (n = l.value) ? void 0 : n.data),
                                        hasAccountData: !!(null == (u = S.params) || null == (d = u.result) || null == (c = d.value) || null == (o = c.account) ? void 0 : o.data),
                                        walletAddress: this.walletAddress
                                    }),
                                    this.isRelevantLimitOrderChange(S) ? (g.R.info("[LimitOrderWebSocket] Relevant limit order change detected, invalidating cache", {
                                        walletAddress: this.walletAddress,
                                        accountPubkey: null == (x = S.params) || null == (h = x.result) || null == (m = h.value) ? void 0 : m.pubkey
                                    }),
                                    this.invalidateLimitOrderCache()) : g.R.debug("[LimitOrderWebSocket] Program notification not relevant to our wallet", {
                                        walletAddress: this.walletAddress,
                                        accountPubkey: null == (f = S.params) || null == (b = f.result) || null == (p = b.value) ? void 0 : p.pubkey,
                                        hasAccountData: !!(null == (j = S.params) || null == (w = j.result) || null == (y = w.value) || null == (v = y.account) ? void 0 : v.data),
                                        hasDirectData: !!(null == (A = S.params) || null == (N = A.result) || null == (k = N.value) ? void 0 : k.data)
                                    });
                                    return
                                }
                            } catch (e) {
                                g.R.warn("[LimitOrderWebSocket] Error parsing WS message:", e)
                            }
                        }
                        ,
                        this.socket.onclose = () => {
                            this.isConnectingWs = !1,
                            this.handleConnectionLoss()
                        }
                        ,
                        this.socket.onerror = () => {}
                    } catch (e) {
                        this.isConnectingWs = !1,
                        g.R.error("[LimitOrderWebSocket] Failed to create WS connection:", e),
                        this.handleConnectionLoss()
                    }
                }
            }
            subscribeToLimitOrders(e) {
                if (this.socket && this.socket.readyState === WebSocket.OPEN)
                    try {
                        g.R.info("[LimitOrderWebSocket] Subscribing to limit orders for wallet:", e);
                        let t = {
                            jsonrpc: "2.0",
                            id: "limit-orders-".concat(e),
                            method: "programSubscribe",
                            params: [this.TITAN_LIMIT_ORDERS_PROGRAM_ID, {
                                encoding: "base64",
                                commitment: "confirmed",
                                filters: [{
                                    dataSize: this.LIMIT_ORDER_ACCOUNT_SIZE
                                }, {
                                    memcmp: {
                                        offset: 0,
                                        bytes: e
                                    }
                                }]
                            }]
                        };
                        this.socket.send(JSON.stringify(t)),
                        g.R.info("[LimitOrderWebSocket] Limit orders subscription message sent:", t)
                    } catch (e) {
                        g.R.warn("[LimitOrderWebSocket] Failed to send limit orders subscribe message:", e)
                    }
            }
            isRelevantLimitOrderChange(e) {
                var t, s, a, i, n;
                if (!this.walletAddress)
                    return !1;
                let l = null == e || null == (t = e.params) ? void 0 : t.result;
                if (!(null == l ? void 0 : l.value))
                    return !1;
                try {
                    let e = l.value.data;
                    if (!e && (null == (s = l.value.account) ? void 0 : s.data) && (e = l.value.account.data),
                    !e)
                        return g.R.debug("[LimitOrderWebSocket] No account data found in notification"),
                        !1;
                    if (g.R.debug("[LimitOrderWebSocket] Checking account data:", {
                        walletAddress: this.walletAddress,
                        accountDataType: typeof e,
                        isArray: Array.isArray(e),
                        dataLength: Array.isArray(e) ? e.length : "not array",
                        hasAccountData: !!(null == (a = l.value.account) ? void 0 : a.data),
                        hasDirectData: !!l.value.data
                    }),
                    Array.isArray(e) && e.length > 0) {
                        let t = f.from(e[0], "base64");
                        if (g.R.debug("[LimitOrderWebSocket] Decoded base64 data:", {
                            bufferLength: t.length,
                            expectedLength: this.LIMIT_ORDER_ACCOUNT_SIZE,
                            matchesExpectedSize: t.length === this.LIMIT_ORDER_ACCOUNT_SIZE
                        }),
                        t.length !== this.LIMIT_ORDER_ACCOUNT_SIZE)
                            return g.R.debug("[LimitOrderWebSocket] Account data size mismatch, not a limit order"),
                            !1;
                        let s = t.subarray(0, 32)
                          , a = new p.PublicKey(this.walletAddress)
                          , i = s.equals(a.toBuffer());
                        return i && g.R.info("[LimitOrderWebSocket] Found matching maker in base64 data"),
                        i
                    }
                    if (e && "object" == typeof e && !Array.isArray(e)) {
                        if ((null == (n = e.parsed) || null == (i = n.info) ? void 0 : i.owner) === this.walletAddress)
                            return g.R.info("[LimitOrderWebSocket] Found matching owner in parsed data"),
                            !0;
                        let t = JSON.stringify(e).includes(this.walletAddress);
                        return t && g.R.info("[LimitOrderWebSocket] Found wallet address in account data"),
                        t
                    }
                    return g.R.debug("[LimitOrderWebSocket] Unknown account data format"),
                    !1
                } catch (e) {
                    return g.R.warn("[LimitOrderWebSocket] Error checking limit order relevance:", e),
                    !1
                }
            }
            invalidateLimitOrderCache() {
                if (!this.queryClient || !this.walletAddress)
                    return;
                g.R.info("[LimitOrderWebSocket] Invalidating limit order cache", {
                    walletAddress: this.walletAddress
                });
                let e = this.queryClient.getQueryData(b.w9.userOrders(this.walletAddress))
                  , t = this.queryClient.getQueryData(b.w9.userOpenOrders(this.walletAddress));
                g.R.info("[LimitOrderWebSocket] Cache state before invalidation:", {
                    walletAddress: this.walletAddress,
                    userOrdersCount: Array.isArray(e) ? e.length : "not array",
                    userOpenOrdersCount: Array.isArray(t) ? t.length : "not array",
                    userOrdersData: e,
                    userOpenOrdersData: t
                }),
                this.queryClient.invalidateQueries({
                    queryKey: b.w9.userOpenOrders(this.walletAddress)
                })
            }
            cleanupConnection() {
                this.socket && (this.socket.close(),
                this.socket = null),
                this.subscriptions.clear()
            }
            handleConnectionLoss() {
                this.scheduleReconnect()
            }
            scheduleReconnect() {
                if (this.reconnectAttempts >= this.maxReconnectAttempts)
                    return void g.R.error("[LimitOrderWebSocket] Max reconnection attempts reached");
                let e = Math.min(this.initialReconnectDelayMs * Math.pow(2, this.reconnectAttempts), this.maxReconnectDelayMs);
                g.R.info("[LimitOrderWebSocket] Scheduling reconnection", {
                    attempt: this.reconnectAttempts + 1,
                    delay: e
                }),
                this.reconnectTimeout = setTimeout( () => {
                    this.reconnectAttempts++,
                    this.connect()
                }
                , e)
            }
            constructor() {
                this.socket = null,
                this.isConnectingWs = !1,
                this.reconnectTimeout = null,
                this.reconnectAttempts = 0,
                this.maxReconnectAttempts = 5,
                this.initialReconnectDelayMs = 1e3,
                this.maxReconnectDelayMs = 3e4,
                this.subscriptions = new Map,
                this.walletAddress = null,
                this.queryClient = null,
                this.apiBaseUrl = "",
                this.rpcWssUrl = "",
                this.TITAN_LIMIT_ORDERS_PROGRAM_ID = new p.PublicKey("TitanLozLMhczcwrioEguG2aAmiATAPXdYpBg3DbeKK"),
                this.LIMIT_ORDER_ACCOUNT_SIZE = 168
            }
        }
        let y = v.getInstance();
        var w = s(93739);
        let j = () => {
            let {isConnected: e, walletAddress: t} = (0,
            x.j)()
              , s = (0,
            m.jE)()
              , {appConfig: i} = (0,
            w.A)()
              , n = null == i ? void 0 : i.RPC_WSS_URL
              , l = (0,
            h.Hb)(t || "")
              , r = (0,
            u.useCallback)( () => {
                if (e && t && n)
                    try {
                        y.initialize({
                            apiBaseUrl: window.location.origin,
                            rpcWssUrl: n
                        }, s),
                        y.startMonitoring(t)
                    } catch (e) {
                        g.R.warn("[LimitOrderWebSocketProvider] Failed to initialize WebSocket service:", e)
                    }
            }
            , [e, t, s, n])
              , o = (0,
            u.useCallback)( () => {
                try {
                    y.stopMonitoring()
                } catch (e) {
                    g.R.warn("[LimitOrderWebSocketProvider] Error stopping WebSocket service:", e)
                }
            }
            , []);
            return (0,
            u.useEffect)( () => (e && t && n ? r() : o(),
            o), [e, t, r, o, n]),
            (0,
            u.useEffect)( () => {}
            , [t]),
            (0,
            u.useEffect)( () => {}
            , [l.isFetching]),
            (0,
            a.jsx)(u.Fragment, {})
        }
        ;
        var k = s(75706)
          , N = s(58827);
        function A(e) {
            let {children: t} = e
              , {appConfig: s} = (0,
            w.A)()
              , i = (0,
            u.useMemo)( () => (null == s ? void 0 : s.PRIVY_APP_ID) ? s.PRIVY_APP_ID : (g.R.warn("No Privy App ID found"),
            ""), [s]);
            return i ? (0,
            a.jsx)(k.si, {
                appId: i,
                config: {
                    loginMethods: ["wallet"],
                    appearance: {
                        showWalletLoginFirst: !0,
                        theme: "dark",
                        walletChainType: "solana-only",
                        walletList: ["detected_solana_wallets", "backpack", "solflare", "phantom", "wallet_connect"],
                        logo: "images/titan-symbol-white.png"
                    },
                    externalWallets: {
                        solana: {
                            connectors: (0,
                            N.dS)()
                        }
                    }
                },
                children: t
            }) : (0,
            a.jsx)(a.Fragment, {})
        }
        var S = s(90529)
          , C = s(73971)
          , R = s(77729);
        let W = () => {
            let {walletAddress: e, connected: t} = (0,
            S.z)()
              , s = !!e
              , i = (0,
            C.v4)(null, t && s ? e : null)
              , {setProfile: n, setProfileLoading: l, setProfileError: r} = (0,
            R.Ay)();
            return (0,
            u.useEffect)( () => {
                s || (n(null),
                r(null))
            }
            , [s, n, r]),
            (0,
            u.useEffect)( () => {
                l(i.isLoading)
            }
            , [i.isLoading, l]),
            (0,
            u.useEffect)( () => {
                r(i.error)
            }
            , [i.error, r]),
            (0,
            u.useEffect)( () => {
                i.data && n(i.data)
            }
            , [i.data, n]),
            (0,
            a.jsx)(u.Fragment, {})
        }
        ;
        var _ = s(41313)
          , O = s(998)
          , T = s(96853)
          , U = s(82945)
          , E = s(25465);
        class P {
            initialize(e, t) {
                this.queryClient = e,
                this.walletAddress = t
            }
            async processAccountNotification(e) {
                if (!this.queryClient || !this.walletAddress)
                    return;
                let {result: t} = e
                  , s = t.value.lamports;
                g.R.debug("[WebSocketBalanceUpdater] Processing SOL balance update:", {
                    lamports: s,
                    slot: t.context.slot
                }),
                await this.updateWalletBalance({
                    solBalance: s
                })
            }
            async processProgramNotification(e) {
                if (!this.queryClient || !this.walletAddress)
                    return void g.R.warn("[WebSocketBalanceUpdater] Missing queryClient or walletAddress");
                let {result: t} = e
                  , s = t.value.account.data.parsed.info;
                if (s.owner !== this.walletAddress)
                    return void g.R.debug("[WebSocketBalanceUpdater] Token account not owned by current wallet:", {
                        owner: s.owner,
                        walletAddress: this.walletAddress,
                        mint: s.mint
                    });
                let a = t.value.account.owner
                  , i = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" === a;
                if ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" !== a && !i)
                    return void g.R.debug("[WebSocketBalanceUpdater] Unknown program ID:", {
                        programId: a,
                        mint: s.mint
                    });
                g.R.debug("[WebSocketBalanceUpdater] Processing token balance update:", {
                    mint: s.mint,
                    owner: s.owner,
                    amount: s.tokenAmount.amount,
                    decimals: s.tokenAmount.decimals,
                    program: i ? "Token-2022" : "SPL Token",
                    slot: t.context.slot
                });
                let n = {
                    mint: s.mint,
                    owner: s.owner,
                    rawAmount: Number(s.tokenAmount.amount),
                    amount: s.tokenAmount.uiAmount || 0,
                    decimals: s.tokenAmount.decimals,
                    tokenAccount: t.value.pubkey,
                    programId: a
                };
                g.R.debug("[WebSocketBalanceUpdater] Updating wallet balance with token account:", n),
                await this.updateWalletBalance({
                    tokenAccount: n
                })
            }
            async updateWalletBalance(e) {
                var t;
                if (!this.queryClient || !this.walletAddress)
                    return void g.R.warn("[WebSocketBalanceUpdater] Missing queryClient or walletAddress in updateWalletBalance");
                this.updateQueue.push(e),
                g.R.debug("[WebSocketBalanceUpdater] Added update to queue:", {
                    queueLength: this.updateQueue.length,
                    updateType: void 0 !== e.solBalance ? "SOL" : "Token",
                    updateMint: null == (t = e.tokenAccount) ? void 0 : t.mint
                }),
                this.queueTimeout && clearTimeout(this.queueTimeout),
                this.queueTimeout = setTimeout( () => {
                    this.processUpdateQueue()
                }
                , this.QUEUE_PROCESSING_DELAY)
            }
            async processUpdateQueue() {
                if (!this.isProcessingQueue && 0 !== this.updateQueue.length) {
                    this.isProcessingQueue = !0,
                    this.queueTimeout = null;
                    try {
                        let e = [...this.updateQueue];
                        this.updateQueue = [],
                        g.R.debug("[WebSocketBalanceUpdater] Processing batched updates:", {
                            updateCount: e.length,
                            updates: e.map(e => {
                                var t;
                                return {
                                    type: void 0 !== e.solBalance ? "SOL" : "Token",
                                    mint: null == (t = e.tokenAccount) ? void 0 : t.mint
                                }
                            }
                            )
                        });
                        let t = this.queryClient.getQueryData(U.l.wallet.balances(this.walletAddress));
                        if (!t) {
                            g.R.debug("[WebSocketBalanceUpdater] No cached data found, invalidating queries"),
                            this.queryClient.invalidateQueries({
                                queryKey: U.l.wallet.balances(this.walletAddress)
                            });
                            return
                        }
                        let s = t;
                        for (let t of e)
                            s = await this.buildUpdatedWalletSummary(s, t);
                        g.R.debug("[WebSocketBalanceUpdater] Batched updates prepared:", {
                            updatedTokenCount: s.tokenBalances.length,
                            totalUsdValue: s.totalUsdValue
                        }),
                        this.queryClient.setQueryData(U.l.wallet.balances(this.walletAddress), s),
                        g.R.debug("[WebSocketBalanceUpdater] Cache updated successfully with batched updates", {
                            updatedTokenCount: s.tokenBalances.length,
                            totalUsdValue: s.totalUsdValue,
                            sampleBalances: s.tokenBalances.slice(0, 3).map(e => ({
                                mint: e.token.address,
                                balance: e.balance,
                                symbol: e.token.symbol
                            }))
                        })
                    } catch (e) {
                        g.R.error("[WebSocketBalanceUpdater] Failed to update balance cache:", e),
                        this.queryClient.invalidateQueries({
                            queryKey: U.l.wallet.balances(this.walletAddress)
                        })
                    } finally {
                        this.isProcessingQueue = !1,
                        this.updateQueue.length > 0 && (this.queueTimeout = setTimeout( () => {
                            this.processUpdateQueue()
                        }
                        , this.QUEUE_PROCESSING_DELAY))
                    }
                }
            }
            async buildUpdatedWalletSummary(e, t) {
                let s = [...e.tokenBalances];
                if (void 0 !== t.solBalance) {
                    let e = s.findIndex(e => e.token.address === E.wV)
                      , a = this.getTokenPriceFromCache(E.wV)
                      , i = (0,
                    E.iU)(t.solBalance)
                      , n = {
                        token: {
                            ...E.Es,
                            address: E.wV,
                            decimals: E.Es.decimals
                        },
                        balance: i,
                        rawBalance: t.solBalance,
                        price: a,
                        usdValue: i * a,
                        accountAddress: this.walletAddress,
                        isStale: !1
                    };
                    e >= 0 ? s[e] = n : s.push(n)
                }
                if (t.tokenAccount) {
                    let {mint: e, amount: a, rawAmount: i, tokenAccount: n, decimals: l} = t.tokenAccount
                      , [r,o] = await Promise.all([this.getTokenMetadata(e), Promise.resolve(this.getTokenPriceFromCache(e))])
                      , c = void 0 !== l ? l : r.decimals;
                    g.R.debug("[WebSocketBalanceUpdater] Token balance creation:", {
                        mint: e,
                        webSocketDecimals: l,
                        metadataDecimals: r.decimals,
                        finalDecimals: c,
                        amount: a,
                        rawAmount: i
                    });
                    let d = {
                        token: {
                            ...r,
                            decimals: c
                        },
                        balance: a,
                        rawBalance: i,
                        price: o,
                        usdValue: a * o,
                        accountAddress: n,
                        isStale: !1
                    }
                      , u = s.findIndex(t => t.token.address === e);
                    a > 0 ? u >= 0 ? s[u] = d : s.push(d) : u >= 0 && e !== E.wV && s.splice(u, 1)
                }
                s.sort( (e, t) => e.token.verified !== t.token.verified ? t.token.verified ? 1 : -1 : t.usdValue !== e.usdValue ? t.usdValue - e.usdValue : t.balance - e.balance);
                let a = s.reduce( (e, t) => e + t.usdValue, 0);
                return {
                    ...e,
                    tokenBalances: s,
                    totalUsdValue: a,
                    lastUpdated: Date.now()
                }
            }
            async getTokenMetadata(e) {
                if (this.tokenMetadataCache[e])
                    return this.tokenMetadataCache[e];
                let t = Date.now();
                if (t - this.lastMetadataFetch < this.METADATA_CACHE_DURATION)
                    return this.createDefaultToken(e);
                try {
                    let s = await fetch("/api/tokens/multiple", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json"
                        },
                        body: JSON.stringify({
                            addresses: [e]
                        })
                    });
                    if (s.ok) {
                        let a = await s.json();
                        if (a.success && a.results && a.results.length > 0) {
                            let s = a.results[0]
                              , i = {
                                address: e,
                                symbol: s.symbol || "UNKNOWN",
                                name: s.name || "Unknown Token",
                                decimals: s.decimals || 0,
                                verified: s.verified || !1,
                                logoURI: s.logoURI
                            };
                            return this.tokenMetadataCache[e] = i,
                            this.lastMetadataFetch = t,
                            i
                        }
                    }
                } catch (e) {
                    g.R.warn("[WebSocketBalanceUpdater] Failed to fetch token metadata:", e)
                }
                return this.createDefaultToken(e)
            }
            getTokenPriceFromCache(e) {
                if (!this.queryClient)
                    return 0;
                let t = U.l.prices.single(e)
                  , s = this.queryClient.getQueryData(t);
                return null != s ? s : 0
            }
            createDefaultToken(e) {
                return {
                    address: e,
                    symbol: "UNKNOWN",
                    name: "Unknown Token",
                    decimals: 0,
                    verified: !1,
                    logoURI: void 0
                }
            }
            flushQueue() {
                this.queueTimeout && (clearTimeout(this.queueTimeout),
                this.queueTimeout = null),
                this.updateQueue.length > 0 && this.processUpdateQueue()
            }
            clearCaches() {
                this.tokenMetadataCache = {},
                this.lastMetadataFetch = 0,
                this.updateQueue = [],
                this.isProcessingQueue = !1,
                this.queueTimeout && (clearTimeout(this.queueTimeout),
                this.queueTimeout = null)
            }
            constructor() {
                this.queryClient = null,
                this.walletAddress = null,
                this.tokenMetadataCache = {},
                this.lastMetadataFetch = 0,
                this.METADATA_CACHE_DURATION = 3e5,
                this.updateQueue = [],
                this.isProcessingQueue = !1,
                this.QUEUE_PROCESSING_DELAY = 10,
                this.queueTimeout = null
            }
        }
        let D = new P
          , L = new p.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
          , M = new p.PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
        class I {
            initialize(e, t) {
                this.apiBaseUrl = e.apiBaseUrl || this.apiBaseUrl,
                this.rpcWssUrl = e.rpcWssUrl || this.rpcWssUrl,
                this.queryClient = t,
                this.rpcWssUrl || g.R.warn("[WalletWebSocket] NEXT_PUBLIC_RPC_WSS_URL is not set")
            }
            startMonitoring(e) {
                this.queryClient && e && (this.stopMonitoring(),
                this.walletAddress = e,
                this.reconnectAttempts = 0,
                D.initialize(this.queryClient, e),
                this.connect())
            }
            stopMonitoring() {
                this.cleanupConnection(),
                this.walletAddress = null,
                this.reconnectAttempts = 0,
                D.clearCaches(),
                this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null)
            }
            connect() {
                this.walletAddress && this.connectViaWebSocket()
            }
            connectViaWebSocket() {
                if (this.walletAddress && !this.isConnectingWs && (!this.socket || this.socket.readyState !== WebSocket.OPEN)) {
                    if (!this.rpcWssUrl)
                        return void g.R.warn("[WalletWebSocket] RPC WSS URL missing; cannot connect");
                    try {
                        this.isConnectingWs = !0,
                        this.socket = new WebSocket(this.rpcWssUrl),
                        this.socket.onopen = () => {
                            this.isConnectingWs = !1,
                            this.reconnectAttempts = 0,
                            g.R.info("[WalletWebSocket] WebSocket connection established"),
                            this.walletAddress && (g.R.info("[WalletWebSocket] Starting subscriptions for wallet: ".concat(this.walletAddress)),
                            this.subscribeToAccount(this.walletAddress),
                            this.subscribeToSplTokens(this.walletAddress),
                            this.subscribeTo2022Tokens(this.walletAddress))
                        }
                        ,
                        this.socket.onmessage = async e => {
                            try {
                                let t = JSON.parse(e.data);
                                if ((null == t ? void 0 : t.result) && "number" == typeof t.result && t.id) {
                                    let e = String(t.id);
                                    e.startsWith("spl-tokens-") || e.startsWith("2022-tokens-") ? (this.tokenSubscriptions.set(e, t.result),
                                    g.R.info("[WalletWebSocket] Token subscription confirmed - ID: ".concat(e, ", Subscription ID: ").concat(t.result))) : (this.subscriptions.set(e, t.result),
                                    g.R.info("[WalletWebSocket] Account subscription confirmed - Address: ".concat(e, ", Subscription ID: ").concat(t.result)));
                                    return
                                }
                                if ((null == t ? void 0 : t.method) === "accountNotification") {
                                    g.R.debug("[WalletWebSocket] Account notification received: ".concat(JSON.stringify(t.params))),
                                    g.R.info("[WalletWebSocket] Account notification received, updating balance directly"),
                                    await D.processAccountNotification(t.params);
                                    return
                                }
                                if ((null == t ? void 0 : t.method) === "programNotification") {
                                    g.R.debug("[WalletWebSocket] Program notification received: ".concat(JSON.stringify(t.params))),
                                    this.isRelevantTokenChange(t) && (g.R.info("[WalletWebSocket] Relevant token change detected, updating balance directly"),
                                    await D.processProgramNotification(t.params));
                                    return
                                }
                            } catch (e) {
                                g.R.warn("[WalletWebSocket] Error parsing WS message:", e)
                            }
                        }
                        ,
                        this.socket.onclose = () => {
                            this.isConnectingWs = !1,
                            this.handleConnectionLoss()
                        }
                        ,
                        this.socket.onerror = () => {}
                    } catch (e) {
                        this.isConnectingWs = !1,
                        g.R.error("[WalletWebSocket] Failed to create WS connection:", e),
                        this.handleConnectionLoss()
                    }
                }
            }
            subscribeToAccount(e) {
                if (this.socket && this.socket.readyState === WebSocket.OPEN)
                    try {
                        this.socket.send(JSON.stringify({
                            jsonrpc: "2.0",
                            id: e,
                            method: "accountSubscribe",
                            params: [e, {
                                commitment: "confirmed"
                            }]
                        }))
                    } catch (e) {
                        g.R.warn("[WalletWebSocket] Failed to send subscribe message:", e)
                    }
            }
            subscribeToSplTokens(e) {
                if (this.socket && this.socket.readyState === WebSocket.OPEN)
                    try {
                        g.R.info("[WalletWebSocket] Subscribing to SPL tokens for wallet:", e);
                        let t = {
                            jsonrpc: "2.0",
                            id: "spl-tokens-".concat(e),
                            method: "programSubscribe",
                            params: [L, {
                                encoding: "jsonParsed",
                                commitment: "confirmed",
                                filters: [{
                                    dataSize: 165
                                }, {
                                    memcmp: {
                                        offset: 32,
                                        bytes: e
                                    }
                                }]
                            }]
                        };
                        this.socket.send(JSON.stringify(t)),
                        g.R.info("[WalletWebSocket] SPL tokens subscription message sent:", t)
                    } catch (e) {
                        g.R.warn("[WalletWebSocket] Failed to send SPL tokens subscribe message:", e)
                    }
            }
            subscribeTo2022Tokens(e) {
                if (this.socket && this.socket.readyState === WebSocket.OPEN)
                    try {
                        g.R.info("[WalletWebSocket] Subscribing to 2022 tokens for wallet:", e);
                        let t = {
                            jsonrpc: "2.0",
                            id: "2022-tokens-".concat(e),
                            method: "programSubscribe",
                            params: [M, {
                                encoding: "jsonParsed",
                                commitment: "confirmed",
                                filters: [{
                                    memcmp: {
                                        offset: 32,
                                        bytes: e
                                    }
                                }]
                            }]
                        };
                        this.socket.send(JSON.stringify(t)),
                        g.R.info("[WalletWebSocket] 2022 tokens subscription message sent:", t)
                    } catch (e) {
                        g.R.warn("[WalletWebSocket] Failed to send 2022 tokens subscribe message:", e)
                    }
            }
            isRelevantTokenChange(e) {
                let t;
                if (!this.walletAddress || !e || "object" != typeof e)
                    return !1;
                if (e.result)
                    t = e.result;
                else {
                    if (!e.params)
                        return !1;
                    t = e.params.result
                }
                let s = null == t ? void 0 : t.value;
                if (!s)
                    return !1;
                let a = s.account;
                if (!a)
                    return !1;
                let i = a.data
                  , n = null == i ? void 0 : i.parsed
                  , l = null == n ? void 0 : n.info
                  , r = null == l ? void 0 : l.owner
                  , o = r === this.walletAddress;
                return o && g.R.debug("[WalletWebSocket] Token change is relevant for wallet:", {
                    owner: r,
                    walletAddress: this.walletAddress,
                    mint: null == l ? void 0 : l.mint,
                    program: a.owner
                }),
                o
            }
            handleConnectionLoss() {
                this.cleanupConnection(),
                this.attemptReconnect()
            }
            attemptReconnect() {
                if (!this.walletAddress)
                    return;
                if (this.reconnectAttempts >= this.maxReconnectAttempts)
                    return void g.R.warn("[WalletWebSocket] Max WS reconnection attempts reached");
                this.reconnectAttempts++;
                let e = Math.min(this.initialReconnectDelayMs * Math.pow(2, this.reconnectAttempts), this.maxReconnectDelayMs);
                this.reconnectTimeout = setTimeout( () => this.connectViaWebSocket(), e)
            }
            cleanupConnection() {
                if (this.reconnectTimeout && (clearTimeout(this.reconnectTimeout),
                this.reconnectTimeout = null),
                this.socket) {
                    try {
                        this.subscriptions.forEach( (e, t) => {
                            this.socket && this.socket.readyState === WebSocket.OPEN && (g.R.info("[WalletWebSocket] Unsubscribing from account: ".concat(t, ", Subscription ID: ").concat(e)),
                            this.socket.send(JSON.stringify({
                                jsonrpc: "2.0",
                                id: t,
                                method: "accountUnsubscribe",
                                params: [e]
                            })))
                        }
                        ),
                        this.tokenSubscriptions.forEach( (e, t) => {
                            this.socket && this.socket.readyState === WebSocket.OPEN && (g.R.info("[WalletWebSocket] Unsubscribing from token subscription: ".concat(t, ", Subscription ID: ").concat(e)),
                            this.socket.send(JSON.stringify({
                                jsonrpc: "2.0",
                                id: t,
                                method: "programUnsubscribe",
                                params: [e]
                            })))
                        }
                        )
                    } catch (e) {}
                    try {
                        this.socket.close()
                    } catch (e) {}
                    this.socket = null
                }
                this.subscriptions.clear(),
                this.tokenSubscriptions.clear(),
                g.R.info("[WalletWebSocket] All subscriptions cleared")
            }
            constructor() {
                this.socket = null,
                this.isConnectingWs = !1,
                this.reconnectTimeout = null,
                this.reconnectAttempts = 0,
                this.maxReconnectAttempts = 5,
                this.initialReconnectDelayMs = 1e3,
                this.maxReconnectDelayMs = 3e4,
                this.subscriptions = new Map,
                this.tokenSubscriptions = new Map,
                this.walletAddress = null,
                this.queryClient = null,
                this.apiBaseUrl = "",
                this.rpcWssUrl = ""
            }
        }
        let z = new I
          , F = () => {
            let {isConnected: e, walletAddress: t} = (0,
            x.j)()
              , s = (0,
            m.jE)()
              , {appConfig: i} = (0,
            w.A)()
              , n = null == i ? void 0 : i.RPC_WSS_URL
              , l = (0,
            T.X2)(t)
              , r = (0,
            u.useCallback)( () => {
                if (e && t && n)
                    try {
                        z.initialize({
                            apiBaseUrl: window.location.origin,
                            rpcWssUrl: n
                        }, s),
                        z.startMonitoring(t)
                    } catch (e) {
                        g.R.warn("[WalletBalanceProvider] Failed to initialize WebSocket service:", e)
                    }
            }
            , [e, t, s, n])
              , o = (0,
            u.useCallback)( () => {
                try {
                    z.stopMonitoring()
                } catch (e) {
                    g.R.warn("[WalletBalanceProvider] Error stopping WebSocket service:", e)
                }
            }
            , []);
            return (0,
            u.useEffect)( () => (e && t && n ? r() : o(),
            o), [e, t, r, o, n]),
            (0,
            u.useEffect)( () => {}
            , [t]),
            (0,
            u.useEffect)( () => {}
            , [l.isFetching]),
            (0,
            a.jsx)(u.Fragment, {})
        }
        ;
        var B = s(55436)
          , q = s(46750)
          , V = s(94187)
          , Q = s(80032)
          , K = s(91015).Buffer;
        async function G(e, t) {
            try {
                g.R.info("Checking sponsored transaction status for: ".concat(t.toString()));
                let s = new p.PublicKey("sponsorKDrY6B1TXJQ5GKUdvGNSbSKRsW8UxGp82Q5Q")
                  , [a] = p.PublicKey.findProgramAddressSync([K.from("sponsorship_tracker"), t.toBuffer()], s)
                  , i = await e.getAccountInfo(a, "confirmed");
                if (!i)
                    return g.R.info("User does not exist in sponsored transaction system"),
                    {
                        exists: !1,
                        count: 0
                    };
                let n = {
                    subsidized_tx_count: i.data.readUInt16LE(32)
                };
                return g.R.info("User exists in system with ".concat(n.subsidized_tx_count, " sponsored transactions")),
                {
                    exists: !0,
                    count: n.subsidized_tx_count
                }
            } catch (e) {
                throw console.error("Error fetching sponsored transaction status:", e),
                e
            }
        }
        var X = s(28764)
          , $ = s(96519);
        let J = {
            trailblazer: {
                icon: "/images/badges/trailblazer_medal.webp",
                image: "/images/badges/trailblazer_bg.svg"
            },
            colossus: {
                icon: "/images/badges/colossus_medal.webp",
                image: "/images/badges/colossus_bg.svg"
            },
            pathfinder: {
                icon: "/images/badges/pathfinder_medal.webp",
                image: "/images/badges/pathfinder_bg.webp"
            },
            legion: {
                icon: "/images/badges/legion_medal.webp",
                image: "/images/badges/legion_medal.webp"
            },
            backpack: {
                icon: "/images/badges/backpack_medal.webp",
                image: "/images/badges/backpack_medal.webp"
            }
        }
          , H = {
            icon: "/images/badges/pathfinder_medal.webp",
            image: "/images/badges/pathfinder_bg.webp"
        }
          , Z = () => {
            let {walletAddress: e} = (0,
            x.j)();
            return !function(e) {
                let {setWalletStats: t, setHasDataError: s} = (0,
                V.Ay)()
                  , a = (0,
                B.I)({
                    queryKey: U.l.walletStats.stats(e || ""),
                    queryFn: async () => {
                        if (!e)
                            throw Error("Wallet address is required");
                        let s = await (0,
                        q.n3)(e);
                        return s && t(s),
                        s
                    }
                    ,
                    enabled: !!e,
                    staleTime: 12e4,
                    gcTime: 3e5
                });
                (0,
                u.useEffect)( () => {
                    a.error && s(!!a.error)
                }
                , [a.error, s])
            }(e),
            !function(e) {
                let {setReferralStats: t, setHasDataError: s} = (0,
                V.Ay)()
                  , a = (0,
                B.I)({
                    queryKey: U.l.walletStats.referralStats(e || ""),
                    queryFn: async () => {
                        if (!e)
                            throw Error("Wallet address is required");
                        let s = await (0,
                        q.L7)(e);
                        return s && t(s),
                        s
                    }
                    ,
                    enabled: !!e,
                    staleTime: 12e4,
                    gcTime: 3e5
                });
                (0,
                u.useEffect)( () => {
                    a.error && s(!!a.error)
                }
                , [a.error, s])
            }(e),
            !function(e) {
                var t;
                let {setBadges: s, setHasBadgeError: a} = (0,
                V.Ay)()
                  , i = (0,
                B.I)({
                    queryKey: U.l.walletStats.badges(e || ""),
                    queryFn: async () => {
                        if (!e)
                            throw Error("Wallet address is required");
                        return (0,
                        q.qL)(e)
                    }
                    ,
                    enabled: !!e,
                    staleTime: 3e5,
                    gcTime: 6e5
                })
                  , n = (0,
                u.useMemo)( () => {
                    var e;
                    if (!(null == (e = i.data) ? void 0 : e.badges))
                        return [];
                    let t = i.data.badges
                      , s = [];
                    return Object.entries(t).forEach(e => {
                        let[t,a] = e;
                        if ("object" == typeof a && null !== a && "has_badge"in a) {
                            let e = t in J ? J[t] : H;
                            s.push({
                                name: t,
                                description: a.description,
                                has_badge: a.has_badge,
                                notified: a.notified,
                                order: a.order || 999,
                                icon: e.icon,
                                image: e.image
                            })
                        }
                    }
                    ),
                    s.sort( (e, t) => e.order !== t.order ? e.order - t.order : e.has_badge !== t.has_badge ? e.has_badge ? -1 : 1 : e.name.localeCompare(t.name))
                }
                , [null == (t = i.data) ? void 0 : t.badges]);
                (0,
                u.useEffect)( () => {
                    s(n)
                }
                , [n, s]),
                (0,
                u.useEffect)( () => {
                    i.error && a(!!i.error)
                }
                , [i.error, a])
            }(e),
            !function(e) {
                let {walletVipStatus: t, setSponsoredTransactionStatus: s} = (0,
                Q.Ay)()
                  , {appConfig: a} = (0,
                w.A)()
                  , i = (0,
                B.I)({
                    queryKey: U.l.walletStats.sponsoredTransactions(e ? "".concat(t, "-").concat(t.isVip) : ""),
                    queryFn: async () => {
                        if (!e)
                            throw Error("Wallet address is required");
                        if (!t.isVip)
                            return {
                                exists: !1,
                                count: 0
                            };
                        let s = (null == a ? void 0 : a.RPC_NODE_URL) || "https://api.mainnet-beta.solana.com"
                          , i = new p.Connection(s,"confirmed")
                          , n = new p.PublicKey(e);
                        return await G(i, n)
                    }
                    ,
                    enabled: !!e,
                    staleTime: 3e4,
                    gcTime: 12e4,
                    retry: 2
                });
                (0,
                u.useEffect)( () => {
                    var e, t;
                    let a = i.isPending || i.isFetching && !i.data;
                    s({
                        exists: (null == (e = i.data) ? void 0 : e.exists) || !1,
                        count: (null == (t = i.data) ? void 0 : t.count) || 0,
                        isLoading: a,
                        error: i.error
                    })
                }
                , [i.data, i.isPending, i.isFetching, i.error, s])
            }(e),
            (0,
            $.tl)(e),
            (0,
            X.S)(),
            (0,
            a.jsx)(u.Fragment, {})
        }
        ;
        var Y = s(55036);
        let ee = e => {
            let {children: t} = e
              , s = n.B.Mainnet
              , {appConfig: m} = (0,
            w.A)()
              , h = (0,
            u.useMemo)( () => (null == m ? void 0 : m.RPC_NODE_URL) ? m.RPC_NODE_URL : Y.env.RPC_NODE_URL ? Y.env.RPC_NODE_URL : (g.R.warn("Using fallback RPC endpoint - configure RPC_NODE_URL environment variable"),
            "https://carolee-5lpkqt-fast-mainnet.helius-rpc.com/"), [m])
              , p = (0,
            u.useMemo)( () => [new c.c, new d.d({
                network: s
            }), new i.h], [s]);
            return (0,
            a.jsx)(A, {
                children: (0,
                a.jsx)(l.S, {
                    endpoint: h,
                    children: (0,
                    a.jsx)(r.r, {
                        autoConnect: !0,
                        wallets: p,
                        children: (0,
                        a.jsx)(o.I, {
                            children: (0,
                            a.jsx)(x.Z, {
                                children: (0,
                                a.jsx)(_.Z6, {
                                    children: (0,
                                    a.jsxs)(O.e, {
                                        children: [(0,
                                        a.jsx)(Z, {}), (0,
                                        a.jsx)(F, {}), (0,
                                        a.jsx)(j, {}), (0,
                                        a.jsx)(W, {}), t]
                                    })
                                })
                            })
                        })
                    })
                })
            })
        }
    }
    ,
    89063: () => {}
    ,
    91447: (e, t, s) => {
        "use strict";
        s.d(t, {
            default: () => r
        });
        var a = s(48876)
          , i = s(26432)
          , n = s(34563)
          , l = s(47337);
        let r = () => {
            let e = (0,
            n.TM)()
              , {setAmmMap: t, setIsLoading: s, setAllAmmList: r, setIsError: o, setError: c, setLastUpdated: d, excludedAmmIds: u} = (0,
            l.A)()
              , m = (0,
            i.useMemo)( () => {
                var t;
                if (!(null == (t = e.data) ? void 0 : t.ammMap))
                    return {};
                let s = e.data.ammMap
                  , a = {};
                return Object.keys(s).forEach(e => {
                    let t = u.includes(e);
                    a[e] = {
                        ...s[e],
                        isExcluded: t
                    }
                }
                ),
                a
            }
            , [e.data, u]);
            return (0,
            i.useEffect)( () => {
                s(e.isLoading)
            }
            , [e.isLoading, s]),
            (0,
            i.useEffect)( () => {
                var t;
                o(e.isError),
                c((null == (t = e.error) ? void 0 : t.message) || null)
            }
            , [e.isError, e.error, o, c]),
            (0,
            i.useEffect)( () => {
                var t, s;
                (null == e || null == (t = e.data) ? void 0 : t.ammMap) && 0 !== Object.keys((null == e || null == (s = e.data) ? void 0 : s.ammMap) || {}).length && r(e.data.ammMap)
            }
            , [e.data.ammMap, r]),
            (0,
            i.useEffect)( () => {
                m && 0 !== Object.keys(m).length && (t(m),
                d(new Date))
            }
            , [m, t, d]),
            (0,
            a.jsx)(i.Fragment, {})
        }
    }
    ,
    97267: (e, t, s) => {
        "use strict";
        s.d(t, {
            default: () => o
        });
        var a = s(48876)
          , i = s(26432)
          , n = s(25465)
          , l = s(32641)
          , r = s(73861);
        let o = () => {
            let e = (0,
            l.Qi)()
              , t = (0,
            l.ns)()
              , {setAllTokens: s, setLstTokens: o, setPopularTokens: c, setVerifiedTokens: d, setIsLoadingToken: u} = (0,
            r.A)()
              , m = (0,
            i.useMemo)( () => {
                let s = e.data || []
                  , a = t.data || []
                  , i = new Map;
                return s.forEach(e => {
                    i.set(e.address, e)
                }
                ),
                a.forEach(e => {
                    i.has(e.address) || i.set(e.address, e)
                }
                ),
                Array.from(i.values())
            }
            , [e.data, t.data])
              , h = (0,
            i.useMemo)( () => {
                var t;
                return (null == e ? void 0 : e.data) ? null == e || null == (t = e.data) ? void 0 : t.filter(e => n.BX.includes(e.address)) : []
            }
            , [e.data])
              , x = e.isLoading || t.isLoading;
            return (0,
            i.useEffect)( () => {
                u(x)
            }
            , [x, u]),
            (0,
            i.useEffect)( () => {
                m && s(m)
            }
            , [m, s]),
            (0,
            i.useEffect)( () => {
                h && c(h)
            }
            , [h, c]),
            (0,
            i.useEffect)( () => {
                e.data && d(e.data)
            }
            , [e.data, d]),
            (0,
            i.useEffect)( () => {
                t.data && o(t.data || [])
            }
            , [t.data, o]),
            (0,
            a.jsx)(i.Fragment, {})
        }
    }
}, e => {
    var t = t => e(e.s = t);
    e.O(0, [7094, 6717, 6690, 410, 2346, 8444, 5721, 3375, 7412, 5679, 8209, 7285, 9259, 3659, 5744, 4324, 8787, 8480, 7229, 9665, 7358], () => t(37962)),
    _N_E = e.O()
}
]);
